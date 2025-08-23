use ratatui::{buffer::Buffer, layout::{Alignment, Rect}, style::{Color, Stylize}, widgets::{Block, BorderType, Paragraph, Widget}};
use tokio::sync::watch;

use crate::tui::viewmodel::Count;

pub struct CounterWidget {
    pub count: watch::Receiver<Count>
}

impl Widget for &CounterWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("event-driven-async-generated")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let text = format!(
            "This is a tui template.\n\
                Press `Ctrl-C` to stop running.\n\
                Press left and right to increment and decrement the counter respectively.\n\
                Counter: {}",
            self.count.borrow().count
        );

        let paragraph = Paragraph::new(text)
            .block(block)
            .fg(Color::Cyan)
            .bg(Color::Black)
            .centered();

        paragraph.render(area, buf);
    }
}
