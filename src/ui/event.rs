pub use termion::event::{Event, Key, MouseEvent};

pub enum KeyEvent {
    Key(Key),
    Ctrl(Key),
    Alt(Key),
}

pub enum TrueEvent {
    KeyEvent(KeyEvent),
    MouseEvent(MouseEvent),
    Unsupported(Vec<u8>),
}

impl TrueEvent {
    pub fn from_termion_event(event: Event) -> TrueEvent {
        match event {
            Event::Key(k) => {
                match k {
                    Key::Ctrl(c) => TrueEvent::KeyEvent(KeyEvent::Ctrl(Key::Char(c))),
                    Key::Alt(c) => TrueEvent::KeyEvent(KeyEvent::Alt(Key::Char(c))),
                    _ => TrueEvent::KeyEvent(KeyEvent::Key(k))
                }
            },
            Event::Mouse(m) => TrueEvent::MouseEvent(m),
            Event::Unsupported(v) => {
                if v == [27, 91, 49, 59, 53, 67]  || v == [27, 79, 99] { // ctrl + right
                    TrueEvent::KeyEvent(KeyEvent::Ctrl(Key::Right))
                } else if v == [27, 91, 49, 59, 53, 68]  || v == [27, 79, 100] { // ctrl + left
                    TrueEvent::KeyEvent(KeyEvent::Ctrl(Key::Left))
                } else if v  == [27, 91, 49, 59, 53, 65] || v == [27, 79, 97] { // ctrl + up
                    TrueEvent::KeyEvent(KeyEvent::Ctrl(Key::Up))
                } else if v  == [27, 91, 49, 59, 53, 66] || v == [27, 79, 98] { // ctrl + down
                    TrueEvent::KeyEvent(KeyEvent::Ctrl(Key::Down))
                } else {
                    TrueEvent::Unsupported(v)
                }
            }
        }
    }
}
