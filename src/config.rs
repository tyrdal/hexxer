use std::path::PathBuf;

use clap::builder::styling;
use clap::{Arg, Command};

#[derive(Debug)]
pub struct Config {
    pub input: Option<PathBuf>,
    pub plain: bool,
    pub cols: u16,
    pub grouping: u16,
    pub seek: usize,
    pub offset: usize,
    pub length: usize,
}

impl Config {
    pub fn new() -> Self {
        let cli = parse_cli();

        let input = cli.get_one::<String>("input").map(PathBuf::from);
        let plain = cli.get_flag("plain");
        let cols = cli
            .get_one::<u16>("cols")
            .copied()
            .unwrap_or(if plain { 30 } else { 16 });
        let grouping = cli.get_one::<u16>("grouping").copied().unwrap_or(2u16);
        let seek = cli.get_one::<usize>("seek").copied().unwrap_or(0usize);
        let offset = cli.get_one::<usize>("offset").copied().unwrap_or(0usize);
        let length = cli.get_one::<usize>("length").copied().unwrap_or(0usize);

        Self {
            input,
            plain,
            cols,
            grouping,
            seek,
            offset,
            length
        }
    }
}

const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::Blue.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());

fn parse_cli() -> clap::ArgMatches {
    Command::new("hexxer")
        .styles(STYLES)
        .version("0.1.0")
        .author("Tyrdal <tyrdal@gmx.de>")
        .about("A clap learning tool")
        .arg(
            Arg::new("input")
                .help("Sets the input file to use, or '-' for stdin.")
                .index(1),
        )
        .arg(
            Arg::new("plain")
                .short('p')
                .long("plain")
                .help("plain text (hex only)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("cols")
                .short('c')
                .long("columns")
                .help("Display <columns> octets per line. Default: 16 (-P/--plain: 30 ) A 0 results in one long line of output.")
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("grouping")
                .short('g')
                .long("grouping")
                .help("Number of octets per group. Default: 2. Not compatible with -P/--plain.")
                .conflicts_with("plain")
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("seek")
                .short('s')
                .long("seek")
                .help("Seek to <offset> before reading.")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("offset")
                .short('o')
                .long("offset")
                .help("Read <offset> bytes from the input.")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("length")
                .short('l')
                .long("length")
                .help("Stop after <length> octets. A 0 means no limit.")
                .value_parser(clap::value_parser!(usize)),
        )
        .get_matches()
}
