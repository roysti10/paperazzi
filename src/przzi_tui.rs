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
    layout::{Constraint, Direction, Layout, Alignment, Rect}, 
    widgets::{Block, Borders, Paragraph, Wrap, BorderType, Clear},
    Terminal,
    Frame,
    text::{Span, Spans},
    style::{Style, Color, Modifier}
};
use url::Url;
use std::io::stdout;
use crate::PRZZIError;
use crate::PRZZIResult;
use crate::PRZZI;


struct Popup {
    show_popup: bool,
    popup_msg: String,
    popup_type: String, //Info by default
    popup_color: Color // yellow for Info, Green for Success and Red for Error
}

impl Popup {
    pub fn new() -> Self {
        Self {
            show_popup: false,
            popup_msg: "".to_string(),
            popup_type: "Info".to_string(),
            popup_color: Color::Yellow,
        }
    }

    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_y) / 2),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage((100 - percent_y) / 2),
                ]
                .as_ref(),
         )
        .split(r);
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1]
    }
    
    fn get_para(&self) -> Paragraph{
        Paragraph::new(self.popup_msg.as_ref())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                .title(Span::styled(self.popup_type.clone(), Style::default().fg(self.popup_color)))
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::White).fg(Color::Black))
            )
            .wrap(Wrap {trim: true})
    }

    fn close(&mut self){
        self.show_popup = false;
    }

    fn open(&mut self, msg: String, popup_type: String){
        if popup_type == "Error!"{
            self.popup_color = Color::Red;
        }
        else if popup_type == "Info" {
            self.popup_color = Color::Yellow;
        }
        else if popup_type == "Success" {
            self.popup_color = Color::Green;
        }
        self.popup_type = popup_type;
        self.popup_msg = msg;
        self.show_popup = true;
    }
}

pub struct PRZZITUI {
    results: Vec<PRZZIResult>,
    result_index: usize,
    scroll: u16,
    popup: Popup,
}

impl PRZZITUI {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            result_index: 0,
            scroll: 0 as u16,
            popup: Popup::new()
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
        if self.popup.show_popup{
            let para = self.popup.get_para(); 
            let area = self.popup.centered_rect(80, 30, size);
            rect.render_widget(Clear, area);
            rect.render_widget(para, area);
        }

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
            .scroll((self.scroll, 0))
    }

    
    fn draw_footer<'a>(&'a self) -> Paragraph<'a> {
        let mut text = vec![Spans::from(vec![
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
                "Open in browser",
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
                "Download paper",
                Style::default().fg(Color::Green)
            ),
            Span::raw(
                "    "
            ),
            Span::styled(
                "↓/↑: ",
                Style::default().fg(Color::Rgb(213, 196, 161))
            ),
            Span::styled(
                "Scroll Abstract",
                Style::default().fg(Color::Green)
            )
        ])];
        if self.popup.show_popup {
            text.push(
                Spans::from(vec![
                    Span::raw(
                        "    ",
                    ),
                    Span::styled(
                        "q: ",
                        Style::default()
                        .fg(Color::Rgb(213, 196, 161))
                    ),
                    Span::styled(
                        "Close Popup",
                        Style::default().fg(Color::Green)
                    ),
                ])  
            )
        }
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
        loop {
            terminal.draw(|f| self.draw(f))?;
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
                        self.popup.close();
                        self.result_index += 1;
                    }
                },
                KeyEvent {
                    code: KeyCode::Char('p'),
                    modifiers: KeyModifiers::NONE
                } => {
                   if self.result_index > 0 {
                       self.popup.close();
                       self.result_index -= 1;
                   }
                },
               KeyEvent {
                    code: KeyCode::Char('r'),
                    modifiers: KeyModifiers::CONTROL
                } => {
                    if webbrowser::open(&self.results[self.result_index].url.as_str()).is_err() {
                        self.popup.open("Redirect failed! Please try again".to_string(), "Error!".to_string());
                    }
                },
                KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::CONTROL
                } => {
                    self.popup.open("Attempting to Download".to_string(), "Info".to_string());
                    terminal.draw(|f| self.draw(f))?;
                    if self.results[self.result_index].url.to_string().contains("doi") {
                        let mut download_url = "https://sci-hub.wf/".to_string();
                        download_url.push_str(self.results[self.result_index].url.as_ref());
                        let doi_url =  Url::parse(&download_url)?;
                        if let Err(_e) = PRZZI::download_doi(doi_url) {
                            self.popup.open("Download failed! This paper is not available to download yet :( if you think this is wrong, raise a issue :) \n Please try redirecting instead".to_string(), "Error!".to_string());
                        }
                        else{
                            self.popup.open("Download Complete :) !!".to_string(), "Success".to_string());

                        }
                    }
                    else {
                        self.popup.open("This paper doesnt have a valid DOI so a download isnt possible just yet :( If you think this is wrong, feel free to raise an issue \n Please try redirect instead".to_string(), "Error!".to_string());
                    }
                },
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE
                } => {
                    self.popup.close();
                },
                KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE
                } => {
                    if self.scroll > 0 {
                        self.scroll-=1;
                    }
                },
                KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE
                } => {
                    self.scroll+=1;
                },
                _ => {}
            }
            
        }
        Ok(())
    }
}
