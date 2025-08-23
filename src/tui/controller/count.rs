use tokio::sync::watch;

use crate::tui::viewmodel::Count;

use super::Controller;

pub struct UpdateCount {
    pub count: watch::Sender<Count>
}

impl Controller<UpdateCountCommand> for UpdateCount {
    fn handle(&self, event: UpdateCountCommand) {
        match event {
            UpdateCountCommand::Increase => self.count.send_modify(|c| { c.count = c.count.saturating_add(1) }),
            UpdateCountCommand::Decrease => self.count.send_modify(|c| { c.count = c.count.saturating_sub(1) }),
        }
    }
}

pub enum UpdateCountCommand {
    Increase,
    Decrease
}

