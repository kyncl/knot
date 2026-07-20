use crate::{
    knot::file_diffs::FileDiffs,
    utils::formatting::{format_hash, format_relative_time},
};
use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table, TableState, Tabs,
    },
};
use std::io::stdout;

struct RenderFile {
    ftype: String,
    path: String,
    mtime: String,
    hash: String,
    type_color: Color,
}

struct RenderConflict {
    path: String,
    src_mtime: String,
    rem_mtime: String,
    src_hash: String,
    rem_hash: String,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum ActiveTab {
    Conflicts,
    SourceUnique,
    RemoteUnique,
    Archived,
}

pub fn file_diff_visualization_interactive(diffs: &FileDiffs) -> Result<()> {
    let source_root_str = diffs.source_root_path.display().to_string();
    let remote_root_str = diffs.remote_root_path.display().to_string();

    let cached_conflicts: Vec<RenderConflict> = diffs
        .conflicts
        .iter()
        .map(|(src, rem)| {
            let path_str = src.path.display().to_string();
            let relative_path = path_str.strip_prefix(&source_root_str);
            let path = {
                if let Some(relative_path) = relative_path {
                    relative_path.to_string()
                } else {
                    path_str
                }
            };
            RenderConflict {
                path,
                src_mtime: format_relative_time(src.mtime),
                rem_mtime: format_relative_time(rem.mtime),
                src_hash: format_hash(src.content_hash),
                rem_hash: format_hash(rem.content_hash),
            }
        })
        .collect();

    let cached_source: Vec<RenderFile> = diffs
        .source_unique
        .iter()
        .map(|file| {
            let path_str = file.path.display().to_string();
            let relative_path = path_str.strip_prefix(&source_root_str);
            let path = {
                if let Some(relative_path) = relative_path {
                    relative_path.to_string()
                } else {
                    path_str
                }
            };
            RenderFile {
                ftype: (if file.is_dir { "DIR" } else { "FILE" }).to_string(),
                path,
                mtime: format_relative_time(file.mtime),
                hash: format_hash(file.content_hash),
                type_color: Color::Green,
            }
        })
        .collect();

    let cached_remote: Vec<RenderFile> = diffs
        .remote_unique
        .iter()
        .map(|file| {
            let path_str = file.path.display().to_string();
            let relative_path = path_str.strip_prefix(&remote_root_str);
            let path = {
                if let Some(relative_path) = relative_path {
                    relative_path.to_string()
                } else {
                    path_str
                }
            };
            RenderFile {
                ftype: (if file.is_dir { "DIR" } else { "FILE" }).to_string(),
                path,
                mtime: format_relative_time(file.mtime),
                hash: format_hash(file.content_hash),
                type_color: Color::Blue,
            }
        })
        .collect();

    let cached_archived: Vec<RenderFile> = diffs
        .archived
        .iter()
        .map(|file| RenderFile {
            ftype: (if file.is_dir { "DIR" } else { "FILE" }).to_string(),
            path: file.path.display().to_string(),
            mtime: format_relative_time(file.mtime),
            hash: format_hash(file.content_hash),
            type_color: Color::Yellow,
        })
        .collect();

    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut active_tab = {
        let mut biggest = 0;
        let mut active = ActiveTab::Conflicts;
        if cached_remote.len() > biggest {
            biggest = cached_remote.len();
            active = ActiveTab::RemoteUnique;
        }
        if cached_source.len() > biggest {
            biggest = cached_source.len();
            active = ActiveTab::SourceUnique;
        }
        if cached_archived.len() > biggest {
            biggest = cached_archived.len();
            active = ActiveTab::Archived;
        }
        if cached_conflicts.len() > biggest {
            active = ActiveTab::Conflicts;
        }
        active
    };
    let mut table_state = TableState::default();
    table_state.select(Some(0));

    let mut needs_redraw = true;

    loop {
        if needs_redraw {
            terminal.draw(|frame| {
                let chunks = Layout::vertical([
                    Constraint::Length(3), // Tab bar
                    Constraint::Min(0),    // Table body
                    Constraint::Length(1), // Hint bar
                ])
                .split(frame.area());

                let tab_titles = vec![
                    Line::from(vec![
                        Span::styled(
                            " !! ",
                            Style::default()
                                .fg(Color::LightRed)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("Conflicts ", Style::default().fg(Color::White)),
                        Span::styled(
                            format!("({})", cached_conflicts.len()),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            " ++ ",
                            Style::default()
                                .fg(Color::LightGreen)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("Source Unique ", Style::default().fg(Color::White)),
                        Span::styled(
                            format!("({})", cached_source.len()),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            " -- ",
                            Style::default()
                                .fg(Color::LightBlue)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("Remote Unique ", Style::default().fg(Color::White)),
                        Span::styled(
                            format!("({})", cached_remote.len()),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            " == ",
                            Style::default()
                                .fg(Color::LightYellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("Archived ", Style::default().fg(Color::White)),
                        Span::styled(
                            format!("({})", cached_archived.len()),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                ];

                let (current_tab_idx, active_border_color) = match active_tab {
                    ActiveTab::Conflicts => (0, Color::LightRed),
                    ActiveTab::SourceUnique => (1, Color::LightGreen),
                    ActiveTab::RemoteUnique => (2, Color::LightBlue),
                    ActiveTab::Archived => (3, Color::LightYellow),
                };

                let tabs = Tabs::new(tab_titles)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Thick)
                            .border_style(Style::default().fg(active_border_color))
                            .title(" SYNCHRONIZATION REPORT ")
                            .title_style(
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ),
                    )
                    .select(current_tab_idx)
                    .highlight_style(
                        Style::default()
                            .bg(Color::Rgb(50, 50, 70))
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                    .divider(Span::styled(" | ", Style::default().fg(Color::DarkGray)));

                frame.render_widget(tabs, chunks[0]);

                // --- TABLE BODY WITH STYLED COLUMNS ---
                let mut rows = Vec::new();
                let header_cols;
                let widths;

                match active_tab {
                    ActiveTab::Conflicts => {
                        header_cols = vec![
                            "File Path",
                            "Source Modified",
                            "Remote Modified",
                            "Source Hash",
                            "Remote Hash",
                        ];
                        widths = vec![
                            Constraint::Percentage(40),
                            Constraint::Percentage(15),
                            Constraint::Percentage(15),
                            Constraint::Percentage(15),
                            Constraint::Percentage(15),
                        ];
                        for item in &cached_conflicts {
                            rows.push(Row::new(vec![
                                Cell::from(item.path.as_str())
                                    .fg(Color::LightRed)
                                    .add_modifier(Modifier::BOLD),
                                Cell::from(item.src_mtime.as_str()).fg(Color::White),
                                Cell::from(item.rem_mtime.as_str()).fg(Color::White),
                                Cell::from(item.src_hash.as_str()).fg(Color::Green),
                                Cell::from(item.rem_hash.as_str()).fg(Color::Blue),
                            ]));
                        }
                    }
                    ActiveTab::SourceUnique | ActiveTab::RemoteUnique | ActiveTab::Archived => {
                        header_cols = vec!["Type", "File Path", "Modified Time", "Content Hash"];
                        widths = vec![
                            Constraint::Length(6),
                            Constraint::Percentage(55),
                            Constraint::Percentage(20),
                            Constraint::Percentage(20),
                        ];
                        let items = match active_tab {
                            ActiveTab::SourceUnique => &cached_source,
                            ActiveTab::RemoteUnique => &cached_remote,
                            ActiveTab::Archived => &cached_archived,
                            _ => unreachable!(),
                        };
                        for item in items {
                            rows.push(Row::new(vec![
                                Cell::from(item.ftype.as_str())
                                    .fg(item.type_color)
                                    .add_modifier(Modifier::BOLD),
                                Cell::from(item.path.as_str()).fg(Color::White),
                                Cell::from(item.mtime.as_str()).fg(Color::Gray),
                                Cell::from(item.hash.as_str()).fg(Color::DarkGray),
                            ]));
                        }
                    }
                }

                let items_count = rows.len();
                let selected_idx = table_state.selected().unwrap_or(0);

                let title_info = {
                    match active_tab {
                        ActiveTab::Archived => format!("{remote_root_str}/~").replace("//", "/"),
                        ActiveTab::Conflicts => {
                            format!("{source_root_str}/~ | {remote_root_str}/~").replace("//", "/")
                        }
                        ActiveTab::SourceUnique => {
                            format!("{source_root_str}/~").replace("//", "/")
                        }
                        ActiveTab::RemoteUnique => {
                            format!("{remote_root_str}/~").replace("//", "/")
                        }
                    }
                };

                let table = Table::new(rows, widths)
                    .header(
                        Row::new(header_cols)
                            .style(
                                Style::default()
                                    .fg(active_border_color)
                                    .add_modifier(Modifier::BOLD),
                            )
                            .bottom_margin(1),
                    )
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::DarkGray))
                            .title(format!(" {title_info} (Items: {items_count}) "))
                            .title_style(Style::default().fg(Color::DarkGray)),
                    )
                    .row_highlight_style(
                        Style::default()
                            .bg(Color::Rgb(45, 45, 60))
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(" ❯ ");

                frame.render_stateful_widget(table, chunks[1], &mut table_state);

                // --- SCROLLBAR ---
                let size_forscrollbar = {
                    let size = crossterm::terminal::size();
                    if let Ok(size) = size {
                        size.1 as usize
                    } else {
                        0
                    }
                };
                if items_count > size_forscrollbar {
                    let mut scrollbar_state =
                        ScrollbarState::new(items_count.saturating_sub(1)).position(selected_idx);

                    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                        .begin_symbol(Some("▲"))
                        .end_symbol(Some("▼"))
                        .track_symbol(Some("│"))
                        .thumb_symbol("█")
                        .style(Style::default().fg(active_border_color));

                    // Render the scrollbar inside the table border area
                    let scrollbar_area = Rect {
                        x: chunks[1].right().saturating_sub(1),
                        y: chunks[1].top() + 1,
                        width: 1,
                        height: chunks[1].height.saturating_sub(2),
                    };

                    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
                }

                // --- HINT BAR ---
                let hint = Line::from(vec![
                    Span::styled(" ←/→/Tab ", Style::default().fg(Color::Cyan)),
                    Span::raw("Switch Tabs | "),
                    Span::styled("↑/↓ ", Style::default().fg(Color::Cyan)),
                    Span::raw("Scroll | "),
                    Span::styled("Space ", Style::default().fg(Color::Cyan)),
                    Span::raw("Page Down | "),
                    Span::styled("Esc/Q ", Style::default().fg(Color::Cyan)),
                    Span::raw("Exit"),
                ]);
                frame.render_widget(hint.dark_gray(), chunks[2]);
            })?;
            needs_redraw = false;
        }

        // --- 3. HIGH-PERFORMANCE EVENT COALESCING ---
        let first_event = event::read()?;
        let mut events = vec![first_event];

        while event::poll(std::time::Duration::from_millis(0))? {
            events.push(event::read()?);
        }

        for event in events {
            match event {
                Event::Key(key) => {
                    if key.kind != crossterm::event::KeyEventKind::Release {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                disable_raw_mode()?;
                                std::io::stdout().execute(LeaveAlternateScreen)?;
                                return Ok(());
                            }
                            KeyCode::Left => {
                                active_tab = match active_tab {
                                    ActiveTab::Conflicts => ActiveTab::Archived,
                                    ActiveTab::SourceUnique => ActiveTab::Conflicts,
                                    ActiveTab::RemoteUnique => ActiveTab::SourceUnique,
                                    ActiveTab::Archived => ActiveTab::RemoteUnique,
                                };
                                table_state.select(Some(0));
                                needs_redraw = true;
                            }
                            KeyCode::Right | KeyCode::Tab => {
                                active_tab = match active_tab {
                                    ActiveTab::Conflicts => ActiveTab::SourceUnique,
                                    ActiveTab::SourceUnique => ActiveTab::RemoteUnique,
                                    ActiveTab::RemoteUnique => ActiveTab::Archived,
                                    ActiveTab::Archived => ActiveTab::Conflicts,
                                };
                                table_state.select(Some(0));
                                needs_redraw = true;
                            }
                            KeyCode::Up => {
                                let max_idx = match active_tab {
                                    ActiveTab::Conflicts => cached_conflicts.len(),
                                    ActiveTab::SourceUnique => cached_source.len(),
                                    ActiveTab::RemoteUnique => cached_remote.len(),
                                    ActiveTab::Archived => cached_archived.len(),
                                };
                                if max_idx > 0 {
                                    let selected = table_state.selected().unwrap_or(0);
                                    if selected > 0 {
                                        table_state.select(Some(selected - 1));
                                    } else {
                                        table_state.select(Some(max_idx - 1));
                                    }
                                    needs_redraw = true;
                                }
                            }
                            KeyCode::Down => {
                                let max_idx = match active_tab {
                                    ActiveTab::Conflicts => cached_conflicts.len(),
                                    ActiveTab::SourceUnique => cached_source.len(),
                                    ActiveTab::RemoteUnique => cached_remote.len(),
                                    ActiveTab::Archived => cached_archived.len(),
                                };
                                if max_idx > 0 {
                                    let selected = table_state.selected().unwrap_or(0);
                                    if selected < max_idx - 1 {
                                        table_state.select(Some(selected + 1));
                                    } else {
                                        table_state.select(Some(0));
                                    }
                                    needs_redraw = true;
                                }
                            }
                            KeyCode::Char(' ') => {
                                let max_idx = match active_tab {
                                    ActiveTab::Conflicts => cached_conflicts.len(),
                                    ActiveTab::SourceUnique => cached_source.len(),
                                    ActiveTab::RemoteUnique => cached_remote.len(),
                                    ActiveTab::Archived => cached_archived.len(),
                                };
                                if max_idx > 0 {
                                    let selected = table_state.selected().unwrap_or(0);
                                    let h = {
                                        let size = crossterm::terminal::size();
                                        if let Ok(size) = size {
                                            size.1 as usize
                                        } else {
                                            0
                                        }
                                    };

                                    let target = (selected + h).min(max_idx - 1);
                                    table_state.select(Some(target));
                                    needs_redraw = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::Resize(_, _) => {
                    needs_redraw = true;
                }
                _ => {}
            }
        }
    }
}
