use axum::body::BoxBody;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

use crate::common::cookie_store::CookieStore;

#[derive(Debug)]
pub enum AppErr {
    InvalidRoom,
    ListenerNotFound,
    AlreadyInRoom,
    AlreadyListener,
    BadStreamingRequest,
    ImpossibleError,
    NothingIsPlaying,
    SNotFound,
    RequesterError(reqwest::Error),
    SessionErr(CookieStoreErr),
    WebSocketErr(WSErr),
    InternalError(std::io::Error),
}
impl IntoResponse for AppErr {
    fn into_response(self) -> Response<BoxBody> {
        match self {
            Self::InvalidRoom => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "invalid room" })),
            ),
            Self::ListenerNotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "invalid listener" })),
            ),
            Self::AlreadyInRoom => (
                StatusCode::IM_A_TEAPOT,
                Json(json!({ "error": "already in room" })),
            ),
            Self::BadStreamingRequest => (
                StatusCode::OK,
                Json(json!({ "error": "bad streaming request" })),
            ),
            Self::AlreadyListener => (
                StatusCode::CONFLICT,
                Json(json!({ "error": "already listener" })),
            ),
            Self::InternalError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            ),
            Self::SessionErr(e) => (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": e.to_string() })),
            ),
            Self::ImpossibleError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "something is broken" })),
            ),
            Self::NothingIsPlaying => (
                StatusCode::OK,
                Json(json!({ "error": "nothing is currently playing" })),
            ),
            Self::RequesterError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            ),
            Self::SNotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "not found" })),
            ),
            Self::WebSocketErr(e) => (StatusCode::OK, Json(json!({ "error": e.to_string() }))),
        }
        .into_response()
    }
}


impl From<reqwest::Error> for AppErr {
    fn from(e: reqwest::Error) -> Self {
        Self::RequesterError(e)
    }
}

#[derive(Debug)]
pub enum CookieStoreErr {
    InvalidSession(CookieStore),
    Empty(CookieStore),
}
impl std::error::Error for CookieStoreErr {}
impl std::fmt::Display for CookieStoreErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err = match self {
            Self::InvalidSession(_) => "invalid session info",
            Self::Empty(_) => "empty session info",
        };
        write!(f, "session err: {}", err)
    }
}

impl From<CookieStoreErr> for AppErr {
    fn from(e: CookieStoreErr) -> Self {
        AppErr::SessionErr(e)
    }
}

#[derive(Debug)]
pub enum WSErr {
    SerializingErr(serde_json::Error),
    WSSessionErr,
}

impl From<WSErr> for AppErr {
    fn from(e: WSErr) -> Self {
        AppErr::WebSocketErr(e)
    }
}

impl std::error::Error for WSErr {}
impl std::fmt::Display for WSErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err = match self {
            WSErr::SerializingErr(e) => e.to_string(),
            WSErr::WSSessionErr => "ws session error".to_string(),
        };
        write!(f, "ws err: {}", err)
    }
}
