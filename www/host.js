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

        noseGoes: {
            isActive: false,
        },

        deathMessage: {
            isActive: false,
            hippoName: null,
        }
    },
});

Vue.component('hippo-head', {
    props: ['hippo'],

    template: `
    <div class="hippo-head-root">
        <div class="hippo-text">
            <div class="name">{{ hippo.player.name }}</div>
            <div class="score">{{ hippo.player.score }}</div>
        </div>
        <div class="head-image-root" :id="hippo.player.id">
            <img src="assets/hippo.png" class="head">
            <transition name="crown">
                <img src="assets/crown.png" class="crown" v-if="hippo.hasCrown">
            </transition>
        </div>
        <transition name="poison">
            <div class="poison-pill" :id="'poison-' + hippo.player.id" v-if="hippo.isDead"></div>
        </transition>
        <transition name="bonus-text">
            <div id="bonus-text-root" v-if="hippo.wonBonus">
                <div id="bonus-text">Speedy Bonus!</div>
            </div>
        </transition>
    </div>
    `,
});

// Helpers to allow us to place hippos in clockwise order. By cycling through this array, we choose
// which sides the hippos are added to and in which proportion. We choose to add twice as many
// hippos to the top and bottom of the screen as to the sides, which looks better on wide-screen
// displays (which is mostly what we're supporting at this point).
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
        addPlayer(payload['PlayerRegister']);
    } else if (payload['HippoEat']) {
        let info = payload['HippoEat'];

        // Find the hippo/player for the player that scored.
        let hippo = app.hippoMap[info.id];
        assert(hippo != null, 'Unable to find hippo for ID: ' + info.id);

        // Updated the local score for the player.
        hippo.player.score = info.score;

        // Animate the hippo head to match the score increase. The direction of the chomp animation
        // depends on the side of the screen that the hippo is on, so we dynamically set the
        // animation property that moves the hippo relative to its side of the screen.
        let element = document.getElementById(hippo.player.id);
        let sideName = SIDE_CSS_NAME[hippo.side.side];

        let from = {};
        from[sideName] = 0;

        let to = { repeat: 1, yoyo: true, overwrite: 'none' };
        to[sideName] = '100px';

        TweenMax.fromTo(element, 0.2, from, to);
    } else if (payload['BeginNoseGoes']) {
        app.noseGoes.isActive = true;
    } else if (payload['BonusWinner']) {
        let event = payload['BonusWinner'];
        let bonusWinner = app.hippoMap[event.id];
        bonusWinner.wonBonus = true;

        // Start the animation once the element has been added to the DOM.
        Vue.nextTick(() => {
            let element = document.getElementById('bonus-text');
            TweenMax.to(element, 0.5, { scale: 1.2, repeat: -1, yoyo: true });
        });
    } else if (payload['EndNoseGoes']) {
        app.noseGoes.isActive = false;

        let info = payload['EndNoseGoes'];
        for (let loser of info.losers) {
            removePlayer(loser);
        }

        if (info.bonus_winner != null) {
            let bonusWinner = app.hippoMap[info.bonus_winner[0]];
            bonusWinner.player.score = info.bonus_winner[1];
            bonusWinner.wonBonus = false;
        }
    } else if (payload['UpdateWinner']) {
        for (let key in app.hippoMap) {
            let hippo = app.hippoMap[key];
            hippo.hasCrown = false;
        }

        let winner = payload['UpdateWinner'].id;
        app.hippoMap[winner].hasCrown = true;
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
        addPlayer(player);
    }
});

/**
 * Creates a hippo for the new player and adds it to one side of the screen.
 */
function addPlayer(player) {
    // Get the side that we're going to add the hippo to.
    let side = SIDES[currentSide];
    currentSide = (currentSide + 1) % SIDES.length;

    // Create a hippo object for the player.
    let hippo = {
        player: player,
        side: side,
        isDead: false,
        hasCrown: player.has_crown,
        wonBonus: false,
    };

    // Add the hippo to the hippo map and its side of the screen.
    assert(app.hippoMap[player.id] == null, 'Hippo already exists for ID: ' + player.id);
    app.hippoMap[player.id] = hippo;
    side.array.push(hippo);
}

/**
 * Removes a player and removes their hippo from the screen.
 */
function removePlayer(player) {
    let hippo = app.hippoMap[player];
    hippo.isDead = true;
    hippo.hasCrown = false;
    app.deathMessage.isActive = true;
    app.deathMessage.hippoName = hippo.player.name;

    // Delay actually removing the hippo until Vue has detected that we've set the `isDead` flag.
    // This ensures that that poison pill animation starts.
    Vue.nextTick(() => {
        assert(delete app.hippoMap[player], 'Unable to remove player for id ' + player);

        // Remove the hippo from its side of the screen.
        let index = hippo.side.array.indexOf(hippo);
        hippo.side.array.splice(index, 1);
    });

    setTimeout(() => { app.deathMessage.isActive = false; }, 5000);
}

// Start the attract animation.
let element = document.getElementById('attract-message');
const ATTRACT_ANIM_DURATION = 0.75;
TweenMax.to(element, ATTRACT_ANIM_DURATION, { scale: 1.3, repeat: -1, yoyo: true });
TweenMax.fromTo(
    element,
    ATTRACT_ANIM_DURATION * 2,
    { rotation: -3 },
    { rotation: 3, ease: Sine.easeInOut, repeat: -1, yoyo: true, delay: ATTRACT_ANIM_DURATION },
);

// Start the nose-goes fidget animation.
let noseGoesElement = document.getElementById('nose-goes');
TweenMax.to(noseGoesElement, 0.13, { scale: 1.1, repeat: -1, yoyo: true });
TweenMax.fromTo(
    noseGoesElement,
    0.1,
    { rotation: -2 },
    { rotation: 2, repeat: -1, yoyo: true },
);
