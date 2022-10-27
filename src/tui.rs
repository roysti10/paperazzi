use crossterm::{cursor::{MoveTo, MoveLeft}, terminal::{Clear, ClearType, size}};
use std::io::{stdout, Write};
use std::io::Stdout;
use crate::PRZZIResult;
use crate::PRZZIError;

#[derive(Clone, Copy)]
struct LinePosition {
    pub x: u16,
    pub y: u16,
    pub len: u16
}

struct Cursor {
    pub lines: Vec<LinePosition>,
    pub current: usize,
    pub current_column: u16,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            lines: Vec::new(),
            current: 0,
            current_column: 0,
        }
    }

    pub fn get_position(&self) -> (u16, u16) {
        let line = self.lines[self.current];
        (line.x + self.current_column, line.y)
    }
}

pub struct PRZZITUI {
    stdout: Stdout,
    results: Vec<PRZZIResult>,
    result_index: usize,
    cursor_pos: Cursor,
}

impl PRZZITUI {
    pub fn new() -> Self {
        Self {
            stdout: stdout(),
            results: Vec::new(),
            result_index: 0,
            cursor_pos: Cursor::new(),
        }
    }

    pub fn set_results(&mut self, results: Vec<PRZZIResult>) {
        self.results = results;
    }

    pub fn flush(& mut self) -> Result<(), PRZZIError> {
        self.stdout.flush()?;
        Ok(())
    }


    pub fn print_results(&mut self) -> Result<(), PRZZIError> {
        crossterm::execute!(self.stdout, Clear(ClearType::All)).unwrap();
        self.flush()?;
        let mut max_word_len = 0;
        let mut current_len = 0;
        let (width, height) = size()?;
        let mut max_width = width*2 / 3;
        const MAX_WORDS_PER_LINE: usize = 15;
        let mut line = Vec::new();
        let mut lines = Vec::new();
        for word in self.results[self.result_index].title.split_whitespace() {
            max_word_len = std::cmp::max(max_word_len, word.len() + 1);
            let new_len = current_len + word.len() as u16 + 1;
            if line.len() < MAX_WORDS_PER_LINE && new_len < max_width {
                line.push(word.clone());
                current_len += word.len() as u16 + 1;
            } else {
                lines.push(line.join(" ") + " ");
                line = vec![word.clone()];
                current_len = word.len() as u16 + 1;
            }
        }
        lines.push(line.join(" ") + " ");
        lines.push(self.results[self.result_index].year.to_string());
        lines.push("\n\n".to_string());
        let mut authors = self.results[self.result_index].authors.clone();
        let mut author_line = Vec::new();
        while !authors.is_empty() {
            let author = authors.pop().unwrap();
            if author_line.len() < 4 {
                author_line.push(author)
            } else {
                lines.push(author_line.join(", "));
                author_line = vec![author];
            }
        }
        lines.push(author_line.join(", "));
        lines.push("\n\n".to_string());
        lines.push("Abstract: ".to_string());
        for (line_no, line) in lines.iter().enumerate() {
            crossterm::execute!(self.stdout, MoveTo(width/2, line_no as u16)).unwrap();
            let len = line.len() as u16;
            write!(self.stdout, "{}", MoveLeft(len/2)).unwrap();
            self.cursor_pos.lines.push(LinePosition {
                x: width/2 - len/2,
                y: line_no as u16,
                len: len,
            });
            write!(self.stdout, "{}", line).unwrap();
        }
        let current_line = lines.len() as u16;
        lines = Vec::new();
        line = Vec::new();
        max_width = width*2 / 2;
        for word in self.results[self.result_index].abs.split_whitespace() {
            max_word_len = std::cmp::max(max_word_len, word.len() + 1);
            let new_len = current_len + word.len() as u16 + 1;
            if line.len() < MAX_WORDS_PER_LINE && new_len < max_width {
                line.push(word.clone());
                current_len += word.len() as u16 + 1;
            } else {
                lines.push(line.join(" ") + " ");
                line = vec![word.clone()];
                current_len = word.len() as u16 + 1;
            }
        }
        lines.push(line.join(" ") + " ");
        for (line_no, line) in lines.iter().enumerate() {
            crossterm::execute!(self.stdout, MoveTo(width/2, current_line + line_no as u16)).unwrap();
            let len = line.len() as u16;
            write!(self.stdout, "{}", MoveLeft(len/2)).unwrap();
            self.cursor_pos.lines.push(LinePosition {
                x: width/2 - len/2,
                y: current_line + line_no as u16,
                len: len
            });
            write!(self.stdout, "{}", line).unwrap();
        }
        crossterm::execute!(self.stdout, MoveTo(self.cursor_pos.lines[0].x, self.cursor_pos.lines[0].y)).unwrap();
        self.flush()?;
        Ok(())
    }
    pub fn read_key(&mut self) -> Result<crossterm::event::KeyEvent, PRZZIError> {
        loop {
            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    return Ok(key);
                }
            }
        }
    }

    pub fn handle_input(&mut self) -> Result<bool, PRZZIError> {
        match self.read_key()? {
            crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Char('c'),
                modifiers: crossterm::event::KeyModifiers::CONTROL,
            } => {
                return Ok(false);
            },
            crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Char('n'),
                modifiers: crossterm::event::KeyModifiers::NONE,
            } => {
                if self.result_index < self.results.len() - 1 {
                    self.result_index += 1;
                    self.print_results().unwrap();
                }
                return Ok(true);
            },
            crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Char('p'),
                modifiers: crossterm::event::KeyModifiers::NONE,
            } => {
                if self.result_index > 0 {
                    self.result_index -= 1;
                    self.print_results().unwrap();
                }
                return Ok(true);
            },
            crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Char('r'),
                modifiers: crossterm::event::KeyModifiers::CONTROL,
            } => {
                if webbrowser::open(&self.results[self.result_index].doi.as_str()).is_err() {
                    write!(self.stdout, "Failed to open browser").unwrap();
                }
                return Ok(true);
            },
            crossterm::event::KeyEvent {
                code: direction @ (
                    crossterm::event::KeyCode::Up | 
                    crossterm::event::KeyCode::Down | 
                    crossterm::event::KeyCode::Left | 
                    crossterm::event::KeyCode::Right
                ),
                modifiers: crossterm::event::KeyModifiers::NONE,
            } => {
                // TODO: if abstract increases screen size
                if direction == crossterm::event::KeyCode::Up {
                    if self.cursor_pos.current > 0 {
                        self.cursor_pos.current -= 1;
                    }
                } else if direction == crossterm::event::KeyCode::Down {
                    if self.cursor_pos.current < self.cursor_pos.lines.len() - 1 {
                        self.cursor_pos.current += 1;
                    }
                } else if direction == crossterm::event::KeyCode::Left {
                    if self.cursor_pos.current_column > 0 {
                        self.cursor_pos.current_column -= 1;
                    }
                } else if direction == crossterm::event::KeyCode::Right {
                    if self.cursor_pos.current_column < self.cursor_pos.lines[self.cursor_pos.current].len {
                        self.cursor_pos.current_column += 1;
                    }
                }
                let (x,y) = self.cursor_pos.get_position();
                crossterm::execute!(self.stdout, MoveTo(x, y)).unwrap();
                return Ok(true);
            },
            _ => {
                return Ok(true);
            }   
        }
    }
}

