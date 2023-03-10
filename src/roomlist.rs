use std::{
    collections::{hash_map::Iter, HashMap},
    sync::{Arc, Weak},
};

use once_cell::sync::Lazy;
use parking_lot::{Mutex, MutexGuard};

use crate::{
    gameroom::{GameRoom, RoomConfiguration},
    util::roomcode::generate_roomcode,
};

static ROOMS: Mutex<Lazy<HashMap<String, OwnedRoom>>> = Mutex::new(Lazy::new(|| HashMap::new()));
pub type RoomsLock<'a> = MutexGuard<'a, Lazy<HashMap<String, OwnedRoom>>>;

pub type OwnedRoom = Arc<Mutex<GameRoom>>;
pub type SharedRoom = Weak<Mutex<GameRoom>>;

pub fn create_room<'a>(config: RoomConfiguration) -> OwnedRoom {
    let lock = ROOMS.lock();
    let mut join_code = generate_roomcode();

    while lock.get(&join_code).is_some() {
        join_code = generate_roomcode();
    }

    let room = Arc::new(Mutex::new(GameRoom::create(config, join_code)));
    lock.insert(join_code, room.clone());
    room
}

pub fn find_room(join_code: String) -> Option<OwnedRoom> {
    ROOMS.lock().get(&join_code).map(|arc| arc.clone())
}

pub fn remove_room(room: OwnedRoom) {
    let lock = ROOMS.lock();

    let to_remove = lock
        .iter()
        .filter(|(_, arc)| Arc::ptr_eq(&room, arc))
        .next();

    if let Some((code, _)) = to_remove {
        lock.remove(code);
    }
}

pub fn iter_all<'a>() -> Iter<'a, String, OwnedRoom> {
    ROOMS.lock().iter()
}
