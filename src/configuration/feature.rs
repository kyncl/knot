use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FeatureConfig {
    pub caching: bool,
    pub gitignore: bool,
    pub compress: bool,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            caching: true,
            gitignore: true,
            compress: false,
        }
    }
}
