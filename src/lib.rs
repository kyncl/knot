pub mod cli;
pub mod configuration;
pub mod connection;
pub mod ignorer;
pub mod knot;
pub mod modes;
pub mod tests;
pub mod utils;

pub const CONFIGURATION_FOLDER: &str = ".knot";
pub const CONFIG_FILE: &str = "config.toml";
pub const IGNORE_PATTERNS_FILE: &str = "knotignore";
pub const KNOTS_CONFIGURATION: &str = "knots.toml";
/// If you name your file with .knot.knot_tmp_, sorry but bad naming
/// (yes extra knot is just for lesser chance of rewriting wrong file)
pub const TEMPORAL_SUFFIX: &str = "_knot.knot_tmp";
pub const IGNORE_PREFIX_FILE: &str = "knot-ignore";
pub const ARCHIVE_PREFIX: &str = "KNOT-ARCHIVE__";
/// This buffer is optimized for cache
pub const BUFFER_SIZE: usize = 16_000;
/// This buffer is optimized for large transfer
pub const BUFFER_SIZE_TRANSFER: usize = 128 * 1024;
pub const STABLE_CHANNELS_PER_SESSION: usize = 16;
pub const COMPRESSION_LEVEL: i32 = 3;
pub const USER_AWAY_MSG: &str = "/ᐠ◞ ᆺ ◟マ Where did you go?";
pub const USER_CAMEBACK_MSG: &str = "/ᐠ•⩊•マ Welcome back!";
