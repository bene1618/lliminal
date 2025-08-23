use tokio::sync::watch;
use tui_input::Input;

use crate::tui::viewmodel::Chat;

use super::Controller;

pub struct ChatController {
    pub chat: watch::Sender<Chat>,
    pub chat_input: watch::Sender<Input>
}

impl Controller<ChatCommand> for ChatController {
    fn handle(&self, event: ChatCommand) {
        match event {
            ChatCommand::Submit => {
                let old_input = self.chat_input.send_replace(Input::default());
                self.chat.send_modify(|chat| {
                    chat.submit_user_input(old_input.value());
                });
            }
        }
    }
}

pub enum ChatCommand {
    Submit
}

