#![allow(dead_code)] // TODO: Remove this once everything is implemented

mod config;
use config::color_choice::ColorChoice;

use owo_colors::{OwoColorize, Stream::Stdout};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;

const SPACE: u8 = 0x20;
const NUL: u8 = 0x00;

// static mut CONFIG : config = onfig::new();

enum Format {
    Hexadecimal,
    Octal,
    Decimal,
    Binary,
}

// #[derive(Debug)]
// struct FrameChars {
//     top_left: char,
//     horizontal: char,
//     vertical: char,
//     top_joint: char,
//     top_right: char,
//     bottom_left: char,
//     bottom_joint: char,
//     bottom_right: char,
// }
//
// impl FrameChars {
//     fn classic() -> Self {
//         Self {
//             top_left: '┌',
//             horizontal: '─',
//             vertical: '│',
//             top_joint: '┬',
//             top_right: '┐',
//             bottom_left: '└',
//             bottom_joint: '┴',
//             bottom_right: '┘',

//         }
//     }
// }

fn discard_bytes<R: Read>(mut reader: R, mut to_skip: usize) -> io::Result<R> {
    let mut buffer = [0u8; 4096];

    while to_skip > 0 {
        let read_len = std::cmp::min(buffer.len(), to_skip);
        let n = reader.read(&mut buffer[..read_len])?;
        if n == 0 {
            break; // EOF
        }
        to_skip -= n;
    }

    Ok(reader)
}

fn colorize(text: &str, color: owo_colors::CssColors, color_choice: ColorChoice) -> String {
    match color_choice {
        ColorChoice::Auto => text
            .if_supports_color(Stdout, |text| text.color(color))
            .to_string(),
        ColorChoice::Never => text.to_string(),
        ColorChoice::Always => text.color(color).to_string(),
    }
}

// fn draw_frame_top(config: &config::Config) {
//     let mut line = String::new();
//     line.push(FrameChars::classic().top_left);
//     (0..config.cols).for_each(|_| {
//         line.push(FrameChars::classic().horizontal);
//     });
//     println!("{}", line);
//     //         FrameChars::classic().horizontal,
//     //         FrameChars::classic().top_joint,
//     //         FrameChars::classic().top_right,
//     //         FrameChars::classic().bottom_left,
//     //         FrameChars::classic().bottom_joint
//     //     )?;
//     // }
// }

// TODO refactor coloring, too much repetition here
// FIXME cols 0 does not work
fn dump<R: Read>(mut reader: R, config: &config::Config) -> io::Result<()> {
    let octets_per_line = config.cols as usize;
    let mut buffer = vec![0u8; octets_per_line]; // Read in chunks of octets_per_line bytes
    let mut offset = config.offset;
    let mut total_read: usize = 0;

    if total_read < config.length || config.length == 0 {
        let mut row_flag = true;
        loop {
            let to_read: usize = if config.length == 0 {
                octets_per_line
            } else {
                std::cmp::min(octets_per_line, config.length - total_read)
            };
            let bytes_read = reader.read(&mut buffer[..to_read])?;
            if bytes_read == 0 {
                // EOF reached
                break;
            }

            if config.plain {
                for &byte in &buffer[..bytes_read] {
                    print!("{:02x}", byte);
                }
                println!();
            } else {
                if config.show_offset {
                    if config.decimal_offset {
                        print!(
                            "{}",
                            colorize(
                                &format!("{:08}: ", offset),
                                if row_flag {
                                    owo_colors::CssColors::LightBlue
                                } else {
                                    owo_colors::CssColors::CadetBlue
                                },
                                config.color_choice,
                            )
                        );
                    } else {
                        print!(
                            "{}",
                            colorize(
                                &format!("{:08x}: ", offset),
                                if row_flag {
                                    owo_colors::CssColors::LightBlue
                                } else {
                                    owo_colors::CssColors::CadetBlue
                                },
                                config.color_choice,
                            )
                        );
                    }
                }

                (0..bytes_read).for_each(|i| {
                    if i != 0 && i % config.grouping as usize == 0 {
                        print!(" "); // Extra space to separate groups
                    }
                    print!(
                        "{}",
                        if buffer[i].is_ascii_graphic() {
                            colorize(
                                &format!("{:02x}", buffer[i]),
                                if row_flag {
                                    owo_colors::CssColors::LightSteelBlue
                                } else {
                                    owo_colors::CssColors::LightSlateGray
                                },
                                config.color_choice,
                            )
                        } else if buffer[i] == NUL {
                            colorize(
                                &format!("{:02x}", buffer[i]),
                                if row_flag {
                                    owo_colors::CssColors::DimGray
                                } else {
                                    owo_colors::CssColors::DarkGray
                                },
                                config.color_choice,
                            )
                        } else if buffer[i].is_ascii_control() || buffer[i] == SPACE {
                            colorize(
                                &format!("{:02x}", buffer[i]),
                                if row_flag {
                                    owo_colors::CssColors::LawnGreen
                                } else {
                                    owo_colors::CssColors::GreenYellow
                                },
                                config.color_choice,
                            )
                        } else {
                            colorize(
                                &format!("{:02x}", buffer[i]),
                                if row_flag {
                                    owo_colors::CssColors::LightCoral
                                } else {
                                    owo_colors::CssColors::IndianRed
                                },
                                config.color_choice,
                            )
                        }
                    );
                });
                print!(" ");

                // Pad for short lines
                if bytes_read < octets_per_line {
                    for i in bytes_read..octets_per_line {
                        print!("  ");
                        if i % config.grouping as usize == 0 {
                            print!(" ");
                        }
                    }
                }

                if config.show_text {
                    for &byte in &buffer[..bytes_read] {
                        let ch = if byte.is_ascii_graphic() {
                            colorize(
                                &format!("{}", byte as char),
                                if row_flag {
                                    owo_colors::CssColors::LightBlue
                                } else {
                                    owo_colors::CssColors::CadetBlue
                                },
                                config.color_choice,
                            )
                        } else if byte == NUL {
                            colorize(
                                &format!("{}", char::from_u32(0x2400 + byte as u32).unwrap_or('�')),
                                if row_flag {
                                    owo_colors::CssColors::DimGray
                                } else {
                                    owo_colors::CssColors::DarkGray
                                },
                                config.color_choice,
                            )
                        } else if byte.is_ascii_control() || byte == SPACE {
                            colorize(
                                &format!("{}", char::from_u32(0x2400 + byte as u32).unwrap_or('�')),
                                if row_flag {
                                    owo_colors::CssColors::LawnGreen
                                } else {
                                    owo_colors::CssColors::GreenYellow
                                },
                                config.color_choice,
                            )
                        } else {
                            colorize(
                                ".",
                                if row_flag {
                                    owo_colors::CssColors::LightCoral
                                } else {
                                    owo_colors::CssColors::IndianRed
                                },
                                config.color_choice,
                            )
                        };
                        print!("{}", ch);
                    }
                }
                println!();
            }

            total_read += bytes_read;
            offset += bytes_read;
            row_flag = !row_flag;
        }
    }

    Ok(())
}

fn get_reader(input: Option<&PathBuf>, seek: i64) -> io::Result<Box<dyn Read>> {
    match input {
        Some(path) => {
            let mut file = File::open(path)?;
            file.seek(if seek >= 0 {
                SeekFrom::Start(seek.unsigned_abs())
            } else {
                SeekFrom::End(seek)
            })?;
            Ok(Box::new(file))
        }
        None => {
            if seek < 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Seeking to negative offsets is not supported when reading from stdin",
                ));
            }
            let mut stdin = io::stdin().lock();
            if seek > 0 {
                discard_bytes(&mut stdin, seek.unsigned_abs() as usize)?;
            }
            Ok(Box::new(stdin))
        }
    }
}

fn main() -> std::io::Result<()> {
    let config = config::Config::new();
    dump(get_reader(config.input.as_ref(), config.seek)?, &config)?;
    Ok(())
}
