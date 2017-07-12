'use strict';

// Initialize Vue.js with some info I guess.
let app = new Vue({
    el: '#app-root',

    data: {
        id: null,
        hippoName: null,
        score: null,
        isPlaying: true,
        noseGoes: {
            isActive: false,
            showMarble: true,
            marbleX: 0,
            marbleY: 0,
        },
    },

    methods: {
        feedMe: function () {
            // If the user taps after they've lost, don't do anything.
            // TODO: Can we have Vue remove the binding when `isPlaying` is false?
            if (!this.isPlaying || this.noseGoes.isActive) {
                return;
            }

            post('/api/feed-me', { id: this.id }, response => {
                this.score = response.score;
            });

            // Animate the text in the center of the screen to give the user some feedback when
            // they tap.
            TweenMax.fromTo(
                '#tap-text',
                0.1,
                { scale: 1 },
                { scale: 1.2, yoyo: true, repeat: 1 },
            );
            TweenMax.fromTo(
                '#tap-text',
                0.1,
                { rotation: 0 },
                { rotation: Math.random() * 6 - 3, yoyo: true, repeat: 1 },
            );
        },

        reload: function () {
            window.location.reload(false);
        },

        poisonMarble: function () {
            this.noseGoes.showMarble = false;
            post(`/api/nose-goes/${this.id}`, {}, response => {
                if (response === 'Survived') {
                    // TODO: What do we do if the player survived?
                } else if (response === 'Died') {
                    // TODO: Do we handle the player's death now or what?
                } else {
                    console.error('Unrecognized nose-goes result:', response);
                }
            });
        }
    },
});

// Initialize WebSocket connetion without waiting for the DOM to be ready. I don't know if that's
// actually a good idea, but whatevs.
let socket = new WebSocket('ws://' + window.location.hostname + ':6768');
socket.onmessage = function(event) {
    // Ignore websocket events if the game is over or there's a nose-goes event.
    if (!app.isPlaying && !app.noseGoes.isActive) {
        return;
    }

    // TODO: Do some kind of validation.
    let payload = JSON.parse(event.data);

    if (payload === 'BeginNoseGoes') {
        app.noseGoes.isActive = true;
        app.noseGoes.showMarble = true;
        app.noseGoes.marbleX = Math.random() * 0.5 + 0.25;
        app.noseGoes.marbleY = Math.random() * 0.5 + 0.25;
    } else if (payload['EndNoseGoes']) {
        // TODO: Do some kind of animation when the player is the one who lost?
        app.noseGoes.isActive = false;
    } else if (payload['HippoEat']) {
        let event = payload['HippoEat'];
        if (event.id === app.id) {
            app.score = event.score;
            app.numMarbles = event.num_marbles;
        }
    } else if (payload['PlayerLose']) {
        let event = payload['PlayerLose'];
        if (event.id === app.id) {
            app.score = event.score;
            app.isPlaying = false;
        }
    } else {
        console.error('Unrecognized player event:', payload);
    }
};

socket.onerror = function(error) {
    console.error(error);
};

socket.onclose = function(event) {
    // TODO: Re-open the connection, if possible.
    console.error('Socket closed I guess: ', event);
};

// Register the player with the backend.
get('/api/register-player', response => {
    app.id = response.id;
    app.hippoName = response.name;
    app.score = 0;
});
