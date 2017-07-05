//! Functionality for broadcasting messages to multiple connected clients.
//!
//! Broadcasts are split between host broadcasts and player broadcasts, based on what info each
//! one needs.

use game::*;
use std::sync::*;
use std::thread;
use ws;

pub type HostBroadcaster = Arc<Broadcaster<HostBroadcast>>;
pub type PlayerBroadcaster = Arc<Broadcaster<PlayerBroadcast>>;

/// A message to be broadcast to connected host clients.
#[derive(Debug, Serialize)]
pub enum HostBroadcast {
    /// A new player has joined the game and should be added to the display.
    PlayerRegister {
        // The ID of the new player.
        id: PlayerId,

        // The player's display name.
        username: String,

        /// The starting score for the player.
        score: usize,
    },

    /// A hippo has eaten a marble from their food pile.
    HippoEat {
        /// The ID for the player whose hippo ate the marble.
        id: PlayerId,

        /// The player's total score.
        score: usize,
    },

    /// A player has lost the game and should be removed from the display.
    PlayerLose {
        id: PlayerId,
    }
}

/// A message to be broadcast to connected player clients.
#[derive(Debug, Serialize)]
pub enum PlayerBroadcast {
    /// A player has lost the game and has been removed.
    PlayerLose {
        /// The ID for the player that thi event applies to.
        id: PlayerId,

        /// The final score for the player that lost.
        score: usize,
    },
}

/// Broadcasts messages to websocket subscribers.
#[derive(Debug)]
pub struct Broadcaster<T> {
    inner: Mutex<mpsc::Sender<T>>,
}

impl<T> Broadcaster<T> {
    /// Sends `broadcast` to all websocket listeners.
    ///
    /// `send` doesn't return a `Result` because it is always possible to make a broadcast; Even
    /// there are no clients listening for the broadcast, the "broadcast" will still be vacuously
    /// successful.
    ///
    /// # Panics
    ///
    /// This method will panic if the broadcast thread (started by calling
    /// [`start_websocket_server`]) has panicked. The broadcast thread panicking indicates that
    /// there is an error in the program (likely JSON serialization of the broadcast failed), so
    /// there is no way to recover at that point, hence why `send` panics in turn.
    ///
    /// [`start_websocket_server`]: ./fn.start_websocket_server.html
    pub fn send(&self, broadcast: T) {
        self.inner
            // This function is the only place where the mutex is locked, so the mutex can only
            // get poisoned if this function panics. This function will only panic if the mutex
            // is poisoned or the broadcast thread has panicked, so either way we can't make
            // broadcasts. As such, we can safely `expect` the lock operation to succeed.
            .lock().expect("Somehow the broadcast mutex got poisoned")

            // The same goes for sending the broadcast: This will only return `Err` if the
            // broadcast thread has panicked, in which case we can't recover anyway, so we may as
            // well panic this thread while we're at it.
            .send(broadcast).expect("The broadcast thread has crashed, can no longer make broadcasts");
    }
}

/// Spawns the websocket server, returning a sender for broadcasting messages.
///
/// The websocket server is run on a separate thread listening on port 6768. This is a necessary
/// workaround because Rocket doesn't yet directly support websockets. The returned `mpsc::Sender`
/// allows for API messages to be sent from any number of threads to the websocket server, at which
/// point they will be broadcast to any connected clients.
pub fn start_server<T>(server_address: &'static str) -> Arc<Broadcaster<T>>
where
    T: 'static + ::serde::ser::Serialize + Send,
{
    // Create a sender/reciever pair so that the Rocket server can send messages to be broadcast
    // to all websocket listeners.
    let (broadcast_sender, broadcast_receiver) = mpsc::channel();
    let (socket_sender, socket_receiver) = mpsc::sync_channel(1);

    // Spawn a thread to host the websocket server. The websocket server must listen on a different
    // port than the Rocket server, so we use 6768. As websocket connections are opened, the sender
    // end of the connection is saved so that
    thread::spawn(move || {
        let mut has_sent = false;
        ws::listen(server_address, |ws_sender| {
            // Send the first `Sender` we get to the broadcast thread. It doesn't matter which
            // one gets sent, any `Sender` can be used to broadcast a message to all websockets
            // connected to the server.
            if !has_sent {
                socket_sender.send(ws_sender).expect("Failed to send socket sender to broadcast thread");
                has_sent = true;
            }

            // Create a noop handler for the connection. We don't care about listening for messages
            // or doing any advanced handling (for now, at least), so we don't need the handler to
            // to do anything.
            |_| { Ok(()) }
        }).expect("Something failed in websocket server");
    });

    // Spawn a thread to read broadcasts and multiplex them to the websockets.
    thread::spawn(move || {
        // Receive the socket that was sent from the websocket thread. We can use any `Sender` to
        // broadcast to all socket connections, so we only need to get one.
        let socket = socket_receiver.recv().expect("Failed to receive a websocket sender ;__;");

        // Pull broadcasts from the channel, serialize each one to JSON, then broadcast it to all
        // websockets connected to the server.
        for broadcast in broadcast_receiver {
            let payload = ::serde_json::to_string(&broadcast).expect("Failed to serialize payload");
            socket.broadcast(payload.clone()).unwrap();
        }
    });

    Arc::new(Broadcaster {
        inner: Mutex::new(broadcast_sender),
    })
}
