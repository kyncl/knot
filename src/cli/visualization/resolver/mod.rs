use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
};
use std::path::{Path, PathBuf};

use crate::{
    USER_AWAY_MSG, USER_CAMEBACK_MSG,
    cli::visualization::rata_utils::{rata_clean, rata_init},
};

pub enum ResolverFiles {
    Archiving,
    SourceRemote,
    UniqueHandle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusState {
    Normal,
    Away,
    Returned,
}

#[derive(Copy, Clone, PartialEq)]
enum FileDestination {
    First,
    Second,
    Skip,
}

pub struct ResolvedFiles {
    /// Recover | Source | Upload
    pub first: Vec<PathBuf>,
    /// Remove  | Remote | Delete
    pub second: Vec<PathBuf>,
    /// Ignore  | Skip | Skip
    pub skipped: Vec<PathBuf>,
}

struct ResolverConfig {
    first_label: &'static str,
    first_color: Color,
    first_icon: &'static str,
    second_label: &'static str,
    second_color: Color,
    second_icon: &'static str,
    skip_label: &'static str,
    skip_color: Color,
    skip_icon: &'static str,
    title: &'static str,
    title_color: Color,
}

impl ResolverFiles {
    fn config(&self) -> ResolverConfig {
        match self {
            ResolverFiles::Archiving => ResolverConfig {
                first_label: "Recover",
                first_color: Color::Green,
                first_icon: "●",
                second_label: "Remove",
                second_color: Color::Red,
                second_icon: "✕",
                skip_label: "Skip",
                skip_color: Color::Gray,
                skip_icon: "◌",
                title: "ARCHIVE RESOLVER",
                title_color: Color::Yellow,
            },
            ResolverFiles::SourceRemote => ResolverConfig {
                first_label: "Source",
                first_color: Color::Green,
                first_icon: "●",
                second_label: "Remote",
                second_color: Color::Cyan,
                second_icon: "◆",
                skip_label: "Ignore",
                skip_color: Color::Gray,
                skip_icon: "◌",
                title: "CONFLICT RESOLVER",
                title_color: Color::LightBlue,
            },
            ResolverFiles::UniqueHandle => ResolverConfig {
                first_label: "Upload",
                first_color: Color::Green,
                first_icon: "●",
                second_label: "Delete",
                second_color: Color::Red,
                second_icon: "✕",
                skip_label: "Skip",
                skip_color: Color::Gray,
                skip_icon: "◌",
                title: "UNIQUE RESOLVER",
                title_color: Color::LightGreen,
            },
        }
    }
}

struct ResolverState {
    selected: usize,
    offset: usize,
    visible_capacity: usize,
    first_count: usize,
    second_count: usize,
    skip_count: usize,
    focus_state: FocusState,
    needs_redraw: bool,
}

impl ResolverState {
    fn set_destination(
        &mut self,
        destinations: &mut [FileDestination],
        idx: usize,
        new_dest: FileDestination,
    ) {
        let old_dest = destinations[idx];
        if old_dest == new_dest {
            return;
        }

        match old_dest {
            FileDestination::First => self.first_count -= 1,
            FileDestination::Second => self.second_count -= 1,
            FileDestination::Skip => self.skip_count -= 1,
        }
        match new_dest {
            FileDestination::First => self.first_count += 1,
            FileDestination::Second => self.second_count += 1,
            FileDestination::Skip => self.skip_count += 1,
        }
        destinations[idx] = new_dest;
        self.needs_redraw = true;
    }
}

pub fn resolve_files<P: AsRef<Path>>(
    files: &[PathBuf],
    resolve: ResolverFiles,
    root_path: Option<P>,
) -> Result<ResolvedFiles> {
    if files.is_empty() {
        return Ok(ResolvedFiles {
            first: vec![],
            second: vec![],
            skipped: vec![],
        });
    }

    let config = resolve.config();
    let mut destinations = vec![FileDestination::Skip; files.len()];

    let mut state = ResolverState {
        selected: 0,
        offset: 0,
        visible_capacity: 20,
        first_count: 0,
        second_count: 0,
        skip_count: files.len(),
        focus_state: FocusState::Normal,
        needs_redraw: true,
    };

    let mut terminal = rata_init()?;

    let root_path_str = root_path
        .map(|p| {
            let path = format!("{}/", p.as_ref().display()).replace("//", "/");
            if let Some(home) = dirs::home_dir() {
                path.replacen(&home.display().to_string(), "~", 1)
            } else {
                path
            }
        })
        .unwrap_or_default();

    let badge_first = Line::from(vec![
        Span::styled(
            format!(" {} ", config.first_icon),
            Style::default().fg(config.first_color),
        ),
        Span::styled(
            format!("{:<8}", config.first_label),
            Style::default()
                .fg(config.first_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let badge_second = Line::from(vec![
        Span::styled(
            format!(" {} ", config.second_icon),
            Style::default().fg(config.second_color),
        ),
        Span::styled(
            format!("{:<8}", config.second_label),
            Style::default()
                .fg(config.second_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let badge_skip = Line::from(vec![
        Span::styled(
            format!(" {} ", config.skip_icon),
            Style::default().fg(config.skip_color),
        ),
        Span::styled(
            format!("{:<8}", config.skip_label),
            Style::default()
                .fg(config.skip_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    loop {
        if state.needs_redraw {
            terminal.draw(|frame| {
                let chunks = Layout::vertical([
                    Constraint::Length(3), // Top Statusline
                    Constraint::Min(0),    // Table with Borders
                    Constraint::Length(1), // Footer Keymap
                ])
                .split(frame.area());

                state.visible_capacity = chunks[1].height.saturating_sub(4).max(1) as usize;

                if state.selected < state.offset {
                    state.offset = state.selected;
                } else if state.selected >= state.offset + state.visible_capacity {
                    state.offset = state
                        .selected
                        .saturating_sub(state.visible_capacity)
                        .saturating_add(1);
                }

                let header_line = Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!(" {} ", config.title),
                        Style::default()
                            .bg(config.title_color)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(config.first_icon, Style::default().fg(config.first_color)),
                    Span::styled(
                        format!(" {} {} ", config.first_label, state.first_count),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("· ", Style::default().fg(Color::DarkGray)),
                    Span::styled(config.second_icon, Style::default().fg(config.second_color)),
                    Span::styled(
                        format!(" {} {} ", config.second_label, state.second_count),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("· ", Style::default().fg(Color::DarkGray)),
                    Span::styled(config.skip_icon, Style::default().fg(config.skip_color)),
                    Span::styled(
                        format!(" {} {} ", config.skip_label, state.skip_count),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]);
                let header_line = Paragraph::new(header_line).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default().fg(config.title_color)),
                );
                frame.render_widget(header_line, chunks[0]);

                // Windowing limit calculation
                let window_end = (state.offset + state.visible_capacity).min(files.len());
                let visible_files = &files[state.offset..window_end];
                let visible_dests = &destinations[state.offset..window_end];

                // Render Table Rows
                let mut rows = Vec::with_capacity(visible_files.len());
                for (i, path) in visible_files.iter().enumerate() {
                    let dest = visible_dests[i];

                    let status_badge = match dest {
                        FileDestination::First => badge_first.clone(),
                        FileDestination::Second => badge_second.clone(),
                        FileDestination::Skip => badge_skip.clone(),
                    };

                    let path_style = if dest == FileDestination::Skip {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    rows.push(Row::new(vec![
                        ratatui::widgets::Cell::from(status_badge),
                        ratatui::widgets::Cell::from(path.display().to_string()).style(path_style),
                    ]));
                }

                let table = Table::new(rows, [Constraint::Length(12), Constraint::Percentage(100)])
                    .header(
                        Row::new(vec!["Status", "File Path"])
                            .style(
                                Style::default()
                                    .fg(config.title_color)
                                    .add_modifier(Modifier::BOLD),
                            )
                            .bottom_margin(1),
                    )
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .title(root_path_str.clone())
                            .border_style(Style::default().fg(Color::DarkGray)),
                    )
                    .row_highlight_style(
                        Style::default()
                            .bg(Color::Rgb(35, 35, 45))
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(" ❯ ");

                let mut local_table_state = TableState::default();
                local_table_state.select(Some(state.selected.saturating_sub(state.offset)));
                frame.render_stateful_widget(table, chunks[1], &mut local_table_state);

                // Scrollbar
                if files.len() > state.visible_capacity {
                    let mut scrollbar_state =
                        ScrollbarState::new(files.len().saturating_sub(1)).position(state.selected);
                    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                        .thumb_symbol("█")
                        .style(Style::default().fg(config.title_color));
                    let scrollbar_area = Rect {
                        x: chunks[1].right().saturating_sub(1),
                        y: chunks[1].top() + 1,
                        width: 1,
                        height: chunks[1].height.saturating_sub(2),
                    };
                    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
                }

                // Footer
                let hint = Line::from(vec![
                    Span::styled(
                        "  Space",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Cycle  ·  "),
                    Span::styled(
                        "1/2/3",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Set  ·  "),
                    Span::styled(
                        "Shift+↑/↓",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Paint  ·  "),
                    Span::styled(
                        "Shift+1/2/3",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" All  ·  "),
                    Span::styled(
                        "Esc/q",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Exit  ·  "),
                    Span::styled(
                        "Enter",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Confirm"),
                ]);
                let footer_chunks =
                    Layout::horizontal([Constraint::Min(0), Constraint::Length(28)])
                        .split(chunks[2]);
                frame.render_widget(hint.dark_gray(), footer_chunks[0]);

                let (egg_txt, egg_col) = match state.focus_state {
                    FocusState::Away => (USER_AWAY_MSG, Color::Yellow),
                    FocusState::Returned => (USER_CAMEBACK_MSG, Color::LightGreen),
                    FocusState::Normal => ("", Color::Reset),
                };
                if !egg_txt.is_empty() {
                    let egg_w = Paragraph::new(Line::from(vec![
                        Span::styled(
                            egg_txt,
                            Style::default().fg(egg_col).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("  "),
                    ]))
                    .alignment(Alignment::Right);
                    frame.render_widget(egg_w, footer_chunks[1]);
                }
            })?;
            state.needs_redraw = false;
        }

        let first_event = event::read()?;
        let mut events = vec![first_event];
        while event::poll(std::time::Duration::from_millis(0))? {
            events.push(event::read()?);
        }

        if let Some(status) = handle_input(&events, files.len(), &mut destinations, &mut state) {
            rata_clean()?;
            if status {
                break;
            } else {
                return Ok(ResolvedFiles {
                    first: vec![],
                    second: vec![],
                    skipped: vec![],
                });
            }
        }
    }

    rata_clean()?;

    let mut resolved = ResolvedFiles {
        first: Vec::new(),
        second: Vec::new(),
        skipped: Vec::new(),
    };

    for (path, dest) in files.iter().zip(destinations.iter()) {
        match dest {
            FileDestination::First => resolved.first.push(path.clone()),
            FileDestination::Second => resolved.second.push(path.clone()),
            FileDestination::Skip => resolved.skipped.push(path.clone()),
        }
    }

    Ok(resolved)
}

fn handle_input(
    events: &[Event],
    file_count: usize,
    destinations: &mut [FileDestination],
    state: &mut ResolverState,
) -> Option<bool> {
    for event in events {
        match event {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                let has_shift = key.modifiers.contains(KeyModifiers::SHIFT);
                let has_control = key.modifiers.contains(KeyModifiers::CONTROL);

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('c') => {
                        if key.code == KeyCode::Char('c') && has_control
                            || key.code != KeyCode::Char('c')
                        {
                            return Some(false);
                        }
                    }
                    KeyCode::Enter => return Some(true),
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                        let next_idx = if state.selected > 0 {
                            state.selected - 1
                        } else {
                            file_count.saturating_sub(1)
                        };
                        if has_shift {
                            state.set_destination(
                                destinations,
                                next_idx,
                                destinations[state.selected],
                            );
                        }
                        state.selected = next_idx;
                        state.needs_redraw = true;
                    }
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                        let next_idx = if state.selected < file_count.saturating_sub(1) {
                            state.selected + 1
                        } else {
                            0
                        };
                        if has_shift {
                            state.set_destination(
                                destinations,
                                next_idx,
                                destinations[state.selected],
                            );
                        }
                        state.selected = next_idx;
                        state.needs_redraw = true;
                    }
                    KeyCode::PageDown => {
                        if file_count > 0 {
                            state.selected =
                                (state.selected + state.visible_capacity).min(file_count - 1);
                            state.needs_redraw = true;
                        }
                    }
                    KeyCode::PageUp => {
                        if file_count > 0 {
                            state.selected = state.selected.saturating_sub(state.visible_capacity);
                            state.needs_redraw = true;
                        }
                    }
                    KeyCode::Tab | KeyCode::Char(' ') => {
                        let next_dest = match destinations[state.selected] {
                            FileDestination::First => FileDestination::Second,
                            FileDestination::Second => FileDestination::Skip,
                            FileDestination::Skip => FileDestination::First,
                        };
                        state.set_destination(destinations, state.selected, next_dest);
                    }
                    KeyCode::Char('1') => {
                        state.set_destination(destinations, state.selected, FileDestination::First)
                    }
                    KeyCode::Char('2') => {
                        state.set_destination(destinations, state.selected, FileDestination::Second)
                    }
                    KeyCode::Char('3')
                    | KeyCode::Char('0')
                    | KeyCode::Backspace
                    | KeyCode::Delete => {
                        state.set_destination(destinations, state.selected, FileDestination::Skip);
                    }
                    KeyCode::Char('!') => {
                        destinations.fill(FileDestination::First);
                        state.first_count = file_count;
                        state.second_count = 0;
                        state.skip_count = 0;
                        state.needs_redraw = true;
                    }
                    KeyCode::Char('@') => {
                        destinations.fill(FileDestination::Second);
                        state.first_count = 0;
                        state.second_count = file_count;
                        state.skip_count = 0;
                        state.needs_redraw = true;
                    }
                    KeyCode::Char('#') | KeyCode::Char(')') => {
                        destinations.fill(FileDestination::Skip);
                        state.first_count = 0;
                        state.second_count = 0;
                        state.skip_count = file_count;
                        state.needs_redraw = true;
                    }
                    _ => {}
                }
            }
            Event::Resize(..) => state.needs_redraw = true,
            Event::FocusLost => {
                state.focus_state = FocusState::Away;
                state.needs_redraw = true;
            }
            Event::FocusGained if state.focus_state == FocusState::Away => {
                state.focus_state = FocusState::Returned;
                state.needs_redraw = true;
            }
            _ => {}
        }
    }
    None
}
