use std::net::SocketAddr;
use tokio::net::TcpSocket;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

pub mod config;
mod protocol;
pub mod util;

#[tokio::main]
async fn main() {
    // Logging setup
    let subscriber = FmtSubscriber::builder()
        .with_max_level(config::LOG_LEVEL)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber");

    let socket = TcpSocket::new_v4().expect("ipv4 socket to be created");
    socket
        .set_reuseaddr(true)
        .expect("socket to be able to be reused");
    socket
        .bind(SocketAddr::from(([0, 0, 0, 0], config::TCP_LISTENING_PORT)))
        .expect("socket address to bind");
    let listener = socket.listen(1024).expect("tcp listener to be created");
    info!("listener started at address {:#?}", listener.local_addr());

    loop {
        let (socket, _) = listener
            .accept()
            .await
            .expect("incoming socket to be accepted");

        info!("accepted a connection");
        tokio::spawn(async move {
            let mut protocol = protocol::Protocol::new(socket);
            protocol.run_loop().await;
        });
    }
}
