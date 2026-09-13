use anyhow::Result;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState, Tabs, Widget,
    },
};
use std::io::Stdout;

use crate::{
    cli::visualization::dashboard::event_handler::{DashEventHandler, SelectedPart, UiRow},
    configuration::loader::{configuration::ConfigurationLoader, remote::RemoteKnotLoader},
    knot::file_diffs::FileDiffs,
};

struct DashDesigner {
    tabs: Rect,
    source: Rect,
    remote: Rect,
    hint: Rect,
}
impl DashDesigner {
    fn chunks(area: Rect) -> Self {
        let vertical_chunks = Layout::vertical([
            Constraint::Length(3), // Top Statusline
            Constraint::Min(0),    // Table with Borders
            Constraint::Length(3), // Footer Keymap
        ])
        .split(area);
        let diff_chunks =
            Layout::horizontal([Constraint::Min(0), Constraint::Min(0)]).split(vertical_chunks[1]);
        Self {
            tabs: vertical_chunks[0],
            hint: vertical_chunks[2],
            source: diff_chunks[0],
            remote: diff_chunks[1],
        }
    }
}

pub struct DashDrawer;
impl DashDrawer {
    fn highlight_block(is_selected: bool, is_highlighted: bool) -> Block<'static> {
        let border_col = if is_selected {
            Color::Rgb(255, 95, 0)
        } else if is_highlighted {
            Color::Gray
        } else {
            Color::DarkGray
        };

        Block::new()
            .borders(Borders::ALL)
            .border_type(if is_selected {
                BorderType::Thick
            } else {
                BorderType::Plain
            })
            .border_style(Style::default().fg(border_col))
    }

    /// Helper to style the scrollbar uniformly
    fn build_scrollbar<'a>(is_selected: bool, is_highlighted: bool) -> Scrollbar<'a> {
        let color = if is_selected {
            Color::Rgb(255, 95, 0)
        } else if is_highlighted {
            Color::Gray
        } else {
            Color::DarkGray
        };

        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_symbol("█")
            .style(Style::default().fg(color))
    }

    /// Helper to assemble the Table component
    fn create_file_table<'a>(
        is_selected: bool,
        is_highlighted: bool,
        title: &'a str,
        bottom_title: String,
        rows: Vec<Row<'static>>,
    ) -> Table<'a> {
        let block = Self::highlight_block(is_selected, is_highlighted)
            .title(title)
            .title_bottom(bottom_title);

        let header = Row::new(vec!["Type", "Status", "File Path"])
            .style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )
            .bottom_margin(1);

        Table::new(
            rows,
            [
                Constraint::Length(5),
                Constraint::Length(9),
                Constraint::Min(0),
            ],
        )
        .header(header)
        .block(block)
        .row_highlight_style(
            Style::default()
                .bg(Color::Rgb(45, 45, 60))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ❯ ")
    }

    /// Helper to calculate layout states (visible rows, scroll offsets, states)
    fn setup_scroll_state(
        area_height: u16,
        total_items: usize,
        selected_idx: usize,
    ) -> (usize, usize, TableState, Option<ScrollbarState>) {
        let visible_rows_count = area_height.saturating_sub(4) as usize;
        let scroll_offset = if visible_rows_count == 0 || selected_idx < visible_rows_count {
            0
        } else {
            (selected_idx + 1)
                .saturating_sub(visible_rows_count)
                .min(total_items.saturating_sub(visible_rows_count))
        };

        let mut table_state = TableState::default();
        if total_items > 0 {
            table_state.select(Some(selected_idx.saturating_sub(scroll_offset)));
        }

        let scrollbar_state = if total_items > visible_rows_count {
            Some(ScrollbarState::new(total_items.saturating_sub(1)).position(selected_idx))
        } else {
            None
        };

        (
            visible_rows_count,
            scroll_offset,
            table_state,
            scrollbar_state,
        )
    }

    pub fn tabs(
        handler: &DashEventHandler,
        loaded_knots: &RemoteKnotLoader,
        is_selected: bool,
    ) -> impl Widget {
        let mut tab_titles = Vec::with_capacity(loaded_knots.knots.len());
        for knot in &loaded_knots.knots {
            let cred = if let Some(ref cred) = knot.config.credentials {
                format!("{}@{}:{}", cred.username, cred.host, cred.port)
            } else {
                // Most likely Local directory
                knot.config.path.display().to_string()
            };
            tab_titles.push(Line::from(vec![Span::styled(
                cred,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]));
        }

        Tabs::new(tab_titles)
            .block(Self::highlight_block(is_selected, false).title(" Remote Knots "))
            .select(handler.selected_tab)
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(50, 50, 70))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .divider(Span::styled(" · ", Style::default().fg(Color::DarkGray)))
    }

    fn build_tree_row(row: &UiRow, is_expanded: bool) -> Row<'static> {
        let indent = "│  ".repeat(row.depth);
        let type_icon = if row.is_dir { "󰉋" } else { "󰈔" };
        let arrow = if row.is_dir {
            if is_expanded { "▼ " } else { "▶ " }
        } else {
            "● "
        };
        let display_path = format!("{indent}{arrow}{}", row.name);
        Row::new(vec![
            Cell::from(type_icon).style(Style::default().fg(row.color)),
            Cell::from(row.status).style(Style::default().fg(row.color)),
            Cell::from(display_path).style(Style::default().fg(row.color)),
        ])
    }

    pub fn source<'a>(
        is_selected: bool,
        is_highlighted: bool,
        loaded_conf: &ConfigurationLoader,
        handler: &DashEventHandler,
        scroll_offset: usize,
        visible_rows_count: usize,
    ) -> Table<'a> {
        let root_str = loaded_conf.source.path.display().to_string();
        // For not revealing any path information while taking pictures
        // also it would look weird and bad
        let bottom_title = if cfg!(debug_assertions) {
            "".to_string()
        } else {
            format!(" {}/~ ", root_str).replace("//", "/")
        };

        let paged_rows = handler
            .source_visible_rows
            .iter()
            .skip(scroll_offset)
            .take(visible_rows_count);

        let rows: Vec<Row<'static>> = paged_rows
            .map(|row| {
                let is_expanded = handler.expanded_dirs.contains(&row.path);
                Self::build_tree_row(row, is_expanded)
            })
            .collect();

        Self::create_file_table(
            is_selected,
            is_highlighted,
            " Source Directory ",
            bottom_title,
            rows,
        )
    }

    pub fn remote<'a>(
        is_selected: bool,
        is_highlighted: bool,
        diffs: &FileDiffs,
        handler: &DashEventHandler,
        scroll_offset: usize,
        visible_rows_count: usize,
    ) -> Table<'a> {
        let root_str = diffs.remote_root_path.display().to_string();
        // Same as source, it would look awful on pictures
        let bottom_title = if cfg!(debug_assertions) {
            "".to_string()
        } else {
            format!(" {}/~ ", root_str).replace("//", "/")
        };

        let paged_rows = handler
            .remote_visible_rows
            .iter()
            .skip(scroll_offset)
            .take(visible_rows_count);

        let rows: Vec<Row<'static>> = paged_rows
            .map(|row| {
                let is_expanded = handler.expanded_dirs.contains(&row.path);
                Self::build_tree_row(row, is_expanded)
            })
            .collect();

        Self::create_file_table(
            is_selected,
            is_highlighted,
            " Remote Directory ",
            bottom_title,
            rows,
        )
    }

    pub fn hint(is_selected: bool) -> impl Widget {
        Paragraph::new(Line::from(vec![
            Span::styled("  ←/→|↑/↓ ", Style::default().fg(Color::Cyan))
                .add_modifier(Modifier::BOLD),
            Span::raw("Move · "),
            Span::styled("Space/PgDn ", Style::default().fg(Color::Cyan))
                .add_modifier(Modifier::BOLD),
            Span::raw("Page Down · "),
            Span::styled("Q ", Style::default().fg(Color::Red)).add_modifier(Modifier::BOLD),
            Span::raw("Exit · "),
            Span::styled("Esc ", Style::default().fg(Color::Red)).add_modifier(Modifier::BOLD),
            Span::raw("Cancel · "),
            Span::styled("Enter ", Style::default().fg(Color::Yellow)).add_modifier(Modifier::BOLD),
            Span::raw("Select · "),
            Span::styled("S ", Style::default().fg(Color::Green)).add_modifier(Modifier::BOLD),
            Span::raw("Synchronize"),
        ]))
        .block(Self::highlight_block(is_selected, false).title(" Hint "))
    }

    pub fn redraw(
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        handler: &DashEventHandler,
        loaded_conf: &ConfigurationLoader,
        loaded_knots: &RemoteKnotLoader,
        diffs: &[FileDiffs],
    ) -> Result<usize> {
        let mut visible_items_count = 0;

        terminal.draw(|frame| {
            let chunks = DashDesigner::chunks(frame.area());
            let is_tabs_selected =
                !handler.is_focused && handler.highlighted_part == SelectedPart::Tabs;
            let is_source_highlighted =
                !handler.is_focused && handler.highlighted_part == SelectedPart::Source;
            let is_source_selected =
                handler.is_focused && handler.highlighted_part == SelectedPart::Source;
            let is_remote_highlighted =
                !handler.is_focused && handler.highlighted_part == SelectedPart::Remote;
            let is_remote_selected =
                handler.is_focused && handler.highlighted_part == SelectedPart::Remote;

            frame.render_widget(
                Self::tabs(handler, loaded_knots, is_tabs_selected),
                chunks.tabs,
            );

            if let Some(diff) = diffs.get(handler.selected_tab) {
                // --- SOURCE PANEL ---
                {
                    let area = chunks.source;
                    let (vis_rows, scroll, mut table_state, scroll_state) =
                        Self::setup_scroll_state(
                            area.height,
                            handler.source_visible_rows.len(),
                            handler.source_idx,
                        );

                    visible_items_count = vis_rows;

                    let table = Self::source(
                        is_source_selected,
                        is_source_highlighted,
                        loaded_conf,
                        handler,
                        scroll,
                        vis_rows,
                    );
                    frame.render_stateful_widget(table, area, &mut table_state);

                    if let Some(mut state) = scroll_state {
                        let scrollbar =
                            Self::build_scrollbar(is_source_selected, is_source_highlighted);
                        let scroll_area = Rect {
                            x: area.right().saturating_sub(1),
                            y: area.top() + 1,
                            width: 1,
                            height: area.height.saturating_sub(2),
                        };
                        frame.render_stateful_widget(scrollbar, scroll_area, &mut state);
                    }
                }

                // --- REMOTE PANEL ---
                {
                    let area = chunks.remote;
                    let (vis_rows, scroll, mut table_state, scroll_state) =
                        Self::setup_scroll_state(
                            area.height,
                            handler.remote_visible_rows.len(),
                            handler.remote_idx,
                        );

                    let table = Self::remote(
                        is_remote_selected,
                        is_remote_highlighted,
                        diff,
                        handler,
                        scroll,
                        vis_rows,
                    );
                    frame.render_stateful_widget(table, area, &mut table_state);

                    if let Some(mut state) = scroll_state {
                        let scrollbar =
                            Self::build_scrollbar(is_remote_selected, is_remote_highlighted);
                        let scroll_area = Rect {
                            x: area.right().saturating_sub(1),
                            y: area.top() + 1,
                            width: 1,
                            height: area.height.saturating_sub(2),
                        };
                        frame.render_stateful_widget(scrollbar, scroll_area, &mut state);
                    }
                }
            } else {
                // Empty Fallback states
                let src_bottom = if cfg!(debug_assertions) {
                    "".to_string()
                } else {
                    format!(" {}/~ ", loaded_conf.source.path.display()).replace("//", "/")
                };
                frame.render_widget(
                    Self::highlight_block(is_source_selected, is_source_highlighted)
                        .title(" Source Directory ")
                        .title_bottom(src_bottom),
                    chunks.source,
                );

                let rem_bottom = if !cfg!(debug_assertions)
                    && let Some(remote) = loaded_knots.knots.get(handler.selected_tab)
                {
                    format!("{}/~", remote.config.path.display()).replace("//", "/")
                } else {
                    "".to_string()
                };

                frame.render_widget(
                    Self::highlight_block(is_remote_selected, is_remote_highlighted)
                        .title(" Remote Directory ")
                        .title_bottom(rem_bottom),
                    chunks.remote,
                );
            }

            frame.render_widget(Self::hint(false), chunks.hint);
        })?;

        Ok(visible_items_count)
    }
}
