use std::{collections::HashMap, process::exit, sync::Arc, time::Duration};

use crate::{api::live::LiveRoom, config::Config, Message, MessageKind};
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use tokio::sync::{mpsc::Receiver, Mutex};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Default)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct UI<B: Backend + std::io::Write> {
    ui_state: UiState,
    terminal: Option<Terminal<B>>,
    live_room: LiveRoom,
    config: Arc<Mutex<Config>>,
}

#[derive(Debug, Default)]
struct UiState {
    /* Channal Receiver */
    msg_rx: Option<Receiver<Message>>,
    rm_info_rx: Option<Receiver<HashMap<String, String>>>,
    rank_info_rx: Option<Receiver<Vec<String>>>,

    /* Tab 1: Chat Room */
    input_mode: InputMode,
    tab_selected: usize,
    input_buf: String,
    chat_history: Vec<Message>,

    /* Tab 2: Rank Info */
    rank_info: Option<Vec<String>>,
    gift_history: Vec<Message>,

    /* Tab 3: Room Info */
    ruid: String,
    room_id: String,
    title: String,
    tags: String,
    description: String,
    area_name: String,
    parent_area_name: String,
    live_start_time: i64,
    watched_show: i64,
    attention: i64,
    uname: String,
    total_likes: i64,
}

impl<B: Backend + std::io::Write> UI<B> {
    pub async fn new(
        term: Terminal<B>,
        msg_rx: Receiver<Message>,
        rm_info_rx: Receiver<HashMap<String, String>>,
        rank_info_rx: Receiver<Vec<String>>,
        room_id: i64,
        config: Arc<Mutex<Config>>,
    ) -> Self {
        let state = UiState {
            msg_rx: Some(msg_rx),
            rm_info_rx: Some(rm_info_rx),
            rank_info_rx: Some(rank_info_rx),
            ..Default::default()
        };

        let live_room = LiveRoom::new(room_id, config.lock().await.credential.clone());

        Self {
            terminal: Some(term),
            ui_state: state,
            live_room,
            config,
        }
    }

    fn tab_next(&mut self) {
        self.ui_state.tab_selected += 1;
        if self.ui_state.tab_selected > 2 {
            self.ui_state.tab_selected = 0;
        }
    }

    pub async fn run(&mut self) -> std::io::Result<()> {
        if self.terminal.is_none() {
            panic!("Err: The terminal af TUI invalid!!!");
        }

        loop {
            // When the length of chat_history and gift_history is greater than or equal to 100,
            // clear up the first 50 chats to ensure that the length of chat_history
            // is not too long.
            if self.ui_state.chat_history.len() >= 100 {
                self.ui_state.chat_history.drain(0..50);
            }

            if self.ui_state.gift_history.len() >= 100 {
                self.ui_state.gift_history.drain(0..50);
            }

            /* Draw UI */
            self.terminal
                .as_mut()
                .unwrap()
                .draw(|f| draw_ui(f, &mut self.ui_state))?;

            /* Receive Message */
            if let Ok(msg) = self.ui_state.msg_rx.as_mut().unwrap().try_recv() {
                match msg.kind {
                    MessageKind::DANMU_MSG => {
                        self.ui_state.chat_history.push(msg);
                    }
                    MessageKind::SEND_GIFT => {
                        self.ui_state.gift_history.push(msg);
                    }
                    _ => {}
                }
            }

            /* Sync Room Info */
            if let Ok(ri) = self.ui_state.rm_info_rx.as_mut().unwrap().try_recv() {
                /* Room Info */
                self.ui_state.ruid = ri["ruid"].clone();
                self.ui_state.room_id = ri["room_id"].clone();
                self.ui_state.title = ri["title"].clone();
                self.ui_state.tags = ri["tags"].clone();
                self.ui_state.description = ri["description"].clone();
                self.ui_state.area_name = ri["area_name"].clone();
                self.ui_state.parent_area_name = ri["parent_area_name"].clone();
                self.ui_state.live_start_time = ri["live_start_time"].parse().unwrap();
                self.ui_state.watched_show = ri["watched_show"].parse().unwrap();
                self.ui_state.attention = ri["attention"].parse().unwrap();
                self.ui_state.uname = ri["uname"].clone();
                self.ui_state.total_likes = ri["total_likes"].parse().unwrap();
            }

            /* Sync The First 50 Of Rank Info */
            if let Ok(rf50) = self.ui_state.rank_info_rx.as_mut().unwrap().try_recv() {
                self.ui_state.rank_info = Some(rf50);
            }

            /* Poll Keyboard Events */
            if crossterm::event::poll(Duration::from_millis(10)).unwrap() {
                if let Event::Key(key) = event::read()? {
                    match self.ui_state.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('e') => {
                                self.ui_state.input_mode = InputMode::Editing;
                            }
                            KeyCode::Char('q') => {
                                /* restore terminal */
                                let stdout = self.terminal.as_mut().unwrap().backend_mut();
                                disable_raw_mode().unwrap();
                                execute!(stdout, LeaveAlternateScreen, DisableMouseCapture,)
                                    .unwrap();
                                self.terminal.as_mut().unwrap().show_cursor().unwrap();
                                exit(0)
                            }
                            KeyCode::Tab => {
                                self.tab_next();
                            }
                            _ => {}
                        },
                        InputMode::Editing => match key.code {
                            KeyCode::Esc => {
                                self.ui_state.input_mode = InputMode::Normal;
                            }
                            KeyCode::Char(c) => {
                                self.ui_state.input_buf.push(c);
                            }
                            KeyCode::Backspace => {
                                self.ui_state.input_buf.pop();
                            }
                            KeyCode::Enter => {
                                if self.ui_state.input_buf.len() > 0 {
                                    let danmaku_text =
                                        self.ui_state.input_buf.drain(..).collect::<String>();
                                    // refresh ui immediately
                                    self.terminal
                                        .as_mut()
                                        .unwrap()
                                        .draw(|f| draw_ui(f, &mut self.ui_state))?;

                                    self.live_room
                                        .send_normal_danmaku(danmaku_text.as_str())
                                        .await;
                                }
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }
}

fn draw_ui<B: Backend>(f: &mut Frame<B>, us: &mut UiState) {
    let chunks = Layout::default()
        .direction(tui::layout::Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Length(1), Constraint::Min(3)].as_ref())
        .split(f.size());
    let tabs_title = vec![
        "Chat Room".to_owned(),
        "Rank Info".to_owned(),
        "Room Info".to_owned(),
    ];
    let tabs_title = tabs_title
        .iter()
        .enumerate()
        .map(|(index, item)| {
            if index == us.tab_selected {
                Spans::from(Span::styled(
                    item.as_str(),
                    Style::default().fg(Color::Blue),
                ))
            } else {
                Spans::from(Span::from(item.as_str()))
            }
        })
        .collect();
    let tabs = Tabs::new(tabs_title).select(us.tab_selected);
    f.render_widget(tabs, chunks[0]);

    match us.tab_selected {
        0 => draw_chat_room(f, us, chunks[1]),
        1 => draw_rank_info(f, us, chunks[1]),
        2 => draw_room_info(f, us, chunks[1]),
        _ => unreachable!(),
    };
}

fn draw_chat_room<B: Backend>(f: &mut Frame<B>, us: &mut UiState, area: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(area);

    /* Chat History */
    let mut chat_history = vec![];
    for msg in us.chat_history.iter() {
        let title = format!("[{}] {}", msg.date.format("%H:%M"), msg.author);
        chat_history.push(
            ListItem::new(Text::from(Spans::from(title))).style(Style::default().fg(Color::Cyan)),
        );
        chat_history.push(ListItem::new(Text::from(Spans::from(msg.content.clone()))));
    }
    chat_history.reverse();
    let chat_history = List::new(chat_history)
        .block(Block::default().borders(Borders::ALL).title("Messages"))
        .start_corner(tui::layout::Corner::BottomLeft);
    f.render_widget(chat_history, chunks[0]);

    /* Input Box */
    let input_box = Paragraph::new(us.input_buf.as_ref())
        .block(Block::default().borders(Borders::ALL).title("Send"));
    f.render_widget(input_box, chunks[1]);
    match us.input_mode {
        InputMode::Normal => {}
        InputMode::Editing => {
            let len = UnicodeWidthStr::width(us.input_buf.as_str());
            f.set_cursor(chunks[1].x + len as u16 + 1, chunks[1].y + 1);
        }
    }
}

fn draw_rank_info<B: Backend>(f: &mut Frame<B>, us: &mut UiState, area: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .direction(tui::layout::Direction::Horizontal)
        .split(area);

    /* rank info */
    let rank_info_items = {
        if let Some(ref data) = us.rank_info {
            Some(
                // format: name,guard_level,score
                data.iter()
                    .map(|i| i.split(',').collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        }
    };
    let mut list_items = vec![];
    if let Some(data) = rank_info_items {
        for (k, v) in data.iter().enumerate() {
            let rank;
            if k < 3 {
                rank = Span::styled((k + 1).to_string() + ": ", Style::default().fg(Color::Red));
            } else {
                rank = Span::raw((k + 1).to_string() + ": ");
            }
            let spans = Spans::from(vec![
                rank,
                Span::styled(v[0].to_owned() + " ", Style::default().fg(Color::Cyan)),
                Span::styled(v[2].to_owned(), Style::default().fg(Color::Blue)),
            ]);
            list_items.push(ListItem::new(Text::from(spans)));
        }
    } else {
        list_items.push(ListItem::new(Text::from("Here is not anyone.")));
    }

    let rank_info_list =
        List::new(list_items).block(Block::default().borders(Borders::ALL).title("Rank"));
    f.render_widget(rank_info_list, chunks[0]);

    /* gift */
    let mut gift_items = us
        .gift_history
        .iter()
        .map(|v| {
            let datetime = v.date.format("[%H:%M]").to_string();
            let gift_ctnt = Spans::from(vec![
                Span::raw(datetime),
                Span::styled(
                    " ".to_owned() + v.author.as_str() + " ",
                    Style::default().fg(Color::Blue),
                ),
                Span::styled(v.content.clone(), Style::default().fg(Color::Green)),
            ]);
            ListItem::new(gift_ctnt)
        })
        .collect::<Vec<_>>();
    gift_items.reverse();
    let gift_list = List::new(gift_items)
        .block(Block::default().borders(Borders::ALL).title("Gift"))
        .start_corner(tui::layout::Corner::BottomLeft);
    f.render_widget(gift_list, chunks[1]);
}

fn draw_room_info<B: Backend>(f: &mut Frame<B>, us: &mut UiState, area: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    /* base info */
    let mut text = vec![
        Spans::from(vec![
            Span::raw("Host: "),
            Span::styled(us.uname.clone(), Style::default().fg(Color::Green)),
        ]),
        Spans::from(vec![
            Span::raw("Room Id: "),
            Span::styled(us.room_id.clone(), Style::default().fg(Color::Cyan)),
        ]),
        Spans::from(vec![
            Span::raw("Title: "),
            Span::styled(us.title.clone(), Style::default().fg(Color::Cyan)),
        ]),
        Spans::from(vec![
            Span::raw("Area: "),
            Span::styled(
                format!("{}/{}", us.parent_area_name.as_str(), us.area_name.as_str()),
                Style::default().fg(Color::Cyan),
            ),
        ]),
    ];
    text.append(&mut crate::utils::parse_description(
        &us.description,
        Style::default().fg(Color::Cyan),
    ));
    let base_info = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
    f.render_widget(base_info, chunks[0]);

    /* other info */
    let duration = crate::utils::duration(us.live_start_time as u64);
    let text = vec![
        Spans::from(vec![
            Span::raw("Live duration: "),
            Span::styled(
                crate::utils::display_duration(duration),
                Style::default().fg(Color::Red),
            ),
        ]),
        Spans::from(vec![
            Span::raw("Total likes: "),
            Span::styled(us.total_likes.to_string(), Style::default().fg(Color::Red)),
        ]),
        Spans::from(vec![
            Span::raw("Attention: "),
            Span::styled(us.attention.to_string(), Style::default().fg(Color::Red)),
        ]),
        Spans::from(vec![
            Span::raw("Watched show: "),
            Span::styled(us.watched_show.to_string(), Style::default().fg(Color::Red)),
        ]),
    ];
    let other_info = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
    f.render_widget(other_info, chunks[1]);
}

impl<B: Backend + std::io::Write> Drop for UI<B> {
    fn drop(&mut self) {
        /* restore terminal */
        let stdout = self.terminal.as_mut().unwrap().backend_mut();
        disable_raw_mode().unwrap();
        execute!(stdout, LeaveAlternateScreen, DisableMouseCapture,).unwrap();
        self.terminal.as_mut().unwrap().show_cursor().unwrap();
    }
}
