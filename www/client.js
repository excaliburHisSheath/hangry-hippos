'use strict';

// Initialize Vue.js with some info I guess.
let app = new Vue({
    el: '#app-root',

    data: {
        id: null,
        username: "",
        score: 0,
    },

    methods: {
        feedMe: function () {
            let payload = {
                player: this.id,
            };

            // Animate the text in the center of the screen to give the user some feedback when
            // they tap.
            TweenMax.fromTo(
                '#centered-text',
                0.1,
                { scale: 1 },
                { scale: 1.2, yoyo: true, repeat: 1 },
            );

            post('api/feed-me', payload, response => {
                this.score = response.score;
            });
        },
    },
});

// Initialize WebSocket connetion without waiting for the DOM to be ready. I don't know if that's
// actually a good idea, but whatevs.
let socket = new WebSocket('ws://' + window.location.hostname + ':6768');
socket.onmessage = function(event) {
    console.log('socket event: ', event);
};

socket.onerror = function(error) {
    console.error(error);
};

socket.onclose = function() {
    // TODO: Re-open the connection, if possible.
    console.log('Socket closed I guess');
};

// Register the player with the backend.
get('api/register-player', response => {
    console.log('Registration result: ', response);
    app.id = response.id;
    app.username = response.username;
});
