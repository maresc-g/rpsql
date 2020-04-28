use std::io::{Write, stdout, stdin};
use termion::event::Key;
use termion::input::TermRead;
use termion::cursor::DetectCursorPos;
use termion::event::Event;
use crate::history::History;

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
}

pub fn init() {
    write!(stdout(),
           "{}{}",
           termion::cursor::Goto(1, 1),
           termion::clear::All)
            .unwrap();
}

pub fn display_vec_on_new_line(tp: &mut TermPos, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>, v: &Vec<String>) {
    print!("\r\n");
    for s in v {
        print!("{}\r\n", s);
    }
    tp.set_cursor_pos(stdout);
}

pub fn display_string_on_new_line(tp: &mut TermPos, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>, s: &String) {
    print!("\r\n{}\r\n", s);
    tp.set_cursor_pos(stdout);
}

pub fn get_input(tp: &mut TermPos, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>, history: &mut History) -> Option<String> {
    tp.reset();
    write!(stdout,
           "{}",
           termion::cursor::Goto(1, tp.y))
            .unwrap();
    print!("{}", PROMPT);
    stdout.flush().unwrap();
    let stdin = stdin();
    let mut buffer_save: Vec<char> = Vec::new();
    for e in stdin.events() {
        let event = e.unwrap();
        match event {
            Event::Key(k) => {
                match k {
                    Key::Ctrl(c) => {
                        match c {
                            'c' | 'd' => {
                                println!("Quit");
                                return None;
                            }
                            'a' => tp.beg(),
                            'e' => tp.end(),
                            _ => {}
                        }
                    }
                    Key::Char(c) => {
                        tp.char(c);
                        if c == '\n' {
                            let ret = tp.buffer.iter().fold(String::new(), |mut acc, &arg| { acc.push(arg); acc });
                            if !ret.trim().is_empty() {
                                history.push(&tp.buffer);
                            }
                            history.reset_index();
                            return Some(ret);
                        }
                    },
                    Key::Backspace => tp.backspace(),
                    Key::Delete => tp.delete(),
                    Key::Left => tp.left(),
                    Key::Right => tp.right(),
                    Key::Up => tp.up(),
                    Key::Down => tp.down(),
                    Key::Home => tp.beg(),
                    Key::End => tp.end(),
                    Key::PageUp => tp.word_left(),
                    Key::PageDown => tp.word_right(),
                    _ => {}
                }
                _display_buffer(tp, stdout);
            },
            Event::Unsupported(v) => {
                let mut print = false;
                if v == [27, 91, 49, 59, 53, 67]  || v == [27, 79, 99] { // ctrl + right
                    tp.word_right();
                    print = true;
                }
                else if v == [27, 91, 49, 59, 53, 68]  || v == [27, 79, 100] { // ctrl + left
                    tp.word_left();
                    print = true;
                }
                else if v  == [27, 91, 49, 59, 53, 65] || v == [27, 79, 97] { // ctrl + up
                    if history.current_command() == -1 {
                        buffer_save = tp.buffer.clone();
                    }
                    if let Some(b) = history.prev() {
                        tp.buffer = b;
                    }
                    print = true;
                    tp.end();
                }
                else if v  == [27, 91, 49, 59, 53, 66] || v == [27, 79, 98] { // ctrl + down
                    if history.current_command() > -1 {
                        if let Some(b) = history.next() {
                            tp.buffer = b;
                        }
                        else {
                            tp.buffer = buffer_save.clone();
                        }
                    }
                    else {
                        tp.buffer = buffer_save.clone();
                    }
                    print = true;
                    tp.end();
                }
                if print {
                    _display_buffer(tp, stdout);
                }
            },
            _ => {}
        }
    }
    None
}

fn _display_buffer(tp: &mut TermPos, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>) {
   print!("{}{}{}{}{}",
           termion::cursor::Goto(1, tp.y),
           termion::clear::AfterCursor,
           PROMPT, tp.buffer.iter().fold(String::new(), |mut acc, &arg| { acc.push(arg); acc }),
           termion::cursor::Goto(tp.x, tp.y + tp.d_y));
    stdout.flush().unwrap();
}
