use std::sync::Arc;

use anyhow::Result;
use futures::future::try_join_all;

use crate::{
    configuration::MainConfig,
    knot::{Knot, file::KnotFile, remote::RemoteKnot},
    utils::behavior::Behavior,
};

pub struct KnotManager {
    pub source: Knot,
    pub remotes: Vec<RemoteKnot>,
}
impl KnotManager {
    pub fn new(source: Knot) -> Self {
        Self {
            source,
            remotes: vec![],
        }
    }

    pub fn add_remote(&mut self, remote: Knot, behavior: Behavior) -> &Self {
        self.remotes.push(RemoteKnot::new(remote, behavior));
        self
    }

    pub async fn update_source(&mut self, main_config: Arc<MainConfig>) -> Result<()> {
        self.source.set_folder(main_config).await
    }
    pub async fn get_source(self, main_config: Arc<MainConfig>) -> Result<Vec<KnotFile>> {
        self.source.crawl_dir(main_config).await
    }

    pub async fn update_remotes(
        remotes: &mut [RemoteKnot],
        main_config: Arc<MainConfig>,
    ) -> Result<()> {
        let knots = remotes;
        let futures = knots
            .iter_mut()
            .map(|remote| remote.knot.set_folder(main_config.clone()));
        try_join_all(futures).await?;
        Ok(())
    }
    pub async fn get_remotes(self, main_config: Arc<MainConfig>) -> Result<Vec<Vec<KnotFile>>> {
        let knots = self.remotes;
        let futures = knots
            .iter()
            .map(|remote| remote.knot.crawl_dir(main_config.clone()));
        try_join_all(futures).await
    }
}
