use std::sync::Arc;
use tokio::sync::RwLock;

use futures::future::join_all;

use crate::protocol::Protocol;

pub struct Channel {
    clients: Vec<Arc<RwLock<Protocol>>>,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, client: Arc<RwLock<Protocol>>) {
        self.clients.push(client);
    }

    pub async fn broadcast(&mut self, message: String) {
        async fn acquire_and_send(protocol: &mut Arc<RwLock<Protocol>>, msg: &str) {
            protocol.write().await.send(msg);
        }

        let futures = self
            .clients
            .iter_mut()
            .map(|protocol| acquire_and_send(protocol, &message));

        join_all(futures).await;
    }
}
