use termion::screen::AlternateScreen;
use std::io::{Stdout, Write};
use std::cmp::{min, max};
use crate::ui::event::{TrueEvent, KeyEvent, Key};

pub struct TextView<'a> {
    data: &'a[String],
    x: usize,
    y: usize,
    max_x: u16,
    max_y: u16,
    max_len: usize,
    #[allow(dead_code)]
    screen: AlternateScreen<Stdout>,
}

impl<'a> TextView<'a> {
    pub fn new(data: &'a[String]) -> TextView<'a> {
        let (max_x, max_y) = termion::terminal_size().unwrap();
        let mut tv = TextView {
            data,
            x: 0,
            y: 0,
            max_x,
            max_y,
            max_len: 0,
            screen: AlternateScreen::from(std::io::stdout())
        };
        tv.max_len = tv._get_current_slice_max_len();
        tv
    }

    pub fn display(&self) {
        print!("{}{}", termion::cursor::Goto(1,1), termion::clear::All);
        let rows = self._get_row_slice();
        for (i, string) in rows.iter().enumerate() {
            let columns = self._get_column_slice(string);
            print!("{}", columns);
            if i < rows.len() - 1 {
                print!("\r\n");
            }
        }
        std::io::stdout().flush().unwrap();
    }

    pub fn handle_event(&mut self, event: TrueEvent) -> bool {
        let mut display = false;
        let mut again = true;
        if let TrueEvent::KeyEvent(ke) = event {
            match ke {
                KeyEvent::Key(k) => {
                    match k {
                        Key::Down => {
                            if self.y + (self.max_y as usize) < self.data.len() {
                                self.y += 1;
                                display = true;
                                self.max_len = self._get_current_slice_max_len();
                            }
                        },
                        Key::Up => {
                            if self.y > 0 {
                                self.y -= 1;
                                display = true;
                                self.max_len = self._get_current_slice_max_len();
                            }
                        },
                        Key::Right => {
                            if self.x + (self.max_x as usize) < self.max_len {
                                self.x += 1;
                                display = true;
                            }
                        },
                        Key::Left => {
                            if self.x > 0 {
                                self.x -= 1;
                                display = true;
                            }
                        },
                        _ => {
                            print!("{}", termion::screen::ToMainScreen);
                            std::io::stdout().flush().unwrap();
                            again = false;
                        }
                    }
                },
                _ => {}
            }
        }

        if display && again {
            self.display();
        }

        again
    }

    fn _get_row_slice(&self) -> &[String] {
        &self.data[self.y..min(self.max_y as usize + self.y, self.data.len())]
    }

    fn _get_column_slice(&self, columns: &'a String) -> &'a str {
        columns.get(self.x..min(self.max_x as usize + self.x, columns.len())).unwrap()
    }

    fn _get_current_slice_max_len(&self) -> usize {
        let rows = self._get_row_slice();
        let mut max_len: usize = 0;
        for string in rows {
            max_len = max(max_len, string.len());
        }
        max_len
    }
}