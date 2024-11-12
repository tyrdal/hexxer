use colored::*;
use std::env;
use std::fs::File;
use std::io::{self, Read};

fn colorize(text: &str, color: Color, use_color: bool) -> String {
    if use_color {
        text.color(color).to_string()
    } else {
        text.to_string()
    }
}

fn hexdump(filename: &str, use_color: bool) -> io::Result<()> {
    let mut file = File::open(filename)?;
    let mut buffer = [0u8; 16]; // Read in chunks of 16 bytes

    let mut offset = 0;
    while let Ok(bytes_read) = file.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        // Print offset (yellow if color is enabled)
        print!(
            "{}",
            colorize(&format!("{:08x}: ", offset), Color::Yellow, use_color)
        );

        // Print hex values (cyan if color is enabled)
        for i in 0..bytes_read {
            if i == 8 {
                print!(" "); // Extra space in the middle for readability
            }
            print!(
                "{}",
                colorize(&format!("{:02x} ", buffer[i]), Color::Cyan, use_color)
            );
        }

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
        print!("|");
        for &byte in &buffer[..bytes_read] {
            let ch = if byte.is_ascii_graphic() || byte.is_ascii_whitespace() {
                colorize(&format!("{}", byte as char), Color::Green, use_color)
            } else {
                colorize(".", Color::Red, use_color)
            };
            print!("{}", ch);
        }
        println!("|");

        offset += bytes_read;
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <filename> [--color]", args[0]);
        return;
    }

    let filename = &args[1];
    let use_color = args.get(2).map_or(false, |arg| arg == "--color");

    if let Err(e) = hexdump(filename, use_color) {
        eprintln!("Error: {}", e);
    }
}
