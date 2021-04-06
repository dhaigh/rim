mod esc;
mod buffer;
use buffer::{Buffer, NORMAL_MODE, INSERT_MODE, COMMAND_MODE};
use std::env;
use std::io::{Read, Write};
use std::fs;

fn flush() {
    std::io::stdout().flush().expect("Couldn't flush stdout");
}

const CTRL_C: char = 0x03 as char;
const TAB: char = 0x09 as char;
const BACKSPACE: char = 0x7F as char;
const ENTER: char = 0x0D as char;
const ESC: char = 0x1B as char;

fn input_loop(buffer: &mut Buffer) {
    buffer.redraw();

    for b in std::io::stdin().bytes() {
        let c = b.unwrap() as char;

        match buffer.mode {
            NORMAL_MODE => {
                match c {
                    'h' => {
                        buffer.left();
                    },

                    'l' => {
                        buffer.right();
                    },

                    'k' => {
                        buffer.up();
                    },

                    'j' => {
                        buffer.down();
                    },

                    'i' => {
                        buffer.mode_insert();
                    },

                    'a' => {
                        buffer.x = match buffer.line().len() {
                            0 => 0,
                            _ => buffer.x + 1,
                        };
                        buffer.mode_insert();
                    },

                    'I' => {
                        buffer.jump_line_start();
                        buffer.mode_insert();
                    },

                    'A' => {
                        buffer.jump_line_end();
                        buffer.x += 1;
                        buffer.mode_insert();
                    },

                    'D' => {
                        buffer.del_after();
                        buffer.redraw();
                    },

                    'C' => {
                        buffer.del_after();
                        buffer.x = match buffer.x {
                            0 => 0,
                            _ => buffer.x + 1,
                        };
                        buffer.redraw();
                        buffer.mode_insert();
                    },

                    'x' => {
                        buffer.del();
                        buffer.redraw();
                    },

                    'X' => {
                        buffer.x = match buffer.x {
                            0 => 0,
                            _ => buffer.x - 1,
                        };
                        buffer.del();
                        buffer.redraw();
                    },

                    'o' => {
                        buffer.insert_line(buffer.y + 1);
                        buffer.y += 1;
                        buffer.x = 0;
                        buffer.mode_insert();
                        buffer.redraw();
                    },

                    'O' => {
                        buffer.insert_line(buffer.y);
                        buffer.x = 0;
                        buffer.mode_insert();
                        buffer.redraw();
                    },

                    's' => {
                        buffer.del();
                        buffer.mode_insert();
                        buffer.redraw();
                    },

                    'S' => {
                        buffer.x = 0;
                        buffer.del_after();
                        buffer.mode_insert();
                        buffer.redraw();
                    },

                    '^' => {
                        buffer.jump_line_start();
                    },

                    '$' => {
                        buffer.jump_line_end();
                    },

                    '0' => {
                        buffer.jump_line_start_abs();
                    },

                    '-' => {
                        buffer.jump_line_end();
                    },

                    ':' => {
                        buffer.mode_command();
                    },

                    'q' => {
                        break;
                    },

                    _ => {}
                }
            },

            INSERT_MODE => {
                match c {
                    ESC | CTRL_C => {
                        buffer.mode_normal();
                    },

                    TAB => {
                        buffer.insert_string(&String::from("    "));
                        buffer.redraw();
                    },

                    ENTER => {
                        buffer.enter();
                        buffer.redraw();
                    },

                    BACKSPACE => {
                        buffer.backspace();
                        buffer.redraw();
                    },

                    _ => {
                        buffer.insert(c);
                        buffer.redraw();
                    },
                }
            },

            COMMAND_MODE => {
                match c {
                    CTRL_C => {
                        buffer.command = String::from("");
                        buffer.mode_normal();
                        buffer.redraw();
                    },

                    ENTER => {
                        match buffer.command.as_str() {
                            "w" => {
                                buffer.save();
                            },
                            "q" => {
                                break;
                            },
                            "wq" => {
                                buffer.save();
                                break;
                            },
                            _ => {},
                        }
                    },

                    _ => {
                        buffer.type_command(c);
                        buffer.redraw();

                        // todo: refactor
                        esc::mv(buffer.command.len() + 1, buffer.height - 1);
                        flush();
                    },
                }
            },


            _ => {}
        }
    }
}

fn open_file() -> Result<(), &'static str> {
    match env::args().nth(1) {
        Some(filename) => {
            match fs::read_to_string(&filename) {
                Ok(string) => {
                    input_loop(&mut Buffer::new(&string, filename));
                    Ok(())
                },
                Err(_) => Err("Couldn't open file"),
            }
        },
        None => Err("Filename not given"),
    }
}

fn main() {
    std::process::exit(match open_file() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("Error: {}", err);
            1
        },
    });
}
