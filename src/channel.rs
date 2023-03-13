use generational_arena::Arena;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Weak;
use tracing::debug;

use crate::socket::{SocketAction, SocketWriter};

pub type ChannelAddress = generational_arena::Index;
static CHANNELS: Mutex<Lazy<Arena<Vec<Weak<SocketWriter>>>>> =
    Mutex::new(Lazy::new(|| Arena::new()));

pub struct Channel<'a>(&'a mut Vec<Weak<SocketWriter>>);

impl<'a> Channel<'a> {
    pub fn subscribe(&mut self, target: Weak<SocketWriter>) {
        self.0.push(target)
    }

    pub fn cleanup(&mut self) {
        let cleaned = self
            .0
            .iter()
            .filter(|writer| writer.strong_count() > 0)
            .map(Weak::clone)
            .collect();
        *self.0 = cleaned;
    }

    pub fn broadcast(&self, message: String) {
        debug!("broadcasting: {}", message);
        self.0.iter().for_each(|writer| {
            writer
                .upgrade()
                .map(|sender| sender.send(SocketAction::Message(message)));
        });
    }
}

pub fn new() -> ChannelAddress {
    CHANNELS.lock().insert(Vec::new())
}

pub fn get<'a>(address: ChannelAddress) -> Option<Channel<'a>> {
    CHANNELS.lock().get_mut(address).map(|c| Channel(c))
}

pub fn remove(address: ChannelAddress) -> bool {
    CHANNELS.lock().remove(address).is_some()
}
