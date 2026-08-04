use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

use crate::TEMPORAL_SUFFIX;

pub fn convert_home_path<P>(path: P, username: Option<String>) -> Result<String>
where
    P: AsRef<Path>,
{
    let mut path = path.as_ref().to_string_lossy().to_string();
    let home_dir = {
        let err = anyhow!("Couldn't get home directory of this system");
        let home_path = dirs::home_dir().ok_or(err)?;

        if let Some(username) = username {
            if path.starts_with("%USERPROFILE%") {
                format!("C:\\Users\\{username}")
            } else {
                format!("/home/{username}")
            }
        } else {
            home_path.to_string_lossy().to_string()
        }
    };

    if path.starts_with("~") {
        path = path.replacen("~", &home_dir, 1);
    }
    if !cfg!(unix) {
        path = path.replace("%USERPROFILE%", &home_dir);
    }
    Ok(path)
}

pub fn normalize_path_key<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Creates name for temporal file, which should be unique
pub fn temporal_file<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow!("Invalid file name"))?;
    let random_suffix = format!("{:016x}", rand::random::<u64>());
    let temp_name = format!(
        ".{}.{random_suffix}{TEMPORAL_SUFFIX}",
        file_name.to_string_lossy()
    );
    Ok(parent.join(temp_name))
}

pub fn relative_path<P: AsRef<Path>, Q: AsRef<Path>>(path: P, root: Q) -> String {
    let root = root.as_ref();
    let path = path.as_ref();

    let rel = match path.strip_prefix(root) {
        Ok(p) => p,
        Err(_) => path,
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
