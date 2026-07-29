use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{
        self, DisableFocusChange, EnableFocusChange, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
};
use std::path::{Path, PathBuf};

use crate::{USER_AWAY_MSG, USER_CAMEBACK_MSG};

pub enum ResolverFiles {
    Archiving,
    SourceRemote,
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
    /// Recover | Source
    pub first: Vec<PathBuf>,
    /// Remove  | Remote
    pub second: Vec<PathBuf>,
    /// Ignore  | Skip
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
                title: "SYNC RESOLVER",
                title_color: Color::LightBlue,
            },
        }
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

    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    std::io::stdout().execute(EnableFocusChange)?;

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut table_state = TableState::default();
    table_state.select(Some(0));
    let mut needs_redraw = true;

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

    let mut focus_state = FocusState::Normal;

    loop {
        if needs_redraw {
            terminal.draw(|frame| {
                let chunks = Layout::vertical([
                    Constraint::Length(1), // Top Statusline
                    Constraint::Min(0),    // Table with Borders
                    Constraint::Length(1), // Footer Keymap
                ])
                .split(frame.area());

                let first_count = destinations
                    .iter()
                    .filter(|&&d| d == FileDestination::First)
                    .count();
                let second_count = destinations
                    .iter()
                    .filter(|&&d| d == FileDestination::Second)
                    .count();
                let skip_count = destinations.len() - first_count - second_count;

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
                    Span::styled(root_path_str.clone(), Style::default().fg(Color::DarkGray)),
                    Span::raw("   "),
                    Span::styled(config.first_icon, Style::default().fg(config.first_color)),
                    Span::styled(
                        format!(" {} {} ", config.first_label, first_count),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("· ", Style::default().fg(Color::DarkGray)),
                    Span::styled(config.second_icon, Style::default().fg(config.second_color)),
                    Span::styled(
                        format!(" {} {} ", config.second_label, second_count),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("· ", Style::default().fg(Color::DarkGray)),
                    Span::styled(config.skip_icon, Style::default().fg(config.skip_color)),
                    Span::styled(
                        format!(" {} {} ", config.skip_label, skip_count),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]);
                frame.render_widget(header_line, chunks[0]);

                // Table Rows
                let mut rows = Vec::with_capacity(files.len());
                for (i, path) in files.iter().enumerate() {
                    let (icon, label, color) = match destinations[i] {
                        FileDestination::First => {
                            (config.first_icon, config.first_label, config.first_color)
                        }
                        FileDestination::Second => {
                            (config.second_icon, config.second_label, config.second_color)
                        }
                        FileDestination::Skip => {
                            (config.skip_icon, config.skip_label, config.skip_color)
                        }
                    };

                    let status_badge = Line::from(vec![
                        Span::styled(format!(" {} ", icon), Style::default().fg(color)),
                        Span::styled(
                            format!("{:<8}", label),
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                    ]);

                    let path_style = if destinations[i] == FileDestination::Skip {
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
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::DarkGray)),
                    )
                    .row_highlight_style(
                        Style::default()
                            .bg(Color::Rgb(35, 35, 45))
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(" ❯ ");
                frame.render_stateful_widget(table, chunks[1], &mut table_state);

                // Scrollbar
                let term_height = chunks[1].height.saturating_sub(2) as usize;
                if files.len() > term_height {
                    let selected_idx = table_state.selected().unwrap_or(0);
                    let mut scrollbar_state =
                        ScrollbarState::new(files.len().saturating_sub(1)).position(selected_idx);
                    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                        .thumb_symbol("█")
                        .style(Style::default().fg(Color::DarkGray));

                    let scrollbar_area = Rect {
                        x: chunks[1].right().saturating_sub(1),
                        y: chunks[1].top() + 1,
                        width: 1,
                        height: chunks[1].height.saturating_sub(2),
                    };
                    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
                }

                let hint = Line::from(vec![
                    Span::styled(
                        "  Space",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" cycle  ·  "),
                    Span::styled(
                        "1/2/3",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" set  ·  "),
                    Span::styled(
                        "Shift+↑/↓",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" paint  ·  "),
                    Span::styled(
                        "Shift+1/2/3",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" all  ·  "),
                    Span::styled(
                        "Enter",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" confirm  ·  "),
                    Span::styled(
                        "Esc/q",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" cancel"),
                ]);
                let footer_chunks =
                    Layout::horizontal([Constraint::Min(0), Constraint::Length(28)])
                        .split(chunks[2]);
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

        let first_event = event::read()?;
        let mut events = vec![first_event];
        while event::poll(std::time::Duration::from_millis(0))? {
            events.push(event::read()?);
        }

        let status = handle_input(
            &events,
            &mut needs_redraw,
            files.len(),
            &mut table_state,
            &mut destinations,
            &mut focus_state,
        );

        if let Some(status) = status {
            cleanup_terminal()?;
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

    cleanup_terminal()?;

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

fn cleanup_terminal() -> Result<()> {
    disable_raw_mode()?;
    std::io::stdout().execute(DisableFocusChange)?;
    std::io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_input(
    events: &[Event],
    needs_redraw: &mut bool,
    file_count: usize,
    table_state: &mut TableState,
    destinations: &mut [FileDestination],
    focus_state: &mut FocusState,
) -> Option<bool> {
    for event in events {
        match event {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Release {
                    let selected = table_state.selected().unwrap_or(0);
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
                            let next_idx = if selected > 0 {
                                selected - 1
                            } else {
                                file_count - 1
                            };
                            if has_shift {
                                destinations[next_idx] = destinations[selected];
                            }
                            table_state.select(Some(next_idx));
                        }
                        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                            let next_idx = if selected < file_count - 1 {
                                selected + 1
                            } else {
                                0
                            };
                            if has_shift {
                                destinations[next_idx] = destinations[selected];
                            }
                            table_state.select(Some(next_idx));
                        }
                        KeyCode::PageDown => {
                            if file_count > 0 {
                                let h = crossterm::terminal::size()
                                    .map(|s| s.1 as usize)
                                    .unwrap_or(0);
                                table_state.select(Some((selected + h).min(file_count - 1)));
                            }
                        }
                        KeyCode::PageUp => {
                            if file_count > 0 {
                                let h = crossterm::terminal::size()
                                    .map(|s| s.1 as usize)
                                    .unwrap_or(0);
                                table_state.select(Some(selected.saturating_sub(h)));
                            }
                        }
                        KeyCode::Tab | KeyCode::Char(' ') => {
                            destinations[selected] = match destinations[selected] {
                                FileDestination::First => FileDestination::Second,
                                FileDestination::Second => FileDestination::Skip,
                                FileDestination::Skip => FileDestination::First,
                            };
                        }
                        KeyCode::Char('1') => destinations[selected] = FileDestination::First,
                        KeyCode::Char('2') => destinations[selected] = FileDestination::Second,
                        KeyCode::Char('3')
                        | KeyCode::Char('0')
                        | KeyCode::Backspace
                        | KeyCode::Delete => {
                            destinations[selected] = FileDestination::Skip;
                        }
                        KeyCode::Char('!') => destinations.fill(FileDestination::First),
                        KeyCode::Char('@') => destinations.fill(FileDestination::Second),
                        KeyCode::Char('#') | KeyCode::Char(')') => {
                            destinations.fill(FileDestination::Skip)
                        }
                        _ => {}
                    }
                    *needs_redraw = true;
                }
            }
            Event::Resize(..) => {
                *needs_redraw = true;
            }
            Event::FocusLost => {
                *focus_state = FocusState::Away;
                *needs_redraw = true;
            }
            Event::FocusGained => {
                if *focus_state == FocusState::Away {
                    *focus_state = FocusState::Returned;
                    *needs_redraw = true;
                }
            }
            _ => {}
        }
    }
    None
}
