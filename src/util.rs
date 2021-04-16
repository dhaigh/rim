use std::io::Write;

pub const CTRL_C: char = 0x03 as char;
pub const TAB: char = 0x09 as char;
pub const BACKSPACE: char = 0x7F as char;
pub const ENTER: char = 0x0D as char;
pub const ESC: char = 0x1B as char;

pub fn clear() {
    print!("\x1B[2J");
}

pub fn mv(x: usize, y: usize) {
    print!("\x1B[{};{}H", y+1, x+1);
}

pub fn flush() {
    std::io::stdout().flush().expect("Couldn't flush stdout");
}
