use rand::*;
use serde::*;
use std::collections::HashMap;
use std::sync::atomic::*;

/// Uniquely identifies a connected player.
///
/// When a new player joins, they use the `/api/register-player` endpoint to register themselves.
/// Registration generates a new `PlayerId`, which is stored inside the server and returned to the
/// client. If the client disconnects and wants to rejoin, they can continue using the previous
/// `PlayerId` to avoid losing the player's progress.
///
/// # Serialization
///
/// `PlayerId` is serialized as a string so that it'll play nice with JavaScript on the client
/// side. The IDs are meant to be treated as opaque, anyway, so sending them across the wire as
/// strings makes sense.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(usize);

impl Serialize for PlayerId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        // TODO: Can we do this without allocating a string?
        let string_id = self.0.to_string();
        serializer.serialize_str(&*string_id)
    }
}

impl Deserialize for PlayerId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
        let string_id = String::deserialize(deserializer)?;
        let id_inner = string_id.parse().map_err(de::Error::custom)?;
        Ok(PlayerId(id_inner))
    }
}

/// Generator for `PlayerId`.
///
/// Meant to be managed as application state by Rocket. Only one should ever be created, and Rocket
/// ensures that only one can ever be registered as managed state.
#[derive(Debug)]
pub struct PlayerIdGenerator(AtomicUsize);

impl PlayerIdGenerator {
    /// Creates a new `PlayerIdGenerator`.
    ///
    /// Only one `PlayerIdGenerator` should be created in the lifetime of the application. A single
    /// generator will never create duplicate IDs, but if there are multiple generators will
    /// produce the same IDs.
    pub fn new() -> PlayerIdGenerator {
        PlayerIdGenerator(ATOMIC_USIZE_INIT)
    }

    /// Generate a unique ID for a player.
    pub fn next_id(&self) -> PlayerId {
        PlayerId(self.0.fetch_add(1, Ordering::Relaxed))
    }
}

/// Generator for a player username.
///
#[derive(Debug)]
pub struct PlayerNameGenerator;

impl PlayerNameGenerator {

    pub fn generate_username(&self) -> String {

        let name_list = vec!
        [
            "Hiphopopotamus",
            "Rhymenocerous",
            "Steve",
            "Peter Potamus",
            "Mr. Wiggles",
            "Seargent Snout",
            "Calamity Hippopatamy",
            "Ringo Potamus",
            "Mrs. Basil E. Frankenhippo",
            "Harry Pottamus",
            "Hermoine Potamus",
            "Buckbeak",
            "Marie Hippolonium",
            "Hippope Francis",
            "Hippaul Potamus",
            "Danerys Mother of Hippos",
            "Darth Potamus",
            "Ann Perkopotamins!",
            "Hippopotahut",
            "Hippopotabell",
            "Combination Hippopotahut and Hippopotabell",
            "Hippchat",
            "Zippo",
            "Let 'er Rippo",
            "Have a Nice Trippo",
            "Tortilla Chippo",
            "Lastey",
            "Jean-Baptiste Emanuel Hippo",
            "Hippo Hipposon",
            "Son of Potamus",
        ];

        match thread_rng().choose(&name_list) {
            None => "".to_string(),
            Some(name) => name.to_string(),
        }
    }
}

pub type Scoreboard = HashMap<PlayerId, usize>;
pub type Usernames = HashMap<PlayerId, String>;

/// The current state for a single player.
#[derive(Debug, Serialize)]
pub struct PlayerData {
    /// A unique identifier for the player.
    pub id: PlayerId,

    /// The player's display name
    pub username: String,

    /// The player's current score.
    pub score: usize,
}
