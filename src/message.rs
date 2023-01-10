use chrono::{DateTime, Utc};

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum MessageKind {
    DANMU_MSG,
    COMBO_SEND,
    SEND_GIFT,
    INTERACT_WORD,
    NOTICE_MSG,
}

#[derive(Debug)]
pub struct Message {
    pub kind: MessageKind,
    pub content: String,
    pub author: String,
    pub date: DateTime<Utc>,
}

impl Message {
    pub fn new(kind: MessageKind, content: String, author: String, date: DateTime<Utc>) -> Self {
        Self {
            kind,
            content,
            author,
            date,
        }
    }
}
