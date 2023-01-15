use std::{process::exit, time::Duration};

use crate::{Message, MessageKind};
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use tokio::sync::mpsc::Receiver;
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};

#[derive(Debug, Default)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

#[derive(Debug)]
pub struct UI<B: Backend + std::io::Write> {
    ui_state: UiState,
    terminal: Option<Terminal<B>>,
}

#[derive(Debug, Default)]
struct UiState {
    /* Channal Receiver */
    mpsc_rx: Option<Receiver<Message>>,

    /* Tab 1: Chat Room */
    input_mode: InputMode,
    tab_selected: usize,
    input_buf: String,
    chat_history: Vec<Message>,
    /* Tab 2: Rank Info */

    /* Tab 3: Live Room Info */
}

impl<B: Backend + std::io::Write> UI<B> {
    pub fn new(term: Terminal<B>, mpsc_rx: Receiver<Message>) -> Self {
        let state = UiState {
            mpsc_rx: Some(mpsc_rx),
            ..Default::default()
        };

        Self {
            terminal: Some(term),
            ui_state: state,
        }
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        if self.terminal.is_none() {
            panic!("Err: The terminal af TUI invalid!!!");
        }

        loop {
            /* Draw UI */
            self.terminal
                .as_mut()
                .unwrap()
                .draw(|f| draw_ui(f, &mut self.ui_state))?;

            /* Receive Message */
            if let Ok(msg) = self.ui_state.mpsc_rx.as_mut().unwrap().try_recv() {
                match msg.kind {
                    MessageKind::DANMU_MSG => {
                        self.ui_state.chat_history.push(msg);
                    }
                    MessageKind::SEND_GIFT => {}
                    MessageKind::COMBO_SEND => {}
                    MessageKind::NOTICE_MSG => {}
                    MessageKind::INTERACT_WORD => {}
                }
            }

            /* Poll Keyboard Events */
            if crossterm::event::poll(Duration::from_millis(200)).unwrap() {
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
        "Live Room Info".to_owned(),
    ];
    let tabs_title = tabs_title
        .iter()
        .map(|item| Spans::from(Span::from(item.as_str())))
        .collect();
    let tabs = Tabs::new(tabs_title).select(us.tab_selected);
    f.render_widget(tabs, chunks[0]);

    match us.tab_selected {
        0 => draw_chat_room(f, us, chunks[1]),
        1 => draw_rank_info(f, us, chunks[1]),
        2 => draw_live_room_info(f, us, chunks[1]),
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
            f.set_cursor(chunks[1].x + us.input_buf.len() as u16 + 1, chunks[1].y + 1);
        }
    }
}

fn draw_rank_info<B: Backend>(f: &mut Frame<B>, us: &mut UiState, area: Rect) {}

fn draw_live_room_info<B: Backend>(f: &mut Frame<B>, us: &mut UiState, area: Rect) {}

impl<B: Backend + std::io::Write> Drop for UI<B> {
    fn drop(&mut self) {
        /* restore terminal */
        let stdout = self.terminal.as_mut().unwrap().backend_mut();
        disable_raw_mode().unwrap();
        execute!(stdout, LeaveAlternateScreen, DisableMouseCapture,).unwrap();
        self.terminal.as_mut().unwrap().show_cursor().unwrap();
    }
}
