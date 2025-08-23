use crossterm::event::Event as CrosstermEvent;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget, DefaultTerminal, Frame};
use tokio::sync::{mpsc::UnboundedSender, watch};
use tui_input::Input;

use super::{controller::{ChatController, Controller, CrosstermController, CursorController}, event::{Event, EventHandler}, view::ChatWidget, viewmodel::{AppState, Chat}};


pub struct App {
    app_state: watch::Receiver<AppState>,
    event_handler: EventHandler,
    crossterm_controller: UnboundedSender<CrosstermEvent>,
    chat_widget: ChatWidget
}

impl Default for App {
    fn default() -> Self {
        // View model
        let (chat_rx, chat_tx) = watch::channel(Chat::default());
        let (chat_input_rx, chat_input_tx) = watch::channel(Input::default());
        let (app_state_rx, app_state_tx) = watch::channel(AppState::default());

        // Controller
        let chat_controller = (ChatController { chat: chat_rx, chat_input: chat_input_rx.clone() }).launch();
        let crossterm_controller = (CrosstermController { app_state: app_state_rx.clone(), chat_input: chat_input_rx, chat_controller: chat_controller.clone() }).launch();
        (CursorController { app_state: app_state_rx, chat_input: chat_input_tx.clone() }).launch();

        // View
        let chat_widget = ChatWidget { chat: chat_tx, chat_input: chat_input_tx };

        Self {
            app_state: app_state_tx,
            event_handler: EventHandler::new(),
            crossterm_controller,
            chat_widget
        }
    }
}

impl App {
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.app_state.borrow().running {
            terminal.draw(|frame| self.draw(frame))?;
            for event in self.event_handler.recv_many().await? {
                match event {
                    Event::Tick => self.tick(),
                    Event::Crossterm(event) => self.crossterm_controller.send(event)?
                }
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
        if let Some(cursor_position) = self.app_state.borrow().cursor_position {
            frame.set_cursor_position(cursor_position);
        }
    }

    fn tick(&mut self) {}
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.chat_widget.render(area, buf);
    }
}
