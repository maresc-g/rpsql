use std::io::{Write, stdout, stdin};
use termion::event::Key;
use termion::input::TermRead;

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
}

pub fn init() {
    write!(stdout(),
           "{}{}",
           termion::cursor::Goto(1, 1),
           termion::clear::All)
            .unwrap();
}

pub fn display_vec_on_new_line(tp: &mut TermPos, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>, v: &Vec<String>) {
    tp.x = 1;
    for s in v {
        write!(stdout,
               "{}{}\n",
               termion::cursor::Goto(1, tp.y),
               s)
                .unwrap();
        tp.y += 1;
        tp.y = std::cmp::min(tp.y, tp.max_y);
    }
    stdout.flush().unwrap();
}

pub fn display_string_on_new_line(tp: &mut TermPos, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>, s: &String) {
    tp.x = 1;
    write!(stdout,
           "{}{}\n",
           termion::cursor::Goto(1, tp.y),
           s)
            .unwrap();
    tp.y += 1;
    tp.y = std::cmp::min(tp.y, tp.max_y);
    stdout.flush().unwrap();
}

pub fn get_input(tp: &mut TermPos, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>) -> Option<String> {
    tp.reset();
    write!(stdout,
           "{}",
           termion::cursor::Goto(1, tp.y))
            .unwrap();
    print!("{}", PROMPT);
    stdout.flush().unwrap();
    let stdin = stdin();
    for c in stdin.keys() {
        match c.unwrap() {
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
                    return Some(tp.buffer.iter().fold(String::new(), |mut acc, &arg| { acc.push(arg); acc }));
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
            _ => {}
        }
        write!(stdout,
               "{}{}{}{}{}",
               termion::cursor::Goto(1, tp.y),
               termion::clear::AfterCursor,
               PROMPT, tp.buffer.iter().fold(String::new(), |mut acc, &arg| { acc.push(arg); acc }),
               termion::cursor::Goto(tp.x, tp.y + tp.d_y))
                .unwrap();
        stdout.flush().unwrap();
    }
    None
}
