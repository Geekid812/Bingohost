use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use serde_repr::Serialize_repr;
use std::io::{Error, ErrorKind, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::config;
use crate::util::version::Version;

pub struct Protocol {
    socket: TcpStream,
    state: ConnectionState,
}

impl Protocol {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            socket,
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
        println!("waiting for handshake");
        let handshake = match self.recv().await {
            Ok(msg) => msg,
            Err(e) => {
                println!("{:#?}", e);
                return;
            }
        };
        println!("received: {}", handshake);
        let req: HandshakeRequest = match from_str(&handshake) {
            Ok(req) => req,
            Err(_) => {
                self.handshake_end(&HandshakeResponse {
                    code: HandshakeCode::ParseError,
                })
                .await;
                return;
            }
        };
        let client_version: Version = match Version::try_from(req.version) {
            Ok(ver) => ver,
            Err(_) => {
                self.handshake_end(&HandshakeResponse {
                    code: HandshakeCode::ParseError,
                })
                .await;
                return;
            }
        };

        // Client version check
        if client_version < config::MINIMUM_CLIENT_VERSION {
            self.handshake_end(&HandshakeResponse {
                code: HandshakeCode::IncompatibleVersion,
            })
            .await;
            return;
        }

        self.handshake_end(&HandshakeResponse {
            code: HandshakeCode::Ok,
        })
        .await;
        self.state = ConnectionState::Connected;
    }

    async fn handshake_end(&mut self, response: &HandshakeResponse) {
        self.send(to_string(response).expect("json conversion to pass"))
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
}

#[derive(PartialEq, Eq)]
enum ConnectionState {
    Closed,
    Connnecting,
    Connected,
    Closing,
}
