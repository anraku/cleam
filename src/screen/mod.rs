pub mod event_search;
pub mod events;
pub mod group_events;
pub mod main;
pub mod viewer;

pub use event_search::EventSearchScreen;
pub use events::EventsScreen;
pub use group_events::GroupEventsScreen;
pub use main::MainScreen;
pub use viewer::ViewerScreen;

use crate::app::LogEvent;

pub enum ScreenAction {
    None,
    Quit,
    Navigate(NavigateTo),
}

pub enum NavigateTo {
    NewEvents {
        group_name: String,
        stream_name: String,
    },
    NewViewer {
        event: LogEvent,
    },
    NewEventSearch {
        group_name: String,
    },
    NewGroupEvents {
        group_name: String,
        start_ms: Option<i64>,
        end_ms: Option<i64>,
        pattern: Option<String>,
        start_display: String,
        end_display: String,
        pattern_display: String,
    },
    Restore(Box<CurrentScreen>),
}

pub enum CurrentScreen {
    Main(MainScreen),
    Events(EventsScreen),
    Viewer(ViewerScreen),
    EventSearch(EventSearchScreen),
    GroupEvents(GroupEventsScreen),
    Transitioning,
}
