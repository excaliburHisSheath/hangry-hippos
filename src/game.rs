use broadcast::*;
use rand::*;
use serde::*;
use std::collections::HashMap;
use std::sync::*;
use std::sync::atomic::*;
use std::thread;
use std::time::*;

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
#[derive(Debug, Default)]
pub struct PlayerIdGenerator(AtomicUsize);

impl PlayerIdGenerator {
    /// Generate a unique ID for a player.
    pub fn next_id(&self) -> PlayerId {
        PlayerId(self.0.fetch_add(1, Ordering::Relaxed))
    }
}

/// Generates a random username for new players.
///
/// Names are chosen from a pre-written list of guaranteed-funny names.
pub fn generate_username() -> String {
    static NAMES: &'static [&'static str] = &[
        "Hiphopopotamus",
        "Rhymenocerous",
        "Steve",
        "Peter Potamus",
        "Mr. Wiggles",
        "Mr. Bubbles",
        "Seargent Snout",
        "General Hipper",
        "Calamity Hippopatamy",
        "Hippaul Potamus",
        "Ringo Potamus",
        "Mrs. Basil E. Frankenhippo",
        "Harry Pottamus",
        "Hermoine Potamus",
        "Ron Potamus",
        "Buckbeak",
        "Hippendor",
        "Hippopuff",
        "Hippoclaw",
        "Hipperin",
        "Marie Hippolonium",
        "Hippope Francis",
        "Danerys Mother of Hippos",
        "Darth Potamus",
        "Hippo the Hutt",
        "Ann Perkopotamins!",
        "Hippopotahut",
        "Hippopotabell",
        "Combination Hippopotahut and Hippopotabell",
        "Hippchat",
        "Slackapotamus",
        "Skyppo",
        "Zippo",
        "Let 'er Rippo",
        "Have a Nice Trippo",
        "Tortilla Chippo",
        "Lastey",
        "Jean-Baptiste Emanuel Hippo",
        "Hippo Hipposon",
        "Son of Potamus",
        "Hippo V. Debs",
        "Hippolyta",
        "Wonder Potamus",
        "Bat-hippo",
        "Hippobrine",
        "H-1000",
        "H-1PO",
        "Hippo of Time",
        "Hippo of Winds",
        "Hippo of Hyrule",
        "Hippo Waker",
        "Breath of the Hippo",
        "Twilight Hippo",
        "Hippo Lippa Lub Dub",
        "Annoying Hippo",
        "Raging Hippo",
        "Raging Rhymenocerous",
        "OMG! Hippopotamus",
        "Hippo Chief 2",
        "Hippo Ex Potamus",
        "Hippo Vodello",
        "Hippo Not Stirred",
        "Hippo the Grey",
        "Hippo the White",
        "The One Hippo",
        "Frippo Hippans",
        "Hippolas",
        "Aragorpotamus",
        "Hippli",
        "Samwise Hippo",
        "Jean-Luc Hippicard",
        "Padmé Potamus",
        "Hippo Skywalker",
        "Rogue Hippo",
        "Revenge of the Hippo",
        "A New Hippo",
        "Attack of the Hippo",
        "Return of the Hippo",
        "Obi-Wan Potamus",
        "Qui-Gon Hippo",
        "The Last Hippo",
        "BB-Hippo",
        "C-3IPPO",
        "R2-HIPPO",
        "Hippo Fett",
        "Hippo Solo",
        "Leia Hipgana",
        "Captain James T. Hippo",
        "Wrath of Hippo",
        "Deep Space Hippo",
        "Leonard McHippo",
        "🦏",
        "The Incredible Hippo",
        "The Amazing Spider-Hippo",
        "Captain Amerihippo",
        "The Winter Hippo",
        "Notorius Hippo G",
        "The More You Hippo",
        "Hippuna Matatamus",
        "Rev. Potomus",
        "Hippocratic",
        "River Horse",
        "Metal Hippo?!",
        "Tactical Hippo Action",
        "Phantom Hippo",
        "Shiny and Hippo",
        "Chekhov's Hippo",
        "The Dude",
        "El Dudedrino",
        "El Hipperino",
        "His Dudeness",
        "His Potamus",
        "Mr. Lebowski",
        "Egopotamus",
        "Markihippo",
        "Mr. Hippo was my father's name",
        "Stardew Hippo",
        "Ninja Hippo",
        "Pirate Hippo",
        "Space Hippo",
        "Space Pirate Ninja Hippo",
        "Hip Hippo",
        "Flavio Hippo",
        "Marita Hippo",
        "Hippocrates",
        "Hippodrome",
        "Hippolytus",
        "Hipposter",
        "Hippias",
        "Hipparchus",
        "Hipparch",
        "Hippasus",
        "Hippo-Packard",
        "Hippovangelist",
        "Charles Hippo",
        "Hugo",
        "Herald",
        "Hippasaurus Rex",
        "Hippius Maximus",
        "Hippo California",
        "Chi-town Potamus",
    ];

    thread_rng().choose(NAMES).unwrap().to_string()
}

/// The current state for a single player.
#[derive(Debug)]
pub struct Player {
    /// A unique identifier for the player.
    pub id: PlayerId,

    /// The player's display name
    pub username: String,

    /// The player's current score.
    pub score: usize,
}

pub type PlayerMap = Arc<RwLock<HashMap<PlayerId, Player>>>;

/// Runs the main logic of the game on a separate thread.
///
/// Spawns a thread that updates game state and broadcasts updates to the players and hosts.
pub fn start_game_loop(
    players: PlayerMap,
    host_broadcaster: HostBroadcaster,
    player_broadcaster: PlayerBroadcaster,
) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(100));
        }
    });
}
