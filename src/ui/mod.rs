mod events_screen;
mod main_screen;
mod viewer_screen;

use ratatui::Frame;

use crate::app::{App, Screen};

pub fn draw(f: &mut Frame, app: &mut App) {
    match app.screen {
        Screen::Main => main_screen::draw(f, app),
        Screen::Events => events_screen::draw(f, app),
        Screen::Viewer => viewer_screen::draw(f, app),
    }
}
