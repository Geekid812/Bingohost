use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::TcpSocket;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

pub mod channel;
pub mod client;
pub mod config;
pub mod gameroom;
pub mod gameteam;
pub mod protocol;
pub mod requests;
pub mod server;
pub mod util;

#[tokio::main]
async fn main() {
    // Logging setup
    let subscriber = FmtSubscriber::builder()
        .with_max_level(config::LOG_LEVEL)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber");

    use config::routes::openplanet as route;
    let client = reqwest::Client::new();
    let authenticator = util::auth::Authenticator::new(
        client,
        (route::BASE.to_owned() + route::AUTH_VALIDATE)
            .parse()
            .expect("authentification route to be valid"),
    );
    let auth_arc = Arc::new(authenticator);

    let server_arc = Arc::new(Mutex::new(server::GameServer::new()));

    let socket = TcpSocket::new_v4().expect("ipv4 socket to be created");
    socket
        .set_reuseaddr(true)
        .expect("socket to be able to be reused");
    socket
        .bind(SocketAddr::from(([0, 0, 0, 0], config::TCP_LISTENING_PORT)))
        .expect("socket address to bind");
    let listener = socket.listen(1024).expect("tcp listener to be created");
    info!(
        "listener started at address {}",
        listener.local_addr().unwrap()
    );

    loop {
        let (socket, _) = listener
            .accept()
            .await
            .expect("incoming socket to be accepted");

        info!("accepted a connection");
        let auth = auth_arc.clone();
        let server = server_arc.clone();
        tokio::spawn(async move {
            let mut protocol = protocol::Protocol::new(socket, auth);
            let identity = match protocol.handshake().await {
                Some(i) => i,
                None => return,
            };
            let player = client::GameClient::new(server, protocol, identity);
            player.run().await;
        });
    }
}
