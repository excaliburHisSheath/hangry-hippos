// Initialize the VueJS app. This is used for debug rendering.
let debug = new Vue({
    el: '#vue-root',
    data: {
        isDebugEnabled: true,
        scores: [],
    }
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

let socket = new WebSocket('ws://' + window.location.hostname + ':6769');
socket.onmessage = (event) => {
    console.log('payload: ', event);

    // TODO: Do some validation on the payload data I guess.
    let payload = JSON.parse(event.data);

    if (payload['PlayerRegistered']) {
        let player_id = payload['PlayerRegistered'];
        debug.scores.push({
            player: player_id,
            score: 0,
        });
    } else if (payload['PlayerScore']) {
        let info = payload['PlayerScore'];

        // Find the existing score object in `debug.scores`. If the player is already in the list,
        // then update their score, otherwise add them to the list.
        // TODO: We probaly shouldn't try to track score for players that haven't been registered,
        // since we wouldn't know there username.
        let existing_score = debug.scores.find(score => score.player == info.player);
        if (existing_score != null) {
            existing_score.score = info.score;
        } else {
            debug.scores.push({
                player: info.player,
                score: info.score,
            });
        }
    }
};
