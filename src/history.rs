use std::fs;
use std::io::{self, Read, Write};
use std::env;

#[derive(Debug)]
pub struct History {
    commands: Vec<Vec<char>>,
    current_command: i32,
    file: Option<fs::File>,
}

impl History {
    fn new() -> History {
        return History {
            commands: Vec::new(),
            current_command: -1,
            file: None,
        }
    }

    pub fn current_command(&self) -> i32 {
        self.current_command
    }

    pub fn prev(&mut self) -> Option<Vec<char>> {
        if self.current_command >= self.commands.len() as i32 - 1 {
            return None;
        }

        self.current_command += 1;
        Some(self.commands.get(self.current_command as usize).unwrap().clone())
    }

    pub fn next(&mut self) -> Option<Vec<char>> {
        if self.current_command == 0 {
            return None;
        }

        self.current_command -= 1;
        Some(self.commands.get(self.current_command as usize).unwrap().clone())
    }

    pub fn push_and_save(&mut self, buffer: &Vec<char>) {
        if self.commands.len() == 0 || self.commands.get(0).unwrap() != buffer {
            self.commands.insert(0, buffer.clone());
            if let Some(f) = &mut self.file {
                let mut s = buffer.iter().fold(String::new(), |mut acc, &arg| { acc.push(arg); acc });
                s.push('\n');
                f.write(s.as_bytes()).unwrap();
            }
        }
    }

    pub fn push(&mut self, buffer: &Vec<char>) {
        if self.commands.len() == 0 || self.commands.get(0).unwrap() != buffer {
            self.commands.insert(0, buffer.clone());
        }
    }

    pub fn reset_index(&mut self) {
        self.current_command = -1;
    }

    pub fn load_from_file() -> History {
        let mut history = History::new();
        let mut path = dirs::config_dir().unwrap_or_else(|| {
            let mut p = std::path::PathBuf::new();
            p.push(env::var("HOME").unwrap_or_default());
            p
        });
        path.push("rpsql");
        let res = fs::create_dir_all(&path);
        if let Err(err) = res {
            match err.kind() {
                io::ErrorKind::NotFound => {
                    if let Err(e) = fs::create_dir_all(&path) {
                        eprintln!("Could not create directory {} : {}", path.to_str().unwrap(), e);
                    }
                },
                _ => {
                    eprintln!("Error while accessing directory {} : {}", path.to_str().unwrap(), err);
                }
            }
        }
        else {
            path.push("history");
            let f = fs::OpenOptions::new().read(true).create(true).write(true).append(true).open(&path);
            match f {
                Ok(mut file) => {
                    println!("Ok");
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).unwrap();
                    for l in contents.lines() {
                        if !l.trim().is_empty() {
                            history.push(&l.chars().collect());
                        }
                    }
                    history.file = Some(file);
                },
                Err(err) => {
                    eprintln!("Could not open history file {} : {}", path.to_str().unwrap(), err);
                }
            }
        }
        history
    }
}
