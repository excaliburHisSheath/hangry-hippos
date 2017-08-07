'use strict';

// Initialize Vue.js with some info I guess.
let app = new Vue({
    el: '#app-root',

    data: {
        id: null,
        hippoName: null,
        score: null,
        isPlaying: true,
        hasCrown: false,
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
                { scale: 1.5, yoyo: true, repeat: 1 },
            );
            TweenMax.fromTo(
                '#tap-text',
                0.1,
                { rotation: 0 },
                { rotation: Math.random() * 10 - 5, yoyo: true, repeat: 1 },
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

    if (payload['BeginNoseGoes']) {
        app.noseGoes.isActive = true;
        app.noseGoes.showMarble = true;
        app.noseGoes.marbleX = Math.random() * 0.5 + 0.25;
        app.noseGoes.marbleY = Math.random() * 0.5 + 0.25;

        window.navigator.vibrate([300, 30, 500, 30, 300]);
    } else if (payload['EndNoseGoes']) {
        // TODO: Do some kind of animation when the player is the one who lost?
        let event = payload['EndNoseGoes'];
        console.log(event);
        if (event.bonus_winner != null && event.bonus_winner[0] === app.id) {
            app.score = event.bonus_winner[1];
        }

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

            localStorage.removeItem('id');
        }
    } else if (payload['UpdateWinner']) {
        let event = payload['UpdateWinner'];
        app.hasCrown = (event.id == app.id);
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

function registerPlayer() {
    // Register the player with the backend.
    get('/api/register-player', response => {
        app.id = response.id;
        app.hippoName = response.name;
        app.score = response.score;
        app.hasCrown = response.has_crown;

        localStorage.setItem('id', response.id);
    });
}

let cachedId = localStorage.getItem('id');
if (cachedId != null) {
    get(
        `/api/player/${cachedId}`,
        response => {
            app.id = response.id;
            app.hippoName = response.name;
            app.score = response.score;
            app.hasCrown = response.has_crown;
        },

        (error, status) => {
            registerPlayer();
        }
    );
} else {
    registerPlayer();
}

// Start the poison marble fidget animation.
let poisonMarbleElement = document.getElementById('poison-marble');
TweenMax.to(poisonMarbleElement, 0.13, { scale: 1.3, repeat: -1, yoyo: true });
TweenMax.fromTo(
    poisonMarbleElement,
    0.1,
    { rotation: -4 },
    { rotation: 4, repeat: -1, yoyo: true },
);

// Start the poison marble flash animation.
let poisonMarbleFlashElement = document.getElementById('poison-marble-flash');
TweenMax.fromTo(
    poisonMarbleFlashElement,
    3,
    { scale: 1, opacity: 1 },
    { scale: 3, opacity: 0, repeat: -1 },
);
