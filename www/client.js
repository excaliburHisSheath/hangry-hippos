'use strict';

// Initialize Vue.js with some info I guess.
let app = new Vue({
    el: '#app-root',

    data: {
        id: null,
        username: null,
        score: null,
        numMarbles: null,
        isPlaying: true,
    },

    methods: {
        feedMe: function () {
            // If the user taps after they've lost, don't do anything.
            // TODO: Can we have Vue remove the binding when `isPlaying` is false?
            if (!this.isPlaying) {
                return;
            }

            let payload = {
                player: this.id,
            };

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

            post('api/feed-me', payload, response => {
                this.numMarbles = response.num_marbles;
            });
        },

        reload: function () {
            window.location.reload(false);
        }
    },
});

// Initialize WebSocket connetion without waiting for the DOM to be ready. I don't know if that's
// actually a good idea, but whatevs.
let socket = new WebSocket('ws://' + window.location.hostname + ':6768');
socket.onmessage = function(event) {
    // TODO: Do some kind of validation.
    let payload = JSON.parse(event.data);

    if (payload['HippoEat']) {
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
get('api/register-player', response => {
    app.id = response.id;
    app.username = response.username;
    app.score = 0;
    app.numMarbles = response.num_marbles;
});
