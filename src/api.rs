use broadcast::*;
use game::*;
use rocket::http::Status;
use rocket::response::*;
use rocket::State;
use rocket_contrib::JSON;
use std::sync::*;

/// The response sent back from the `/register-player` endpoint.
#[derive(Debug, Serialize)]
pub struct RegisterPlayerResponse {
    /// The `PlayerId` that was generated for the new player.
    id: PlayerId,
}

/// Generates a `PlayerId` for a new player.
// TODO: Allow players to specify a username when registering.
#[get("/register-player")]
pub fn register_player(
    player_id_generator: State<PlayerIdGenerator>,
    scoreboard: State<Mutex<Scoreboard>>,
    broadcaster: State<HostBroadcaster>,
) -> JSON<RegisterPlayerResponse>
{
    let player_id = player_id_generator.next_id();

    // Add the player to the scoreboard.
    {
        let mut scoreboard = scoreboard.lock().expect("Scoreboard mutex was poisoned");
        let old = scoreboard.insert(player_id, 0);
        assert_eq!(None, old, "Player ID was registered twice");
    }

    // Broadcast to all hosts that a new player has joined.
    broadcaster.send(HostBroadcast::PlayerRegistered(player_id));

    // Respond to the client.
    JSON(RegisterPlayerResponse {
        id: player_id,
    })
}

/// The request expected from the client for the `/feed-me` endpoint.
#[derive(Debug, Deserialize)]
pub struct FeedPlayerRequest {
    /// The `PlayerId` for the player that clicked their "Feed Me" button.
    player: PlayerId,
}

/// The response sent back from the `/feed-me` endpoint.
#[derive(Debug, Serialize)]
pub struct FeedPlayerResponse {
    score: usize,
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
    scoreboard: State<Mutex<Scoreboard>>,
    broadcaster: State<HostBroadcaster>,
) -> Result<JSON<FeedPlayerResponse>>
{
    let payload = payload.into_inner();
    let player_id = payload.player;

    // Add 1 to the player's score, returning the new score.
    let score = {
        let mut scoreboard = scoreboard.lock().expect("Scoreboard mutex was poisoned");

        // Get the player's current score, or return an `InvalidPlayer` error if it's not in
        // the scoreboard.
        let score = scoreboard
            .get_mut(&player_id)
            .ok_or(Error::InvalidPlayer(player_id))?;
        *score += 1;
        *score
    };

    // Broadcast the new score to all hosts.
    broadcaster.send(HostBroadcast::PlayerScore {
        player: player_id,
        score: score,
    });

    // Respond to the client.
    Ok(JSON(FeedPlayerResponse {
        score: score,
    }))
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