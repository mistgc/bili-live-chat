use bili_live_chat::App;
use bili_live_chat::Credential;

#[tokio::main]
async fn main() {
    let mut cret = Credential::new();
    cret.session_data = "your session_data".to_owned();
    cret.bili_jct = "your bili_jct".to_owned();
    cret.buvid3 = "your buvid3".to_owned();

    // 3044248 魔法Zc目录 直播间
    let mut app = App::new(3044248, cret);
    app.run().await;
}
