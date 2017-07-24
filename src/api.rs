use broadcast::*;
use game;
use game::*;
use rocket::http::Status;
use rocket::response::*;
use rocket::State;
use std::mem;

/// The current state for a player that is needed by the host site.
///
/// This doesn't include all of the player's internal state data, only the information needed
/// by the display site.
#[derive(Debug, Serialize, Responder)]
pub struct PlayerData {
    /// The player's ID.
    id: PlayerId,

    /// The player's display name.
    name: String,

    /// The player's current score.
    score: usize,

    /// Whether or not the player has the crown (i.e. if the player is winning).
    has_crown: bool,
}

/// Generates a `PlayerId` for a new player.
// TODO: Allow players to specify a username when registering.
#[get("/register-player")]
pub fn register_player(
    players: State<PlayerMap>,
    winner: State<Winner>,
    host_broadcaster: State<HostBroadcaster>,
    player_broadcaster: State<PlayerBroadcaster>,
) -> PlayerData {
    let id = PlayerId::new();
    let name = game::generate_username();
    let score = 0;

    let player = Player {
        id,
        name: name.clone(),
        score,
    };

    // Add the player to the game state.
    let mut players = players.write().expect("Players map was poisoned!");
    let old = players.insert(id, player);
    assert!(old.is_none(), "Player ID was registered twice");

    // Broadcast to all hosts that a new player has joined.
    host_broadcaster.send(HostBroadcast::PlayerRegister {
        id,
        name: name.clone(),
        score,
    });

    // Update winner if this is the first player.
    let mut winner = winner.lock().expect("Winner was poisoned!");
    let has_crown = winner.is_none();
    if winner.is_none() {
        *winner = Some(id);
        host_broadcaster.send(HostBroadcast::UpdateWinner { id });
        player_broadcaster.send(PlayerBroadcast::UpdateWinner { id });
    }

    // Respond to the client.
    PlayerData { id, name, score, has_crown }
}

/// The request expected from the client for the `/feed-me` endpoint.
#[derive(Debug, Deserialize, FromData)]
pub struct FeedMeRequest {
    /// The `PlayerId` for the player that clicked their "Feed Me" button.
    pub id: PlayerId,
}

/// The response sent back from the `/feed-me` endpoint.
#[derive(Debug, Serialize, Responder)]
pub struct FeedMeResponse {
    pub score: usize,
}

/// Feeds a player's hippo, increasing the player's score.
///
/// # Errors
///
/// If the `player` member of `payload` isn't a valid `PlayerId` (i.e. the ID isn't in `scores`),
/// Then `Err(InvalidPlayer)` is returned.
#[post("/feed-me", format = "application/json", data = "<payload>")]
pub fn feed_player(
    payload: FeedMeRequest,
    players: State<PlayerMap>,
    winner: State<Winner>,
    host_broadcaster: State<HostBroadcaster>,
    player_broadcaster: State<PlayerBroadcaster>,
) -> Result<FeedMeResponse> {
    let id = payload.id;

    // Add 1 to the player's score, returning the new score. We create an explicit scope here to
    // limit how long we hold the lock on the player map.
    let mut players = players.write().expect("Player map was poisoned!");

    // Get the player's current score, or return an `InvalidPlayer` error if it's not in
    // the scoreboard.
    let score = {
        let player = players
            .get_mut(&id)
            .ok_or(Error::InvalidPlayer(id))?;

        player.score += 1;
        player.score
    };

    // Update the host displays.
    host_broadcaster.send(HostBroadcast::HippoEat { id, score });

    let mut winner = winner.lock().expect("Winner was poisoned!");
    let winner = winner.as_mut().expect("There must be a winner if a hippo is being fed");
    let winner_score = players.get(winner).unwrap().score;
    if score > winner_score && id != *winner {
        // Make the current player the new winner.
        mem::replace(winner, id);
        host_broadcaster.send(HostBroadcast::UpdateWinner { id });
        player_broadcaster.send(PlayerBroadcast::UpdateWinner { id });
    }

    Ok(FeedMeResponse { score })
}

#[derive(Debug, Serialize, Responder)]
pub enum NoseGoesResponse {
    Survived,
    Died,
}

#[post("/nose-goes/<id>")]
pub fn nose_goes(
    id: PlayerId,
    nose_goes: State<NoseGoesState>,
) -> Result<NoseGoesResponse> {
    let mut nose_goes = nose_goes.lock().expect("Nose-goes state was poisoned!");
    match *nose_goes {
        NoseGoes::Inactive { .. } => {
            Err(Error::InvalidNoesGoes)
        }

        NoseGoes::InProgress { ref mut remaining_players, .. } => {
            // It's an error for the player to not be part of the nose-goes event.
            if !remaining_players.contains(&id) {
                return Err(Error::InvalidNoesGoes);
            }

            // If there are multiple players still in the event, remove the player. If the player
            // is the last one left, they die.
            if remaining_players.len() > 1 {
                remaining_players.remove(&id);
                Ok(NoseGoesResponse::Survived)
            } else {
                Ok(NoseGoesResponse::Died)
            }
        }
    }
}

/// The response sent back from the `/scoreboard` endpoint.
///
/// Contains the list of current players and all information about each player, useful for giving
/// new hosts the current state of the game.
#[derive(Debug, Serialize, Responder)]
pub struct PlayersResponse {
    pub players: Vec<PlayerData>,
}

#[get("/player/<id>")]
pub fn get_player(
    id: PlayerId,
    players: State<PlayerMap>,
    winner: State<Winner>,
) -> Option<PlayerData> {
    let players = players.read().expect("Player map was poisoned!");
    let winner = winner.lock().expect("Winner was poisoned!");

    players.get(&id).map(|player| PlayerData {
        id: player.id,
        name: player.name.clone(),
        score: player.score,
        has_crown: Some(player.id) == *winner,
    })
}

/// Returns a list of players and their scores.
///
/// This is used by new host connections to update thier display to match the current state of the
/// game.
#[get("/players")]
pub fn get_players(players: State<PlayerMap>, winner: State<Winner>) -> PlayersResponse {
    let players = players.read().expect("Player map was poisoned!");
    let winner = winner.lock().expect("Winner was poisoned!");
    let players = players.values()
        .map(|player| {
            PlayerData {
                id: player.id,
                name: player.name.clone(),
                score: player.score,
                has_crown: Some(player.id) == *winner,
            }
        })
        .collect();

    PlayersResponse { players }
}

/// The error type for an API requests that can fail.
#[derive(Debug, Serialize)]
pub enum Error {
    /// Indicates that an invalid player was specified for the operation.
    ///
    /// This might occur if the client code cached the player ID from a previous session, and is
    /// now trying to use the ID in a session where it is no longer valid. Re-registering the
    /// player to generate a new ID should fix the issue.
    InvalidPlayer(PlayerId),

    /// Indicates that a noes-goes request was not valid.
    ///
    /// This can occur for two reasons:
    ///
    /// - The request arrived when no noes-goes event was active.
    /// - The player was not a part of the active noes-goes event.
    InvalidNoesGoes,
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, request: &::rocket::request::Request) -> ::std::result::Result<Response<'r>, Status> {
        use rocket::response::status::Custom;

        Custom(Status::BadRequest, ::rocket_contrib::Json(self)).respond_to(request)
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;
