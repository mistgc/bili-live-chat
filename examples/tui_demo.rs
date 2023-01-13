#![allow(dead_code)]

use std::{io, time::SystemTime};

use chrono::{DateTime, Datelike, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, ModifierKeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Widget},
    Frame, Terminal,
};

#[derive(Debug)]
enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug)]
struct App<'a, B: Backend> {
    input: String,
    terminal: Option<&'a mut Terminal<B>>,
    app_widgets: AppWidgets,
}

#[derive(Debug)]
struct AppWidgets {
    input_mode: InputMode,
    tab_selected: usize,
    input: String,
    chat_history: Vec<String>,
}

impl AppWidgets {
    pub fn new() -> Self {
        Self {
            input_mode: InputMode::Normal,
            tab_selected: 0,
            input: String::new(),
            chat_history: vec![],
        }
    }
}

impl<'a, B: Backend> App<'a, B> {
    pub fn new(term: &'a mut Terminal<B>) -> Self {
        Self {
            input: String::new(),
            terminal: Some(term),
            app_widgets: AppWidgets::new(),
        }
    }

    fn tab_next(&mut self) {
        self.app_widgets.tab_selected += 1;
        if self.app_widgets.tab_selected > 2 {
            self.app_widgets.tab_selected = 0;
        }
    }

    fn tab_prev(&mut self) {
        if self.app_widgets.tab_selected == 0 {
            self.app_widgets.tab_selected = 2;
        } else {
            self.app_widgets.tab_selected -= 1;
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        if self.terminal.is_none() {
            println!("Err: The terminal of App invalid!!!");
            return Ok(());
        }
        loop {
            // render ui
            self.terminal
                .as_mut()
                .unwrap()
                .draw(|f| ui(f, &mut self.app_widgets))?;

            // bind keycode event
            if let Event::Key(key) = event::read()? {
                match self.app_widgets.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            self.app_widgets.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Tab => {
                            self.tab_next();
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Esc => self.app_widgets.input_mode = InputMode::Normal,
                        KeyCode::Char(c) => {
                            self.app_widgets.input.push(c);
                        }
                        KeyCode::Backspace => {
                            self.app_widgets.input.pop();
                        }
                        KeyCode::Enter => {
                            self.app_widgets
                                .chat_history
                                .push(self.app_widgets.input.drain(..).collect());
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

struct ChatHistoryItem<'a> {
    title: ListItem<'a>,
    content: ListItem<'a>,
}

struct ChatHistory<'a> {
    messages: Vec<ChatHistoryItem<'a>>,
}

impl<'a> FromIterator<ChatHistoryItem<'a>> for ChatHistory<'a> {
    fn from_iter<T: IntoIterator<Item = ChatHistoryItem<'a>>>(iter: T) -> Self {
        let mut res = vec![];
        for i in iter {
            res.push(i);
        }

        Self { messages: res }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, aw: &mut AppWidgets) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Length(1), Constraint::Min(3)].as_ref())
        .split(f.size());
    let tabs_title = vec!["tab1".to_owned(), "tab2".to_owned(), "tab3".to_owned()];
    let title = tabs_title
        .iter()
        .map(|item| Spans::from(Span::from(item.as_str())))
        .collect();
    let tabs = Tabs::new(title).select(aw.tab_selected);
    f.render_widget(tabs, chunks[0]);

    match aw.tab_selected {
        0 => draw_first_tab(f, aw, chunks[1]),
        1 => draw_second_tab(f, aw, chunks[1]),
        2 => draw_third_tab(f, aw, chunks[1]),
        _ => unreachable!(),
    };
}

fn draw_first_tab<B: Backend>(f: &mut Frame<B>, aw: &mut AppWidgets, area: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let list: ChatHistory = aw
        .chat_history
        .iter()
        .enumerate()
        .map(|(_, m)| {
            let dt = DateTime::<Utc>::from(SystemTime::now());
            let title = Spans::from(format!("{}", dt));
            let content = Spans::from(Span::raw(m));
            ChatHistoryItem {
                title: ListItem::new(Text::from(title)),
                content: ListItem::new(Text::from(content)),
            }
        })
        .collect();
    let chat_history = {
        let mut res = vec![];
        for i in list.messages {
            res.push(i.title.style(Style::default().fg(Color::Cyan)));
            res.push(i.content);
        }
        res
    };
    let chat_history =
        List::new(chat_history).block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(chat_history, chunks[0]);

    let input = Paragraph::new(aw.input.as_ref())
        .block(Block::default().borders(Borders::ALL).title("Send"));
    f.render_widget(input, chunks[1]);
    match aw.input_mode {
        InputMode::Normal => {}
        InputMode::Editing => {
            f.set_cursor(chunks[1].x + aw.input.len() as u16 + 1, chunks[1].y + 1);
        }
    }
}

fn draw_second_tab<B: Backend>(f: &mut Frame<B>, aw: &mut AppWidgets, area: Rect) {
    let block = Block::default().title("Tab 2").borders(Borders::ALL);
    f.render_widget(block, area);
}

fn draw_third_tab<B: Backend>(f: &mut Frame<B>, aw: &mut AppWidgets, area: Rect) {
    let block = Block::default().title("Tab 3").borders(Borders::ALL);
    f.render_widget(block, area);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend);

    // create app and run it
    let mut app = App::new(terminal.as_mut().unwrap());
    let res = app.run();

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.as_mut().unwrap().backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.as_mut().unwrap().show_cursor()?;
    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
