mod app;
mod aws;
mod screen;
mod tui;
mod ui;

use anyhow::Result;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    let client = aws::build_client().await?;
    let mut app = App::new(client);
    let mut terminal = tui::init()?;
    let result = app.run(&mut terminal).await;
    tui::restore()?;
    result
}
