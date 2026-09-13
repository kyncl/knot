use std::io::{Stdout, stdout};

use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{DisableFocusChange, EnableFocusChange},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

/// Creates new ratatui terminal and enter the terminal into alternative
/// screen with monitoring of focus
pub fn rata_init() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(EnableFocusChange)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// This will disable raw mode and leave alternative screen
pub fn rata_clean() -> Result<()> {
    disable_raw_mode()?;
    std::io::stdout().execute(DisableFocusChange)?;
    std::io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
