use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Rect}, widgets::{Block, Paragraph, Widget, Wrap}};
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
        let [input_area, messages_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(1)
        ]).areas(area);

        let input = Paragraph::new(chat_input.value())
            .block(Block::bordered().title("Input"));
        input.render(input_area, buf);

        let messages = self.chat.borrow().messages.iter().rev().map(|msg| match msg {
            lliminal::llm::Message::User { parts } => {
                "User: ".to_owned() + &parts.iter().map(|p| match &p.content {
                    lliminal::llm::UserMessageContent::Text { text } => text.clone(),
                }).collect::<Vec<_>>().join("\n")
            },
            lliminal::llm::Message::Assistant { parts } => {
                "Assistant: ".to_owned() + &parts.iter().map(|p| match &p.content {
                    lliminal::llm::AssistantMessageContent::Text { text } => text.clone(),
                }).collect::<Vec<_>>().join("\n")
            }
        }).collect::<Vec<_>>().join("\n");
        let paragraph = Paragraph::new(messages)
            .block(Block::bordered().title("Messages"))
            .wrap(Wrap { trim: true });
        paragraph.render(messages_area, buf);
    }
}
