use anyhow::{Result, anyhow};
use chrono::{DateTime, Duration, Utc};

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

pub fn parse_human_time(input: &str) -> Result<i64> {
    let input = input.trim().to_lowercase();
    let parts: Vec<&str> = input.split_whitespace().collect();

    let (amount, unit) = match parts.as_slice() {
        [num_str, unit, "ago"] => {
            let num: i64 = num_str.parse().map_err(|_| anyhow!("Invalid number"))?;
            (num, *unit)
        }
        _ if input.len() > 1 => {
            let (num_str, unit_str) = input.split_at(input.len() - 1);
            let num: i64 = num_str.parse().map_err(|_| anyhow!("Invalid format"))?;
            (num, unit_str)
        }
        _ => return Err(anyhow!("Unsupported format")),
    };

    let duration = match unit.trim_end_matches('s') {
        "s" | "sec" | "second" => Duration::seconds(amount),
        "m" | "min" | "minute" => Duration::minutes(amount),
        "h" | "hr" | "hour" => Duration::hours(amount),
        "d" | "day" => Duration::days(amount),
        "w" | "week" => Duration::weeks(amount),
        _ => return Err(anyhow!("Unknown unit: {unit}")),
    };
    let target_time = Utc::now() - duration;
    Ok(target_time.timestamp())
}

pub fn format_hash(hash: Option<u64>) -> String {
    match hash {
        Some(h) => {
            if h == 0 {
                "ARCHIVE".to_string()
            } else {
                format!("0x{:08x}", h as u32)
            }
        }
        None => "N/A (Dir)".to_string(),
    }
}
