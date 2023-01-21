use bili_live_chat::config::Config;
use bili_live_chat::App;
use bili_live_chat::Credential;

#[tokio::main]
async fn main() {
    let conf = Config {
        credential: Credential {
            session_data: "your session data".to_owned(),
            bili_jct: "your bili_jct".to_owned(),
            buvid3: "your buvid3".to_owned(),
        },
    };

    // 3044248 魔法Zc目录 直播间
    let mut app = App::new(3044248, conf).await;
    app.run().await;
}
