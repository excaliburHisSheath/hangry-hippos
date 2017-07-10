use broadcast::*;
use rand::*;
use rocket::request::FromParam;
use serde::*;
use std::collections::{ HashMap, HashSet };
use std::mem;
use std::str::FromStr;
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

impl<'a> FromParam<'a> for PlayerId {
    type Error = <usize as FromStr>::Err;

    fn from_param(param: &'a str) -> Result<PlayerId, Self::Error> {
        let inner = param.parse()?;
        Ok(PlayerId(inner))
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
    pub name: String,

    /// The player's current score.
    pub score: usize,
}

pub type PlayerMap = Arc<RwLock<HashMap<PlayerId, Player>>>;

/// Runs the main logic of the game on a separate thread.
///
/// Spawns a thread that updates game state and broadcasts updates to the players and hosts.
pub fn start_game_loop(
    players: PlayerMap,
    nose_goes: NoseGoesState,
    host_broadcaster: HostBroadcaster,
    player_broadcaster: PlayerBroadcaster,
) {
    thread::spawn(move || {
        // NOTE: These should be `const`, but you can't make a const `Duration`.
        let nose_goes_duration = Duration::from_millis(10_000);
        let nose_goes_interval = Duration::from_millis(30_000);

        loop {
            let now = Instant::now();

            let mut nose_goes = nose_goes.lock().expect("Nose-goes state was poisoned!");

            // Match the current nose-goes state, and return the new state.
            //
            // NOTE: We use `mem::replace` to move the current state out of the `Mutex`, so that we
            // can safely destructure and mutate it.
            *nose_goes = match mem::replace(&mut *nose_goes, NoseGoes::default()) {
                NoseGoes::Inactive { next_start_time } => {
                    let players = players.read().expect("Player map was poisoned!");
                    if now > next_start_time {
                        if players.len() > 1 {
                            // Add all players to the nose-goes event.
                            let remaining_players: HashSet<PlayerId> = players.keys().cloned().collect();

                            host_broadcaster.send(HostBroadcast::BeginNoseGoes {
                                duration: nose_goes_duration,
                                players: remaining_players.iter().cloned().collect(),
                            });
                            player_broadcaster.send(PlayerBroadcast::BeginNoseGoes);

                            NoseGoes::InProgress {
                                start_time: next_start_time,
                                end_time: next_start_time + nose_goes_duration,
                                remaining_players,
                            }
                        } else {
                            // There aren't enough players to run the nose-goes event. Delay until
                            // later.
                            let next_start_time = next_start_time + nose_goes_interval;
                            NoseGoes::Inactive { next_start_time }
                        }
                    } else {
                        NoseGoes::Inactive { next_start_time }
                    }
                }

                NoseGoes::InProgress { start_time, end_time, remaining_players } => {
                    if now > end_time {
                        // Pick a random player to be the loser.
                        let loser = remaining_players
                            .iter()
                            .nth(thread_rng().gen_range(0, remaining_players.len()))
                            .cloned()
                            .unwrap();

                        // Remove the player from the player map.
                        let mut players = players.write().expect("Player map was poisoned");
                        let loser_info = players.remove(&loser).expect("Loser wasn't in player map");

                        // Broadcast player loss to players and hosts.
                        host_broadcaster.send(HostBroadcast::EndNoseGoes { loser });
                        player_broadcaster.send(PlayerBroadcast::EndNoseGoes { loser });
                        player_broadcaster.send(PlayerBroadcast::PlayerLose {
                            id: loser,
                            score: loser_info.score,
                        });

                        NoseGoes::Inactive { next_start_time: end_time + nose_goes_interval }
                    } else {
                        NoseGoes::InProgress { start_time, end_time, remaining_players }
                    }
                }
            };

            thread::sleep(Duration::from_millis(100));
        }
    });
}

/// State information for nose-goes events.
#[derive(Debug)]
pub enum NoseGoes {
    Inactive {
        next_start_time: Instant,
    },

    InProgress {
        start_time: Instant,
        end_time: Instant,
        remaining_players: HashSet<PlayerId>,
    }
}

impl Default for NoseGoes {
    fn default() -> NoseGoes {
        NoseGoes::Inactive {
            next_start_time: Instant::now() + Duration::from_millis(10_000),
        }
    }
}

pub type NoseGoesState = Arc<Mutex<NoseGoes>>;
