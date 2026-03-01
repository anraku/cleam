mod event_search_screen;
mod events_screen;
mod group_events_screen;
mod main_screen;
mod viewer_screen;

use ratatui::Frame;

use crate::screen::CurrentScreen;

pub fn draw(f: &mut Frame, screen: &mut CurrentScreen) {
    match screen {
        CurrentScreen::Main(s) => main_screen::draw(f, s),
        CurrentScreen::Events(s) => events_screen::draw(f, s),
        CurrentScreen::Viewer(s) => viewer_screen::draw(f, s),
        CurrentScreen::EventSearch(s) => event_search_screen::draw(f, s),
        CurrentScreen::GroupEvents(s) => group_events_screen::draw(f, s),
        CurrentScreen::Transitioning => {}
    }
}
