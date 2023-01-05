#![allow(dead_code)]

use crate::client;

#[derive(Debug)]
pub struct App {
    danmu_client: client::DanmuClient, /* danmu client */
                                       /* Todo: config */
}

impl App {
    pub fn new() -> Self {
        todo!()
    }

    pub fn run(&mut self) {}
}
