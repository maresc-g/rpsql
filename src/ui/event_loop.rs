use std::io::{Write, stdout, stdin};
use termion::input::TermRead;
use crate::history::History;
use crate::ui::event::{TrueEvent, KeyEvent};
use crate::ui::text_input::{TermPos, TextInput, TextInputEvent};

const PROMPT: &'static str = "$> ";

pub enum Event {
    Quit,
    Buffer(String),
    None,
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
    // tp.set_cursor_pos(stdout);
}

pub fn display_string_on_new_line(tp: &mut TermPos, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>, s: &String) {
    print!("\r\n{}\r\n", s);
    // tp.set_cursor_pos(stdout);
}

pub fn get_input(tp: &mut TermPos, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>, history: &mut History) -> Event {
    // tp.reset();
    // write!(stdout,
    //        "{}",
    //        termion::cursor::Goto(1, tp.y))
    //         .unwrap();
    print!("{}", PROMPT);
    stdout.flush().unwrap();
    let stdin = stdin();
    let mut ti = TextInput::new(PROMPT);
    for e in stdin.events() {
        let event = e.unwrap();
        let true_event = TrueEvent::from_termion_event(event);
        let res = ti.handle_event(true_event, stdout);
        match res {
            TextInputEvent::HistoryPrev => {
                // if self.history.current_command() == -1 {
                //     self.buffer_save = self.tp.buffer.clone();
                // }
                // if let Some(b) = self.history.prev() {
                //     self.tp.buffer = b;
                // }
                // self.tp.end();
            },
            TextInputEvent::HistoryNext => {
                // if self.history.current_command() > -1 {
                //     if let Some(b) = self.history.next() {
                //         self.tp.buffer = b;
                //     }
                //     else {
                //         self.tp.buffer = self.buffer_save.clone();
                //     }
                // }
                // else {
                //     self.tp.buffer = self.buffer_save.clone();
                // }
                // self.tp.end();
            },
            TextInputEvent::None => {}
            TextInputEvent::Quit => return Event::Quit,
            TextInputEvent::Buffer(raw, buffer) => {
                history.push_and_save(&raw);
                history.reset_index();
                return Event::Buffer(buffer)
            }
        }
    }
    Event::None
}

