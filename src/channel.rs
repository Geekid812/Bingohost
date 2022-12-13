use futures::future::join_all;

use crate::protocol::Protocol;

pub struct Channel {
    clients: Vec<Protocol>,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, client: Protocol) {
        self.clients.push(client);
    }

    pub async fn broadcast(&mut self, message: String) {
        let futures = self
            .clients
            .iter_mut()
            .map(|protocol| protocol.send(&message));

        join_all(futures).await;
    }
}
