use crate::{
    USER_AWAY_MSG, USER_CAMEBACK_MSG,
    cli::visualization::{
        rata_utils::{rata_clean, rata_init},
        resolver::FocusState,
    },
    knot::file_diffs::FileDiffs,
    utils::formatting::{format_hash, format_relative_time},
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use indicatif::HumanBytes;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState, Tabs,
    },
};
use std::time::Duration;

#[derive(Copy, Clone, Debug, PartialEq)]
enum ActiveTab {
    Conflicts,
    SourceUnique,
    RemoteUnique,
    Archived,
}

pub fn file_diff_visualization_interactive(
    diffs: &FileDiffs,
) -> Result<bool, Box<dyn std::error::Error>> {
    let source_root_str = diffs.source_root_path.display().to_string();
    let remote_root_str = diffs.remote_root_path.display().to_string();

    let conflicts_len = diffs.conflicts.len();
    let source_len = diffs.source_unique.len();
    let remote_len = diffs.remote_unique.len();
    let archived_len = diffs.archived.len();

    // Default to tab with the most items
    let mut active_tab = ActiveTab::Conflicts;
    let mut max_len = conflicts_len;
    if source_len > max_len {
        max_len = source_len;
        active_tab = ActiveTab::SourceUnique;
    }
    if remote_len > max_len {
        max_len = remote_len;
        active_tab = ActiveTab::RemoteUnique;
    }
    if archived_len > max_len {
        active_tab = ActiveTab::Archived;
    }

    let mut terminal = rata_init()?;
    let mut selected_idx: usize = 0;
    let mut needs_redraw = true;
    let mut focus_state = FocusState::Normal;

    let res = (|| -> Result<bool, Box<dyn std::error::Error>> {
        loop {
            let active_len = match active_tab {
                ActiveTab::Conflicts => conflicts_len,
                ActiveTab::SourceUnique => source_len,
                ActiveTab::RemoteUnique => remote_len,
                ActiveTab::Archived => archived_len,
            };

            if active_len > 0 && selected_idx >= active_len {
                selected_idx = active_len - 1;
            }

            if needs_redraw {
                terminal.draw(|frame| {
                    let chunks = Layout::vertical([
                        Constraint::Length(3), // Tab bar
                        Constraint::Min(0),    // Table body
                        Constraint::Length(1), // Hint bar
                    ])
                    .split(frame.area());

                    let table_area = chunks[1];
                    // Subtract 2 for top/bottom borders and 2 for header + header margin
                    let visible_rows_count = table_area.height.saturating_sub(4) as usize;

                    // --- TAB BAR ---
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
                                format!("({conflicts_len})"),
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
                                format!("({source_len})"),
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
                                format!("({remote_len})"),
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
                                format!("({archived_len})"),
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
                        .divider(Span::styled(" · ", Style::default().fg(Color::DarkGray)));

                    frame.render_widget(tabs, chunks[0]);

                    // --- VIEWPORT COMPUTATION ---
                    // Calculate top offset so selected_idx is always in view
                    let scroll_offset =
                        if visible_rows_count == 0 || selected_idx < visible_rows_count {
                            0
                        } else {
                            (selected_idx + 1)
                                .saturating_sub(visible_rows_count)
                                .min(active_len.saturating_sub(visible_rows_count))
                        };

                    // --- LAZY ROW RENDERING (O(visible) instead of O(N)) ---
                    let (rows, header_cols, widths): (Vec<Row>, Vec<&str>, Vec<Constraint>) =
                        match active_tab {
                            ActiveTab::Conflicts => {
                                let headers = vec![
                                    "File Path",
                                    "Source Modified",
                                    "Remote Modified",
                                    "Source Hash",
                                    "Remote Hash",
                                    "Size diff",
                                ];
                                let constraints = vec![
                                    Constraint::Percentage(45),
                                    Constraint::Percentage(10),
                                    Constraint::Percentage(10),
                                    Constraint::Percentage(10),
                                    Constraint::Percentage(10),
                                    Constraint::Percentage(15),
                                ];

                                let visible_items = diffs
                                    .conflicts
                                    .iter()
                                    .skip(scroll_offset)
                                    .take(visible_rows_count);
                                let rows = visible_items
                                    .map(|(src, rem)| {
                                        let path_str = src.path.display().to_string();
                                        let path = path_str
                                            .strip_prefix(&source_root_str)
                                            .unwrap_or(&path_str);

                                        Row::new(vec![
                                            Cell::from(path.to_string())
                                                .fg(Color::LightRed)
                                                .add_modifier(Modifier::BOLD),
                                            Cell::from(format_relative_time(src.mtime))
                                                .fg(Color::White),
                                            Cell::from(format_relative_time(rem.mtime))
                                                .fg(Color::White),
                                            Cell::from(format_hash(src.content_hash))
                                                .fg(Color::Green),
                                            Cell::from(format_hash(rem.content_hash))
                                                .fg(Color::Blue),
                                            Cell::from(format!(
                                                "{} | {}",
                                                HumanBytes(src.size),
                                                HumanBytes(rem.size)
                                            ))
                                            .fg(Color::White),
                                        ])
                                    })
                                    .collect();

                                (rows, headers, constraints)
                            }
                            ActiveTab::SourceUnique
                            | ActiveTab::RemoteUnique
                            | ActiveTab::Archived => {
                                let headers = vec![
                                    "Type",
                                    "File Path",
                                    "Modified Time",
                                    "Content Hash",
                                    "Size",
                                ];
                                let constraints = vec![
                                    Constraint::Length(7),
                                    Constraint::Percentage(65),
                                    Constraint::Percentage(10),
                                    Constraint::Percentage(10),
                                    Constraint::Percentage(10),
                                ];

                                let (slice, root_str, color) = match active_tab {
                                    ActiveTab::SourceUnique => (
                                        &diffs.source_unique,
                                        source_root_str.as_str(),
                                        Color::Green,
                                    ),
                                    ActiveTab::RemoteUnique => (
                                        &diffs.remote_unique,
                                        remote_root_str.as_str(),
                                        Color::Blue,
                                    ),
                                    ActiveTab::Archived => (&diffs.archived, "", Color::Yellow),
                                    _ => unreachable!(),
                                };

                                let visible_items =
                                    slice.iter().skip(scroll_offset).take(visible_rows_count);
                                let rows = visible_items
                                    .map(|file| {
                                        let path_str = file.path.display().to_string();
                                        let path = if root_str.is_empty() {
                                            &path_str
                                        } else {
                                            path_str.strip_prefix(root_str).unwrap_or(&path_str)
                                        };

                                        let ftype =
                                            if file.is_dir { "󰉋 DIR" } else { "󰈔 FILE" };

                                        Row::new(vec![
                                            Cell::from(ftype)
                                                .fg(color)
                                                .add_modifier(Modifier::BOLD),
                                            Cell::from(path.to_string()).fg(Color::White),
                                            Cell::from(format_relative_time(file.mtime))
                                                .fg(Color::Gray),
                                            Cell::from(format_hash(file.content_hash))
                                                .fg(Color::DarkGray),
                                            Cell::from(HumanBytes(file.size).to_string())
                                                .fg(Color::White),
                                        ])
                                    })
                                    .collect();

                                (rows, headers, constraints)
                            }
                        };

                    let title_info = match active_tab {
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
                                .title(format!(" {title_info} "))
                                .title_style(Style::default().fg(Color::DarkGray)),
                        )
                        .row_highlight_style(
                            Style::default()
                                .bg(Color::Rgb(45, 45, 60))
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )
                        .highlight_symbol(" ❯ ");

                    // Local relative selection index for current visible slice
                    let relative_selection = selected_idx.saturating_sub(scroll_offset);
                    let mut table_state = TableState::default();
                    if active_len > 0 {
                        table_state.select(Some(relative_selection));
                    }

                    frame.render_stateful_widget(table, table_area, &mut table_state);

                    // --- SCROLLBAR ---
                    if active_len > visible_rows_count {
                        let mut scrollbar_state = ScrollbarState::new(active_len.saturating_sub(1))
                            .position(selected_idx);

                        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                            .thumb_symbol("█")
                            .style(Style::default().fg(active_border_color));

                        let scrollbar_area = Rect {
                            x: table_area.right().saturating_sub(1),
                            y: table_area.top() + 1,
                            width: 1,
                            height: table_area.height.saturating_sub(2),
                        };

                        frame.render_stateful_widget(
                            scrollbar,
                            scrollbar_area,
                            &mut scrollbar_state,
                        );
                    }

                    let footer_chunks =
                        Layout::horizontal([Constraint::Min(0), Constraint::Length(28)])
                            .split(chunks[2]);
                    // --- HINT BAR ---
                    let hint = Line::from(vec![
                        Span::styled("  ←/→/Tab ", Style::default().fg(Color::Cyan))
                            .add_modifier(Modifier::BOLD),
                        Span::raw("Switch Tabs · "),
                        Span::styled("↑/↓ ", Style::default().fg(Color::Cyan))
                            .add_modifier(Modifier::BOLD),
                        Span::raw("Scroll · "),
                        Span::styled("Space/PgDn ", Style::default().fg(Color::Cyan))
                            .add_modifier(Modifier::BOLD),
                        Span::raw("Page Down · "),
                        Span::styled("Esc/Q ", Style::default().fg(Color::Red))
                            .add_modifier(Modifier::BOLD),
                        Span::raw("Exit · "),
                        Span::styled("Enter/S ", Style::default().fg(Color::Green))
                            .add_modifier(Modifier::BOLD),
                        Span::raw("Synchronize"),
                    ]);
                    frame.render_widget(hint.dark_gray(), footer_chunks[0]);
                    let (easter_egg_text, easter_egg_color) = match focus_state {
                        FocusState::Away => (USER_AWAY_MSG, Color::Yellow),
                        FocusState::Returned => (USER_CAMEBACK_MSG, Color::LightGreen),
                        FocusState::Normal => ("", Color::Reset),
                    };
                    if !easter_egg_text.is_empty() {
                        let egg_widget = Paragraph::new(Line::from(vec![
                            Span::styled(
                                easter_egg_text,
                                Style::default()
                                    .fg(easter_egg_color)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw("  "),
                        ]))
                        .alignment(Alignment::Right);
                        frame.render_widget(egg_widget, footer_chunks[1]);
                    }
                })?;
                needs_redraw = false;
            }

            // --- EVENT LOOP WITH COALESCING ---
            let first_event = event::read()?;
            let mut events = vec![first_event];

            while event::poll(Duration::from_millis(0))? {
                events.push(event::read()?);
            }

            for event in events {
                match event {
                    Event::Key(key) if key.kind != event::KeyEventKind::Release => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
                        KeyCode::Enter | KeyCode::Char('s') => return Ok(false),
                        KeyCode::Left => {
                            active_tab = match active_tab {
                                ActiveTab::Conflicts => ActiveTab::Archived,
                                ActiveTab::SourceUnique => ActiveTab::Conflicts,
                                ActiveTab::RemoteUnique => ActiveTab::SourceUnique,
                                ActiveTab::Archived => ActiveTab::RemoteUnique,
                            };
                            selected_idx = 0;
                            needs_redraw = true;
                        }
                        KeyCode::Right | KeyCode::Tab => {
                            active_tab = match active_tab {
                                ActiveTab::Conflicts => ActiveTab::SourceUnique,
                                ActiveTab::SourceUnique => ActiveTab::RemoteUnique,
                                ActiveTab::RemoteUnique => ActiveTab::Archived,
                                ActiveTab::Archived => ActiveTab::Conflicts,
                            };
                            selected_idx = 0;
                            needs_redraw = true;
                        }
                        KeyCode::Up => {
                            if active_len > 0 {
                                selected_idx = if selected_idx > 0 {
                                    selected_idx - 1
                                } else {
                                    active_len - 1
                                };
                                needs_redraw = true;
                            }
                        }
                        KeyCode::Down => {
                            if active_len > 0 {
                                selected_idx = if selected_idx < active_len - 1 {
                                    selected_idx + 1
                                } else {
                                    0
                                };
                                needs_redraw = true;
                            }
                        }
                        KeyCode::Char(' ') | KeyCode::PageDown => {
                            if active_len > 0 {
                                selected_idx = (selected_idx + 20).min(active_len - 1);
                                needs_redraw = true;
                            }
                        }
                        KeyCode::PageUp => {
                            if active_len > 0 {
                                selected_idx = selected_idx.saturating_sub(20);
                                needs_redraw = true;
                            }
                        }
                        KeyCode::Home => {
                            selected_idx = 0;
                            needs_redraw = true;
                        }
                        KeyCode::End if active_len > 0 => {
                            selected_idx = active_len - 1;
                            needs_redraw = true;
                        }
                        _ => {}
                    },
                    Event::Resize(_, _) => {
                        needs_redraw = true;
                    }
                    Event::FocusLost => {
                        focus_state = FocusState::Away;
                        needs_redraw = true;
                    }
                    Event::FocusGained if focus_state == FocusState::Away => {
                        focus_state = FocusState::Returned;
                        needs_redraw = true;
                    }
                    _ => {}
                }
            }
        }
    })();

    rata_clean()?;
    res
}
