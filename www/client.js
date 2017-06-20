'use strict';

// Initialize Vue.js with some info I guess.
let app = new Vue({
    el: '#app-root',
    data: {
        id: null,
    }
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
get('/api/register-player', response => {
    console.log('Registration result: ', response)
    app.id = response.id;
});

// Callback for "Feed Me" button. Sends a message to the backend notifying that
// a hippo has been fed.
function feedMe() {
    let payload = {
        player: app.id,
    };
    post('/api/feed-me', payload, response => {
        console.log('feed-me response: ', response);
    });
}
