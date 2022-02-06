mod listener;
mod room;
mod ws;

use crate::common::state::{SharedState, State};
use crate::error::AppErr;
use axum::extract::TypedHeader;
use axum::response::Redirect;
use axum::routing::{get, get_service, post};
use axum::{AddExtensionLayer, Router};
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::{services::ServeDir, trace::TraceLayer};

pub fn get_router() -> Router {
    let shared_state: SharedState = State::new();

    Router::new()
        .nest(
            "/room",
            Router::new()
                .route("/stream", get(room::get_stream))
                .route("/playing", post(room::set_room_song))
                .route("/", get(room::get_current_room)),
        )
        .nest(
            "/static",
            get_service(ServeDir::new("server/static"))
                .handle_error(|e: std::io::Error| async move { AppErr::InternalError(e) }),
        )
        .route(
            "/",
            get(
                |TypedHeader(user_agent): TypedHeader<headers::UserAgent>| async move {
                    {
                        // yes this is how i determine if it is a browser
                        if user_agent.as_str().contains("Mozilla") {
                            Redirect::permanent("/static".parse().unwrap())
                        } else {
                            Redirect::permanent("/".parse().unwrap())
                        }
                    }
                },
            ),
        )
        .route("/connect/:new_room_id", get(room::connect_to_room))
        .route("/ws", get(ws::ws_handler))
        .route("/new_listener", get(listener::new_listener))
        .route_layer(
            ServiceBuilder::new()
                .layer(AddExtensionLayer::new(shared_state))
                .layer(CookieManagerLayer::new()),
        )
        .layer(TraceLayer::new_for_http())
}

// async fn stream_song(
//     Extension(shared_state): Extension<SharedState>,
//     cookies: CookieStore,
//     TypedHeader(b_range): TypedHeader<headers::Range>,
// ) -> Result<impl IntoResponse, AppErr> {
//     let mut state = shared_state.lock().unwrap();
//     let room = state
//         .rooms
//         .get_room_by_id_mut(cookies.room_id.unwrap())
//         .unwrap();

//     let buf_full = room.playing.as_ref().unwrap().bytes.as_slice();
//     let full_len = buf_full.len() as u64;

//     let bounded_range = b_range.iter().next().unwrap();
//     let s = match bounded_range.0 {
//         Bound::Included(b) => b,
//         Bound::Excluded(b) => (b + 1),
//         Bound::Unbounded => 0,
//     };

//     let mut e = match bounded_range.1 {
//         Bound::Included(b) => b,
//         Bound::Excluded(b) => (b + 1),
//         Bound::Unbounded => full_len,
//     };

//     if e > full_len {
//         e = full_len;
//     }

//     let buf = buf_full[s as usize..e as usize].to_vec();

//     let mut h = HeaderMap::new();
//     h.typed_insert(headers::ContentRange::bytes(s..e, Some(full_len)).unwrap());
//     h.typed_insert(headers::AcceptRanges::bytes());
//     Ok((StatusCode::PARTIAL_CONTENT, h, buf))
// }
