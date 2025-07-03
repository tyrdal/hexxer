#![allow(dead_code)] // TODO: Remove this once everything is implemented

mod config;

use config::{Language, SubCommand, color_choice::ColorChoice};
use owo_colors::{OwoColorize, Stream::Stdout};
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process;

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

#[allow(clippy::needless_range_loop)]
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
                write!(io::stdout(), "{}", config.format.value(byte))?;
            }
            if config.cols > 0 {
                writeln!(io::stdout(),)?;
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
                write!(io::stdout(), "{offset_block}",)?;
            }

            for i in 0..bytes_read {
                // TODO: use iterators so that write! can be used with ? operator
                if i != 0 && i % config.grouping as usize == 0 {
                    write!(io::stdout(), " ")?; // Extra space to separate groups
                }

                write!(
                    io::stdout(),
                    "{}",
                    colorize(
                        &config.format.value(buffer[i]),
                        if buffer[i].is_ascii_graphic() {
                            config.colors.dump_text.get(row_flag)
                        } else if buffer[i] == NUL {
                            config.colors.nul_char.get(row_flag)
                        } else if buffer[i].is_ascii_control() || buffer[i] == SPACE {
                            config.colors.control_char.get(row_flag)
                        } else {
                            config.colors.undefined_char.get(row_flag)
                        },
                        config.color_choice
                    )
                )?;
            }
            write!(io::stdout(), " ")?;

            // Pad for short lines
            if bytes_read < octets_per_line {
                for i in bytes_read..octets_per_line {
                    write!(io::stdout(), "  ")?;
                    if i % config.grouping as usize == 0 {
                        write!(io::stdout(), " ")?;
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
                    write!(io::stdout(), "{ch}")?;
                }
            }
            writeln!(io::stdout(),)?;
        }

        total_read += bytes_read;
        offset += bytes_read;
        row_flag = !row_flag;
    }

    Ok(())
}

fn generate_array<R: Read>(mut reader: R, config: &config::Config) -> io::Result<()> {
    let var_name = if config.capitalize {
        config.var_name.to_uppercase()
    } else {
        config.var_name.clone()
    };

    let octets_per_line = if config.cols > 0 {
        config.cols as usize
    } else {
        12
    };
    let mut buffer = vec![0u8; octets_per_line]; // Read in chunks of octets_per_line bytes
    let mut total_read: usize = 0;

    if config.input.is_some() {
        let array_size = std::cmp::min(
            fs::metadata(config.input.as_ref().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "No input file specified")
            })?)
            .map(|meta| meta.len())
            .expect("Could not get file size!")
                - config.seek.unsigned_abs(),
            config.length as u64,
        );
        match config.language {
            Language::C => {
                writeln!(
                    io::stdout(),
                    "#include <stdint.h>\n\nuint8_t {var_name}[] = {{"
                )?;
            }
            Language::Cpp => {
                if config.vector {
                    writeln!(
                        io::stdout(),
                        "#include <vector>\n#include <cstdint>\n\nstd::vector<uint8_t> {var_name} = {{"
                    )?;
                } else {
                    writeln!(
                        io::stdout(),
                        "#include <array>\n#include <cstdint>\n\nstd::array<uint8_t, {array_size}> {var_name} = {{"
                    )?;
                }
            }
            Language::Rust => {
                if config.vector {
                    writeln!(io::stdout(), "pub let {var_name} = vec![")?;
                } else {
                    writeln!(io::stdout(), "pub const {var_name}: [u8; {array_size}] = [")?;
                }
            }
            Language::Python => {
                writeln!(io::stdout(), "{var_name} = [")?;
            }
        }
    }

    loop {
        let to_read: usize = std::cmp::min(octets_per_line, config.length - total_read);
        let bytes_read = reader.read(&mut buffer[..to_read])?;
        if bytes_read == 0 {
            // EOF reached
            break;
        }

        write!(io::stdout(), "  ")?;
        // avoid trailing space for last byte
        for (i, &byte) in buffer[..bytes_read].iter().enumerate() {
            if i > 0 {
                write!(io::stdout(), ", ")?;
            }
            write!(io::stdout(), "0x{}", config.format.value(byte))?;
        }
        writeln!(io::stdout(), ",")?;

        total_read += bytes_read;
    }

    if config.input.is_some() {
        match config.language {
            Language::C | Language::Cpp => {
                writeln!(io::stdout(), "}};")?;
            }
            Language::Python => {
                writeln!(io::stdout(), "]")?;
            }
            Language::Rust => {
                writeln!(io::stdout(), "];")?;
            }
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

fn run() -> io::Result<()> {
    let config = config::Config::new()?;
    let reader = get_reader(config.input.as_ref(), config.seek)?;
    match config.subcommand {
        SubCommand::Dump => dump(reader, &config)?,
        SubCommand::Generate => generate_array(reader, &config)?,
        SubCommand::Reverse => todo!(),
    };
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        if err.kind() == io::ErrorKind::BrokenPipe {
            process::exit(0);
        }
        eprintln!("Error: {err}");
        process::exit(1);
    }
}
