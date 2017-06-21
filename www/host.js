'use strict';

const TOP_SIDE = 0;
const RIGHT_SIDE = 1;
const BOTTOM_SIDE = 2;
const LEFT_SIDE = 3;

// Initialize the VueJS app. This is used for app rendering.
let app = new Vue({
    el: '#vue-root',
    data: {
        // Track which hippos should be on each side of the screen.
        topHippos: [],
        rightHippos: [],
        bottomHippos: [],
        leftHippos: [],

        // Keep a map to allow us ot lookup hippos by player ID. The hippos keep a reference to
        // their player so this also allows us to lookup players by ID.
        hippoMap: {},
    },
});

Vue.component('hippo-head', {
    props: ['name', 'score', 'id'],
    template: `
    <div class="hippo-head">
        <div class="hippo-text">
            <div class="hippo-name">{{ name }}</div>
            <div class="hippo-score">Score: {{ score }}</div>
        </div>
        <img src="assets/hippo.jpg" class="hippo-head-image" :id="id">
    </div>
    `,
});

// Helpers to allow us to place hippos in clockwise order.
let sides = [
    app.topHippos,
    app.rightHippos,
    app.bottomHippos,
    app.leftHippos,
];
let currentSide = 0;

// Setup a websocket to listen for updates from the server.
let socket = new WebSocket('ws://' + window.location.hostname + ':6769');
socket.onmessage = (event) => {
    // TODO: Do some validation on the payload data I guess.
    let payload = JSON.parse(event.data);

    if (payload['PlayerRegistered']) {
        registerPlayer(payload['PlayerRegistered']);
    } else if (payload['PlayerScore']) {
        let info = payload['PlayerScore'];

        // Find the hippo/player for the player that scored.
        let hippo = app.hippoMap[info.id];
        assert(hippo != null, 'Unable to find hippo for ID: ' + info.id);

        // Updated the local score for the player.
        hippo.player.score = info.score;

        // Animate the hippo head to match the score increase. The direction of the chomp animation
        // depends on the side of the screen that the hippo is on.
        let element = document.getElementById(hippo.player.id);
        switch (hippo.side) {
            case TOP_SIDE: {
                TweenMax.fromTo(
                    element,
                    .2,
                    { top: 0 },
                    { top: '100px', repeat: 1, yoyo: true, overwrite: 'none' },
                );
            } break;

            case RIGHT_SIDE: {
                TweenMax.fromTo(
                    element,
                    .2,
                    { right: 0 },
                    { right: '100px', repeat: 1, yoyo: true, overwrite: 'none' },
                );
            } break;

            case BOTTOM_SIDE: {
                TweenMax.fromTo(
                    element,
                    .2,
                    { bottom: 0 },
                    { bottom: '100px', repeat: 1, yoyo: true, overwrite: 'none' },
                );
            } break;

            case LEFT_SIDE: {
                TweenMax.fromTo(
                    element,
                    .2,
                    { left: 0 },
                    { left: '100px', repeat: 1, yoyo: true, overwrite: 'none' },
                );
            } break;

            default: throw new Error('Unrecognized hippo side: ' + hippo.side);
        }
    }
};

// When we first boot up we need to get the current list of players.
get('/api/players', response => {
    let players = response['players'];
    assert(players != null, '/api/players response was missing a "players" member');

    // Add players to the player map, so we can find them by ID.
    for (let player of players) {
        registerPlayer(player);
    }
});

/**
 * Creates a hippo for the new player and adds it to one side of the screen.
 */
function registerPlayer(player) {
    // Create a hippo object for the player.
    let hippo = {
        player: player,
        side: currentSide,
    };

    // Add the hippo to the hippo map.
    assert(app.hippoMap[player.id] == null, 'Hippo already exists for ID: ' + player.id);
    app.hippoMap[player.id] = hippo;

    // Add the hippo to one of the sides of the screen. We cycle the sides clockwise to make sure
    // the hippos are distributed evenly.
    sides[currentSide].push(hippo);
    currentSide = (currentSide + 1) % 4;
}

// Start the attract an
let element = document.getElementById('attract-message');
const ATTRACT_ANIM_DURATION = 0.75;
TweenMax.to(element, ATTRACT_ANIM_DURATION, { scale: 1.3, repeat: -1, yoyo: true });
TweenMax.fromTo(
    element,
    ATTRACT_ANIM_DURATION * 2,
    { rotation: -3 },
    { rotation: 3, ease: Sine.easeInOut, repeat: -1, yoyo: true, delay: ATTRACT_ANIM_DURATION },
);
