#![allow(dead_code)]

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::{
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};
use tokio_tungstenite as tungstenite;
use tungstenite::{tungstenite::protocol::Message, MaybeTlsStream};

use crate::utils;

type WebSocketStream = tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;

pub trait Pack {
    fn into_bytes(self) -> Vec<u8>;

    fn from_bytes<'de>(data: Vec<u8>) -> Self
    where
        Self: Sized + Deserialize<'de>,
    {
        let de_json_raw: serde_json::Value = serde_json::from_slice(data.as_slice()).unwrap();
        Self::deserialize(de_json_raw).unwrap()
    }
}

#[derive(Debug, Default)]
pub struct DanmuClient {
    client: reqwest::Client,                                 /* Http Client */
    auth: CookieAuth,                                        /* Cookie Auth */
    room_id: u32,                                            /* Room ID */
    token: String,                                           /* Token */
    host_list: Vec<HostServer>,                              /* Danmu Host Server List */
    host_index: u8, /* Index of Danmu Host Server Connected */
    conn_write: Option<SplitSink<WebSocketStream, Message>>, /* Connection with Danmu Host Server */
    conn_read: Option<SplitStream<WebSocketStream>>, /* Connection with Danmu Host Server */
    mpsc_tx: Option<Sender<Message>>, /* Channel Sender */
    mpsc_rx: Option<Receiver<Message>>, /* Channel Receiver */
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
    Auth, /* Authentication Pack */
    AuthResp, /* Authentication Response Pack */
          // HeartBeat,     /* Heart Beat Pack */
          // HeartBeatResp, /* Heart Beat Response Pack */
          // Normal,        /* Normal Pack */
}

#[derive(Debug, Deserialize, Serialize, Default)]
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
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct AuthRespPack {}

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
                "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?id={}&type=0",
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
        // create a channel
        let (tx, rx) = mpsc::channel(512);
        self.mpsc_tx = Some(tx);
        self.mpsc_rx = Some(rx);

        Ok(())
    }

    async fn shake_hands(&mut self) {
        for (i, j) in self.host_list.iter().enumerate() {
            match tungstenite::connect_async(format!("wss://{}:{}/sub", j.host, j.wss_port)).await {
                Ok(conn_raw) => {
                    let (write, read) = conn_raw.0.split();
                    self.conn_write = Some(write);
                    self.conn_read = Some(read);
                    self.host_index = i as u8;
                    println!("Danmu Host Server Response: {:?}", conn_raw.1);
                    break;
                }
                Err(e) => eprintln!("{:#?}", e),
            }
        }
    }

    async fn send(&mut self, data: &[u8]) {
        self.conn_write
            .as_mut()
            .unwrap()
            .send(Message::from(data))
            .await
            .unwrap();
    }

    async fn read(&mut self) -> Vec<u8> {
        let res = self
            .conn_read
            .as_mut()
            .unwrap()
            .next()
            .await
            .unwrap()
            .unwrap()
            .into_data();

        res
    }

    pub async fn send_auth(&mut self) -> Result<(), reqwest::Error> {
        let auth_pack_body =
            AuthPack::new(0, self.room_id, 3, "web".to_owned(), 2, self.token.clone());
        let ser_body = serde_json::to_vec(&auth_pack_body).unwrap();
        let mut auth_pack: Vec<u8> = vec![0; ser_body.len() + 16];

        utils::fill_datapack_header(DataPack::Auth, auth_pack.as_mut_slice(), 1);
        let mut offset = 16;
        for byte in ser_body.as_slice() {
            auth_pack[offset] = *byte;
            offset += 1;
        }

        self.send(&auth_pack).await;
        println!(
            "auth_resp_pack: {:?}",
            std::str::from_utf8(&self.read().await.as_slice()[16..])
        );

        Ok(())
    }

    pub async fn connect(&mut self) -> Result<(), reqwest::Error> {
        // initialize danmu client
        self.init_client().await?;

        // shake hands
        self.shake_hands().await;

        // send authentication pack
        self.send_auth().await.unwrap();

        Ok(())
    }
}

impl AuthPack {
    pub fn new(
        _uid: u32,
        _room_id: u32,
        _proto_ver: u8,
        _platform: String,
        _type: u32,
        _key: String,
    ) -> Self {
        Self {
            _uid,
            _room_id,
            _proto_ver,
            _platform,
            _type,
            _key,
        }
    }
}

impl Pack for AuthPack {
    fn into_bytes(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

impl Pack for AuthRespPack {
    fn into_bytes(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}
