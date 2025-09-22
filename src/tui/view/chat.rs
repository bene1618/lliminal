use lliminal::llm::{AssistantMessageContent, AssistantMessagePart, UserMessageContent, UserMessagePart};
use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Position, Rect}, style::{Style, Stylize}, text::Line, widgets::{Block, Paragraph, Widget}};
use tokio::sync::watch;
use tui_input::Input;

use crate::tui::viewmodel::{AppState, Chat};

pub struct ChatWidget {
    pub app_state: watch::Sender<AppState>,
    pub chat: watch::Receiver<Chat>,
    pub chat_input: watch::Receiver<Input>
}

impl Widget for &ChatWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [messages_area, input_area] = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(3),
        ]).areas(area);
        self.render_messages(messages_area, buf);
        self.render_input(input_area, buf);
    }
}

impl ChatWidget {
    fn render_messages(&self, area: Rect, buf: &mut Buffer) {
        let scroll = self.chat.borrow().scroll;
        let mut y = area.y + area.height - 1;
        for line in self.chat.borrow().messages.iter().flat_map(|msg| match msg {
            lliminal::llm::Message::User { parts } => user_message_lines(parts, area.width),
            lliminal::llm::Message::Assistant { parts } => assistant_message_lines(parts, area.width)
        }).rev().skip(scroll) {
            if y < area.y {
                break;
            }
            line.render(Rect { x: area.x, y, width: area.width, height: 1 }, buf);
            if y == 0 {
                break;
            }
            y -= 1;
        }
    }

    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        let chat_input = self.chat_input.borrow().clone();
        let width = area.width.saturating_sub(3);
        let scroll = chat_input.visual_scroll(width as usize);

        let input = Paragraph::new(chat_input.value())
            .scroll((0, u16::try_from(scroll).unwrap()))
            .block(Block::bordered().title("Input"));

        input.render(area, buf);

        if self.chat.borrow().user_input {
            self.app_state.send_modify(|app_state| {
                app_state.cursor_position = Some(Position::from((
                    area.x + u16::try_from(chat_input.visual_cursor().saturating_sub(scroll)).unwrap() + 1,
                    area.y + 1
                )));
            });
        } else {
            self.app_state.send_modify(|state| { state.cursor_position = None; });
        }
    }
}

fn user_message_lines(parts: &[UserMessagePart], width: u16) -> Vec<Line> {
    let text = parts.iter().map(|UserMessagePart { content }| {
        match content {
            UserMessageContent::Text { text } => "> ".to_owned() + text,
        }
    }).collect::<Vec<_>>().join("\n");
    into_formatted_lines(&text, width, &Style::default().italic())
}

fn assistant_message_lines(parts: &[AssistantMessagePart], width: u16) -> Vec<Line> {
    let text = parts.iter().map(|AssistantMessagePart { content, complete }| {
        match content {
            AssistantMessageContent::Text { text } => text.clone() + if *complete { "" } else { " ..." },
        }
    }).collect::<Vec<_>>().join("\n");
    into_formatted_lines(&text, width, &Style::default())
}

fn into_formatted_lines<S>(text: &str, width: u16, style: &S) -> Vec<Line<'static>>
    where S: Into<Style> + Clone
{
    textwrap::wrap(text, width as usize).iter()
        .map(|line_cow| String::from(line_cow.clone()))
        .map(|line| Line::styled(line, style.clone()))
        .collect()
}
