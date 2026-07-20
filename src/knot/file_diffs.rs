use crate::{
    ARCHIVE_PREFIX,
    cli::visualization::file_diffs::{
        interactive::file_diff_visualization_interactive, table::file_diff_visualization_table,
    },
    knot::{Knot, file::KnotFile},
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// When comparing Vec<KnotFile> FilesDiffs can separate
/// all possible differences
pub struct FileDiffs {
    /// Files that are missing inside remote folder  
    pub source_unique: Vec<KnotFile>,

    /// Files that are missing inside source folder  
    pub remote_unique: Vec<KnotFile>,

    /// Files that has both directories, but different hash
    pub conflicts: Vec<(KnotFile, KnotFile)>,

    /// Files that has Archive prefix in name
    pub archived: Vec<KnotFile>,

    /// Path, where the source files lives
    pub source_root_path: PathBuf,

    /// Path, where the remote files lives
    pub remote_root_path: PathBuf,
}
impl FileDiffs {
    pub fn new<P>(
        source_file: &[KnotFile],
        source_path: P,
        remote_file: &[KnotFile],
        remote_path: P,
    ) -> Self
    where
        P: AsRef<Path> + Sync,
    {
        let source_path = source_path.as_ref().to_path_buf();
        let remote_path = remote_path.as_ref().to_path_buf();
        let source_map: HashMap<String, &KnotFile> = source_file
            .iter()
            .map(|f| (f.relative_path(&source_path), f))
            .collect();
        let remote_map: HashMap<String, &KnotFile> = remote_file
            .iter()
            .map(|f| (f.relative_path(&remote_path), f))
            .collect();
        let source_unique: Vec<KnotFile> = source_map
            .iter()
            .filter_map(|(path, file)| {
                if !remote_map.contains_key(path) {
                    Some((*file).clone())
                } else {
                    None
                }
            })
            .collect();
        let mut archived = vec![];
        let remote_unique: Vec<KnotFile> = remote_map
            .iter()
            .filter_map(|(path, file)| {
                let name = file.name();
                if let Ok(name) = name {
                    if !source_map.contains_key(path) {
                        if name.starts_with(ARCHIVE_PREFIX) {
                            archived.push((*file).clone());
                            None
                        } else {
                            Some((*file).clone())
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        let conflicts: Vec<(KnotFile, KnotFile)> = remote_map
            .iter()
            .filter_map(|(path, remote_file)| {
                let source_file = source_map.get(path);
                if let Some(source_file) = source_file
                    && source_file.content_hash != remote_file.content_hash
                {
                    Some(((*source_file).clone(), (*remote_file).clone()))
                } else {
                    None
                }
            })
            .collect();

        FileDiffs {
            source_unique,
            remote_unique,
            conflicts,
            archived,
            source_root_path: source_path,
            remote_root_path: remote_path,
        }
    }

    pub fn visualization(&self) {
        if let Err(error_msg) = file_diff_visualization_interactive(self) {
            println!("Couldn't create interactive visualization, because of: {error_msg}");
            file_diff_visualization_table(self);
        }
    }
}

impl Knot {
    pub fn difference(&self, remote_knot: &Knot) -> FileDiffs {
        FileDiffs::new(
            &self.files,
            &self.path,
            &remote_knot.files,
            &remote_knot.path,
        )
    }
}
