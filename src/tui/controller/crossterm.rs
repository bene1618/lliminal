use crossterm::event::Event;
use tokio::sync::mpsc;

use super::{count::UpdateCountCommand, Controller};

pub struct CrosstermController {
    pub update_count: mpsc::UnboundedSender<UpdateCountCommand>
}

impl Controller<Event> for CrosstermController {
    fn handle(&self, event: Event) {
        match event {
            Event::Key(key_event) => {
                match key_event.code {
                    crossterm::event::KeyCode::Left => { self.update_count.send(UpdateCountCommand::Decrease).unwrap() },
                    crossterm::event::KeyCode::Right => { self.update_count.send(UpdateCountCommand::Increase).unwrap() },
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
