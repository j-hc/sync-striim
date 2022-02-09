use super::listener::Listeners;
use super::room::Rooms;

use std::sync::Arc;
use tokio::sync::Mutex;

pub type SharedState = Arc<State>;

pub struct State {
    pub rooms: Mutex<Rooms>,
    pub listeners: Mutex<Listeners>,
}
impl State {
    pub fn new() -> SharedState {
        Arc::new(Self {
            rooms: Mutex::new(Rooms::default()),
            listeners: Mutex::new(Listeners::default()),
        })
    }
}
