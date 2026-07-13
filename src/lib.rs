pub mod cli;
pub mod configuration;
pub mod connection;
pub mod ignorer;
pub mod knot;
pub mod modes;
pub mod utils;

pub const APP_FOLDER: &str = ".knot";
pub const IGNORE_PREFIX_FILE: &str = "knot-ignore";
pub const ARCHIVE_PREFIX: &str = "KNOT-ARCHIVE__";
pub const BUFFER_SIZE: u32 = 16_000;
