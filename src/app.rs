#![allow(dead_code)]

use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use std::{io::Stdout, sync::Arc, time::Duration};
use tokio::sync::{mpsc, Mutex};
use tui::{backend::CrosstermBackend, Terminal};

use crate::{client, UI};

pub struct App {
    ui: Arc<Mutex<UI<CrosstermBackend<Stdout>>>>, /* UI */
    danmu_client: Arc<Mutex<client::DanmuClient>>, /* danmu client */
                                                  /* Todo: config */
}

impl App {
    pub fn new(room_id: u32, cookie: client::CookieAuth) -> Self {
        let (tx, rx) = mpsc::channel(512);

        /* setup terminal */
        let mut stdout = std::io::stdout();
        enable_raw_mode().unwrap();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let term = Terminal::new(backend).unwrap();

        let client = Arc::new(Mutex::new(client::DanmuClient::new(room_id, cookie, tx)));
        let ui = Arc::new(Mutex::new(UI::new(term, rx)));

        Self {
            danmu_client: client,
            ui,
        }
    }

    pub async fn run(&mut self) {
        self.danmu_client.lock().await.connect().await.unwrap();
        let client1 = self.danmu_client.clone();
        let beat = tokio::spawn(async move {
            loop {
                client1.lock().await.send_heart_beat().await;
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });
        let client2 = self.danmu_client.clone();
        let recv_msg = tokio::spawn(async move {
            loop {
                client2.lock().await.receive().await;
                tokio::time::sleep(Duration::from_secs_f32(0.5)).await;
            }
        });
        let ui = self.ui.clone();
        let draw_ui = tokio::spawn(async move {
            ui.lock().await.run().unwrap();
        });
        tokio::join!(beat, recv_msg, draw_ui).0.unwrap();
    }
}
