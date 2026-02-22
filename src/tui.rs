use anyhow::Result;
use ratatui::DefaultTerminal;

pub fn init() -> Result<DefaultTerminal> {
    // ratatui::init() handles enable_raw_mode + EnterAlternateScreen internally
    Ok(ratatui::init())
}

pub fn restore() -> Result<()> {
    // ratatui::restore() handles disable_raw_mode + LeaveAlternateScreen internally
    ratatui::restore();
    Ok(())
}
