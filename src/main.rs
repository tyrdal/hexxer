mod config;

use colored::*;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

const BUFFER_SIZE: usize = 16;

enum Format {
    Hexadecimal,
    Octal,
    Decimal,
    Binary,
}

fn colorize(text: &str, color: Color, use_color: bool) -> String {
    if use_color {
        text.color(color).to_string()
    } else {
        text.to_string()
    }
}

fn dump<R: Read>(mut reader: R, config: config::Config) -> io::Result<()> {
    let mut buffer = [0u8; BUFFER_SIZE]; // Read in chunks of 16 bytes
    let mut offset = config.offset;
    let mut total_read: usize = 0;

    if total_read < config.length {
        loop {
            let to_read = std::cmp::min(BUFFER_SIZE, config.length - total_read);
            let bytes_read = reader.read(&mut buffer[..to_read])?;
            if bytes_read == 0 || total_read >= config.length {
                // EOF reached
                break;
            }

            print!("{:08x}: ", offset);
            (0..bytes_read).for_each(|i| {
                if i == 8 {
                    print!(" "); // Extra space in the middle for readability
                }
                print!("{:02x} ", buffer[i]);
            });

            // Pad for short lines
            if bytes_read < 16 {
                for i in bytes_read..16 {
                    print!("   ");
                    if i == 7 {
                        print!(" ");
                    }
                }
            }

            // Print ASCII representation
            for &byte in &buffer[..bytes_read] {
                let ch = if byte.is_ascii_graphic() || byte.is_ascii_whitespace() {
                    colorize(&format!("{}", byte as char), Color::Green, true)
                } else {
                    colorize(".", Color::Red, true)
                };
                print!("{}", ch);
            }
            println!();

            total_read += bytes_read;
            offset += bytes_read
        }
        //
        //     let mut offset = 0;
        //     while let Ok(bytes_read) = file.read(&mut buffer) {
        //         if bytes_read == 0 {
        //             break;
        //         }
        //
        //         // Print offset (yellow if color is enabled)
        //         print!(
        //             "{}",
        //             colorize(&format!("{:08x}: ", offset), Color::Yellow, use_color)
        //         );
        //
        //         // Print hex values (cyan if color is enabled)
        //         for i in 0..bytes_read {
        //             if i == 8 {
        //                 print!(" "); // Extra space in the middle for readability
        //             }
        //             print!(
        //                 "{}",
        //                 colorize(&format!("{:02x} ", buffer[i]), Color::Cyan, use_color)
        //             );
        //         }
        //
        //         // Pad for short lines
        //         if bytes_read < 16 {
        //             for i in bytes_read..16 {
        //                 print!("   ");
        //                 if i == 7 {
        //                     print!(" ");
        //                 }
        //             }
        //         }
        //
        //         // Print ASCII representation
        //         print!("|");
        //         for &byte in &buffer[..bytes_read] {
        //             let ch = if byte.is_ascii_graphic() || byte.is_ascii_whitespace() {
        //                 colorize(&format!("{}", byte as char), Color::Green, use_color)
        //             } else {
        //                 colorize(".", Color::Red, use_color)
        //             };
        //             print!("{}", ch);
        //         }
        //         println!("|");
        //
        //         offset += bytes_read;
        //     }
    }

    Ok(())
}

fn get_reader(input: Option<&PathBuf>) -> io::Result<Box<dyn Read>> {
    match input {
        Some(path) => Ok(Box::new(File::open(path)?)),
        None => Ok(Box::new(io::stdin().lock())),
    }
}

fn main() -> std::io::Result<()> {
    let config = config::Config::new();
    dump(get_reader(config.input.as_ref())?, config)?;
    Ok(())
}
