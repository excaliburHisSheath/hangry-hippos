'use strict';

// Initialize the VueJS app. This is used for app rendering.
let app = new Vue({
    el: '#vue-root',
    data: {
        // Keep a list of all players, and a map for looking players up by ID.
        players: [],
        playerMap: {},

        // Track which hippos should be on each side of the screen.
        topHippos: [],
        rightHippos: [],
        bottomHippos: [],
        leftHippos: [],
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
        let player = payload['PlayerRegistered'];
        app.players.push(player);
        app.playerMap[player.id] = player;

        sides[currentSide].push({
            player: player,
        });
        currentSide = (currentSide + 1) % 4;
    } else if (payload['PlayerScore']) {
        let info = payload['PlayerScore'];

        let player = app.playerMap[info.id];
        assert(player != null, 'Unable to find player for ID: ' + info.id);

        player.score = info.score;

        // Animate the hippo head to match the score increase.
        let element = document.getElementById(player.id);
        TweenMax.fromTo(element, .2, { right: 0 }, { right: '100px', repeat: 1, yoyo: true, overwrite: 'none' });
    }
};

// When we first boot up we need to get the current list of players.
get('/api/players', response => {
    app.players = response['players'];

    // Add players to the player map, so we can find them by ID.
    for (let player of app.players) {
        app.playerMap[player.id] = player;

        sides[currentSide].push({
            player: player,
        });
        currentSide = (currentSide + 1) % 4;
    }
});
