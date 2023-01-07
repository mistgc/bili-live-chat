use bili_live_chat::{client::CookieAuth, App};

#[tokio::main]
async fn main() {
    let mut app = App::new(3, CookieAuth::default());
    app.run().await;
}
