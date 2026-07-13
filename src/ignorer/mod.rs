use crate::configuration::MainConfig;
use anyhow::Result;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::Path;

pub fn should_ignore(file: &Path, project_path: &Path, ignorer: &Gitignore) -> bool {
    if let Ok(relative_path) = file.strip_prefix(project_path) {
        let is_match = ignorer.matched(relative_path, false);
        is_match.is_ignore()
    } else {
        false
    }
}

/// It's not really specified, when gitignorer will return error,
/// so maybe rewrite functions that uses it on Option, but rn it's going to be hard requirement
fn make_git_ignore<P>(
    ignore_patterns: &[impl AsRef<str>],
    gitignore_path: Option<P>,
) -> Result<Gitignore>
where
    P: AsRef<Path>,
{
    let mut builder = GitignoreBuilder::new(".");
    if let Some(gitignore_path) = gitignore_path {
        builder.add(gitignore_path);
    }
    for pattern in ignore_patterns {
        let pattern_str = pattern.as_ref();
        if pattern_str.trim().is_empty() {
            continue;
        }
        builder.add_line(None, pattern_str)?;
    }
    Ok(builder.build()?)
}

pub fn setup_ignorer<P>(
    source_dir: P,
    config: &MainConfig,
    ignore_list: &[impl AsRef<str>],
) -> Result<Gitignore>
where
    P: AsRef<Path>,
{
    let git_ignore_fl = source_dir.as_ref().join(".gitignore");
    let git_ignore_opt = {
        if config.features.gitignore {
            Some(git_ignore_fl)
        } else {
            None
        }
    };
    let ignorer = make_git_ignore(ignore_list, git_ignore_opt)?;
    Ok(ignorer)
}
