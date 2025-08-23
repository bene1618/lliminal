use ratatui::layout::Position;
use tokio::sync::watch;
use tui_input::Input;

use crate::tui::viewmodel::AppState;

pub struct CursorController {
    pub app_state: watch::Sender<AppState>,
    pub chat_input: watch::Receiver<Input>,
}

impl CursorController {
    pub fn launch(mut self) {
        tokio::spawn(async move {
            self.chat_input.mark_changed();
            while let Ok(_) = self.chat_input.changed().await {
                let cursor = self.chat_input.borrow_and_update().visual_cursor();
                self.app_state.send_modify(|app_state| { app_state.cursor_position = Some(Position::from((cursor as u16 + 1, 1))); });
            }
        });
    }
}
