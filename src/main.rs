use std::{
    net::SocketAddr,
    sync::{atomic::AtomicU32, Arc},
};
use tokio::net::TcpSocket;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

use crate::context::ClientContext;

pub mod channel;
pub mod client;
pub mod config;
pub mod context;
pub mod events;
pub mod gamecommon;
pub mod gamedata;
pub mod gamemap;
pub mod gameroom;
pub mod gameteam;
pub mod handlers;
pub mod handshake;
pub mod reconnect;
pub mod requests;
pub mod rest;
pub mod roomlist;
pub mod socket;
pub mod sync;
pub mod util;

pub static CLIENT_COUNT: AtomicU32 = AtomicU32::new(0);

#[tokio::main]
async fn main() {
    // Logging setup
    let subscriber = FmtSubscriber::builder()
        .with_max_level(config::LOG_LEVEL)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber");

    // Auth setup
    use config::routes::openplanet as route;
    let client = reqwest::Client::new();
    let authenticator = rest::auth::Authenticator::new(
        client,
        (route::BASE.to_owned() + route::AUTH_VALIDATE)
            .parse()
            .expect("authentification route to be valid"),
    );
    let auth_arc = Arc::new(authenticator);
    if config::AUTHENTICATION_API_SECRET.is_none() {
        info!("Openplanet authentification is disabled. This is intended to be used only on unofficial servers!");
    }

    // Socket creation
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
        let (incoming, _) = listener
            .accept()
            .await
            .expect("incoming socket to be accepted");

        info!("accepted a connection");
        let auth = auth_arc.clone();
        tokio::spawn(async move {
            let (writer, reader) = socket::spawn(incoming);
            let ctx = match handshake::read_handshake(&mut reader, auth).await {
                Ok(identity) => {
                    let ctx = reconnect::recover(&identity);
                    let data = handshake::HandshakeSuccess {
                        username: identity.display_name,
                        can_reconnect: ctx.is_some(),
                    };
                    handshake::accept_socket(writer, data);
                    Some(ClientContext::new(identity, ctx, Arc::new(writer)))
                }
                Err(code) => {
                    handshake::deny_socket(writer, code);
                    None
                }
            };

            if ctx.is_none() {
                return;
            }
            client::run_loop(ctx.unwrap(), reader);
        });
    }
}
