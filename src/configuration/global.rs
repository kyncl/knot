use ignore::gitignore::Gitignore;

pub struct GlobalConfig {
    pub ignorer: Gitignore,
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
