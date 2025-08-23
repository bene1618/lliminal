use ratatui::{buffer::Buffer, layout::Rect, widgets::{Block, Paragraph, Widget}};
use tokio::sync::watch;
use tui_input::Input;

use crate::tui::viewmodel::Chat;

pub struct ChatWidget {
    pub chat: watch::Receiver<Chat>,
    pub chat_input: watch::Receiver<Input>
}

impl Widget for &ChatWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chat_input = self.chat_input.borrow();
        let input = Paragraph::new(chat_input.value())
            .block(Block::bordered().title("Input"));
        input.render(area, buf);
    }
}
