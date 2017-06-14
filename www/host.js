let app = new Vue({
    el: '#app',
    data: {
        scores: [],
    }
});

let socket = new WebSocket('ws://' + window.location.hostname + ':6769');
socket.onmessage = (event) => {
    console.log('payload: ', event);

    // TODO: Do some validation on the payload data I guess.
    let payload = JSON.parse(event.data);

    if (payload['PlayerRegistered']) {
        let player_id = payload['PlayerRegistered'];
        app.scores.push({
            player: player_id,
            score: 0,
        });
    } else if (payload['PlayerScore']) {
        let info = payload['PlayerScore'];

        // Find the existing score object in `app.scores`. If the player is already in the list,
        // then update their score, otherwise add them to the list.
        // TODO: We probaly shouldn't try to track score for players that haven't been registered,
        // since we wouldn't know there username.
        let existing_score = app.scores.find(score => score.player == info.player);
        if (existing_score != null) {
            existing_score.score = info.score;
        } else {
            app.scores.push({
                player: info.player,
                score: info.score,
            });
        }
    }
};
