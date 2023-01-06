use generational_arena::Arena;
use std::sync::{Arc, RwLock};
use tracing::info;

use crate::{events::ServerEventVariant, protocol::Protocol};

pub type ChannelAddress = generational_arena::Index;

pub struct Channel {
    clients: RwLock<Vec<Arc<Protocol>>>,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(Vec::new()),
        }
    }

    pub fn subscribe(&self, client: Arc<Protocol>) {
        self.clients.write().expect("lock poisoned").push(client);
    }

    pub fn broadcast(&self, message: String) {
        info!("Broadcasting: {}", message);

        let msg = Arc::new(message);
        for client in &*self.clients.read().expect("lock poisoned") {
            tokio::spawn(Channel::send(client.clone(), msg.clone()));
        }
    }

    async fn send(protocol: Arc<Protocol>, msg: Arc<String>) {
        protocol.send(&msg).await.expect("channel broadcast failed");
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

    pub fn subscribe(&self, address: ChannelAddress, client: Arc<Protocol>) {
        if let Some(channel) = self.arena.read().expect("lock poisioned").get(address) {
            channel.subscribe(client)
        }
    }

    pub fn broadcast(&self, address: ChannelAddress, event: ServerEventVariant) {
        let message = serde_json::to_string(&event).expect("event serialization");
        self.arena
            .read()
            .expect("lock poisoned")
            .get(address)
            .expect("broadcasting in a channel that exists")
            .broadcast(message)
    }
}
