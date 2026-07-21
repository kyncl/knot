use crate::APP_FOLDER;
use anyhow::{Result, anyhow};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize, with::AsString};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
};

pub type CacheMap = HashMap<String, KnotFile>;
pub type ArchivedCacheMap = <CacheMap as Archive>::Archived;
pub fn load_cache(folder: &Path) -> Option<HashMap<String, KnotFile>> {
    let path = folder.join(APP_FOLDER).join("cache");
    let bytes = std::fs::read(path).ok()?;
    rkyv::from_bytes::<HashMap<String, KnotFile>, rkyv::rancor::Error>(&bytes).ok()
}

pub fn save_cache(folder: &Path, files: &[KnotFile]) -> Result<()> {
    let app_dir = folder.join(APP_FOLDER);
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)?;
    }
    let cache_path = app_dir.join("cache");
    let mut cache_map: CacheMap = HashMap::new();
    for file in files {
        if file.content_hash.is_some() {
            let path_str = file.path.to_string_lossy().into_owned();
            cache_map.insert(path_str, file.clone());
        }
    }
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&cache_map)
        .map_err(|e| anyhow!("Failed to serialize cache with rkyv: {e}"))?;
    let mut file = std::fs::File::create(cache_path)?;
    file.write_all(&bytes)?;
    file.flush()?;
    Ok(())
}

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
