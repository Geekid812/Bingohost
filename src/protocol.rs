use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use serde_repr::Serialize_repr;
use std::io::{Error, ErrorKind, Result};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{error, info};

use crate::config;
use crate::util::{
    auth::{Authenticator, ValidationError},
    version::Version,
};

pub struct Protocol {
    socket: TcpStream,
    auth: Arc<Authenticator>,
    state: ConnectionState,
}

impl Protocol {
    pub fn new(socket: TcpStream, auth: Arc<Authenticator>) -> Self {
        Self {
            socket,
            auth,
            state: ConnectionState::Closed,
        }
    }

    pub async fn run_loop(&mut self) {
        self.handshake().await;

        if self.state != ConnectionState::Connected {
            return;
        }
        loop {
            if self.recv().await.is_ok() {
                println!("received a message in runloop");
            } else {
                return;
            }
        }
    }

    async fn handshake(&mut self) {
        self.state = ConnectionState::Connnecting;

        // Receive opening handshake
        let handshake = match self.recv().await {
            Ok(msg) => msg,
            Err(e) => {
                println!("{:#?}", e);
                return;
            }
        };
        let req: HandshakeRequest = match from_str(&handshake) {
            Ok(req) => req,
            Err(_) => {
                self.handshake_end(HandshakeCode::ParseError).await;
                return;
            }
        };
        let client_version: Version = match Version::try_from(req.version) {
            Ok(ver) => ver,
            Err(_) => {
                self.handshake_end(HandshakeCode::ParseError).await;
                return;
            }
        };

        // Client version check
        if client_version < config::MINIMUM_CLIENT_VERSION {
            self.handshake_end(HandshakeCode::IncompatibleVersion).await;
            return;
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
                return;
            }
        };

        info!("Authentificated client: {:?}", identity);
        self.handshake_end(HandshakeCode::Ok).await;
        self.state = ConnectionState::Connected;
    }

    async fn handshake_end(&mut self, code: HandshakeCode) {
        self.send(to_string(&HandshakeResponse { code }).expect("json conversion to pass"))
            .await
            .unwrap_or_default();
    }

    async fn recv(&mut self) -> Result<String> {
        let mut buf = [0; 4];
        self.socket.read_exact(&mut buf).await?;
        let size = i32::from_le_bytes(buf);
        let mut msg_buf = vec![0; size as usize];
        self.socket.read_exact(&mut msg_buf).await?;
        let message =
            String::from_utf8(msg_buf).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        Ok(message)
    }

    async fn send(&mut self, message: String) -> Result<()> {
        let f = (message.len() as i32).to_le_bytes();
        let msg_buf = message.as_bytes();
        self.socket.write_all(&f).await?;
        self.socket.write_all(&msg_buf).await?;
        Ok(())
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

#[derive(PartialEq, Eq)]
enum ConnectionState {
    Closed,
    Connnecting,
    Connected,
    #[allow(unused)] // For now...
    Closing,
}
