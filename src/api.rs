use broadcast::*;
use game;
use game::*;
use rocket::http::Status;
use rocket::response::*;
use rocket::State;
use rocket_contrib::JSON;
use std::time::*;

/// The response sent back from the `/register-player` endpoint.
#[derive(Debug, Serialize)]
pub struct RegisterPlayerResponse {
    /// The `PlayerId` that was generated for the new player.
    pub id: PlayerId,
    pub username: String,
    pub balls: usize,
}

/// Generates a `PlayerId` for a new player.
// TODO: Allow players to specify a username when registering.
#[get("/register-player")]
pub fn register_player(
    player_id_generator: State<PlayerIdGenerator>,
    players: State<PlayerMap>,
    broadcaster: State<HostBroadcaster>,
) -> JSON<RegisterPlayerResponse>
{
    let id = player_id_generator.next_id();
    let username = game::generate_username();
    let balls = 10;

    let player = Player {
        id,
        username: username.clone(),
        score: 0,
        balls,
        next_eat_time: Instant::now() + Duration::from_millis(1000),
    };

    // Add the player to the game state.
    {
        let mut players = players.write().expect("Players map was poisoned!");
        let old = players.insert(id, player);
        assert!(old.is_none(), "Player ID was registered twice");
    }

    // Broadcast to all hosts that a new player has joined.
    broadcaster.send(HostBroadcast::PlayerRegister {
        id,
        username: username.clone(),
        score: 0,
        balls,
    });

    // Respond to the client.
    JSON(RegisterPlayerResponse {
        id,
        username,
        balls,
    })
}

/// The request expected from the client for the `/feed-me` endpoint.
#[derive(Debug, Deserialize)]
pub struct FeedPlayerRequest {
    /// The `PlayerId` for the player that clicked their "Feed Me" button.
    pub player: PlayerId,
}

/// The response sent back from the `/feed-me` endpoint.
#[derive(Debug, Serialize)]
pub struct FeedPlayerResponse {
    balls: usize,
}

/// Feeds a player's hippo, increasing the player's score.
///
/// # Errors
///
/// If the `player` member of `payload` isn't a valid `PlayerId` (i.e. the ID isn't in `scores`),
/// Then `Err(InvalidPlayer)` is returned.
#[post("/feed-me", format = "application/json", data = "<payload>")]
pub fn feed_player(
    payload: JSON<FeedPlayerRequest>,
    players: State<PlayerMap>,
    broadcaster: State<HostBroadcaster>,
) -> Result<JSON<FeedPlayerResponse>>
{
    let payload = payload.into_inner();
    let id = payload.player;

    // Add 1 to the player's score, returning the new score.
    let balls = {
        let mut players = players.write().expect("Players were poisoned");

        // Get the player's current score, or return an `InvalidPlayer` error if it's not in
        // the scoreboard.
        let player = players
            .get_mut(&id)
            .ok_or(Error::InvalidPlayer(id))?;
        player.balls += 1;
        player.balls
    };

    // Update the host displays and respond to the player.
    broadcaster.send(HostBroadcast::AddBall { id, balls });
    Ok(JSON(FeedPlayerResponse { balls }))
}

/// The response sent back from the `/scoreboard` endpoint.
///
/// Contains the list of current players and all information about each player, useful for giving
/// new hosts the current state of the game.
#[derive(Debug, Serialize)]
pub struct PlayersResponse {
    pub players: Vec<PlayerData>,
}

#[derive(Debug, Serialize)]
pub struct PlayerData {
    id: PlayerId,
    username: String,
    score: usize,
    balls: usize,
}

/// Returns a list of players and their scores.
///
/// This is used by new host connections to update thier display to match the current state of the
/// game.
#[get("/players")]
pub fn get_players(players: State<PlayerMap>) -> JSON<PlayersResponse> {
    let players = players.read().expect("Player map was poisoned!");
    let players = players.values()
        .map(|player| {
            PlayerData {
                id: player.id,
                username: player.username.clone(),
                score: player.score,
                balls: player.balls,
            }
        })
        .collect();

    JSON(PlayersResponse { players })
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
}

impl<'r> Responder<'r> for Error {
    fn respond(self) -> ::std::result::Result<Response<'r>, Status> {
        use rocket::response::status::Custom;

        Custom(Status::BadRequest, JSON(self)).respond()
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;
