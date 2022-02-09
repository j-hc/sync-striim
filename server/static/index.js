// i have no idea tbh

var xhr = new XMLHttpRequest();
xhr.open('GET', '/new_listener', true);
xhr.onload = function () {
    updateLogin();
    connectWS();
    newRoom();
};
xhr.send();


const parseCookie = str =>
    str
        .split(';')
        .map(v => v.split('='))
        .reduce((acc, v) => {
            acc[decodeURIComponent(v[0].trim())] = parseInt(decodeURIComponent(v[1].trim()));
            return acc;
        }, {});


var is_mod = true;
var login_frame;
var socket;
const player = document.getElementById("mainPlayer");
player.addEventListener('play', (event) => {
    if (is_mod) {
        let m = JSON.stringify({ "pos": player.currentTime, "kind": "Resume" });
        socket.send(m);
    }
});

player.addEventListener('pause', (event) => {
    if (is_mod) {
        let m = JSON.stringify({ "pos": player.currentTime, "kind": "Pause" });
        socket.send(m);
    }
});
console.log("loaded player callbacks");




function updateLogin() {
    console.log("login updated");
    login_frame = JSON.stringify(parseCookie(document.cookie));
}

function renderRoom(room) {
    document.getElementById("playing").innerHTML = room.playing.title;
    document.getElementById("listeners").innerHTML = room.listeners.listeners.map((l) => l.listener_id).join(", ")
    document.getElementById("modlistener").innerHTML = room.mod_id.toString();
    document.getElementById("cur_roomid").innerHTML = room.room_id;
    let parsed_login_frame = JSON.parse(login_frame);
    document.getElementById("listenerid").innerHTML = parsed_login_frame.listener_id;
    is_mod = room.mod_id == parsed_login_frame.listener_id;
}


function updateRoom() {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', '/room', true);
    xhr.onload = function () {
        let room = JSON.parse(xhr.responseText);
        renderRoom(room);
    };
    xhr.send();
}

function newRoom() {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', '/room', true);
    xhr.onload = function () {
        let room = JSON.parse(xhr.responseText);
        renderRoom(room);
        if (room.playing.is_loaded) {
            playerLoad();
            player.currentTime = room.playing.pos;
            if (room.playing.is_playing) {
                player.play();
            }
        }
    };
    xhr.send();
}


const room_id_to_join = document.getElementById("roomid");
const join_btn = document.getElementById("join-room-btn");
const yt_search = document.getElementById("yt-search");
const yt_search_btn = document.getElementById("yt-search-btn");


yt_search_btn.addEventListener("click", () => {
    var xhr = new XMLHttpRequest();
    xhr.open('POST', '/room/playing', true);
    xhr.onload = function () {
        let err_elem = document.getElementById("setplayingerror");
        if (xhr.status == 403) {
            err_elem.style.visibility = '';
            err_elem.innerHTML = "You are not the mod!";
        } else {
            err_elem.style.visibility = 'hidden';
            playerLoad();
        }
    };
    xhr.setRequestHeader("Content-Type", "application/json");
    xhr.send(JSON.stringify({ "query": yt_search.value }));
});


join_btn.addEventListener("click", () => {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', '/connect/' + room_id_to_join.value, true);
    xhr.onload = function () {
        let room = JSON.parse(xhr.responseText);

        let err_elem = document.getElementById("joinerror");
        if (xhr.status == 200) {
            updateLogin();
            renderRoom(room);
            if (room.playing.is_loaded) {
                playerLoad();
                player.currentTime = room.playing.pos;
                if (room.playing.is_playing) {
                    player.play();
                }
            }
            connectWS();
            err_elem.style.visibility = 'hidden';
        } else if (xhr.status == 418) {
            err_elem.style.visibility = ''
            err_elem.innerHTML = "Already in the room";
        } else if (xhr.status == 404) {
            err_elem.style.visibility = '';
            err_elem.innerHTML = "No room exists with that ID";
        }
    };
    xhr.send();
});


function connectWS() {
    if (location.protocol === "https:") {
        ws_protocol = "wss:";
    } else {
        ws_protocol = "ws:";
    }
    ws_uri = ws_protocol + '//' + location.host + '/ws';
    socket = new WebSocket(ws_uri);
    setInterval(() => {
        // ping the server every 40 secs
        socket.send(JSON.stringify({ event: "ping" }));
        // update the room info every 40 through HTTP request, change this to use websocket frames instead
        updateRoom();
    }, 40000);

    socket.addEventListener('open', function (event) {
        socket.send(login_frame);
    });

    socket.addEventListener('message', function (event) {
        let f = JSON.parse(event.data);
        player.currentTime = f.pos;
        if (f.kind === "Pause" && !f.paused) {
            player.pause();
        } else if (f.kind === "Resume") {
            player.play();
        }
    });
}

function playerLoad() {
    player.src = "../room/stream";
    player.load();
}

