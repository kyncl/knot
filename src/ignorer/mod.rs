use anyhow::Result;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::Path;

pub fn should_ignore(file: &Path, project_path: &Path, ignorer: &Gitignore, is_dir: bool) -> bool {
    if let Ok(relative_path) = file.strip_prefix(project_path) {
        let is_match = ignorer.matched(relative_path, is_dir);
        is_match.is_ignore()
    } else {
        false
    }
}

pub struct NormalizedPattern {
    pub override_fmt: String,
    pub gitignore_fmt: String,
}

pub fn normalize_pattern(raw: &str) -> Option<NormalizedPattern> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    if let Some(stripped) = trimmed.strip_prefix('!') {
        let clean = stripped.trim_end_matches('/');
        let override_rule = if clean != stripped {
            format!("{clean}/**")
        } else {
            clean.to_string()
        };
        Some(NormalizedPattern {
            override_fmt: override_rule,
            gitignore_fmt: format!("!{stripped}"),
        })
    } else {
        let clean = trimmed.trim_end_matches('/');
        let override_rule = if clean != trimmed {
            format!("!{clean}/**")
        } else {
            format!("!{clean}")
        };
        Some(NormalizedPattern {
            override_fmt: override_rule,
            gitignore_fmt: trimmed.to_string(),
        })
    }
}

/// It's not really specified, when gitignorer will return error,
/// so maybe rewrite functions that uses it on Option, but rn it's going to be hard requirement
pub fn make_git_ignore<P>(root_path: P, ignore_patterns: &[impl AsRef<str>]) -> Result<Gitignore>
where
    P: AsRef<Path>,
{
    let root_path = root_path.as_ref();
    let mut builder = GitignoreBuilder::new(root_path);
    builder.add(root_path.join(".gitignore"));
    for pattern in ignore_patterns {
        if let Some(pattern) = normalize_pattern(pattern.as_ref()) {
            builder.add_line(None, &pattern.gitignore_fmt)?;
        }
    }
    Ok(builder.build()?)
}
