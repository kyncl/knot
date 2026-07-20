use chrono::{DateTime, Utc};

pub fn format_relative_time(mtime_sec: i64) -> String {
    let mtime_dt = match DateTime::from_timestamp(mtime_sec, 0) {
        Some(dt) => dt,
        None => return "Unknown time".to_string(),
    };

    let now = Utc::now();
    let duration = now.signed_duration_since(mtime_dt);

    if duration.num_seconds() < 0 {
        "in the future".to_string()
    } else if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{} mins ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_days() < 30 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_days() < 365 {
        format!("{} months ago", duration.num_days() / 30)
    } else {
        format!("{} years ago", duration.num_days() / 365)
    }
}

pub fn format_hash(hash: Option<u64>) -> String {
    match hash {
        Some(h) => format!("0x{:08x}", h as u32),
        None => "N/A (Dir)".to_string(),
    }
}
