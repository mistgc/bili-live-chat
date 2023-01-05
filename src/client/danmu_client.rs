#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite as tungstenite;
use tungstenite::MaybeTlsStream;

type WebSocketStream = tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Default)]
pub struct DanmuClient {
    client: reqwest::Client,       /* Http Client */
    auth: CookieAuth,              /* Cookie Auth */
    room_id: u32,                  /* Room ID */
    token: String,                 /* Token */
    host_list: Vec<HostServer>,    /* Danmu Host Server List */
    host_index: u8,                /* Index of Danmu Host Server Connected */
    conn: Option<WebSocketStream>, /* Connection with Danmu Host Server */
}

#[derive(Debug, Default)]
pub struct CookieAuth {
    dede_user_id: String,
    dede_user_id_ckmd5: String,
    sess_data: String,
    bili_jct: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HostServer {
    host: String,
    port: u32,
    ws_port: u32,
    wss_port: u32,
}

#[derive(Debug)]
pub enum DataPack {
    Auth,          /* Authentication Pack */
    AuthResp,      /* Authentication Response Pack */
    HeartBeat,     /* Heart Beat Pack */
    HeartBeatResp, /* Heart Beat Response Pack */
    Normal,        /* Normal Pack */
}

#[derive(Debug, Serialize)]
pub struct AuthPack {
    #[serde(rename = "uid")]
    _uid: u32,
    #[serde(rename = "roomid")]
    _room_id: u32,
    #[serde(rename = "protover")]
    _proto_ver: u8,
    #[serde(rename = "platform")]
    _platform: String,
    #[serde(rename = "type")]
    _type: u32,
    #[serde(rename = "key")]
    _key: String,
}

impl DanmuClient {
    pub fn new(room_id: u32, auth: CookieAuth) -> Self {
        Self {
            room_id,
            auth,
            ..Default::default()
        }
    }

    async fn init_client(&mut self) -> Result<(), reqwest::Error> {
        // send request to get token
        let resp = self
            .client
            .get(format!(
                "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?id={}",
                self.room_id
            ))
            .send()
            .await?;
        // convert to 'serde_json::Value' instance
        let json: serde_json::Value = serde_json::from_str(&resp.text().await?).unwrap();
        let token = json["data"]["token"].as_str().unwrap();
        self.token = token.to_owned();
        // extract the danmu host server list and append into 'self.host_list'
        let host_list_raw = json["data"]["host_list"].as_array().unwrap();
        for obj in host_list_raw {
            self.host_list.push(HostServer::deserialize(obj).unwrap());
        }
        Ok(())
    }

    async fn shake_hands(&mut self) {
        for (i, j) in self.host_list.iter().enumerate() {
            match tungstenite::connect_async(format!("wss://{}:{}/sub", j.host, j.wss_port)).await {
                Ok(conn_raw) => {
                    self.conn = Some(conn_raw.0);
                    self.host_index = i as u8;
                    println!("Danmu Host Server Response: {:?}", conn_raw.1);
                    break;
                }
                Err(e) => eprintln!("{:#?}", e),
            }
        }
    }

    pub async fn connect(&mut self) -> Result<(), reqwest::Error> {
        // initialize danmu client
        self.init_client().await?;

        // shake hands
        self.shake_hands().await;
        Ok(())
    }
}
