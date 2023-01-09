use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use serde_repr::Serialize_repr;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{error, info, warn};

use crate::config;
use crate::rest::auth::{Authenticator, PlayerIdentity, ValidationError};
use crate::util::version::Version;

pub struct Protocol {
    socket: TcpStream,
    auth: Arc<Authenticator>,
    state: Mutex<ConnectionState>,
}

impl Protocol {
    pub fn new(socket: TcpStream, auth: Arc<Authenticator>) -> Self {
        Self {
            socket: socket,
            auth,
            state: Mutex::new(ConnectionState::Closed),
        }
    }

    pub async fn handshake(&mut self) -> Option<PlayerIdentity> {
        if self.state() != ConnectionState::Closed {
            warn!(
                "Handshake was requested when the connection was not closed: {:?}",
                self.state
            );
            return None;
        }
        self.set_state(ConnectionState::Connnecting);

        // Receive opening handshake
        let handshake = match self.recv().await {
            Ok(msg) => msg,
            Err(e) => {
                println!("{:#?}", e);
                return None;
            }
        };
        let req: HandshakeRequest = match from_str(&handshake) {
            Ok(req) => req,
            Err(_) => {
                self.handshake_end(HandshakeCode::ParseError).await;
                return None;
            }
        };
        let client_version: Version = match Version::try_from(req.version) {
            Ok(ver) => ver,
            Err(_) => {
                self.handshake_end(HandshakeCode::ParseError).await;
                return None;
            }
        };

        // Client version check
        if client_version < config::MINIMUM_CLIENT_VERSION {
            self.handshake_end(HandshakeCode::IncompatibleVersion).await;
            return None;
        }

        // Authentification
        let validation_result = self.auth.validate(req.token).await;
        let identity = match validation_result {
            Ok(i) => i,
            Err(e) => {
                error!("{}", e);
                let code = match e {
                    ValidationError::RequestError(_) => HandshakeCode::AuthFailure,
                    ValidationError::BackendError(_) => HandshakeCode::AuthRefused,
                };
                self.handshake_end(code).await;
                return None;
            }
        };

        info!("Authentificated client: {:?}", identity);
        self.handshake_success(&identity).await;
        self.set_state(ConnectionState::Connected);
        return Some(identity);
    }

    fn state(&self) -> ConnectionState {
        *self.state.lock().expect("lock poisoned")
    }

    fn set_state(&self, state: ConnectionState) {
        *self.state.lock().expect("lock poisoned") = state;
    }

    async fn handshake_end(&self, code: HandshakeCode) {
        self.send_inner(
            &to_string(&HandshakeResponse {
                code,
                username: None,
            })
            .expect("json conversion to pass"),
        )
        .await
        .unwrap_or_default();
    }

    async fn handshake_success(&self, identity: &PlayerIdentity) {
        self.send_inner(
            &to_string(&HandshakeResponse {
                code: HandshakeCode::Ok,
                username: Some(identity.display_name.clone()),
            })
            .expect("json conversion to pass"),
        )
        .await
        .unwrap_or_default();
    }

    pub async fn recv(&self) -> io::Result<String> {
        let mut buf = [0; 4];
        let mut reader = &self.socket;
        reader.read_exact(&mut buf).await?;
        let size = i32::from_le_bytes(buf);
        let mut msg_buf = vec![0; size as usize];
        reader.read_exact(&mut msg_buf).await?;
        let message = String::from_utf8(msg_buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(message)
    }

    async fn send_inner(&self, message: &str) -> io::Result<()> {
        let f = (message.len() as i32).to_le_bytes();
        let msg_buf = message.as_bytes();
        let mut writer = &self.socket;
        writer.write_all(&f).await?;
        writer.write_all(&msg_buf).await?;
        Ok(())
    }

    pub async fn send(&self, message: &str) -> io::Result<()> {
        if self.state() != ConnectionState::Connected {
            return Err(io::Error::from(io::ErrorKind::NotConnected));
        }
        self.send_inner(message).await
    }

    pub fn close(&self) {
        self.set_state(ConnectionState::Closing)
    }

    pub async fn error(&self, err: &str) {
        // TODO: stub
        error!(err);
    }
}

#[derive(Deserialize)]
struct HandshakeRequest {
    version: String,
    token: String,
}

#[derive(Serialize)]
struct HandshakeResponse {
    code: HandshakeCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
}

#[derive(Serialize_repr)]
#[repr(i32)]
enum HandshakeCode {
    Ok = 0,
    ParseError = 1,
    IncompatibleVersion = 2,
    AuthFailure = 3,
    AuthRefused = 4,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ConnectionState {
    Closed,
    Connnecting,
    Connected,
    Closing,
}
