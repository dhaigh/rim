use crate::esc;
use std::io::Write;
use std::fs;

pub const NORMAL_MODE: char = 'N';
pub const INSERT_MODE: char = 'I';
pub const COMMAND_MODE: char = 'C';

fn flush() {
    std::io::stdout().flush().expect("Couldn't flush stdout");
}

pub struct Buffer {
    pub filename: String,
    pub lines: Vec<String>,
    pub mode: char, // todo: restrict to known modes?
    pub command: String,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Buffer {
    pub fn new(string: &String, filename: String) -> Self {
        let mut lines: Vec<String> = Vec::new();

        for line in string.lines() {
            lines.push(line.to_owned());
        }

        Self {
            filename,
            lines,
            mode: NORMAL_MODE,
            command: String::from(""),
            x: 0,
            y: 0,

            // todo: use a crate to un-hardcode this and listen for SIGWINCH
            width: 80,
            height: 24,
        }
    }

    // jumps
    // -------------------------------------------------------------------------
    pub fn left(&mut self) {
        self.x = if self.x == 0 {
            self.x
        } else {
            self.x - 1
        };
        self.mv();
    }

    pub fn right(&mut self) {
        let len = self.line().len();
        self.x = if len == 0 {
            0
        } else if self.x == len - 1 {
            self.x
        } else {
            self.x + 1
        };
        self.mv();
    }

    pub fn up(&mut self) {
        self.y = if self.y == 0 {
            self.y
        } else {
            self.y - 1
        };
        self.mv();
    }

    pub fn down(&mut self) {
        self.y = if self.y == self.lines.len() - 1 {
            self.y
        } else {
            self.y + 1
        };
        self.mv();
    }

    pub fn jump_line_start(&mut self) {
        let mut i = 0;
        loop {
            match self.line().chars().nth(i) {
                Some(' ') => {
                    i += 1;
                },
                Some(_) | None => {
                    break;
                },
            }
        }
        self.x = i;
        self.mv();
    }

    pub fn jump_line_start_abs(&mut self) {
        self.x = 0;
        self.mv();
    }

    pub fn jump_line_end(&mut self) {
        let len = self.line().len();
        self.x = match len {
            0 => 0,
            _ => len - 1,
        };
        self.mv();
    }

    // mutation
    // -------------------------------------------------------------------------
    fn leading_whitespace(&self) -> bool {
        for c in self.line()[..self.x].chars() {
            if c != ' ' {
                return false;
            }
        }
        return true;
    }

    pub fn backspace(&mut self) {
        if self.x == 0 {
            return;
        }

        let x: usize = match self.leading_whitespace() {
            true => {
                match self.x % 4 {
                    0 => self.x - 4,
                    1 => self.x - 1,
                    2 => self.x - 2,
                    3 => self.x - 3,
                    _ => self.x,
                }
            },
            false => {
                self.x - 1
            },
        };

        let mut modified = self.line()[..x].to_owned();
        modified.push_str(&self.line()[self.x..]);
        self.lines[self.y] = modified.to_owned();
        self.x = x;
    }

    pub fn del(&mut self) {
        let mut modified = self.line()[..self.x].to_owned();
        modified.push_str(&self.line()[(self.x + 1)..]);
        self.lines[self.y] = modified.to_owned();
    }

    pub fn del_after(&mut self) {
        self.lines[self.y] = self.line()[..self.x].to_owned();
        if self.x > 0 {
            self.x -= 1;
        }
    }

    pub fn insert_line(&mut self, index: usize) {
        let new_len = self.lines.len() + 1;
        self.lines.resize(new_len, "".to_owned());

        let mut y = new_len - 1;
        while y > index {
            self.lines.swap(y, y - 1);
            y -= 1;
        }
    }

    pub fn enter(&mut self) {
        self.insert_line(self.y + 1);
        self.lines[self.y+1] = self.line()[self.x..].to_owned();
        self.lines[self.y] = self.line()[..self.x].to_owned();
        self.y += 1;
        self.x = 0;
    }

    pub fn insert_string(&mut self, string: &String) {
        let mut line = self.line()[..self.x].to_owned();
        line.push_str(string);
        line.push_str(&self.line()[self.x..]);
        self.lines[self.y] = line.to_owned();
        self.x += string.len();
    }

    pub fn insert(&mut self, char_to_insert: char) {
        self.insert_string(&char_to_insert.to_string());
    }

    // mode commands
    // -------------------------------------------------------------------------
    fn set_mode(&mut self, mode: char) {
        self.mode = mode;
        self.draw_mode();
        self.mv();
    }

    pub fn mode_normal(&mut self) {
        self.set_mode(NORMAL_MODE);
        self.left();
    }

    pub fn mode_insert(&mut self) {
        self.set_mode(INSERT_MODE);
    }

    pub fn mode_command(&mut self) {
        self.set_mode(COMMAND_MODE);
        self.draw_command();
        flush();
    }

    // draw commands
    // -------------------------------------------------------------------------
    pub fn redraw(&self) {
        esc::clear();
        for (i, line) in self.lines.iter().enumerate() {
            esc::mv(0, i);
            print!("{}", line);
            flush();
        }
        self.draw_command();
        self.draw_mode();
        self.mv();
    }

    fn draw_mode(&self) {
        esc::mv(1, self.height - 2);
        print!("{}", match self.mode {
            'N' => "NORMAL ",
            'I' => "INSERT ",
            'C' => "COMMAND",
            _ => "",
        });
    }

    fn draw_command(&self) {
        if self.mode == COMMAND_MODE {
            esc::mv(0, self.height - 1);
            print!(":{}", self.command);
        }
    }

    fn mv(&self) {
        esc::mv(self.x, self.y);
        flush();
    }

    // command mode
    // -------------------------------------------------------------------------
    pub fn type_command(&mut self, char_to_insert: char) {
        self.command.push_str(&char_to_insert.to_string());
    }

    pub fn save(&self) {
        fs::write(&self.filename, self.stringify()).expect("Write failed");
    }

    // helpers
    // -------------------------------------------------------------------------
    pub fn line(&self) -> &String {
        &self.lines[self.y]
    }

    fn stringify(&self) -> String {
        let mut string = String::from("");

        for line in &self.lines {
            string.push_str(&line);
            string.push('\n');
        }

        string
    }
}
