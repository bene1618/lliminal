use ratatui::layout::Position;
use tokio::sync::watch;
use tui_input::Input;

use crate::tui::viewmodel::{AppState, Chat};

pub struct CursorController {
    pub app_state: watch::Sender<AppState>,
    pub chat: watch::Receiver<Chat>,
    pub chat_input: watch::Receiver<Input>,
}

impl CursorController {
    pub fn launch(mut self) {
        tokio::spawn(async move {
            self.chat_input.mark_changed();
            while let Ok(_) = tokio::select! {
                c = self.chat_input.changed() => c,
                c = self.chat.changed() => c
            } {
                let chat_input = self.chat_input.borrow_and_update();
                if self.chat.borrow_and_update().user_input {
                    self.app_state.send_modify(|app_state| { app_state.cursor_position = Some(Position::from((chat_input.visual_cursor() as u16 + 1, 1))); });
                } else {
                    self.app_state.send_modify(|app_state| { app_state.cursor_position = None; });
                }
            }
        });
    }
}
