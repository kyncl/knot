use ignore::gitignore::Gitignore;

pub struct GlobalConfig {
    pub ignorer: Gitignore,
}
impl GlobalConfig {
    pub fn new() -> Self {
        Self {
            ignorer: Gitignore::empty(),
        }
    }

    pub fn ignorer(mut self, ignorer: Gitignore) -> Self {
        self.ignorer = ignorer;
        self
    }
}
