use crate::common::listener::ListenerID;
use crate::common::room::{RoomID, RoomSync};
use crate::common::state::SharedState;
use crate::error::{AppErr, WSErr};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::Extension;
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct WSSession {
    room_id: RoomID,
    listener_id: ListenerID,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(shared_state): Extension<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(|ws| handle_ws(ws, shared_state))
}

async fn handle_ws(ws: WebSocket, shared_state: SharedState) -> Result<(), AppErr> {
    let (mut ws_send, mut ws_recv) = ws.split();

    let session: WSSession = loop {
        if let Some(Ok(Message::Text(msg))) = ws_recv.next().await {
            break serde_json::from_str(&msg).map_err(WSErr::SerializingErr)?;
        }
    };
    println!("session from client ws: {:?}", session);

    let is_mod = {
        let mut rooms = shared_state.rooms.lock().await;
        let room = rooms
            .get_room_by_id_mut(session.room_id)
            .ok_or(WSErr::WSSessionErr)?;
        room.is_listener_mod(session.listener_id)
    };

    if is_mod {
        while let Some(Ok(Message::Text(msg))) = ws_recv.next().await {
            if let Ok(sync_msg) = serde_json::from_str::<RoomSync>(&msg) {
                let mut rooms = shared_state.rooms.lock().await;
                let room = rooms
                    .get_room_by_id_mut(session.room_id)
                    .ok_or(WSErr::WSSessionErr)?;
                if let Some(p) = &mut room.playing {
                    p.sync(&sync_msg);
                }
                // if err, no receiver exists or all of them are dropped
                room.tx.send(sync_msg).ok();
            }
        }
    } else {
        let mut rx = {
            let mut rooms = shared_state.rooms.lock().await;
            let room = rooms
                .get_room_by_id_mut(session.room_id)
                .ok_or(WSErr::WSSessionErr)?;
            room.tx.subscribe()
        };
        while let Ok(msg) = rx.recv().await {
            let sync_msg = serde_json::to_string(&msg).map_err(WSErr::SerializingErr)?;
            if ws_send.send(Message::Text(sync_msg)).await.is_err() {
                let mut rooms = shared_state.rooms.lock().await;
                let _ = rooms
                    .get_room_by_id_mut(session.room_id)
                    .ok_or(WSErr::WSSessionErr)?
                    .listeners
                    .take_listener_by_id(session.listener_id);
                break;
            }
        }
    }
    Ok(())
}
