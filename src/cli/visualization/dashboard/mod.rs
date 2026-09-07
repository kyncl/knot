use std::path::PathBuf;

use anyhow::{Result, anyhow};

use crate::modes::setup::setup;

pub async fn show_dashborad(config_path: Option<PathBuf>, _notifications: bool) -> Result<()> {
    let (_main_config, _knots) = setup(config_path)
        .await
        .map_err(|e| anyhow!("Setup failed: {e}"))?;
    todo!("Dashboard is not done");
}
