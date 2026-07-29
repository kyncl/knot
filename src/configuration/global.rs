use ignore::gitignore::Gitignore;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct GlobalConfig {
    #[serde(skip)]
    pub ignorer: Gitignore,
    // Will be replaced with custom ignore file
    #[serde(skip)]
    pub ignore_patterns: Vec<String>,
}
impl Default for GlobalConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalConfig {
    pub fn new() -> Self {
        Self {
            ignorer: Gitignore::empty(),
            ignore_patterns: vec![],
        }
    }

    pub fn ignorer(mut self, ignorer: Gitignore) -> Self {
        self.ignorer = ignorer;
        self
    }
}
