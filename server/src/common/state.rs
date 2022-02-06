use super::listener::Listeners;
use super::room::Rooms;

use std::sync::Arc;
use tokio::sync::Mutex;

pub type SharedState = Arc<State>;

pub struct State {
    pub rooms: Mutex<Rooms>,
    pub listeners: Mutex<Listeners>,
    pub tx: tokio::sync::broadcast::Sender<String>,
}
impl State {
    pub fn new() -> SharedState {
        let (tx, _) = tokio::sync::broadcast::channel(100);
        Arc::new(Self {
            rooms: Mutex::new(Rooms::default()),
            listeners: Mutex::new(Listeners::default()),
            tx,
        })
    }
}
