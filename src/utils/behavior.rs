use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Serialize, Deserialize, Debug)]
pub struct Behavior {
    pub uniques: UniqueBehavior,
    pub conflicts: ConflictBehavior,
}
impl Behavior {
    pub fn new(conflicts: ConflictBehavior, uniques: UniqueBehavior) -> Self {
        Self { conflicts, uniques }
    }
}

/// If Knot finds unique files, this will determinant how to handle it
#[derive(Default, Debug, Display, Serialize, Deserialize)]
pub enum UniqueBehavior {
    #[default]
    Archive,
    /// Unique SOURCE files will be add to REMOTE
    /// and REMOTE uniques will be removed
    MirrorSource,
    /// Unique REMOTE files will be add to SOURCE
    /// and SOURCE uniques will be removed
    MirrorRemote,
    Ask,
    Skip,
    /// Will never remove any file, just makes unique in both folders
    OnlyAdd,
}

/// If content of files differs, this will determinant how to handle it
#[derive(Default, Debug, Display, Serialize, Deserialize)]
pub enum ConflictBehavior {
    /// If you want to always prioritize newer based on modified time
    #[default]
    Newer,
    /// If you want to always prioritize older based on modified time
    Older,
    /// If you want to always prioritize source
    Source,
    /// If you want to always prioritize remote
    Remote,
    /// Ask user, how to handle it
    Ask,
    /// Skip all files
    Skip,
}
