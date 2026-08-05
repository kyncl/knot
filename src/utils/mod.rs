pub mod behavior;
pub mod compression;
pub mod crypto;
pub mod formatting;
pub mod notifications;
pub mod password;
pub mod paths;
pub mod shell_complete;
pub mod toml;

use std::collections::HashSet;

/// Will remove all duplicate values in array
pub fn remove_duplicates(data: &[impl AsRef<str>]) -> Vec<String> {
    let set: HashSet<_> = data.iter().map(|s| s.as_ref().to_string()).collect();
    set.into_iter().collect()
}

/// Strips all separators ('-', '_', ' ') and convert to lowercase
pub fn normalize_property(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}
