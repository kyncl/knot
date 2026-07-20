pub mod behavior;
pub mod compression;
pub mod formatting;
pub mod paths;

use std::collections::HashSet;

/// Will remove all duplicate values in array
pub fn remove_duplicates(data: &[impl AsRef<str>]) -> Vec<String> {
    let set: HashSet<_> = data.iter().map(|s| s.as_ref().to_string()).collect();
    set.into_iter().collect()
}
