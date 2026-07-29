use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct FeatureConfig {
    pub caching: bool,
    pub gitignore: bool,
    pub compress: bool,
}
// Not sure if it's good to have features
// off by default, but it's not nothing to be changed later
// impl Default for FeatureConfig {
//     fn default() -> Self {
//         Self {
//             caching: true,
//             gitignore: true,
//         }
//     }
// }
