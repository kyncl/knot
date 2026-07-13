use crate::{
    ARCHIVE_PREFIX,
    knot::{Knot, file::KnotFile},
};
use std::{collections::HashMap, path::Path};

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
        }
    }

    pub fn print_visualization(&self) {
        use colored::Colorize;
        use comfy_table::{Attribute, Cell, Color as TableColor, Table};

        let total_issues = self.source_unique.len()
            + self.remote_unique.len()
            + self.conflicts.len()
            + self.archived.len();
        if total_issues == 0 {
            println!(
                "{}",
                "✔ Both directories are perfectly synchronized."
                    .green()
                    .bold()
            );
            return;
        }

        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Status").add_attribute(Attribute::Bold),
            Cell::new("File Name").add_attribute(Attribute::Bold),
            Cell::new("Resolution / Detail").add_attribute(Attribute::Bold),
        ]);

        for file in &self.source_unique {
            table.add_row(vec![
                Cell::new("+ To Upload").fg(TableColor::Green),
                Cell::new(file.name().unwrap_or("unknown".to_string())),
                Cell::new("Missing in remote directory"),
            ]);
        }

        for file in &self.remote_unique {
            table.add_row(vec![
                Cell::new("- Missing Locally").fg(TableColor::Red),
                Cell::new(file.name().unwrap_or("unknown".to_string())),
                Cell::new("Exists only in remote directory"),
            ]);
        }

        for (source, _remote) in &self.conflicts {
            table.add_row(vec![
                Cell::new("~ Conflict").fg(TableColor::Yellow),
                Cell::new(source.name().unwrap_or("unknown".to_string())),
                Cell::new("Hash mismatch (needs resolution)"),
            ]);
        }

        for file in &self.archived {
            table.add_row(vec![
                Cell::new("* Archived").fg(TableColor::Cyan),
                Cell::new(file.name().unwrap_or("unknown".to_string())),
                Cell::new("Ignored/Stored as Archive"),
            ]);
        }

        println!("{table}");
        println!(
            "\nSummary: {} pending uploads | {} missing locally | {} conflicts",
            self.source_unique.len().to_string().green(),
            self.remote_unique.len().to_string().red(),
            self.conflicts.len().to_string().yellow(),
        );
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
