use bili_live_chat::client::CookieAuth;
use bili_live_chat::client::DanmuClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = DanmuClient::new(3, CookieAuth::default());
    let resp = app.connect().await?;
    Ok(())
}
