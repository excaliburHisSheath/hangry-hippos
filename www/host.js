'use strict';

// GIVE ME TAU OR GIVE ME DEATH!
Math.TAU = 2 * Math.PI;

const TOP_SIDE = 0;
const RIGHT_SIDE = 1;
const BOTTOM_SIDE = 2;
const LEFT_SIDE = 3;

const SIDE_CSS_NAME = ['top', 'right', 'bottom', 'left'];

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
            <div class="hippo=marbles">Marbles: {{ hippo.marbles.length }}</div>
        </div>
        <div class="food-pile">
            <transition-group v-on:enter="enter" v-bind:css="false">
                <div
                    class="marble"
                    v-for="marble in hippo.marbles"
                    :key="marble.key"
                    v-bind:style="{ backgroundColor: marble.color, transform: 'translate(' + marble.x + 'px, ' + marble.y + 'px)' }"
                    v-bind:x="marble.x"
                    v-bind:y="marble.y"
                ></div>
            </transition-group>
        </div>
        <img src="assets/hippo.jpg" class="hippo-head-image" :id="hippo.player.id">
    </div>
    `,

    methods: {
        enter: function (element, done) {
            let x = Number(element.getAttribute('x'));
            let y = Number(element.getAttribute('y'));

            TweenMax.from(
                element,
                0.8,
                {
                    x: x * 3,
                    y: y * 3,
                    opacity: 0.3,
                    scale: 3.0,
                    zIndex: 10,
                    ease: Bounce.easeOut,
                    onComplete: done,
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
    } else if (payload['AddMarble']) {
        let info = payload['AddMarble'];

        // Find the hippo/player for the player that scored.
        let hippo = app.hippoMap[info.id];
        assert(hippo != null, 'Unable to find hippo for ID: ' + info.id);

        hippo.marbles.push(generateMable(info.marble));
        assert(hippo.marbles.length === info.num_marbles, 'Hippo marbles out of sync with server');
    } else if (payload['HippoEat']) {
        let info = payload['HippoEat'];

        // Find the hippo/player for the player that scored.
        let hippo = app.hippoMap[info.id];
        assert(hippo != null, 'Unable to find hippo for ID: ' + info.id);

        // Updated the local score for the player.
        hippo.player.score = info.score;

        // Remove the eaten marble.
        let index = hippo.marbles.findIndex(marble => marble.key === info.marble_key);
        assert(index >= 0, 'Eaten marble was not in food pile');
        hippo.marbles.splice(index, 1);
        assert(hippo.marbles.length === info.num_marbles, 'Hippo marbles out of sync with server');

        // Animate the hippo head to match the score increase. The direction of the chomp animation
        // depends on the side of the screen that the hippo is on, so we dynamically set the
        // animation property that moves the hippo relative to its side of the screen.
        let element = document.getElementById(hippo.player.id);
        let sideName = SIDE_CSS_NAME[hippo.side.side];

        let from = {};
        from[sideName] = 0;

        let to = { repeat: 1, yoyo: true, overwrite: 'none' };
        to[sideName] = '100px';

        TweenMax.fromTo(element, .2, from, to);
    } else if (payload['PlayerLose']) {
        let info = payload['PlayerLose'];

        // Retreive the hippo and remove it from the hippo map.
        let hippo = app.hippoMap[info.id];
        assert(delete app.hippoMap[info.id], 'Unable to remove player for id ' + info.id);

        // Remove the hippo from its side of the screen.
        let index = hippo.side.array.indexOf(hippo);
        hippo.side.array.splice(index, 1);
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
    for (let marble of player.marbles) {
        marbles.push(generateMable(marble));
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

const MARBLE_PILE_RADIUS = 40;
function generateMable(apiMarble) {
    return {
        key: apiMarble.key,
        color: apiMarble.color,
        angle: apiMarble.angle,
        radius: apiMarble.radius,
        x: Math.cos(apiMarble.angle) * apiMarble.radius * MARBLE_PILE_RADIUS,
        y: Math.sin(apiMarble.angle) * apiMarble.radius * MARBLE_PILE_RADIUS,
    }
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
