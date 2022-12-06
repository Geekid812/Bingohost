use crate::util::version::Version;
use tracing::Level;

pub const LOG_LEVEL: Level = Level::INFO;
pub const TCP_LISTENING_PORT: u16 = 6600;
pub const MINIMUM_CLIENT_VERSION: Version = Version(3, 0);
pub const AUTHENTICATION_API_SECRET: &'static str = env!("AUTH_SECRET");

pub mod routes {
    pub mod openplanet {
        pub const BASE: &'static str = "https://openplanet.dev";

        pub const AUTH_VALIDATE: &'static str = "/api/auth/validate";
    }
}
