pub mod count;
pub mod crossterm;

use tokio::sync::mpsc;

pub trait Controller<C> {
    fn handle(&self, command: C);

    fn launch(self) -> mpsc::UnboundedSender<C> where Self: Sized + Send + 'static, C: Send + 'static {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Some(cmd) = receiver.recv().await {
                self.handle(cmd);
            }
        });
        sender
    }
}

