use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::DefaultTerminal;
use std::io;

pub fn init() -> Result<DefaultTerminal> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let terminal = ratatui::init();
    Ok(terminal)
}

pub fn restore() -> Result<()> {
    ratatui::restore();
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
