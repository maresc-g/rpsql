use std::io::{Write, stdout, stdin};
use termion::input::TermRead;
use crate::history::History;
use crate::ui::event::{TrueEvent, KeyEvent};
use crate::ui::text_input::{TermPos, TextInput};

const PROMPT: &'static str = "$> ";

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

pub fn get_input(tp: &mut TermPos, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>, history: &mut History) -> Option<String> {
    // tp.reset();
    // write!(stdout,
    //        "{}",
    //        termion::cursor::Goto(1, tp.y))
    //         .unwrap();
    print!("{}", PROMPT);
    stdout.flush().unwrap();
    let stdin = stdin();
    let mut ti = TextInput::new(PROMPT, history);
    for e in stdin.events() {
        let event = e.unwrap();
        let true_event = TrueEvent::from_termion_event(event);
        ti.handle_event(true_event, stdout);
    }
    None
}

