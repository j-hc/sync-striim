pub type ListenerID = usize;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Listeners {
    total_listeners: usize,
    listeners: Vec<Listener>,
}
impl Listeners {
    pub fn push_listener(&mut self, listener: Listener) {
        self.listeners.push(listener);
        self.total_listeners += 1;
    }

    pub fn new_listener(&mut self) -> Listener {
        let l = Listener {
            listener_id: self.total_listeners,
        };
        self.total_listeners += 1;
        l
    }

    pub fn take_listener_by_id(&mut self, listener_id: ListenerID) -> Option<Listener> {
        let i = self
            .listeners
            .iter()
            .position(|l| l.listener_id == listener_id)?;
        Some(self.listeners.remove(i))
    }

    pub fn is_listener_inside(&self, listener_id: ListenerID) -> bool {
        self.listeners.iter().any(|l| l.listener_id == listener_id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Listener {
    pub listener_id: ListenerID,
}
