use crate::{
    cli::visualization::{
        dashboard::file_node::{UiFileNode, status_weight},
        resolver::FocusState,
    },
    knot::remote::RemoteKnot,
};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::style::Color;
use std::{collections::HashSet, path::PathBuf};

#[derive(PartialEq, Eq, Default)]
pub enum SelectedPart {
    #[default]
    Tabs,
    Source,
    Remote,
}

#[derive(Debug, Clone)]
pub struct UiRow {
    pub depth: usize,
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub status: &'static str,
    pub color: Color,
}

pub struct DashEventHandler {
    pub end: bool,
    pub sync: bool,
    pub redraw: bool,
    pub is_focused: bool,
    pub focus_state: FocusState,
    pub highlighted_part: SelectedPart,
    pub selected_tab: usize,
    pub expanded_dirs: HashSet<PathBuf>,
    pub source_idx: usize,
    pub remote_idx: usize,
    pub source_visible_rows: Vec<UiRow>,
    pub remote_visible_rows: Vec<UiRow>,
}
impl Default for DashEventHandler {
    fn default() -> Self {
        Self {
            focus_state: FocusState::Normal,
            sync: Default::default(),
            end: Default::default(),
            redraw: Default::default(),
            highlighted_part: Default::default(),
            selected_tab: Default::default(),
            source_idx: Default::default(),
            remote_idx: Default::default(),
            is_focused: Default::default(),
            expanded_dirs: Default::default(),
            source_visible_rows: Vec::new(),
            remote_visible_rows: Vec::new(),
        }
    }
}

impl DashEventHandler {
    pub fn init_tree(&mut self, source_nodes: &[UiFileNode], remote_nodes: &[UiFileNode]) {
        self.refresh_rows(source_nodes, remote_nodes);
    }

    pub fn refresh_rows(&mut self, source_nodes: &[UiFileNode], remote_nodes: &[UiFileNode]) {
        self.source_visible_rows.clear();
        if let Some(tree) = source_nodes.get(self.selected_tab) {
            let mut rows = Vec::new();
            self.flatten_into_rows(tree, 0, &mut rows);
            self.source_visible_rows = rows;
        }

        self.remote_visible_rows.clear();
        if let Some(tree) = remote_nodes.get(self.selected_tab) {
            let mut rows = Vec::new();
            self.flatten_into_rows(tree, 0, &mut rows);
            self.remote_visible_rows = rows;
        }
    }

    fn flatten_into_rows(&self, node: &UiFileNode, depth: usize, rows: &mut Vec<UiRow>) {
        if depth > 0 {
            rows.push(UiRow {
                depth: depth - 1,
                path: node.path.clone(),
                name: node.name.clone(),
                is_dir: node.is_dir,
                status: node.status,
                color: node.color,
            });
        }

        if depth == 0 || (node.is_dir && self.expanded_dirs.contains(&node.path)) {
            let mut children: Vec<_> = node.children.values().collect();
            children.sort_by(|a, b| {
                b.is_dir
                    .cmp(&a.is_dir)
                    .then_with(|| status_weight(b.status).cmp(&status_weight(a.status))) // Unique/Conflict before Synced
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())) // Alphabetical order
            });

            for child in children {
                self.flatten_into_rows(child, depth + 1, rows);
            }
        }
    }

    pub fn expand_selected(&mut self, source_nodes: &[UiFileNode], remote_nodes: &[UiFileNode]) {
        let (rows, idx) = match self.highlighted_part {
            SelectedPart::Source => (&self.source_visible_rows, self.source_idx),
            SelectedPart::Remote => (&self.remote_visible_rows, self.remote_idx),
            _ => return,
        };
        if let Some(row) = rows.get(idx)
            && row.is_dir {
                self.expanded_dirs.insert(row.path.clone());
                self.refresh_rows(source_nodes, remote_nodes);
            }
    }

    pub fn collapse_selected(&mut self, source_nodes: &[UiFileNode], remote_nodes: &[UiFileNode]) {
        let (rows, idx) = match self.highlighted_part {
            SelectedPart::Source => (&self.source_visible_rows, self.source_idx),
            SelectedPart::Remote => (&self.remote_visible_rows, self.remote_idx),
            _ => return,
        };
        if let Some(row) = rows.get(idx)
            && row.is_dir {
                self.expanded_dirs.remove(&row.path);
                self.refresh_rows(source_nodes, remote_nodes);
            }
    }

    pub fn toggle_selected(&mut self, source_nodes: &[UiFileNode], remote_nodes: &[UiFileNode]) {
        let (rows, idx) = match self.highlighted_part {
            SelectedPart::Source => (&self.source_visible_rows, self.source_idx),
            SelectedPart::Remote => (&self.remote_visible_rows, self.remote_idx),
            _ => return,
        };
        if let Some(row) = rows.get(idx)
            && row.is_dir
        {
            if self.expanded_dirs.contains(&row.path) {
                self.expanded_dirs.remove(&row.path);
            } else {
                self.expanded_dirs.insert(row.path.clone());
            }
            self.refresh_rows(source_nodes, remote_nodes);
        }
    }

    pub fn handle(
        &mut self,
        events: &[Event],
        remotes: &[RemoteKnot],
        visible_items: usize,
        source_nodes: &[UiFileNode],
        remote_nodes: &[UiFileNode],
    ) {
        for event in events {
            match event {
                Event::Key(key) if key.kind != KeyEventKind::Release => {
                    let has_control = key.modifiers.contains(KeyModifiers::CONTROL);

                    match key.code {
                        // Global Exit
                        KeyCode::Char('q') | KeyCode::Char('c') => {
                            if key.code == KeyCode::Char('c') && has_control
                                || key.code != KeyCode::Char('c')
                            {
                                self.end = true;
                                continue;
                            }
                        }
                        KeyCode::Char('s') => {
                            self.sync = true;
                            self.redraw = true;
                        }
                        // Unselect / Unlock focus
                        KeyCode::Esc if self.is_focused => {
                            self.is_focused = false;
                            self.redraw = true;
                        }
                        // Lock focus into Source or Remote
                        KeyCode::Enter => {
                            if self.is_focused {
                                self.toggle_selected(source_nodes, remote_nodes);
                            } else {
                                self.is_focused = true;
                                if self.highlighted_part == SelectedPart::Tabs {
                                    self.highlighted_part = SelectedPart::Source;
                                }
                            }
                            self.redraw = true;
                        }
                        // Navigate Left / Previous Tab
                        KeyCode::Left => {
                            if self.is_focused {
                                self.collapse_selected(source_nodes, remote_nodes);
                            } else {
                                match self.highlighted_part {
                                    SelectedPart::Tabs => {
                                        if self.selected_tab > 0 {
                                            self.selected_tab -= 1;
                                        } else if !remotes.is_empty() {
                                            self.selected_tab = remotes.len().saturating_sub(1);
                                        }
                                        self.source_idx = 0;
                                        self.remote_idx = 0;
                                    }
                                    SelectedPart::Remote => {
                                        self.highlighted_part = SelectedPart::Source;
                                    }
                                    _ => {}
                                }
                            }
                            self.redraw = true;
                        }
                        // Navigate Right / Next Tab
                        KeyCode::Right => {
                            if self.is_focused {
                                self.expand_selected(source_nodes, remote_nodes);
                            } else {
                                match self.highlighted_part {
                                    SelectedPart::Tabs => {
                                        if !remotes.is_empty() {
                                            self.selected_tab =
                                                (self.selected_tab + 1) % remotes.len();
                                        }
                                        self.source_idx = 0;
                                        self.remote_idx = 0;
                                    }
                                    SelectedPart::Source => {
                                        self.highlighted_part = SelectedPart::Remote;
                                    }
                                    _ => {}
                                }
                            }
                            self.redraw = true;
                        }
                        // Navigate Up / Scroll Up
                        KeyCode::Up => {
                            if self.is_focused {
                                if self.highlighted_part == SelectedPart::Source {
                                    if self.source_idx == 0 {
                                        self.source_idx =
                                            self.source_visible_rows.len().saturating_sub(1);
                                    } else {
                                        self.source_idx = self.source_idx.saturating_sub(1);
                                    }
                                    self.redraw = true;
                                } else if self.highlighted_part == SelectedPart::Remote {
                                    if self.remote_idx == 0 {
                                        self.remote_idx =
                                            self.remote_visible_rows.len().saturating_sub(1);
                                    } else {
                                        self.remote_idx = self.remote_idx.saturating_sub(1);
                                    }
                                    self.redraw = true;
                                }
                            } else if self.highlighted_part != SelectedPart::Tabs {
                                self.highlighted_part = SelectedPart::Tabs;
                                self.redraw = true;
                            }
                        }
                        // Navigate Down / Scroll Down
                        KeyCode::Down => {
                            if self.is_focused {
                                if self.highlighted_part == SelectedPart::Source {
                                    if self.source_idx
                                        == self.source_visible_rows.len().saturating_sub(1)
                                    {
                                        self.source_idx = 0;
                                    } else {
                                        self.source_idx = self
                                            .source_idx
                                            .saturating_add(1)
                                            .min(self.source_visible_rows.len().saturating_sub(1));
                                    }
                                    self.redraw = true;
                                } else if self.highlighted_part == SelectedPart::Remote {
                                    if self.remote_idx
                                        == self.remote_visible_rows.len().saturating_sub(1)
                                    {
                                        self.remote_idx = 0;
                                    } else {
                                        self.remote_idx = self
                                            .remote_idx
                                            .saturating_add(1)
                                            .min(self.remote_visible_rows.len().saturating_sub(1));
                                    }
                                    self.redraw = true;
                                }
                            } else if self.highlighted_part == SelectedPart::Tabs {
                                self.highlighted_part = SelectedPart::Source;
                                self.redraw = true;
                            } else if self.highlighted_part == SelectedPart::Source
                                || self.highlighted_part == SelectedPart::Remote
                            {
                                self.is_focused = true;
                                self.redraw = true;
                            }
                        }
                        KeyCode::PageDown | KeyCode::Char(' ') if self.is_focused => {
                            if self.highlighted_part == SelectedPart::Source {
                                self.source_idx = self
                                    .source_idx
                                    .saturating_add(visible_items)
                                    .min(self.source_visible_rows.len().saturating_sub(1));
                                self.redraw = true;
                            } else if self.highlighted_part == SelectedPart::Remote {
                                self.remote_idx = self
                                    .remote_idx
                                    .saturating_add(visible_items)
                                    .min(self.remote_visible_rows.len().saturating_sub(1));
                                self.redraw = true;
                            }
                        }
                        KeyCode::PageUp if self.is_focused => {
                            if self.highlighted_part == SelectedPart::Source {
                                self.source_idx = self.source_idx.saturating_sub(visible_items);
                                self.redraw = true;
                            } else if self.highlighted_part == SelectedPart::Remote {
                                self.remote_idx = self.remote_idx.saturating_sub(visible_items);
                                self.redraw = true;
                            }
                        }
                        _ => {}
                    }
                }
                Event::Resize(..) => self.redraw = true,
                Event::FocusLost => {
                    self.focus_state = FocusState::Away;
                    self.redraw = true;
                }
                Event::FocusGained if self.focus_state == FocusState::Away => {
                    self.focus_state = FocusState::Returned;
                    self.redraw = true;
                }
                _ => {}
            }
        }
    }
}
