use std::path::PathBuf;

use anyhow::{Result, anyhow};

pub fn convert_home_path<T>(path: T) -> Result<String>
where
    T: Into<PathBuf> + ToString,
{
    let mut path = path.to_string();
    let home_dir = {
        let err = anyhow!("Couldn't get home directory of this system");
        let home_path = dirs::home_dir().ok_or(err)?;
        home_path.to_string_lossy().to_string()
    };

    if cfg!(unix) {
        path = path.replace("~", &home_dir);
    } else {
        path = path.replace("%USERPROFILE%", &home_dir);
    }
    Ok(path)
}
