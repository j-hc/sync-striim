use crate::common::cookie_store::CookieStore;
use crate::common::state::SharedState;
use crate::error::AppErr;
use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct SetRoomPlayingReq {
    query: String,
}
pub async fn set_room_song(
    Json(set_room_song_req): Json<SetRoomPlayingReq>,
    cookies: CookieStore,
    Extension(shared_state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppErr> {
    let mut rooms = shared_state.rooms.lock().await;
    let room = rooms.get_room_by_id_mut(cookies.room_id)?;
    if room.is_listener_mod(cookies.listener_id) {
        let results = yt_rs::search(&set_room_song_req.query, "en-GB", "US")
            .await?;
        room.playing
            .set_stream(results[0].video_id().ok_or(AppErr::SNotFound)?, set_room_song_req.query)
            .await?;
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::FORBIDDEN)
    }
}

pub async fn get_current_room(
    cookie_store: CookieStore,
    Extension(shared_state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppErr> {
    let rooms = shared_state.rooms.lock().await;
    if let Some(r) = rooms.get_room_by_id(cookie_store.room_id) {
        Ok(Json(json!(r)))
    } else {
        Err(AppErr::InvalidRoom)
    }
}

pub async fn connect_to_room(
    Path(new_room_id): Path<String>,
    mut cookie_store: CookieStore,
    Extension(shared_state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppErr> {
    let new_room_id: usize = new_room_id.parse().map_err(|_| AppErr::InvalidRoom)?;
    let old_room_id = cookie_store.room_id;
    if new_room_id == old_room_id {
        return Err(AppErr::AlreadyInRoom);
    }
    let mut rooms = shared_state.rooms.lock().await;

    if !rooms.is_room_valid(new_room_id) {
        return Err(AppErr::InvalidRoom);
    }

    let cur_room = rooms.get_room_by_id_mut(old_room_id)?;

    let listener = cur_room
        .listeners
        .take_listener_by_id(cookie_store.listener_id)
        .ok_or(AppErr::ListenerNotFound)?;

    // dont care about the unwrap, it wont panic
    let new_room = rooms.get_room_by_id_mut(new_room_id).unwrap();

    cookie_store.set_cookies(Some(new_room.room_id), None).await;
    new_room.listeners.push_listener(listener);
    Ok(Json(json!(new_room)))
}
