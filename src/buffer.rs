use crate::esc;
use std::io::Write;
use std::fs;

pub const NORMAL_MODE: char = 'N';
pub const NORMAL_PREFIX_MODE: char = 'P';
pub const INSERT_MODE: char = 'I';
pub const COMMAND_MODE: char = 'C';

fn flush() {
    std::io::stdout().flush().expect("Couldn't flush stdout");
}

pub struct Buffer {
    // The changes to the current buffer are maintained in this string vec
    pub lines: Vec<String>,

    // The filename we read from and will write to
    pub filename: String,

    // NORMAL_MODE or INSERT_MODE or COMMAND_MODE
    pub mode: char, // todo: restrict to known modes?

    // Characters you type afrer pressing ":" e.g. "wq"
    pub command: String,

    // x/y position of cursor
    pub x: usize,
    pub y: usize,

    // width/height of terminal
    pub width: usize,
    pub height: usize,

    // Does cursor cling to the end of the line? Set to true after pressing $
    pub cling_to_end: bool,

    pub normal_prefix: String,
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

            cling_to_end: false,
            normal_prefix: String::from(""),
        }
    }

    // jumps
    // -------------------------------------------------------------------------
    pub fn left(&mut self) {
        self.cling_to_end = false;
        self.add_x(-1);
        self.mv();
    }

    pub fn right(&mut self) {
        self.cling_to_end = false;
        self.add_x(1);
        self.mv();
    }

    pub fn up(&mut self) {
        self.add_y(-1);
        self.add_x(0);
        self.mv();
    }

    pub fn down(&mut self) {
        self.add_y(1);
        self.add_x(0);
        self.mv();
    }

    pub fn jump_line_start(&mut self) {
        self.x = 0;
        loop {
            match self.line().chars().nth(self.x) {
                Some(' ') => {
                    self.x += 1;
                },
                Some(_) | None => {
                    break;
                },
            }
        }
        self.mv();
    }

    pub fn jump_line_start_abs(&mut self) {
        self.x = 0;
        self.mv();
    }

    pub fn jump_line_end_abs(&mut self) {
        self.cling_to_end = true;
        let len = self.line().len();
        self.x = match len {
            0 => 0,
            _ => len - 1,
        };
        self.mv();
    }

    pub fn jump_top(&mut self) {
        self.y = 0;
        self.add_x(0);
        self.mv();
    }

    pub fn jump_middle(&mut self) {
        self.y = self.lines.len() / 2;
        self.add_x(0);
        self.mv();
    }

    pub fn jump_bottom(&mut self) {
        self.y = self.lines.len() - 1;
        self.add_x(0);
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

    // prefixed commands
    // -------------------------------------------------------------------------
    pub fn prefix(&mut self, prefix_char: char) {
        let mut matched = true;
        self.normal_prefix.push_str(&prefix_char.to_string());
        match self.normal_prefix.as_ref() {
            "dd" => {
                self.lines.remove(self.y);
                if self.y == self.lines.len() {
                    self.y -= 1;
                }
            },
            _ => {
                matched = false;
            },
        };
        if matched {
            self.mode_normal();
        }
    }

    // mode commands
    // -------------------------------------------------------------------------
    fn set_mode(&mut self, mode: char) {
        self.mode = mode;
        self.draw_mode();
        self.mv();
    }

    pub fn mode_normal(&mut self) {
        self.normal_prefix = String::from("");
        self.command = String::from("");
        self.set_mode(NORMAL_MODE);
        self.left();
    }

    pub fn mode_normal_prefix(&mut self) {
        self.set_mode(NORMAL_PREFIX_MODE);
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
        self.draw_mode();
        self.draw_prefix();
        self.draw_command();
        self.mv();
    }

    fn draw_mode(&self) {
        esc::mv(1, self.height - 2);
        print!("{}", match self.mode {
            'N' | 'P' => "NORMAL ",
            'I'       => "INSERT ",
            'C'       => "COMMAND",
            _ => "",
        });
    }

    fn draw_prefix(&self) {
        esc::mv(self.width - 10, self.height - 2);
        print!("{}", self.normal_prefix);
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
    pub fn type_command(&mut self, command_char: char) {
        self.command.push_str(&command_char.to_string());
    }

    pub fn save(&self) {
        fs::write(&self.filename, self.stringify()).expect("Write failed");
    }

    // helpers
    // -------------------------------------------------------------------------
    pub fn line(&self) -> &String {
        &self.lines[self.y]
    }

    pub fn add_x(&mut self, n: isize) {
        let sum: isize = (self.x as isize) + n;
        let len = self.line().chars().count() as isize;

        // todo: refactor + simplify
        self.x = match self.cling_to_end {
            true => {
                if len == 0 {
                    0
                } else {
                    (len - 1) as usize
                }
            },
            false => {
                if sum < 0 {
                    0
                } else {
                    if len == 0 {
                        0
                    } else if sum >= len - 1 {
                        (len - 1) as usize
                    } else {
                        sum as usize
                    }
                }
            },
        };
    }

    pub fn add_y(&mut self, n: isize) {
        let sum: isize = (self.y as isize) + n;

        // todo: refactor + simplify
        self.y = if sum < 0 {
            0
        } else {
            let len = self.lines.len() as isize;
            if len == 0 {
                0
            } else if sum >= len - 1 {
                (len - 1) as usize
            } else {
                sum as usize
            }
        }
    }

    fn stringify(&self) -> String {
        let mut string = String::from("");

        for line in &self.lines {
            string.push_str(line);
            string.push('\n');
        }

        string
    }
}
