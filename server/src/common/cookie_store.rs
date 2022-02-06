use crate::error::{AppErr, CookieStoreErr};

use super::{listener::ListenerID, room::RoomID};
use crate::common::state::SharedState;
use async_trait::async_trait;
use axum::extract::{Extension, FromRequest, RequestParts};
use tower_cookies::{Cookie, Cookies};

#[derive(Debug)]
pub struct CookieStore {
    cookies: Cookies,
    pub room_id: RoomID,
    pub listener_id: ListenerID,
}
impl CookieStore {
    const COOKIE_LISTENER_ID: &'static str = "listener_id";
    const COOKIE_ROOM_ID: &'static str = "room_id";

    pub async fn from_extensions(
        ext_cookies: Cookies,
        ext_state: &SharedState,
    ) -> Result<Self, AppErr> {
        if let Some((room_id, listener_id)) = Self::get_cookies_from_ext(&ext_cookies) {
            if Self::validate_cookies(ext_state, room_id, listener_id).await {
                Ok(Self {
                    cookies: ext_cookies,
                    room_id,
                    listener_id,
                })
            } else {
                Err(CookieStoreErr::InvalidSession(Self {
                    cookies: ext_cookies,
                    room_id,
                    listener_id,
                })
                .into())
            }
        } else {
            Err(CookieStoreErr::Empty(Self {
                cookies: ext_cookies,
                room_id: 0,
                listener_id: 0,
            })
            .into())
        }
    }

    pub async fn set_cookies(&mut self, room_id: Option<RoomID>, listener_id: Option<ListenerID>) {
        if let Some(room_id) = room_id {
            let mut cookie = Cookie::new(Self::COOKIE_ROOM_ID, room_id.to_string());
            cookie.set_path("/");
            self.cookies.add(cookie);
            self.room_id = room_id;
        }
        if let Some(listener_id) = listener_id {
            let mut cookie = Cookie::new(Self::COOKIE_LISTENER_ID, listener_id.to_string());
            cookie.set_path("/");
            self.cookies.add(cookie);
            self.listener_id = listener_id;
        }
    }

    pub fn clear_cookies(&mut self) {
        self.cookies
            .remove(Cookie::new(Self::COOKIE_LISTENER_ID, ""));
        self.cookies.remove(Cookie::new(Self::COOKIE_ROOM_ID, ""));
    }

    fn get_cookies_from_ext(ext_cookies: &Cookies) -> Option<(RoomID, ListenerID)> {
        if let (Some(room_c), Some(listener_c)) = (
            ext_cookies.get(CookieStore::COOKIE_ROOM_ID),
            ext_cookies.get(CookieStore::COOKIE_LISTENER_ID),
        ) {
            if let (Ok(room_id), Ok(listener_id)) = (
                room_c.value().parse::<RoomID>(),
                listener_c.value().parse::<ListenerID>(),
            ) {
                return Some((room_id, listener_id));
            }
        }
        None
    }

    async fn validate_cookies(
        ext_state: &SharedState,
        room_id: RoomID,
        listener_id: ListenerID,
    ) -> bool {
        if let Some(room) = ext_state.rooms.lock().await.get_room_by_id(room_id) {
            if room.listeners.is_listener_inside(listener_id) {
                return true;
            }
        }
        false
    }
}

#[async_trait]
impl<T: Send> FromRequest<T> for CookieStore {
    type Rejection = AppErr;

    async fn from_request(req: &mut RequestParts<T>) -> Result<Self, Self::Rejection> {
        let Extension(ext_cookies) = Extension::<Cookies>::from_request(req)
            .await
            .map_err(|_| AppErr::ImpossibleError)?;
        let Extension(ext_state) = Extension::<SharedState>::from_request(req)
            .await
            .map_err(|_| AppErr::ImpossibleError)?;

        CookieStore::from_extensions(ext_cookies, &ext_state).await
    }
}
