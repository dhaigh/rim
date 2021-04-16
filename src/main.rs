mod esc;
mod buffer;
use buffer::{
    Buffer,
    NORMAL_MODE,
    NORMAL_PREFIX_MODE,
    INSERT_MODE,
    COMMAND_MODE,
};
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

    // todo: read utf-8 characters instead of bytes
    for b in std::io::stdin().bytes() {
        let c = b.unwrap() as char;

        match buffer.mode {
            NORMAL_MODE => {
                match c {
                    // jumps
                    // ---------------------------------------------------------
                    'h' => { buffer.left(); },
                    'l' => { buffer.right(); },
                    'k' => { buffer.up(); },
                    'j' => { buffer.down(); },
                    '^' => { buffer.jump_line_start(); },
                    '0' => { buffer.jump_line_start_abs(); },
                    '$' => { buffer.jump_line_end_abs(); },
                    'H' => { buffer.jump_top(); },
                    'M' => { buffer.jump_middle(); },
                    'L' => { buffer.jump_bottom(); },

                    // mode change
                    // ---------------------------------------------------------
                    ':' => {
                        buffer.mode_command();
                    },

                    'i' => {
                        buffer.mode_insert();
                    },

                    'a' => {
                        buffer.add_x(1);
                        buffer.mode_insert();
                    },

                    'I' => {
                        buffer.jump_line_start();
                        buffer.mode_insert();
                    },

                    'A' => {
                        buffer.jump_line_end_abs();
                        buffer.x += 1;
                        buffer.mode_insert();
                    },

                    // modification
                    // ---------------------------------------------------------
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

                    // modification with prefix
                    // ---------------------------------------------------------
                    'd' => {
                        buffer.mode_normal_prefix();
                        buffer.prefix('d');
                        buffer.redraw();
                    },

                    _ => {}
                }
            },

            NORMAL_PREFIX_MODE => {
                match c {
                    CTRL_C => {
                        buffer.mode_normal();
                        buffer.redraw();
                    },

                    _ => {
                        buffer.prefix(c);
                        buffer.redraw();
                    },
                }
            },

            INSERT_MODE => {
                match c {
                    ESC | CTRL_C => {
                        buffer.mode_normal();
                    },

                    TAB => {
                        // tabs are 4 spaces--deal with it htaters
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
                        buffer.mode_normal();
                        buffer.redraw();
                    },

                    ENTER => {
                        match buffer.command.as_str() {
                            "w" => {
                                buffer.save();
                                buffer.mode_normal();
                                buffer.redraw();
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

            _ => {
                panic!("Unknown mode");
            },
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
