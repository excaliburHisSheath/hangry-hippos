'use strict';

function get(endpoint, onResponse, onError) {
    let request = new XMLHttpRequest();
    request.addEventListener('load', () => {
        if (request.status >= 200 && request.status < 300) {
            let response = JSON.parse(request.response);
            onResponse(response, request.status);
        } else if (onError != null) {
            onError(request.status);
        }
    });

    request.addEventListener('error', onError);
    request.open('GET', endpoint);
    request.send();
}

function post(endpoint, payload, onResponse, onError) {
    let request = new XMLHttpRequest();
    request.addEventListener('load', () => {
        if (request.status >= 200 && request.status < 300) {
            let response = JSON.parse(request.response);
            onResponse(response, request.status);
        } else if (onError != null) {
            onError(request.status);
        }
    });
    request.open('POST', endpoint);
    request.setRequestHeader('Content-Type', 'application/json;charset=UTF-8');
    request.send(JSON.stringify(payload));
}
