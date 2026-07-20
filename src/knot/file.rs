use anyhow::{Result, anyhow};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize, with::AsString};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct KnotFile {
    #[rkyv(with = AsString)]
    pub path: PathBuf,
    pub content_hash: Option<u64>,
    pub is_dir: bool,
    /// modified time
    pub mtime: i64,
}
impl KnotFile {
    pub fn is_symling(&self) -> bool {
        !self.is_dir && self.content_hash.is_none()
    }
    pub fn name(&self) -> Result<String> {
        let name = self.path.file_name();
        if let Some(name) = name {
            Ok(name.to_string_lossy().to_string())
        } else {
            Err(anyhow!(
                "Couldn't get file name from {}",
                self.path.to_string_lossy()
            ))
        }
    }
    pub fn new<P>(path: P, mtime: i64, is_dir: bool, content_hash: Option<u64>) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            path: path.as_ref().to_path_buf(),
            mtime,
            content_hash,
            is_dir,
        }
    }
    pub fn relative_path<P: AsRef<Path>>(&self, root: P) -> String {
        let root = root.as_ref();
        let rel = match self.path.strip_prefix(root) {
            Ok(p) => p,
            Err(_) => &self.path,
        };
        rel.components()
            .filter_map(|c| match c {
                std::path::Component::Normal(s) => s.to_str(),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("/")
            .replace("\\", "/")
    }
}
