#![allow(dead_code)] // TODO: Remove this once everything is implemented
mod config;

use colored::*;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;

const SPACE: u8 = 0x20;

enum Format {
    Hexadecimal,
    Octal,
    Decimal,
    Binary,
}

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

fn colorize(text: &str, color: Color, use_color: bool) -> String {
    if use_color {
        text.color(color).to_string()
    } else {
        text.to_string()
    }
}

fn dump<R: Read>(mut reader: R, config: config::Config) -> io::Result<()> {
    let octets_per_line = config.cols as usize;
    let mut buffer = vec![0u8; octets_per_line]; // Read in chunks of octets_per_line bytes
    let mut offset = config.offset;
    let mut total_read: usize = 0;

    if total_read < config.length || config.length == 0 {
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
                print!("{:08x}: ", offset);
                (0..bytes_read).for_each(|i| {
                    if i != 0 && i % config.grouping as usize == 0 {
                        print!(" "); // Extra space to separate groups
                    }
                    print!("{:02x}", buffer[i]);
                });
                print!(" ");

                // Pad for short lines
                if bytes_read < octets_per_line {
                    for i in bytes_read..octets_per_line {
                        print!("  ");
                        if i == config.grouping as usize {
                            print!(" ");
                        }
                    }
                }

                // Print text representation
                for &byte in &buffer[..bytes_read] {
                    let ch = if byte.is_ascii_graphic() || byte == SPACE {
                        colorize(&format!("{}", byte as char), Color::Green, true)
                    } else if byte.is_ascii_control() {
                        colorize(
                            &format!("{}", char::from_u32(0x2400 + byte as u32).unwrap_or('�')),
                            Color::Blue,
                            true,
                        )
                    } else {
                        colorize(".", Color::Red, true)
                    };
                    print!("{}", ch);
                }
                println!();
            }

            total_read += bytes_read;
            offset += bytes_read
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
    dump(get_reader(config.input.as_ref(), config.seek)?, config)?;
    Ok(())
}
