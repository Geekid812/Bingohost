use generational_arena::{Arena, Index, Iter};
use once_cell::sync::Lazy;
use parking_lot::{Mutex, MutexGuard};

use crate::{
    gameroom::{GameRoom, RoomConfiguration},
    util::roomcode::generate_roomcode,
};

static ROOMS: Mutex<Lazy<Arena<GameRoom>>> = Mutex::new(Lazy::new(|| Arena::new()));
pub type RoomsLock<'a> = MutexGuard<'a, Lazy<Arena<GameRoom>>>;

pub struct RoomIdentifier(Index);

pub fn create_room<'a>(config: RoomConfiguration) -> (RoomIdentifier, &'a mut GameRoom) {
    let lock = ROOMS.lock();
    let mut join_code = generate_roomcode();

    while lock_find_room(&lock, join_code).is_some() {
        join_code = generate_roomcode();
    }

    let room = GameRoom::create(config, join_code);
    let id = RoomIdentifier(lock.insert(room));
    (id, lock.get_mut(id.0).expect("room exists after creation"))
}

pub fn find_room(join_code: String) -> Option<RoomIdentifier> {
    lock_find_room(&ROOMS.lock(), join_code)
}

pub fn get_room<'a>(id: RoomIdentifier) -> Option<&'a mut GameRoom> {
    ROOMS.lock().get_mut(id.0)
}

pub fn remove_room(id: RoomIdentifier) -> Option<GameRoom> {
    ROOMS.lock().remove(id.0)
}

pub fn iter_all<'a>() -> Iter<'a, GameRoom> {
    ROOMS.lock().iter()
}

fn lock_find_room(lock: &RoomsLock, join_code: String) -> Option<RoomIdentifier> {
    let result = lock
        .iter()
        .filter(|(_, room)| room.join_code() == join_code)
        .next()?;
    Some(RoomIdentifier(result.0))
}
