use crate::request::Request;
use crate::Credential;
use std::collections::HashMap;

#[derive(Debug)]
pub struct LiveRoom {
    room_display_id: i64,
    credential: Credential,
}

impl LiveRoom {
    pub fn new(room_display_id: i64, credential: Credential) -> Self {
        Self {
            room_display_id,
            credential,
        }
    }

    pub async fn send_normal_danmaku(&self, danmaku_text: &str) {
        let mut danmaku = HashMap::new();
        let timestamp = (chrono::Utc::now() + chrono::Duration::hours(8)).timestamp();

        danmaku.insert("roomid".to_owned(), self.room_display_id.to_string());
        danmaku.insert("color".to_owned(), 0xffffff.to_string());
        danmaku.insert("fontsize".to_owned(), 25.to_string());
        danmaku.insert("mode".to_owned(), 1.to_string());
        danmaku.insert("msg".to_owned(), danmaku_text.to_owned());
        danmaku.insert("rnd".to_owned(), timestamp.to_string());
        danmaku.insert("bubble".to_owned(), 0.to_string());

        Request::send(
            "POST",
            "https://api.live.bilibili.com/msg/send",
            None,
            Some(&mut danmaku),
            Some(&self.credential),
            false,
        )
        .await
        .unwrap();
    }
}
