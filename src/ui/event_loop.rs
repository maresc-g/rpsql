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
    let mut buffer_save: Vec<char> = Vec::new();
    for e in stdin.events() {
        let event = e.unwrap();
        let true_event = TrueEvent::from_termion_event(event);
        let res = ti.handle_event(true_event, stdout);
        match res {
            TextInputEvent::HistoryPrev => {
                if history.current_command() == -1 {
                    buffer_save = ti.data();
                }
                if let Some(b) = history.prev() {
                    ti.set_data(b, stdout);
                }
            },
            TextInputEvent::HistoryNext => {
                if history.current_command() > -1 {
                    if let Some(b) = history.next() {
                        ti.set_data(b, stdout);
                    }
                    else {
                        ti.set_data(buffer_save.clone(), stdout);
                    }
                }
                else {
                    ti.set_data(buffer_save.clone(), stdout);
                }
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

