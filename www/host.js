'use strict';

// Initialize the VueJS app. This is used for app rendering.
let app = new Vue({
    el: '#vue-root',
    data: {
        players: [],
        playerMap: {},
    },
});

let players = [];

// Setup a websocket to listen for updates from the server.
let socket = new WebSocket('ws://' + window.location.hostname + ':6769');
socket.onmessage = (event) => {
    // TODO: Do some validation on the payload data I guess.
    let payload = JSON.parse(event.data);

    if (payload['PlayerRegistered']) {
        let player = payload['PlayerRegistered'];
        app.players.push(player);
        app.playerMap[player.id] = player;
    } else if (payload['PlayerScore']) {
        let info = payload['PlayerScore'];

        let player = app.playerMap[info.id];
        assert(player != null, 'Unable to find player for ID: ' + info.id);

        player.score = info.score;
    }
};

// When we first boot up we need to get the current list of players.
get('/api/players', response => {
    app.players = response['players'];

    // Add players to the player map, so we can find them by ID.
    for (let player of app.players) {
        app.playerMap[player.id] = player;
    }
});
