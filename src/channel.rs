use futures::future::join_all;
use std::sync::Arc;

use crate::protocol::Protocol;

pub struct Channel {
    clients: Vec<Arc<Protocol>>,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, client: Arc<Protocol>) {
        self.clients.push(client);
    }

    pub async fn broadcast(&mut self, message: String) {
        async fn acquire_and_send(protocol: &mut Arc<Protocol>, msg: &str) {
            protocol.send(msg).await.expect("channel broadcast failed");
        }

        let futures = self
            .clients
            .iter_mut()
            .map(|protocol| acquire_and_send(protocol, &message));

        join_all(futures).await;
    }
}
