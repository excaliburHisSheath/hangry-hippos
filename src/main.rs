#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rand;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate ws;

use broadcast::*;
use game::*;
use rocket::response::*;
use std::io;
use std::path::*;
use std::sync::*;

mod api;
mod broadcast;
mod game;

/// Routes `/` to the player landing page.
///
/// Serves `www/client.html` when the player navigates directly to `localhost:6767`, that way people
/// don't have to manually go to `localhost:6767/client.html`.
#[get("/")]
fn static_serve_player() -> io::Result<NamedFile> {
    NamedFile::open(Path::new("www/client.html"))
}

/// Routes `/host` to the host landing page.
///
/// Server `www/host.html` when navigating to to `localhost:6767/host`, that way people don't have
/// to manually add the `.html` to the end.
#[get("/host")]
fn static_serve_display() -> io::Result<NamedFile> {
    NamedFile::open(Path::new("www/host.html"))
}

/// Fallback static file server route.
///
/// Any requests that aren't matched against an API route and aren't the special case `/` and `/host`
/// routes will be served as static files, returning a 404 error if the file doesn't exist.
#[get("/<file..>")]
fn static_serve(file: PathBuf) -> io::Result<NamedFile> {
    NamedFile::open(Path::new("www/").join(file))
}

fn main() {
    // Start websocket servers for broadcasting messages to host clients and player clients. The
    // resulting `Broadcaster<T>` objects are given to Rocket as managed state so that any API
    // endpoint can broadcast state changes as necessary.
    let client_broadcaster = broadcast::start_server::<PlayerBroadcast>("0.0.0.0:6768");
    let host_broadcaster = broadcast::start_server::<HostBroadcast>("0.0.0.0:6769");

    // Start the main Rocket application.
    rocket::ignite()
        .mount("/", routes![
            static_serve,
            static_serve_player,
            static_serve_display,
        ])
        .mount("/api", routes![
            api::register_player,
            api::feed_player,
            api::get_players,
        ])
        .manage(PlayerIdGenerator::new())
        .manage(host_broadcaster)
        .manage(client_broadcaster)
        .manage(Mutex::new(Scoreboard::new()))
        .manage(Mutex::new(Usernames::new()))
        .launch();
}
