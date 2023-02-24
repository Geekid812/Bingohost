use std::collections::HashMap;

use once_cell::sync::Lazy;
use parking_lot::RwLock;

use crate::{
    channel::{Channel, ChannelSink},
    gameteam::TeamIdentifier,
    messageable::Messageable,
    roomlist::RoomIdentifier,
};

static CHANNELS: RwLock<Lazy<HashMap<Messageable, Channel>>> =
    RwLock::new(Lazy::new(|| HashMap::new()));

pub fn create_channel(addr: Messageable) {
    CHANNELS.write().insert(addr, Channel::new());
}

pub fn remove_channel(addr: Messageable) {
    CHANNELS.write().remove(&addr);
}

pub fn subscribe(addr: Messageable, sink: ChannelSink) {
    if let Some(channel) = CHANNELS.read().get(&addr) {
        channel.subscribe(sink)
    }
}

pub fn send(addr: Messageable, message: String) {
    if let Some(channel) = CHANNELS.read().get(&addr) {
        channel.broadcast(message)
    }
}
