use ratatui::style::Color;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::{knot::file::KnotFile, utils::paths::convert_home_path};

#[derive(Debug, Clone)]
pub struct UiFileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub status: &'static str,
    pub color: Color,
    pub children: BTreeMap<String, UiFileNode>,
}

pub fn status_weight(status: &str) -> u8 {
    match status {
        "Conflict" => 3,
        "Unique" => 2,
        "Modified" => 1,
        "Synced" => 0,
        _ => 0,
    }
}

impl UiFileNode {
    pub fn build_tree<'a, I>(items: I, root_path: &Path) -> UiFileNode
    where
        I: Iterator<Item = (&'static str, &'a KnotFile, Color)>,
    {
        let root_name = root_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| root_path.to_string_lossy().to_string());

        let mut root = UiFileNode {
            path: PathBuf::from(
                convert_home_path(root_path, None).unwrap_or(root_path.display().to_string()),
            ),
            name: root_name,
            is_dir: true,
            status: "Synced",
            color: Color::DarkGray,
            children: BTreeMap::new(),
        };

        let canonical_root = root
            .path
            .canonicalize()
            .unwrap_or_else(|_| root.path.to_path_buf());

        for (status, file, color) in items {
            if !file.path.starts_with(&root.path) {
                continue;
            }

            let canonical_file_path = file
                .path
                .canonicalize()
                .unwrap_or_else(|_| file.path.clone());
            let relative = match canonical_file_path.strip_prefix(&canonical_root) {
                Ok(rel) => rel,
                Err(_) => file.path.strip_prefix(&root.path).unwrap_or(&file.path),
            };

            let components: Vec<_> = relative.components().collect();
            if components.is_empty() {
                continue;
            }

            let mut current_node = &mut root;
            let mut current_path = current_node.path.to_path_buf();

            for (i, comp) in components.iter().enumerate() {
                let comp_str = comp.as_os_str().to_string_lossy().to_string();
                current_path.push(&comp_str);

                let is_last = i == components.len() - 1;

                current_node = current_node
                    .children
                    .entry(comp_str.clone())
                    .or_insert_with(|| UiFileNode {
                        path: current_path.clone(),
                        name: comp_str,
                        is_dir: if is_last { file.is_dir } else { true },
                        status: "Synced",
                        color: Color::DarkGray,
                        children: BTreeMap::new(),
                    });

                if status_weight(status) > status_weight(current_node.status) {
                    current_node.status = status;
                    current_node.color = color;
                }

                if is_last {
                    current_node.is_dir = file.is_dir;
                }
            }
        }
        root
    }
}
