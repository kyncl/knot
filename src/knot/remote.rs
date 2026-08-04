use crate::{
    configuration::loader::remote::{ProvidesRemoteConfig, RemoteKnotConfig},
    knot::Knot,
    utils::behavior::Behavior,
};

pub struct RemoteKnot {
    pub knot: Knot,
    pub behavior: Behavior,
}
impl ProvidesRemoteConfig for RemoteKnot {
    fn get_config(&self) -> RemoteKnotConfig {
        RemoteKnotConfig {
            config: self.knot.to_config(),
            behavior: self.behavior,
        }
    }
}
impl RemoteKnot {
    pub fn new(knot: Knot, behavior: Behavior) -> Self {
        Self { knot, behavior }
    }
}
