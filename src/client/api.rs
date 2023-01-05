use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug)]
struct Api {
    base: String,
    endpoint: String,
    method: String,
    params: Option<HashMap<String, String>>,
}

impl Api {
    pub fn new(
        base: String,
        endpoint: String,
        method: String,
        params: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            base,
            endpoint,
            method,
            params,
        }
    }
}

#[derive(Debug)]
enum BiliUrl {
    BiliLiveUrl,
}

impl BiliUrl {
    pub fn get_url(variant: BiliUrl) -> String {
        match variant {
            BiliUrl::BiliLiveUrl => "https://api.live.bilibili.com/".to_owned(),
        }
    }
}
