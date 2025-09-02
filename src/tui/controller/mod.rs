mod chat;
mod crossterm;

pub use chat::*;
pub use crossterm::*;

use tokio::sync::mpsc;

pub trait Controller<C> {
    fn handle(&self, command: C);

    fn register_self_sender(&mut self, _sender: mpsc::UnboundedSender<C>) {
    }

    fn launch(mut self) -> mpsc::UnboundedSender<C> where Self: Sized + Send + 'static, C: Send + 'static {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        self.register_self_sender(sender.clone());
        tokio::spawn(async move {
            while let Some(cmd) = receiver.recv().await {
                self.handle(cmd);
            }
        });
        sender
    }
}

