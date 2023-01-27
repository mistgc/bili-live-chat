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

    pub async fn get_room_info(room_display_id: i64) -> Option<HashMap<String, String>> {
        let mut params = HashMap::new();
        params.insert("room_id".to_owned(), room_display_id.to_string());
        let resp = Request::send(
            "GET",
            "https://api.live.bilibili.com/xlive/web-room/v1/index/getInfoByRoom",
            Some(&params),
            None,
            None,
            true,
        )
        .await;

        match resp {
            Ok(data) => {
                let value: serde_json::Value =
                    serde_json::from_str(data.text().await.unwrap().as_str()).unwrap();
                let mut out = HashMap::new();
                out.insert(
                    "uid".to_owned(),
                    value["data"]["room_info"]["uid"]
                        .to_string()
                        .trim_start_matches("\"")
                        .trim_end_matches("\"")
                        .to_string(),
                );
                out.insert(
                    "room_id".to_owned(),
                    value["data"]["room_info"]["room_id"]
                        .to_string()
                        .trim_start_matches("\"")
                        .trim_end_matches("\"")
                        .to_string(),
                );
                out.insert(
                    "title".to_owned(),
                    value["data"]["room_info"]["title"]
                        .to_string()
                        .trim_start_matches("\"")
                        .trim_end_matches("\"")
                        .to_string(),
                );
                out.insert(
                    "tags".to_owned(),
                    value["data"]["room_info"]["tags"]
                        .to_string()
                        .trim_start_matches("\"")
                        .trim_end_matches("\"")
                        .to_string(),
                );
                out.insert(
                    "description".to_owned(),
                    value["data"]["room_info"]["description"]
                        .to_string()
                        .trim_start_matches("\"")
                        .trim_end_matches("\"")
                        .to_string(),
                );
                out.insert(
                    "area_name".to_owned(),
                    value["data"]["room_info"]["area_name"]
                        .to_string()
                        .trim_start_matches("\"")
                        .trim_end_matches("\"")
                        .to_string(),
                );
                out.insert(
                    "parent_area_name".to_owned(),
                    value["data"]["room_info"]["parent_area_name"]
                        .to_string()
                        .trim_start_matches("\"")
                        .trim_end_matches("\"")
                        .to_string(),
                );
                out.insert(
                    "live_start_time".to_owned(),
                    value["data"]["room_info"]["live_start_time"]
                        .to_string()
                        .trim_start_matches("\"")
                        .trim_end_matches("\"")
                        .to_string(),
                );
                out.insert(
                    "watched_show".to_owned(),
                    value["data"]["watched_show"]["num"]
                        .to_string()
                        .trim_start_matches("\"")
                        .trim_end_matches("\"")
                        .to_string(),
                );
                out.insert(
                    "attention".to_owned(),
                    value["data"]["anchor_info"]["relation_info"]["attention"]
                        .to_string()
                        .trim_start_matches("\"")
                        .trim_end_matches("\"")
                        .to_string(),
                );
                out.insert(
                    "uname".to_owned(),
                    value["data"]["anchor_info"]["base_info"]["uname"]
                        .to_string()
                        .trim_start_matches("\"")
                        .trim_end_matches("\"")
                        .to_string(),
                );

                Some(out)
            }
            Err(_) => None,
        }
    }
}
