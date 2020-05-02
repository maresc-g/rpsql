use std::io::{stdin};
use termion::input::TermRead;
use crate::history::History;
use crate::ui::event::TrueEvent;
use crate::ui::text_input::{TextInput, TextInputEvent};

pub enum Event {
    Quit,
    Buffer(String),
    None,
}

pub fn init() {
   print!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All);
}

pub fn display_vec(v: &[String]) {
    for s in v {
        display_string(s);
    }
}

pub fn display_string(s: &str) {
    print!("{}\r\n", s);
}

pub fn get_input(stdout: &mut termion::raw::RawTerminal<std::io::Stdout>, history: &mut History) -> Event {
    let stdin = stdin();
    let mut ti = TextInput::new(stdout);
    let mut buffer_save: Vec<char> = Vec::new();
    for e in stdin.events() {
        let event = e.unwrap();
        let true_event = TrueEvent::from_termion_event(event);
        let res = ti.handle_event(true_event);
        match res {
            TextInputEvent::HistoryPrev => {
                if history.current_command() == -1 {
                    buffer_save = ti.data();
                }
                if let Some(b) = history.prev() {
                    ti.set_data(b);
                }
            },
            TextInputEvent::HistoryNext => {
                if history.current_command() > -1 {
                    if let Some(b) = history.next() {
                        ti.set_data(b);
                    }
                    else {
                        ti.set_data(buffer_save.clone());
                    }
                }
                else {
                    ti.set_data(buffer_save.clone());
                }
            },
            TextInputEvent::None => {}
            TextInputEvent::Quit => return Event::Quit,
            TextInputEvent::Buffer(raw, buffer) => {
                if !buffer.is_empty() {
                    history.push_and_save(&raw);
                }
                history.reset_index();
                return Event::Buffer(buffer)
            }
        }
    }
    Event::None
}

