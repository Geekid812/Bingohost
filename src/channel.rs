use generational_arena::Arena;
use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex, RwLock},
};
use tracing::{debug, info};

use crate::{
    client::{ClientId, GameClient},
    events::ServerEvent,
    protocol::Protocol,
};

pub type ChannelAddress = generational_arena::Index;

pub struct Channel {
    clients: Mutex<HashMap<ClientId, Arc<Protocol>>>,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            clients: Mutex::new(HashMap::new()),
        }
    }

    pub fn subscribe(&self, id: ClientId, client: Arc<Protocol>) {
        self.clients
            .lock()
            .expect("lock poisoned")
            .insert(id, client);
    }

    pub fn remove(&self, id: ClientId) {
        self.clients.lock().expect("lock poisoned").remove(&id);
    }

    pub fn broadcast(&self, message: String) {
        debug!("Broadcasting: {}", message);

        let msg = Arc::new(message);
        for client in self.clients.lock().expect("lock poisoned").values() {
            tokio::spawn(Channel::send(client.clone(), msg.clone()));
        }
    }

    async fn send(protocol: Arc<Protocol>, msg: Arc<String>) {
        match protocol.send(&msg).await {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != io::ErrorKind::NotConnected {
                    protocol.error(&e.to_string()).await
                };
            }
        };
    }
}

pub struct ChannelCollection {
    arena: RwLock<Arena<Channel>>,
}

impl ChannelCollection {
    pub fn new() -> Self {
        Self {
            arena: RwLock::new(Arena::new()),
        }
    }

    pub fn create_one(&self) -> ChannelAddress {
        let channel = Channel::new();
        self.arena.write().expect("lock poisoned").insert(channel)
    }

    pub fn subscribe(&self, address: ChannelAddress, client: &GameClient) {
        if let Some(channel) = self.arena.read().expect("lock poisioned").get(address) {
            channel.subscribe(client.get_id(), client.get_protocol());
        }
    }

    pub fn unsubscribe(&self, address: ChannelAddress, client: ClientId) {
        if let Some(channel) = self.arena.read().expect("lock poisioned").get(address) {
            channel.remove(client);
        }
    }

    pub fn broadcast(&self, address: ChannelAddress, event: ServerEvent) {
        let message = serde_json::to_string(&event).expect("event serialization");
        self.arena
            .read()
            .expect("lock poisoned")
            .get(address)
            .expect("broadcasting in a channel that exists")
            .broadcast(message)
    }

    pub fn remove(&self, address: ChannelAddress) {
        self.arena.write().expect("lock poisioned").remove(address);
    }
}
