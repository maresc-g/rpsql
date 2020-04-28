#[derive(Debug)]
pub struct History {
    commands: Vec<Vec<char>>,
    current_command: i32,
}

impl History {
    pub fn new() -> History {
        return History {
            commands: Vec::new(),
            current_command: -1,
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

    pub fn push(&mut self, buffer: &Vec<char>) {
        if self.commands.len() == 0 || self.commands.get(0).unwrap() != buffer {
            self.commands.insert(0, buffer.clone());
        }
    }

    pub fn reset_index(&mut self) {
        self.current_command = -1;
    }
}
