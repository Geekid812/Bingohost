use std::time::Duration;

use crate::util::version::Version;
use tracing::Level;

pub const LOG_LEVEL: Level = Level::INFO;
pub const TCP_LISTENING_PORT: u16 = 6600;
pub const MINIMUM_CLIENT_VERSION: Version = Version(3, 0);
pub const AUTHENTICATION_API_SECRET: &'static str = env!("AUTH_SECRET");

pub const MAP_QUEUE_SIZE: usize = 0;
pub const TMX_FETCH_TIMEOUT: Duration = Duration::from_secs(20);
pub const TMX_USERAGENT: &'static str = env!("TMX_USERAGENT");

pub const TEAMS: [(&'static str, &'static str); 3] =
    [("Red", "FF0000"), ("Green", "00FF00"), ("Blue", "0000FF")];

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
