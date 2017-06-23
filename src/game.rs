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
        "Seargent Snout",
        "Calamity Hippopatamy",
        "Hippaul Potamus",
        "Ringo Potamus",
        "Mrs. Basil E. Frankenhippo",
        "Harry Pottamus",
        "Hermoine Potamus",
        "Buckbeak",
        "Hippendor",
        "Hippopuff",
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
        "Hippobrine",
        "H-1000",
        "H-1PO",
        "Hippo of Time",
        "Hippo of Winds",
        "Hippo of Hyrule",
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
        "Jean-Luc Hippicard",
        "Padmé Potamus",
        "🦏",
        "The Incredible Hippo",
        "The Amazing Spider-Hippo",
        "Notorius Hippo G",
        "The More You Hippo",
        "Hippuna Matatamus",
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

    /// The number of balls in the player's food pile.
    pub balls: usize,

    /// The time at which the player's hippo will next eat a ball.
    pub next_eat_time: Instant,
}

pub type PlayerMap = Arc<RwLock<HashMap<PlayerId, Player>>>;

pub fn start_game_loop(players: PlayerMap, host_broadcaster: HostBroadcaster) {
    thread::spawn(move || {
        loop {
            let now = Instant::now();
            {
                let mut players = players.write().expect("Hippo map was poisoned!");
                players.retain(|&id, player| {
                    // Ignore hippos that are not ready to eat.
                    if now < player.next_eat_time { return true; }


                    // Try to eat a ball. If there's one for the hippo to eat, we get a point.
                    // Otherwise, the hippo is le dead.
                    if player.balls > 0 {
                        // Eat a ball, get a point.
                        player.balls -= 1;
                        player.score += 1;

                        // Broadcast the new score to all hosts.
                        host_broadcaster.send(HostBroadcast::HippoEat {
                            id,
                            score: player.score,
                            balls: player.balls,
                        });

                        // Determine the next time the player's hippo will eat.
                        player.next_eat_time += Duration::from_millis(750);

                        true
                    } else {
                        // Notify the hosts the the player lost.
                        host_broadcaster.send(HostBroadcast::PlayerLose { id });

                        // TODO: Notify the player that they lost.

                        // Remove the player from the players map.
                        false
                    }
                });
            }

            thread::sleep(Duration::from_millis(100));
        }
    });
}
