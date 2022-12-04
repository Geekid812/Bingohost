use crate::util::version::Version;
use tracing::Level;

pub const LOG_LEVEL: Level = Level::TRACE;
pub const TCP_LISTENING_PORT: u16 = 6600;
pub const MINIMUM_CLIENT_VERSION: Version = Version(3, 0);
