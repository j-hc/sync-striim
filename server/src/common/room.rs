use crate::error::AppErr;

use super::listener::{ListenerID, Listeners};
use super::playing::Playing;
use serde::{Deserialize, Serialize};

pub type RoomID = usize;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum RoomSyncKind {
    Pause,
    Resume,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RoomSync {
    pub pos: f32,
    pub kind: RoomSyncKind,
}

#[derive(Serialize, Debug)]
pub struct Room {
    pub playing: Playing,
    pub room_id: RoomID,
    pub listeners: Listeners,
    pub mod_id: ListenerID,

    #[serde(skip)]
    pub tx: tokio::sync::broadcast::Sender<RoomSync>,
}
impl Room {
    pub fn is_listener_mod(&self, listener_id: ListenerID) -> bool {
        self.mod_id == listener_id
    }
}

#[derive(Serialize, Debug, Default)]
pub struct Rooms {
    pub rooms: Vec<Room>,
}

impl Rooms {
    pub fn new_room_with_mod(&mut self, mod_id: ListenerID) -> &mut Room {
        let l = self.rooms.len();
        let (tx, _) = tokio::sync::broadcast::channel(100);
        self.rooms.push(Room {
            mod_id,
            playing: Default::default(),
            room_id: l,
            listeners: Listeners::default(),
            tx,
        });
        // dont mind this unwrap it wont panic
        self.rooms.get_mut(l).unwrap()
    }

    pub fn get_room_by_id_mut(&mut self, room_id: RoomID) -> Result<&mut Room, AppErr> {
        self.rooms
            .iter_mut()
            .find(|r| r.room_id == room_id)
            .ok_or(AppErr::InvalidRoom)
    }

    pub fn get_room_by_id(&self, room_id: RoomID) -> Option<&Room> {
        self.rooms.iter().find(|r| r.room_id == room_id)
    }

    pub fn is_room_valid(&self, room_id: RoomID) -> bool {
        self.rooms.iter().any(|r| r.room_id == room_id)
    }
}
