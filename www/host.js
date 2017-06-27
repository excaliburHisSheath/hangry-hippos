'use strict';

const TOP_SIDE = 0;
const RIGHT_SIDE = 1;
const BOTTOM_SIDE = 2;
const LEFT_SIDE = 3;

let marbleCounter = 0;

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
    props: ['hippo'],

    template: `
    <div class="hippo-head">
        <div class="hippo-text">
            <div class="hippo-name">{{ hippo.player.name }}</div>
            <div class="hippo-score">Score: {{ hippo.player.score }}</div>
            <div class="hippo=balls">Balls: {{ hippo.player.balls }}</div>
        </div>
        <div class="food-pile">
            <transition-group v-on:enter="enter" v-bind:css="false">
                <div class="marble" v-for="marble in hippo.marbles" :key="marble.key"></div>
            </transition-group>
        </div>
        <img src="assets/hippo.jpg" class="hippo-head-image" :id="hippo.player.id">
    </div>
    `,

    methods: {
        enter: function (element, doneProc) {
            let x = Math.random() * 100 - 50;
            let y = Math.random() * 100 - 50;
            TweenMax.fromTo(
                element,
                0.8,
                {
                    x: x * 3,
                    y: y * 3,
                    opacity: 0.3,
                    scale: 3.0,
                    zIndex: 10,
                },
                {
                    x: x,
                    y: y,
                    opacity: 1,
                    scale: 1,
                    zIndex: 0,

                    ease: Bounce.easeOut,
                    onComplete: doneProc,
                },
            );
        },
    },
});

// Helpers to allow us to place hippos in clockwise order. By cycling through this array, we choose
// which sides the hippos are added to and in which proportion. We choose to add twice as many
// hippos to the top and bottom of the screen as to the sides, which looks better on wide-screen
// displays (which is moslty what we're supporting at this point).
const SIDES = [
    { array: app.topHippos, side: TOP_SIDE },
    { array: app.topHippos, side: TOP_SIDE },
    { array: app.topHippos, side: TOP_SIDE },
    { array: app.rightHippos, side: RIGHT_SIDE },
    { array: app.bottomHippos, side: BOTTOM_SIDE },
    { array: app.bottomHippos, side: BOTTOM_SIDE },
    { array: app.bottomHippos, side: BOTTOM_SIDE },
    { array: app.leftHippos, side: LEFT_SIDE },
];
let currentSide = 0;

// Setup a websocket to listen for updates from the server.
let socket = new WebSocket('ws://' + window.location.hostname + ':6769');
socket.onmessage = (event) => {
    // TODO: Do some validation on the payload data I guess.
    let payload = JSON.parse(event.data);

    if (payload['PlayerRegister']) {
        registerPlayer(payload['PlayerRegister']);
    } else if (payload['AddBall']) {
        let info = payload['AddBall'];

        // Find the hippo/player for the player that scored.
        let hippo = app.hippoMap[info.id];
        assert(hippo != null, 'Unable to find hippo for ID: ' + info.id);

        hippo.player.balls = info.balls;

        while (hippo.marbles.length < info.balls) {
            hippo.marbles.push({
                key: marbleCounter,
            });
            marbleCounter += 1;
        }
    } else if (payload['HippoEat']) {
        let info = payload['HippoEat'];

        // Find the hippo/player for the player that scored.
        let hippo = app.hippoMap[info.id];
        assert(hippo != null, 'Unable to find hippo for ID: ' + info.id);

        // Updated the local score for the player.
        hippo.player.score = info.score;
        hippo.player.balls = info.balls;

        while (hippo.marbles.length > info.balls) {
            hippo.marbles.splice(0, 1);
        }

        // Animate the hippo head to match the score increase. The direction of the chomp animation
        // depends on the side of the screen that the hippo is on.
        let element = document.getElementById(hippo.player.id);
        switch (hippo.side.side) {
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

            default: throw new Error('Unrecognized hippo side: ' + hippo.side.side);
        }
    } else if (payload['PlayerLose']) {
        let info = payload['PlayerLose'];

        // Retreive the hippo and remove it from the hippo map.
        let hippo = app.hippoMap[info.id];
        assert(delete app.hippoMap[info.id], 'Unable to remove player for id ' + info.id);

        // Remove the hippo from its side of the screen.
        let index = hippo.side.array.indexOf(hippo);
    } else {
        console.error('Unrecognized host event:', payload);
    }
};
socket.onclose = (event) => {
    console.error('Socket closed:', event);
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
    // Get the side that we're going to add the hippo to.
    let side = SIDES[currentSide];
    currentSide = (currentSide + 1) % SIDES.length;

    let marbles = [];
    for (let count = 0; count < player.balls; count += 1) {
        marbles.push({
            key: marbleCounter,
        });
        marbleCounter += 1;
    }

    // Create a hippo object for the player.
    let hippo = {
        player: player,
        side: side,
        marbles: marbles,
    };

    // Add the hippo to the hippo map and its side of the screen.
    assert(app.hippoMap[player.id] == null, 'Hippo already exists for ID: ' + player.id);
    app.hippoMap[player.id] = hippo;
    side.array.push(hippo);
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
