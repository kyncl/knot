use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ExperimentalConfig {
    /// This will do the synchronization asynchronously
    /// You'll get faster synchronization with multiple Remote Knots for price of broken UI,
    /// and possible race conditioning.
    pub async_sync: bool,
}
