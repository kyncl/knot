use crate::utils::paths::convert_home_path;
use inquire::{Autocomplete, autocompletion::Replacement};

#[derive(Clone, Default)]
pub struct FilePathCompleter {
    suggestion: Option<String>,
    use_files: bool,
}
impl FilePathCompleter {
    pub fn new(use_files: bool) -> Self {
        Self {
            suggestion: None,
            use_files,
        }
    }
}

impl Autocomplete for FilePathCompleter {
    fn get_suggestions(
        &mut self,
        input: &str,
    ) -> Result<Vec<String>, inquire::error::CustomUserError> {
        let input_path = convert_home_path(input, None)?;
        let mut results = Vec::new();
        let dir_read = match std::fs::read_dir(&input_path) {
            Ok(dir) => dir,
            Err(_) => {
                let path = std::path::Path::new(&input_path)
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("."));
                match std::fs::read_dir(path) {
                    Ok(optional_dir) => optional_dir,
                    Err(_) => return Ok(vec![]),
                }
            }
        };

        let home_dir = dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        for entry in dir_read.flatten() {
            let file_path = entry.path().to_string_lossy().to_string();
            if file_path.starts_with(&input_path) {
                let metadata = entry.metadata()?;
                if metadata.is_file() && !self.use_files {
                    continue;
                }

                let mut hint = entry.path().to_string_lossy().to_string();
                if metadata.is_dir() {
                    hint.push('/');
                }

                if input.starts_with('~') && !home_dir.is_empty() && hint.starts_with(&home_dir) {
                    hint = hint.replacen(&home_dir, "~", 1);
                } else if input.starts_with("%USERPROFILE%")
                    && !home_dir.is_empty()
                    && hint.starts_with(&home_dir)
                {
                    hint = hint.replacen(&home_dir, "%USERPROFILE%", 1);
                }

                results.push(hint);
            }
        }
        self.suggestion = results.first().cloned();
        Ok(results)
    }

    fn get_completion(
        &mut self,
        _input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, inquire::error::CustomUserError> {
        if let Some(selected) = highlighted_suggestion {
            Ok(Replacement::Some(selected))
        } else if let Some(suggestion) = &self.suggestion {
            Ok(Replacement::Some(suggestion.to_string()))
        } else {
            Ok(Replacement::None)
        }
    }
}
