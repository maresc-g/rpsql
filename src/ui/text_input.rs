use termion::cursor::DetectCursorPos;
use crate::ui::event::{TrueEvent, KeyEvent};
use termion::event::Key;
use crate::history::History;
use std::io::{Write, stdout, stdin};

const PROMPT: &'static str = "$> ";

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
    pub fn new() -> TermPos {
        let (max_x, max_y) = termion::terminal_size().unwrap();
        TermPos {
            x: PROMPT.len() as u16 + 1,
            y: 1,
            buffer: Vec::new(),
            current_index: 0,
            d_y: 0,
            max_x: max_x,
            max_y: max_y,
            prompt_len: PROMPT.len()
        }
    }

    fn reset(&mut self) {
        self.buffer = Vec::new();
        self.current_index = 0;
        self.d_y = 0;
        self.x = self.prompt_len as u16 + 1;
    }

    fn set_cursor_pos(&mut self, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>) {
        let pos = stdout.cursor_pos().unwrap();
        self.x = pos.0;
        self.y = pos.1;
    }

    fn _compute_max_dy(&self) -> u16 {
        ((self.prompt_len + self.buffer.len()) / self.max_x as usize) as u16
    }

    fn char(&mut self, c: char) {
        if c == '\n' {
            self.y += self._compute_max_dy() + 1;
            self.y = std::cmp::min(self.y, self.max_y);
            if self.y == self.max_y {
                print!("{}\n", termion::cursor::Goto(1, self.y));
            }
        }
        else {
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

pub struct TextInput<'a> {
    tp: TermPos,
    prompt: String,
    history: &'a mut History,
    buffer_save: Vec<char>,
}

impl<'a> TextInput<'a> {
    pub fn new(prompt: &str, history: &'a mut History) -> TextInput<'a> {
        TextInput {
            tp: TermPos::new(),
            prompt: prompt.to_string(),
            history: history,
            buffer_save: Vec::new(),
        }
    }

    pub fn handle_event(&mut self, event: TrueEvent, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>) -> Option<String> {
        match event {
            TrueEvent::KeyEvent(ke) => {
                match ke {
                    KeyEvent::Key(k) => {
                        match k {
                            Key::Char(c) => {
                                self.tp.char(c);
                                if c == '\n' {
                                    let ret = self.tp.buffer.iter().fold(String::new(), |mut acc, &arg| { acc.push(arg); acc });
                                    if !ret.trim().is_empty() {
                                        self.history.push_and_save(&self.tp.buffer);
                                    }
                                    self.history.reset_index();
                                    return Some(ret);
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
                                        println!("Quit");
                                        return None;
                                    }
                                    'a' => self.tp.beg(),
                                    'e' => self.tp.end(),
                                    'l' => self.tp.clear_term(),
                                    _ => {}
                                }
                            }
                            Key::Left => self.tp.word_left(),
                            Key::Right => self.tp.word_right(),
                            Key::Up => {
                                if self.history.current_command() == -1 {
                                    self.buffer_save = self.tp.buffer.clone();
                                }
                                if let Some(b) = self.history.prev() {
                                    self.tp.buffer = b;
                                }
                                self.tp.end();
                            },
                            Key::Down => {
                                if self.history.current_command() > -1 {
                                    if let Some(b) = self.history.next() {
                                        self.tp.buffer = b;
                                    }
                                    else {
                                        self.tp.buffer = self.buffer_save.clone();
                                    }
                                }
                                else {
                                    self.tp.buffer = self.buffer_save.clone();
                                }
                                self.tp.end();
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
                _display_buffer(&mut self.tp, stdout);
            },
            _ => {}
        }
        None
    }
}

fn _display_buffer(tp: &mut TermPos, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>) {
    print!("{}{}{}{}{}",
           termion::cursor::Goto(1, tp.y),
           termion::clear::AfterCursor,
           PROMPT, tp.buffer.iter().fold(String::new(), |mut acc, &arg| { acc.push(arg); acc }),
           termion::cursor::Goto(tp.x, tp.y + tp.d_y));
    stdout.flush().unwrap();
}