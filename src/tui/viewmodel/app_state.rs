use ratatui::layout::Position;

pub struct AppState {
    pub running: bool,
    pub cursor_position: Option<Position>
}

impl Default for AppState {
    fn default() -> Self {
        Self { running: true, cursor_position: None }
    }
}
