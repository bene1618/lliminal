use app::App;

mod app;
mod controller;
mod event;
mod view;
mod viewmodel;

pub async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::default().run(terminal).await;
    ratatui::restore();
    result
}
