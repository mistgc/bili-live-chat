#![allow(dead_code)]

use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use std::{collections::HashMap, io::Stdout, sync::Arc, time::Duration};
use tokio::sync::{mpsc, Mutex};
use tui::{backend::CrosstermBackend, Terminal};

use crate::{api, client, config::Config, Message, UI};

pub struct App {
    ui: Arc<Mutex<UI<CrosstermBackend<Stdout>>>>,    /* UI */
    danmu_client: Arc<Mutex<client::DanmakuClient>>, /* danmu client */
    config: Arc<Mutex<Config>>,                      /* config */
    room_id: u32,                                    /* room id */
    msg_tx: mpsc::Sender<Message>,                   /* sender for message */
    rm_info_tx: mpsc::Sender<HashMap<String, String>>, /* sender for room information */
    rank_info_tx: mpsc::Sender<Vec<String>>,         /* sender for rank info */
}

impl App {
    pub async fn new(room_id: u32, config: Config) -> Self {
        let conf = Arc::new(Mutex::new(config));
        let (msg_tx, msg_rx) = mpsc::channel(512);
        let (rm_info_tx, rm_info_rx) = mpsc::channel(4);
        let (rank_info_tx, rank_info_rx) = mpsc::channel(4);

        /* setup terminal */
        let mut stdout = std::io::stdout();
        enable_raw_mode().unwrap();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let term = Terminal::new(backend).unwrap();

        let client = Arc::new(Mutex::new(client::DanmakuClient::new(
            room_id,
            msg_tx.clone(),
        )));
        let ui = Arc::new(Mutex::new(
            UI::new(
                term,
                msg_rx,
                rm_info_rx,
                rank_info_rx,
                room_id as i64,
                conf.clone(),
            )
            .await,
        ));

        Self {
            ui,
            danmu_client: client,
            config: conf.clone(),
            room_id,
            msg_tx,
            rm_info_tx,
            rank_info_tx,
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
                tokio::time::sleep(Duration::from_secs_f32(0.3)).await;
            }
        });

        let ui = self.ui.clone();
        let draw_ui = tokio::spawn(async move {
            ui.lock().await.run().await.unwrap();
        });

        let rm_info_tx = self.rm_info_tx.clone();
        let rank_info_tx = self.rank_info_tx.clone();
        let room_id = self.room_id;
        let sync_room_info = tokio::spawn(async move {
            loop {
                if let Some(data) = api::live::LiveRoom::get_room_info(room_id as i64).await {
                    let ruid = data["ruid"].parse().unwrap();
                    rm_info_tx.send(data).await.unwrap();
                    if let Some(data) =
                        api::live::LiveRoom::get_rank_info_first_50(room_id as i64, ruid).await
                    {
                        rank_info_tx.send(data).await.unwrap();
                    }
                }

                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });

        tokio::join!(beat, recv_msg, draw_ui, sync_room_info)
            .0
            .unwrap();
    }
}
