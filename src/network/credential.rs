use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct Credential {
    pub session_data: String,
    pub bili_jct: String,
    pub buvid3: String,
}

impl Credential {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_vec(src: &Vec<String>) -> Option<Self> {
        if src.len() > 3 {
            return None;
        }

        Some(Self {
            session_data: src[0].to_owned(),
            bili_jct: src[1].to_owned(),
            buvid3: src[2].to_owned(),
        })
    }

    pub fn get_cookies(&self) -> Option<HashMap<String, String>> {
        let mut hm = HashMap::new();
        hm.insert("SESSDATA".to_owned(), self.session_data.clone());
        hm.insert("buvid3".to_owned(), self.buvid3.clone());
        hm.insert("bili_jct".to_owned(), self.bili_jct.clone());

        Some(hm)
    }
}
