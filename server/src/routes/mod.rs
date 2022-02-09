mod listener;
mod room;
mod ws;

use std::ops::Bound;

use crate::common::cookie_store::CookieStore;
use crate::common::state::{SharedState, State};
use crate::error::AppErr;
use axum::extract::{Extension, TypedHeader};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, get_service, post};
use axum::{AddExtensionLayer, Router};
use headers::{HeaderMap, HeaderMapExt};
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::{services::ServeDir, trace::TraceLayer};

pub fn get_router() -> Router {
    let shared_state: SharedState = State::new();

    Router::new()
        .nest(
            "/room",
            Router::new()
                .route("/stream", get(get_stream))
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
                            Redirect::permanent("/static".parse().unwrap()).into_response()
                        } else {
                            ().into_response()
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

async fn get_stream(
    Extension(shared_state): Extension<SharedState>,
    cookies: CookieStore,
    TypedHeader(b_range): TypedHeader<headers::Range>,
) -> Result<impl IntoResponse, AppErr> {
    let mut state = shared_state.rooms.lock().await;
    let room = state.get_room_by_id_mut(cookies.room_id)?;
    let sh = room
        .playing
        .stream_helper
        .as_mut()
        .ok_or(AppErr::NothingIsPlaying)?;

    let bounded_range = b_range.iter().next().unwrap();
    let s = match bounded_range.0 {
        Bound::Included(b) => b,
        Bound::Excluded(b) => b + 1,
        Bound::Unbounded => 0,
    };

    let e = match bounded_range.1 {
        Bound::Included(b) => b,
        Bound::Excluded(b) => b + 1,
        Bound::Unbounded => sh.content_length,
    };

    let chunk = sh.striim(s as usize..e as usize).await?.to_vec();

    let mut h = HeaderMap::new();
    h.typed_insert(
        headers::ContentRange::bytes(s..s + chunk.len() as u64, Some(sh.content_length)).unwrap(),
    );
    h.typed_insert(headers::AcceptRanges::bytes());

    Ok((StatusCode::PARTIAL_CONTENT, h, chunk))
}
