#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Account {
    #[serde(rename = "mid")]
    pub mid: i64,
    #[serde(rename = "uname")]
    pub uname: String,
    #[serde(rename = "userid")]
    pub user_id: String,
    #[serde(rename = "sign")]
    pub sign: String,
    #[serde(rename = "birthday")]
    pub brithday: String,
    #[serde(rename = "sex")]
    pub sex: String,
    #[serde(rename = "nick_free")]
    pub nick_free: bool,
    #[serde(rename = "rank")]
    pub rank: String,
}
