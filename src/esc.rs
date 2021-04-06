pub fn clear() {
    print!("\x1B[2J\x1B[1;1H");
}

pub fn mv(x: usize, y: usize) {
    print!("\x1B[{};{}H", y+1, x+1);
}
