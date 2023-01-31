use chrono::{DateTime, Utc};

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum MessageKind {
    DANMU_MSG,
    COMBO_SEND,
    SEND_GIFT,
    INTERACT_WORD,
    NOTICE_MSG,
    SUPER_CHAT_MESSAGE,
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

    // If the SC still valid, return Some(..), otherwise return None.
    //
    // @Return: Option<(content, author, endtime, left_display_time)>
    pub fn parse_sc(&self) -> Option<(&str, &str, i64, i64)> {
        match self.kind {
            MessageKind::SUPER_CHAT_MESSAGE => {
                let data = self.author.as_str().split(",").collect::<Vec<_>>();
                let (author, _, endtime) = (data[0], data[1], data[2].parse::<i64>().unwrap());
                let left_display_time = endtime - chrono::Utc::now().timestamp();
                if left_display_time <= 0 {
                    None
                } else {
                    Some((self.content.as_str(), author, endtime, left_display_time))
                }
            }
            _ => None,
        }
    }
}
