use serde::{Deserialize, Deserializer, Serialize, de};
use strum::Display;

use crate::utils::normalize_property;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
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
#[derive(Default, Debug, Display, Serialize, Clone, Copy)]
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
impl<'de> Deserialize<'de> for UniqueBehavior {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match normalize_property(&s).as_str() {
            "archive" => Ok(UniqueBehavior::Archive),
            "mirrorsource" => Ok(UniqueBehavior::MirrorSource),
            "mirrorremote" => Ok(UniqueBehavior::MirrorRemote),
            "ask" => Ok(UniqueBehavior::Ask),
            "skip" => Ok(UniqueBehavior::Skip),
            "onlyadd" => Ok(UniqueBehavior::OnlyAdd),
            _ => Err(de::Error::unknown_variant(
                &s,
                &[
                    "Archive",
                    "MirrorSource",
                    "MirrorRemote",
                    "Ask",
                    "Skip",
                    "OnlyAdd",
                ],
            )),
        }
    }
}

/// If content of files differs, this will determinant how to handle it
#[derive(Default, Debug, Display, Serialize, Clone, Copy)]
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
    /// Skips all conflict files
    Skip,
}
impl<'de> Deserialize<'de> for ConflictBehavior {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match normalize_property(&s).as_str() {
            "newer" => Ok(ConflictBehavior::Newer),
            "older" => Ok(ConflictBehavior::Older),
            "source" => Ok(ConflictBehavior::Source),
            "remote" => Ok(ConflictBehavior::Remote),
            "ask" => Ok(ConflictBehavior::Ask),
            "skip" => Ok(ConflictBehavior::Skip),
            _ => Err(de::Error::unknown_variant(
                &s,
                &["Newer", "Older", "Source", "Remote", "Ask", "Skip"],
            )),
        }
    }
}
