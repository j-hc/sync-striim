// i have no idea tbh


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
var player;


function updateLogin() {
    console.log("login updated");
    login_frame = JSON.stringify(parseCookie(document.cookie));
}


var xhr = new XMLHttpRequest();
xhr.open('GET', '/new_listener', true);
xhr.onload = function () {
    updateLogin();
    connectWS();
    renderRoom();
};
xhr.send();


function renderRoom() {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', '/room', true);
    xhr.onload = function () {
        let r = JSON.parse(xhr.responseText);
        let p;
        if (r.playing != undefined) {
            p = r.playing.stream_url;
        } else {
            p = "nothing"
        }
        document.getElementById("playing").innerHTML = p;
        document.getElementById("listeners").innerHTML = r.listeners.listeners.map((l) => l.listener_id).join(", ")
        document.getElementById("modlistener").innerHTML = r.mod_id.toString();
        document.getElementById("cur_roomid").innerHTML = r.room_id;
        is_mod = r.mod_id == JSON.parse(login_frame).listener_id;
    };
    xhr.send();
}


const room_id_to_join = document.getElementById("roomid");
const join_btn = document.getElementById("join-room-btn");
const yt_search = document.getElementById("yt-search");
const yt_search_btn = document.getElementById("yt-search-btn");



join_btn.addEventListener("click", () => {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', '/connect/' + room_id_to_join.value, true);
    xhr.onload = function () {
        let err_elem = document.getElementById("joinerror");
        if (xhr.status == 200) {
            updateLogin();
            renderRoom();
            connectWS();
            err_elem.style.visibility = 'hidden';
        } else if (xhr.status == 418) {
            document.getElementById("joinerror").style.visibility = ''
            err_elem.innerHTML = "Already in the room";
        } else if (xhr.status == 404) {
            document.getElementById("joinerror").style.visibility = '';
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
        renderRoom();
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


function onPlayerLoad() {
    player = document.getElementById("mainPlayer");

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
}
