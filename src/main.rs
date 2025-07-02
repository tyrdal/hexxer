#![allow(dead_code)] // TODO: Remove this once everything is implemented

mod config;

use config::{SubCommand, color_choice::ColorChoice};
use owo_colors::{OwoColorize, Stream::Stdout};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;

const SPACE: u8 = 0x20;
const NUL: u8 = 0x00;

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

fn colorize(text: &str, color: owo_colors::DynColors, color_choice: ColorChoice) -> String {
    match color_choice {
        ColorChoice::Auto => text
            .if_supports_color(Stdout, |text| text.color(color))
            .to_string(),
        ColorChoice::Never => text.to_string(),
        ColorChoice::Always => text.color(color).to_string(),
    }
}

fn dump<R: Read>(mut reader: R, config: &config::Config) -> io::Result<()> {
    let octets_per_line = if config.cols > 0 {
        config.cols as usize
    } else {
        32
    };
    let mut buffer = vec![0u8; octets_per_line]; // Read in chunks of octets_per_line bytes
    let mut offset = config.offset;
    let mut total_read: usize = 0;

    let mut row_flag = true;
    loop {
        let to_read: usize = std::cmp::min(octets_per_line, config.length - total_read);
        let bytes_read = reader.read(&mut buffer[..to_read])?;
        if bytes_read == 0 {
            // EOF reached
            break;
        }

        if config.plain {
            for &byte in &buffer[..bytes_read] {
                print!("{}", config.format.value(byte));
            }
            if config.cols > 0 {
                println!();
            }
        } else {
            if config.show_offset {
                let offset_block = if config.decimal_offset {
                    colorize(
                        &format!("{offset:08}: "),
                        config.colors.panel_text.get(row_flag),
                        config.color_choice,
                    )
                } else {
                    colorize(
                        &format!("{offset:08x}: "),
                        config.colors.panel_text.get(row_flag),
                        config.color_choice,
                    )
                };
                print!("{offset_block}",);
            }

            (0..bytes_read).for_each(|i| {
                if i != 0 && i % config.grouping as usize == 0 {
                    print!(" "); // Extra space to separate groups
                }
                print!(
                    "{}",
                    if buffer[i].is_ascii_graphic() {
                        colorize(
                            &config.format.value(buffer[i]),
                            config.colors.dump_text.get(row_flag),
                            config.color_choice,
                        )
                    } else if buffer[i] == NUL {
                        colorize(
                            &config.format.value(buffer[i]),
                            config.colors.nul_char.get(row_flag),
                            config.color_choice,
                        )
                    } else if buffer[i].is_ascii_control() || buffer[i] == SPACE {
                        colorize(
                            &config.format.value(buffer[i]),
                            config.colors.control_char.get(row_flag),
                            config.color_choice,
                        )
                    } else {
                        colorize(
                            &config.format.value(buffer[i]),
                            config.colors.undefined_char.get(row_flag),
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
                            config.colors.panel_text.get(row_flag),
                            config.color_choice,
                        )
                    } else if byte == NUL {
                        colorize(
                            &format!("{}", char::from_u32(0x2400 + byte as u32).unwrap_or('�')),
                            config.colors.nul_char.get(row_flag),
                            config.color_choice,
                        )
                    } else if byte.is_ascii_control() || byte == SPACE {
                        colorize(
                            &format!("{}", char::from_u32(0x2400 + byte as u32).unwrap_or('�')),
                            config.colors.control_char.get(row_flag),
                            config.color_choice,
                        )
                    } else {
                        colorize(
                            ".",
                            config.colors.undefined_char.get(row_flag),
                            config.color_choice,
                        )
                    };
                    print!("{ch}");
                }
            }
            println!();
        }

        total_read += bytes_read;
        offset += bytes_read;
        row_flag = !row_flag;
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
    match config.subcommand {
        SubCommand::Dump => dump(get_reader(config.input.as_ref(), config.seek)?, &config)?,
        SubCommand::Generate => todo!(),
        SubCommand::Reverse => todo!(),
    }
    Ok(())
}
