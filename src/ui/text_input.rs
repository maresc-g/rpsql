use termion::cursor::DetectCursorPos;
use crate::ui::event::{TrueEvent, KeyEvent, Key};
use std::io::{Write};

const PROMPT: &str = "$> ";

pub struct TermPos {
    x: u16,
    y: u16,
    buffer: Vec<char>,
    current_index: usize,
    d_y: u16,
    max_x: u16,
    max_y: u16,
    prompt_len: usize,
}

impl TermPos {
    pub fn new(stdout: &mut termion::raw::RawTerminal<std::io::Stdout>) -> TermPos {
        let (max_x, max_y) = termion::terminal_size().unwrap();
        let pos = stdout.cursor_pos().unwrap();
        TermPos {
            x: PROMPT.len() as u16 + 1,
            y: pos.1,
            buffer: Vec::new(),
            current_index: 0,
            d_y: 0,
            max_x,
            max_y,
            prompt_len: PROMPT.len()
        }
    }

    fn _compute_max_dy(&self) -> u16 {
        ((self.prompt_len + self.buffer.len()) / self.max_x as usize) as u16
    }

    fn char(&mut self, c: char) {
        self.buffer.insert(self.current_index, c);
        self.current_index += 1;
        self.x += 1;
        if self.x > self.max_x {
            self.x = 1;
            self.d_y += 1;
        }
        if self.y + self._compute_max_dy() > self.max_y {
            self.y -= 1;
            print!("{}\n{}", termion::cursor::Goto(1, self.max_y), termion::clear::CurrentLine);
        }
    }

    fn left(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
            if self.x == 1 {
                self.x = self.max_x;
                self.d_y -= 1;
            }
            else {
                self.x -= 1;
            }
        }
    }

    fn right(&mut self) {
        if self.current_index < self.buffer.len() {
            self.current_index += 1;
            if self.x == self.max_x {
                self.x = 1;
                self.d_y += 1;
            }
            else {
                self.x += 1;
            }
        }
    }

    fn up(&mut self) {
        if self.d_y > 0 {
            self.current_index -= self.max_x as usize;
            self.d_y -= 1;
        }
        else {
            self.beg();
        }
    }

    fn down(&mut self) {
        if self.current_index + (self.max_x as usize) < self.buffer.len() {
            self.current_index += self.max_x as usize;
            self.d_y += 1;
        }
        else {
            self.end();
        }
    }

    fn backspace(&mut self) {
        if self.current_index > 0 {
            self.buffer.remove(self.current_index - 1);
            self.left();
        }
    }

    fn delete(&mut self) {
        if self.current_index < self.buffer.len() {
            self.buffer.remove(self.current_index);
        }
    }

    fn beg(&mut self) {
        self.current_index = 0;
        self.x = self.prompt_len as u16 + 1;
        self.d_y = 0;
    }

    fn end(&mut self) {
        self.current_index = self.buffer.len();
        self.x = ((self.prompt_len + self.buffer.len()) % self.max_x as usize) as u16 + 1;
        self.d_y = self._compute_max_dy();
    }

    fn word_left(&mut self) {
        let rev_index = self.buffer.len() - self.current_index;
        let mut skipped = rev_index;
        let first_non_whitespace_res = self.buffer.iter().rev().skip(skipped).position(|&c| !c.is_whitespace());
        if let Some(p) = first_non_whitespace_res {
            skipped = p + rev_index;
        }
        let result = self.buffer.iter().rev().skip(skipped).position(|&c| c.is_whitespace());
        if let Some(p) = result {
            for _ in 0..(p + (skipped - rev_index)) {
                self.left();
            }
        }
        else {
            self.beg();
        }
    }

    fn word_right(&mut self) {
        let first_non_whitespace_res = self.buffer.iter().skip(self.current_index).position(|&c| c.is_whitespace());
        if let Some(p) = first_non_whitespace_res {
            let skipped = p + self.current_index;
            let result = self.buffer.iter().skip(skipped).position(|&c| !c.is_whitespace());
            if let Some(p) = result {
                for _ in 0..(p + (skipped - self.current_index)) {
                    self.right();
                }
            }
            else {
                self.end();
            }
        } else {
            self.end();
        }
    }

    fn clear_term(&mut self) {
        self.y = 1;
        print!("{}", termion::clear::All);
    }
}

#[derive(PartialEq)]
pub enum TextInputEvent {
    Quit,
    HistoryPrev,
    HistoryNext,
    Buffer(Vec<char>, String),
    None,
}

pub struct TextInput {
    tp: TermPos,
}

impl TextInput {
    pub fn new(stdout: &mut termion::raw::RawTerminal<std::io::Stdout>) -> TextInput {
        let ti = TextInput {
            tp: TermPos::new(stdout),
        };
        ti._display_buffer();
        ti
    }

    pub fn handle_event(&mut self, event: TrueEvent) -> TextInputEvent {
        if let TrueEvent::KeyEvent(ke) = event {
            match ke {
                KeyEvent::Key(k) => {
                    match k {
                        Key::Char(c) => {
                            if c == '\n' {
                                let ret = self.tp.buffer.iter().fold(String::new(), |mut acc, &arg| { acc.push(arg); acc });
                                return TextInputEvent::Buffer(self.tp.buffer.clone(), ret);
                            } else {
                                self.tp.char(c);
                            }
                        },
                        Key::Backspace => self.tp.backspace(),
                        Key::Delete => self.tp.delete(),
                        Key::Left => self.tp.left(),
                        Key::Right => self.tp.right(),
                        Key::Up => self.tp.up(),
                        Key::Down => self.tp.down(),
                        Key::Home => self.tp.beg(),
                        Key::End => self.tp.end(),
                        Key::PageUp => self.tp.word_left(),
                        Key::PageDown => self.tp.word_right(),
                        _ => {}
                    }
                },
                KeyEvent::Ctrl(k) => {
                    match k {
                        Key::Char(c) => {
                            match c {
                                'c' | 'd' => {
                                    print!("Quit\r\n");
                                    return TextInputEvent::Quit;
                                }
                                'a' => self.tp.beg(),
                                'e' => self.tp.end(),
                                'l' => self.tp.clear_term(),
                                _ => {}
                            }
                        }
                        Key::Left => self.tp.word_left(),
                        Key::Right => self.tp.word_right(),
                        Key::Up => return TextInputEvent::HistoryPrev,
                        Key::Down => return TextInputEvent::HistoryNext,
                        _ => {}
                    }
                },
                _ => {}
            }
            self._display_buffer();
        }
        TextInputEvent::None
    }

    pub fn data(&self) -> Vec<char> {
        self.tp.buffer.clone()
    }

    pub fn set_data(&mut self, d: Vec<char>) {
        self.tp.buffer = d;
        self.tp.end();
        self._display_buffer();
    }

    fn _display_buffer(&self) {
        print!("{}{}{}{}{}",
               termion::cursor::Goto(1, self.tp.y),
               termion::clear::AfterCursor,
               PROMPT, self.tp.buffer.iter().fold(String::new(), |mut acc, &arg| { acc.push(arg); acc }),
               termion::cursor::Goto(self.tp.x, self.tp.y + self.tp.d_y));
        std::io::stdout().flush().unwrap();
    }
}
