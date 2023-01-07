#![allow(dead_code)]

use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

use crate::client;

#[derive(Debug)]
pub struct App {
    danmu_client: Arc<Mutex<client::DanmuClient>>, /* danmu client */
                                                   /* Todo: config */
}

unsafe impl Send for App {}

impl App {
    pub fn new(room_id: u32, cookie: client::CookieAuth) -> Self {
        let client = Arc::new(Mutex::new(client::DanmuClient::new(room_id, cookie)));
        Self {
            danmu_client: client,
        }
    }

    pub async fn run(&mut self) {
        self.danmu_client.lock().await.connect().await.unwrap();
        let client1 = self.danmu_client.clone();
        let beat = tokio::spawn(async move {
            loop {
                client1.lock().await.send_heart_beat().await;
                println!("hello");
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });
        tokio::join!(beat);
    }
}
