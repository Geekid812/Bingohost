use crate::util::version::Version;
use tracing::Level;

pub const LOG_LEVEL: Level = Level::INFO;
pub const TCP_LISTENING_PORT: u16 = 6600;
pub const MINIMUM_CLIENT_VERSION: Version = Version(3, 0);
pub const AUTHENTICATION_API_SECRET: &'static str = env!("AUTH_SECRET");

pub const TEAMS: [(&'static str, &'static str); 1] = [("Red", "FF0000")];

pub const JOINCODE_LENGTH: u32 = 6;
pub const JOINCODE_CHARS: [char; 26] = [
    'A', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'T', 'U', 'V', 'W',
    'X', 'Y', '3', '4', '6', '7', '9',
];

pub mod routes {
    pub mod openplanet {
        pub const BASE: &'static str = "https://openplanet.dev";

        pub const AUTH_VALIDATE: &'static str = "/api/auth/validate";
    }
}
