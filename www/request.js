'use strict';

function get(endpoint, onResponse) {
    let request = new XMLHttpRequest();
    request.addEventListener('load', () => {
        // TODO: Check the status code and handle any errors.
        let response = JSON.parse(request.response);
        onResponse(response);
    });
    request.open('GET', endpoint);
    request.send();
}

function post(endpoint, payload, onResponse) {
    let request = new XMLHttpRequest();
    request.addEventListener('load', () => {
        // TODO: Check the status code and handle any errors.
        let response = JSON.parse(request.response);
        onResponse(response);
    });
    request.open('POST', endpoint);
    request.setRequestHeader('Content-Type', 'application/json;charset=UTF-8');
    request.send(JSON.stringify(payload));
}
