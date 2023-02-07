#![allow(dead_code)]

use crate::{Message, MessageKind};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, sync::mpsc::Sender};
use tokio_tungstenite as tungstenite;
use tungstenite::{tungstenite::protocol::Message as WssMessage, MaybeTlsStream};

use crate::client::Account;
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
pub struct DanmakuClient {
    client: reqwest::Client,    /* Http Client */
    account: Account,           /* BiliBili Account */
    room_id: u32,               /* Room ID */
    token: String,              /* Token */
    host_list: Vec<HostServer>, /* Danmu Host Server List */
    host_index: u8,             /* Index of Danmu Host Server Connected */
    // When the function connect() finishes, conn_write will be taken and returned to outside.
    conn_write: Option<SplitSink<WebSocketStream, WssMessage>>, /* Connection with Danmu Host Server */
    conn_read: Option<SplitStream<WebSocketStream>>, /* Connection with Danmu Host Server */
    mpsc_tx: Option<Sender<Message>>,                /* Channel Sender */
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

impl DanmakuClient {
    pub fn new(room_id: u32, mpsc_tx: Sender<Message>) -> Self {
        Self {
            room_id,
            mpsc_tx: Some(mpsc_tx),
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
                    break;
                }
                Err(e) => eprintln!("{:#?}", e),
            }
        }
    }

    async fn send(&mut self, data: &[u8]) {
        match self
            .conn_write
            .as_mut()
            .unwrap()
            .send(WssMessage::from(data))
            .await
        {
            Err(e) => eprintln!("{:#?}", e),
            _ => {}
        }
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

    pub async fn send_auth(&mut self) {
        let auth_pack_body =
            // If 'protover' is 2, the response pack will be compressed by 'zlib'.
            AuthPack::new(0, self.room_id, 2, "web".to_owned(), 2, self.token.clone());
        let ser_body = serde_json::to_vec(&auth_pack_body).unwrap();
        let mut auth_pack: Vec<u8> = vec![0; ser_body.len() + 16];

        utils::fill_datapack_header(DataPack::Auth, auth_pack.as_mut_slice(), 1);
        let mut offset = 16;
        for byte in ser_body.as_slice() {
            auth_pack[offset] = *byte;
            offset += 1;
        }

        self.send(&auth_pack).await;
    }

    pub async fn send_heart_beat(&mut self) {
        let mut beat_pack: Vec<u8> = vec![0; 16];
        utils::fill_datapack_header(DataPack::HeartBeat, beat_pack.as_mut_slice(), 1);
        self.send(&beat_pack).await;
    }

    pub async fn connect(
        &mut self,
    ) -> Result<SplitSink<WebSocketStream, WssMessage>, reqwest::Error> {
        // initialize danmu client
        self.init_client().await?;

        // shake hands
        self.shake_hands().await;

        // send authentication pack
        self.send_auth().await;

        if let Some(conn_write) = self.conn_write.take() {
            Ok(conn_write)
        } else {
            panic!("Connecting bilibili danmaku server failed.");
        }
    }

    pub async fn receive(&mut self) {
        let msg = self.read().await;
        if msg.len() >= 16 {
            if msg[7] == 2 {
                // data compressed by zlib, then need to decompressing
                let dec_data = utils::zlib_dec(&msg[16..]).unwrap();
                let packs = utils::split_packs(&dec_data);
                for p in packs {
                    self.handle_msg(p.as_slice()).await;
                }
            }
        }
    }

    async fn handle_msg(&mut self, msg: &[u8]) {
        let json: serde_json::Value = serde_json::from_slice(msg).unwrap();
        match json["cmd"].to_string().as_str() {
            "\"DANMU_MSG\"" => {
                let content = json["info"][1].to_string();
                let author = json["info"][2][1].to_string();
                let datetime = utils::timestamp_to_datetime_utc8(
                    json["info"][9]["ts"].to_string().parse().unwrap(),
                );
                let msg = Message::new(
                    MessageKind::DANMU_MSG,
                    content[1..content.len() - 1].to_owned(),
                    author[1..author.len() - 1].to_owned(),
                    datetime,
                );
                /* Send Message to Channel */
                self.mpsc_tx.as_mut().unwrap().send(msg).await.unwrap();
            }

            "\"SUPER_CHAT_MESSAGE\"" => {
                let content = json["data"]["message"]
                    .to_string()
                    .trim_start_matches("\"")
                    .trim_end_matches("\"")
                    .to_owned();
                let mut author = json["data"]["user_info"]["uname"]
                    .to_string()
                    .trim_start_matches("\"")
                    .trim_end_matches("\"")
                    .to_owned();
                // separator: ","
                // format: "username,sc_duration,endtime"
                author = author
                    + ","
                    + json["data"]["time"].to_string().as_str()
                    + ","
                    + json["data"]["end_time"].to_string().as_str();
                let datetime = utils::timestamp_to_datetime_utc8(
                    json["data"]["start_time"].to_string().parse().unwrap(),
                );
                let msg = Message::new(MessageKind::SUPER_CHAT_MESSAGE, content, author, datetime);
                /* Send Message to Channel */
                self.mpsc_tx.as_mut().unwrap().send(msg).await.unwrap();
            }

            "\"COMBO_SEND\"" => {}
            "\"SEND_GIFT\"" => {
                let action = json["data"]["action"]
                    .to_string()
                    .trim_start_matches("\"")
                    .trim_end_matches("\"")
                    .to_owned();
                let gift_name = json["data"]["giftName"]
                    .to_string()
                    .trim_start_matches("\"")
                    .trim_end_matches("\"")
                    .to_owned();
                let gift_num = json["data"]["num"].to_string();
                let uname = json["data"]["uname"]
                    .to_string()
                    .trim_start_matches("\"")
                    .trim_end_matches("\"")
                    .to_owned();
                let datetime = utils::timestamp_to_datetime_utc8(
                    json["data"]["timestamp"].to_string().parse().unwrap(),
                );
                // TODO: implment i18n
                let msg = Message::new(
                    MessageKind::SEND_GIFT,
                    action + "了" + gift_num.as_str() + "个" + gift_name.as_str(),
                    uname,
                    datetime,
                );
                /* Send Message to Channel */
                self.mpsc_tx.as_mut().unwrap().send(msg).await.unwrap();
            }
            "\"INTERACT_WORD\"" => {}
            "\"NOTICE_MSG\"" => {}
            _ => {}
        }
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
