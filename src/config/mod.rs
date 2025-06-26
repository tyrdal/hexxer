pub mod color_choice;
use clap::value_parser;
use color_choice::ColorChoice;

use std::path::PathBuf;

use clap::builder::styling;
use clap::{Arg, Command};

#[derive(Debug)]
pub struct Config {
    pub input: Option<PathBuf>,
    pub plain: bool,
    pub cols: u16,
    pub grouping: u16,
    pub seek: i64,
    pub offset: usize,
    pub length: usize,
    pub show_offset: bool,
    pub show_text: bool,
    pub decimal_offset: bool,
    pub color_choice: ColorChoice,
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
        let seek = cli.get_one::<i64>("seek").copied().unwrap_or(0i64);
        let offset = cli.get_one::<usize>("offset").copied().unwrap_or(0usize);
        let length = cli.get_one::<usize>("length").copied().unwrap_or(0usize);
        let show_offset = !cli.get_flag("no_offset");
        let show_text = !cli.get_flag("no_text");
        let decimal_offset = cli.get_flag("decimal_offset");
        let color_choice = cli
            .get_one::<ColorChoice>("color")
            .expect("Invalid color choice")
            .to_owned();

        Self {
            input,
            plain,
            cols,
            grouping,
            seek,
            offset,
            length,
            show_offset,
            show_text,
            decimal_offset,
            color_choice,
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
        .about("A hexdump tool")
        .arg(
            Arg::new("input")
                .help("Sets the input file to use, if not present stdin is used.")
                .index(1),
        )
        .arg(
            Arg::new("plain")
                .short('p')
                .long("plain")
                .help("Plain text (hex only).")
                .conflicts_with("offset")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no_offset")
                .long("no-offset")
                .help("Don't show the offset part")
                .default_value("false")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no_text")
                .long("no-text")
                .help("Don't show the text part")
                .default_value("false")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("decimal_offset")
                .short('d')
                .long("decimal")
                .help("Show offset in decimal instead of hex")
                .default_value("false")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("cols")
                .short('c')
                .long("columns")
                .help("Display <columns> octets per line. [default: 16 (-p/--plain: 30)] With -p/--plain, 0 results in one long line of output.")
                .num_args(1)
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("grouping")
                .short('g')
                .long("grouping")
                .help("Number of octets per group. [default: 2] Not compatible with -P/--plain.")
                .num_args(1)
                .conflicts_with("plain")
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("seek")
                .value_name("position")
                .short('s')
                .long("seek")
                .help("Seek to <offset> before reading.")
                .num_args(1)
                .allow_negative_numbers(true)
                .value_parser(clap::value_parser!(i64)),
        )
        .arg(
            Arg::new("offset")
                .value_name("display offset")
                .short('o')
                .long("offset")
                .help("Add <offset> to the displayed file position.")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("length")
                .short('l')
                .long("length")
                .help("Stop after <length> octets.")
                .num_args(1)
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .alias("colour")
                .help("Color output. [default: auto]")
                .num_args(1)
                .value_name("when")
                .value_parser(value_parser!(ColorChoice)),
        )
        .get_matches()
}
