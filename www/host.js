'use strict';

// Initialize the VueJS app. This is used for debug rendering.
let debug = new Vue({
    el: '#vue-root',
    data: {
        isDebugEnabled: true,
        players: [],
    },
});

// Initialize the Phaser game. Phaser provides most of the functionality that
// we use to build the display.
let game = new Phaser.Game({
    width: 800,
    height: 600,
    parent: 'phaser-root',
    state: {
        preload: preload,
        create: create,
    },
});

let players = [];

function preload() {
    game.load.image('hippo', 'assets/hippo.jpg');
    game.load.image('background', 'assets/background.jpg');
}

function create() {
    // Create a sprite for the backdrop image and have it fill the canvas.
    let background = game.add.sprite(0, 0, 'background');
    background.width = 800;
    background.height = 600;

    // Create a sprite for the hippo head and make is relatively small.
    let sprite = game.add.sprite(80, 0, 'hippo');
    sprite.width = 100;
    sprite.height = 100;
}

// Setup a websocket to listen for updates from the server.
let socket = new WebSocket('ws://' + window.location.hostname + ':6769');
socket.onmessage = (event) => {
    // TODO: Do some validation on the payload data I guess.
    let payload = JSON.parse(event.data);

    if (payload['PlayerRegistered']) {
        let player = payload['PlayerRegistered'];
        players.push(player);
    } else if (payload['PlayerScore']) {
        let info = payload['PlayerScore'];

        // Find the existing score object in `debug.scores`. If the player is already in the list,
        // then update their score, otherwise add them to the list.
        // TODO: We probaly shouldn't try to track score for players that haven't been registered,
        // since we wouldn't know there username.
        let existing_score = players.find(player => player.id === info.id);
        assert(existing_score != null, 'Got a score for an unregistered player: ' + info.player);
        existing_score.score = info.score;
    }
};

// When we first boot up we need to get the current list of players.
get('/api/players', response => {
    players = response['players'];

    // Set the `debug.players` to be `players`, that way any changes to
    // `players` will automatically be reflected in the debug information.
    debug.players = players;

    // TODO: Create a hippo for each player.
});
