use crossterm::{
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
    event::{EnableMouseCapture, DisableMouseCapture, Event::{Key}, poll, read, KeyEvent, KeyCode, KeyModifiers},
    execute,
    ExecutableCommand
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Alignment},
    widgets::{Block, Borders, Paragraph, Wrap, BorderType},
    Terminal,
    Frame,
    text::{Span, Spans},
    style::{Style, Color, Modifier}
};
//use url::Url;
use std::io::{stdout, Write};
use crate::PRZZIError;
use crate::PRZZIResult;

pub struct PRZZITUI {
    results: Vec<PRZZIResult>,
    result_index: usize,
}

impl PRZZITUI {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            result_index: 0,
        }
    }
    
    pub fn set_results(&mut self, results: Vec<PRZZIResult>) {
        self.results = results;
    }

    fn read_key(&mut self) -> Result<KeyEvent, PRZZIError> {
        loop {
            if poll(std::time::Duration::from_millis(100))? {
                if let Key(key) = read()? {
                    return Ok(key);
                }
            }
        }
    }
    
    fn draw<B> (&mut self, rect: &mut Frame<B>)
    where
        B: Backend,
    {
        let size = rect.size();
        let block = Block::default().style(Style::default().bg(Color::Rgb(40, 40, 40)).fg(Color::LightBlue));
        rect.render_widget(block, size);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(25),
                    Constraint::Percentage(65),
                    Constraint::Percentage(10)
                ]
            )
            .split(size);
        let title = self.draw_title();
        rect.render_widget(title, chunks[0]);
        let abs = self.draw_abstract();
        rect.render_widget(abs, chunks[1]);
        let footer = self.draw_footer();
        rect.render_widget(footer, chunks[2]);
    }
    
    fn draw_title<'a>(&'a self) -> Paragraph<'a> {
        let text = vec![
            Spans::from(Span::styled(
                self.results[self.result_index].title.as_str(),
                Style::default()
                .fg(Color::Rgb(213, 196, 161))
                .add_modifier(Modifier::ITALIC)
            )),
            Spans::from("\n\n"),
            Spans::from(Span::styled(
                self.results[self.result_index].year.to_string(),
                Style::default().fg(Color::Rgb(213, 196, 161))
            )),
            Spans::from("\n\n"),
            Spans::from(Span::styled(
                self.results[self.result_index].authors.join(", "),
                Style::default().fg(Color::Rgb(213, 196, 161))
            ))
        ];
        Paragraph::new(text.clone())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                .title(Span::styled(
                    "Paperazzi",
                    Style::default().fg(Color::Red)
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
            )
            .wrap(Wrap {trim: true})    
    }

    
    fn draw_abstract<'a>(&'a self) -> Paragraph<'a> {
        let text = vec![
            Spans::from(Span::styled(
                "Abstract",
                Style::default()
                .fg(Color::Rgb(213, 196, 161))
                .add_modifier(Modifier::ITALIC)
            )),
            Spans::from("\n\n"),
            Spans::from(Span::styled(
                self.results[self.result_index].abs.to_string(),
                Style::default().fg(Color::Rgb(213, 196, 161))
            )),
        ];
        Paragraph::new(text.clone())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                .title(Span::styled(
                    "Paperazzi",
                    Style::default().fg(Color::Red)
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
            )
            .wrap(Wrap {trim: true})    
    }

    
    fn draw_footer<'a>(&'a self) -> Paragraph<'a> {
        let text = Spans::from(vec![
            Span::styled(
                "p: ",
                Style::default()
                .fg(Color::Rgb(213, 196, 161))
            ),
            Span::styled(
                "Previous",
                Style::default().fg(Color::Green)
            ),
            Span::raw(
                "    ",
            ),
            Span::styled(
                "n: ",
                Style::default()
                .fg(Color::Rgb(213, 196, 161))
            ),
            Span::styled(
                "Next",
                Style::default().fg(Color::Green)
            ),
            Span::raw(
                "    ",
            ),
            Span::styled(
                "Ctrl-r: ",
                Style::default()
                .fg(Color::Rgb(213, 196, 161))
            ),
            Span::styled(
                "Open in browser\t",
                Style::default().fg(Color::Green)
            ),
            Span::raw(
                "    ",
            ),
            Span::styled(
                "Ctrl-d: ",
                Style::default()
                .fg(Color::Rgb(213, 196, 161))
            ),
            Span::styled(
                "Download paper (In Development)",
                Style::default().fg(Color::Green)
            ),
        ]);
        Paragraph::new(text.clone())
            .alignment(Alignment::Center)
            .wrap(Wrap {trim: true})    
    }
    pub fn start_ui(&mut self) -> Result<(), PRZZIError> {
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let mut backend = CrosstermBackend::new(stdout);
        backend.execute(SetTitle("Paperazzi"))?;
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        terminal.draw(|f| self.draw(f))?;
        loop {
            match self.read_key()? {
                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL
                } => {
                    terminal.clear()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    break;
                },
                KeyEvent {
                    code: KeyCode::Char('n'),
                    modifiers: KeyModifiers::NONE
                } => {
                    if self.result_index < self.results.len() - 1 {
                        self.result_index += 1;
                        terminal.draw(|f| self.draw(f))?;
                    }
                },
                KeyEvent {
                    code: KeyCode::Char('p'),
                    modifiers: KeyModifiers::NONE
                } => {
                   if self.result_index > 0 {
                       self.result_index -= 1;
                       terminal.draw(|f| self.draw(f))?;
                   }
                },
                KeyEvent {
                    code: KeyCode::Char('r'),
                    modifiers: KeyModifiers::CONTROL
                } => {
                    if webbrowser::open(&self.results[self.result_index].url.as_str()).is_err() {
                       let block = Block::default().title("Error!").borders(Borders::ALL);
                       let mut frame = terminal.get_frame();
                       let size = frame.size();
                       let popup_layout = Layout::default()
                           .direction(Direction::Vertical)
                           .constraints(
                                [
                                    Constraint::Percentage(40),
                                    Constraint::Percentage(20),
                                    Constraint::Percentage(40),
                                ].as_ref(),
                            )
                           .split(size);
                       let area = Layout::default()
                           .direction(Direction::Horizontal)
                           .constraints(
                               [
                                    Constraint::Percentage(20),
                                    Constraint::Percentage(60),
                                    Constraint::Percentage(20),
                               ].as_ref(),
                            )
                           .split(popup_layout[1])[1];
                       frame.render_widget(block, area)
                    }
                },
                /*
                KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::CONTROL
                } => {
                    let client = reqwest::blocking::Client::new();
                    let url = self.results[self.result_index].url.as_str();
                    let down_url = Url::parse(url).unwrap();
                    println!("{}", down_url);
                    let res = client.get(down_url)
                        // user agent
                        .header("User-Agent", "Paperazzi")
                        .send()?;  
                    let filename = res.url().path_segments().unwrap().last().unwrap();
                    let mut file = std::fs::File::create(filename)?;
                    file.write_all(res.bytes()?.as_ref())?;
                    println!("Done");
                },
                */
                _ => {
                    continue;
                }
            }
            
        }
        Ok(())
    }
}
