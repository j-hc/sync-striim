use crate::common::cookie_store::CookieStore;
use crate::common::listener::ListenerID;
use crate::common::state::SharedState;
use crate::error::{AppErr, CookieStoreErr};
use axum::extract::Extension;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use tower_cookies::Cookies;

#[derive(Serialize)]
struct ListenerCreatedResp {
    id: ListenerID,
}
pub async fn new_listener(
    Extension(ext_cookies): Extension<Cookies>,
    Extension(ext_state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppErr> {
    match CookieStore::from_extensions(ext_cookies, &ext_state).await {
        Ok(_) => Err(AppErr::AlreadyListener),
        Err(AppErr::SessionErr(e)) => {
            let mut cs = match e {
                CookieStoreErr::InvalidSession(mut cs) => {
                    cs.clear_cookies();
                    cs
                }
                CookieStoreErr::Empty(cs) => cs,
            };
            let listener = ext_state.listeners.lock().await.new_listener();
            let listener_id = listener.listener_id;

            let mut rooms = ext_state.rooms.lock().await;
            let r = rooms.new_room_with_mod(listener_id);
            r.listeners.push_listener(listener);
            cs.set_cookies(Some(r.room_id), Some(listener_id)).await;
            Ok(Json(ListenerCreatedResp { id: listener_id }))
        }
        _ => unreachable!(),
    }
}
