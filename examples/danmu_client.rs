use bili_live_chat::{client::DanmakuClient, Message};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, _) = mpsc::channel::<Message>(512);
    let mut app = DanmakuClient::new(3, tx);
    let _ = app.connect().await?;
    Ok(())
}
