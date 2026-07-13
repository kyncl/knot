pub mod behavior;
pub mod compression;
pub mod paths;

use std::collections::HashSet;

/// Will remove all duplicate values in array
pub fn remove_duplicates(data: &[String]) -> Vec<String> {
    let set: HashSet<_> = data.iter().cloned().collect();
    set.into_iter().collect()
}
