use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget, DefaultTerminal};
use tokio::sync::{mpsc::UnboundedSender, watch};

use super::{controller::{count::UpdateCount, crossterm::CrosstermController, Controller}, event::{Event, EventHandler}, view::CounterWidget, viewmodel::Count};

pub struct App {
    running: bool,
    event_handler: EventHandler,
    crossterm_controller: UnboundedSender<CrosstermEvent>,
    counter_widget: CounterWidget
}

impl Default for App {
    fn default() -> Self {
        // View model
        let (count_rx, count_tx) = watch::channel(Count::default());

        // Controller
        let update_count = (UpdateCount { count: count_rx }).launch();
        let crossterm_controller = (CrosstermController { update_count: update_count.clone() }).launch();

        // View
        let counter_widget = CounterWidget { count: count_tx };

        Self {
            running: true,
            event_handler: EventHandler::new(),
            crossterm_controller,
            counter_widget
        }
    }
}

impl App {
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.event_handler.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(CrosstermEvent::Key(
                    KeyEvent { modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('c' | 'C'), .. }
                )) => self.quit(),
                Event::Crossterm(event) => self.crossterm_controller.send(event)?
            }
        }
        Ok(())
    }

    fn tick(&mut self) {}

    fn quit(&mut self) {
        self.running = false;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.counter_widget.render(area, buf);
    }
}
