use std::time::Duration;

use crate::util::version::Version;
use tracing::Level;

#[cfg(not(any(feature = "preview", feature = "live")))]
mod default {
    use super::*;

    pub const LOG_LEVEL: Level = Level::INFO;
    pub const TCP_LISTENING_PORT: u16 = 6600;
    pub const MINIMUM_CLIENT_VERSION: Version = Version(3, 0);

    pub const MAP_QUEUE_SIZE: usize = 10;
    pub const MAP_QUEUE_CAPACITY: usize = 30;
    pub const TMX_FETCH_TIMEOUT: Duration = Duration::from_secs(20);
}
#[cfg(not(any(feature = "preview", feature = "live")))]
pub use default::*;

#[cfg(feature = "preview")]
mod preview {
    use super::*;

    pub const LOG_LEVEL: Level = Level::INFO;
    pub const TCP_LISTENING_PORT: u16 = 6699;
    pub const MINIMUM_CLIENT_VERSION: Version = Version(3, 0);

    pub const MAP_QUEUE_SIZE: usize = 100;
    pub const MAP_QUEUE_CAPACITY: usize = 200;
    pub const TMX_FETCH_TIMEOUT: Duration = Duration::from_secs(20);
}
#[cfg(feature = "preview")]
pub use preview::*;

#[cfg(feature = "live")]
mod live {
    use super::*;

    pub const LOG_LEVEL: Level = Level::INFO;
    pub const TCP_LISTENING_PORT: u16 = 6900;
    pub const MINIMUM_CLIENT_VERSION: Version = Version(3, 0);

    pub const MAP_QUEUE_SIZE: usize = 100;
    pub const MAP_QUEUE_CAPACITY: usize = 200;
    pub const TMX_FETCH_TIMEOUT: Duration = Duration::from_secs(20);
}
#[cfg(feature = "live")]
pub use live::*;

pub const AUTHENTICATION_API_SECRET: &'static str = env!("AUTH_SECRET");
pub const TMX_USERAGENT: &'static str = env!("TMX_USERAGENT");

pub const TEAMS: [(&'static str, &'static str); 6] = [
    ("Red", "D84315"),
    ("Green", "8BC34A"),
    ("Blue", "0095FF"),
    ("Cyan", "4DD0E1"),
    ("Pink", "D81B60"),
    ("Yellow", "FFFF00"),
];

pub const JOINCODE_LENGTH: u32 = 6;
pub const JOINCODE_CHARS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

pub mod routes {
    pub mod openplanet {
        pub const BASE: &'static str = "https://openplanet.dev";

        pub const AUTH_VALIDATE: &'static str = "/api/auth/validate";
    }

    pub mod tmexchange {
        pub const BASE: &'static str = "https://trackmania.exchange";

        pub const MAP_SEARCH: &'static str = "/mapsearch2/search";
        pub const MAPPACK_MAPS: &'static str = "/api/mappack/get_mappack_tracks/";
    }
}
