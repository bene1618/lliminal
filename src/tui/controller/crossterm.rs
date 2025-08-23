use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::{mpsc, watch};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::tui::viewmodel::{AppState, Chat};

use super::{ChatCommand, Controller};

pub struct CrosstermController {
    pub app_state: watch::Sender<AppState>,
    pub chat: watch::Receiver<Chat>,
    pub chat_input: watch::Sender<Input>,
    pub chat_controller: mpsc::UnboundedSender<ChatCommand>,
}

impl Controller<Event> for CrosstermController {
    fn handle(&self, event: Event) {
        let chat = self.chat.borrow();
        match event {
            Event::Key(
                KeyEvent { modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('c' | 'C'), .. }
            ) => self.app_state.send_modify(|state| { state.running = false; }),
            Event::Key(
                KeyEvent { modifiers: KeyModifiers::NONE, code: KeyCode::Enter, .. }
            ) if chat.user_input => self.chat_controller.send(ChatCommand::Submit).unwrap(),
            _ if chat.user_input => self.chat_input.send_modify(|input| {
                input.handle_event(&event);
            }),
            _ => {}
        }
    }
}
