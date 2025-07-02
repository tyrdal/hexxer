pub mod color_choice;
use color_choice::{ColorChoice, LineColorConfig};

use std::path::PathBuf;
use std::process;

use clap::builder::styling;
use clap::{Arg, ArgMatches, Command, ValueEnum, arg, value_parser};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Language {
    C,
    Cpp,
    Rust,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum Format {
    Hexadecimal,
    Octal,
    Decimal,
    Binary,
}

impl Format {
    pub fn value(&self, val: u8) -> String {
        match self {
            Format::Hexadecimal => format!("{val:02x}"),
            Format::Octal => format!("{val:03o}"),
            Format::Decimal => format!("{val:03}"),
            Format::Binary => format!("{val:08b}"),
        }
    }
}

#[derive(Debug)]
pub enum SubCommand {
    Dump,
    Generate,
    Reverse,
}

#[derive(Debug)]
pub struct Config {
    pub color_choice: ColorChoice,
    pub colors: LineColorConfig,
    pub cols: u16,
    pub decimal_offset: bool,
    pub format: Format,
    pub grouping: u16,
    pub input: Option<PathBuf>,
    pub language: Language,
    pub length: usize,
    pub offset: usize,
    pub plain: bool,
    pub seek: i64,
    pub subcommand: SubCommand,
    pub show_offset: bool,
    pub show_text: bool,
    pub vector: bool,
}

fn parse_dump(matches: &ArgMatches, config: &mut Config) {
    config.subcommand = SubCommand::Dump;

    config.input = matches.get_one::<String>("infile").map(PathBuf::from);
    config.plain = matches.get_flag("plain");
    config.cols = matches
        .get_one::<u16>("cols")
        .copied()
        .unwrap_or(if config.plain { 30 } else { 16 });
    config.format = matches
        .get_one::<Format>("format")
        .expect("Invalid format choice")
        .to_owned();
    config.grouping = matches.get_one::<u16>("grouping").copied().unwrap_or(2u16);
    config.seek = matches.get_one::<i64>("seek").copied().unwrap_or(0i64);
    config.offset = matches
        .get_one::<usize>("display_offset")
        .copied()
        .unwrap_or(0usize);
    config.length = matches
        .get_one::<usize>("length")
        .copied()
        .unwrap_or(usize::MAX);
    config.show_offset = !matches.get_flag("no-offset");
    config.show_text = !matches.get_flag("no-text");
    config.decimal_offset = matches.get_flag("decimal-offset");
    config.color_choice = matches
        .get_one::<ColorChoice>("color")
        .expect("Invalid color choice")
        .to_owned();
}

fn parse_generate(matches: &ArgMatches, config: &mut Config) {
    if matches.get_flag("list-languages") {
        let langs: Vec<_> = Language::value_variants()
            .iter()
            .map(ToString::to_string)
            .collect();
        for lang in langs.iter() {
            println!("{lang}");
        }
        process::exit(0);
    }

    config.subcommand = SubCommand::Generate;

    config.input = matches.get_one::<String>("infile").map(PathBuf::from);
    config.cols = matches.get_one::<u16>("cols").copied().unwrap_or(12);
    config.seek = matches.get_one::<i64>("seek").copied().unwrap_or(0i64);
    config.length = matches
        .get_one::<usize>("length")
        .copied()
        .unwrap_or(0usize);
    config.language = matches
        .get_one::<Language>("language")
        .expect("Invalid language choice")
        .to_owned();
}

fn parse_reverse(matches: &ArgMatches, config: &mut Config) {
    config.subcommand = SubCommand::Reverse;

    config.input = matches.get_one::<String>("infile").map(PathBuf::from);
    config.plain = matches.get_flag("plain");
    config.length = matches
        .get_one::<usize>("length")
        .copied()
        .unwrap_or(0usize);
}

impl Config {
    pub fn new() -> Self {
        let cli = parse_cli();
        let mut config = Config {
            cols: 0,
            color_choice: ColorChoice::Auto,
            colors: LineColorConfig::default(),
            decimal_offset: false,
            format: Format::Hexadecimal,
            grouping: 0,
            input: None,
            language: Language::C,
            length: usize::MAX,
            plain: false,
            seek: 0,
            subcommand: SubCommand::Dump,
            offset: 0,
            show_offset: false,
            show_text: false,
            vector: false,
        };

        match cli.subcommand() {
            Some(("dump", dump)) => parse_dump(dump, &mut config),
            Some(("generate", generate)) => parse_generate(generate, &mut config),
            Some(("reverse", reverse)) => parse_reverse(reverse, &mut config),
            _ => process::exit(0), // we should never get here
        }

        config
    }
}

const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::Blue.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());

fn parse_cli() -> clap::ArgMatches {
    let input_arg = arg!([infile] "Sets the input file to use, if not present stdin is used.");

    Command::new("hexxer")
        .styles(STYLES)
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .help_template(
            "\
{name} {version}
{about}

usage:
    {usage}

options:
{all-args}
"
        )
        .subcommand_required(true)
        .subcommand(
            Command::new("dump")
                .about("Dump a file to the terminal")
                .visible_alias("dp")
                .arg(input_arg.clone())
                .arg(
                    arg!( -p --plain "Plain text.")
                        .conflicts_with("display_offset")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("no-offset")
                        .long("no-offset").help( "Don't show the offset part.")
                        .default_value("false")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("no-text")
                        .long("no-text")
                        .help( "Don't show the text part.")
                        .default_value("false")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("decimal-offset")
                        .short('d')
                        .long("decimal-offset")
                        .help("Show offset in decimal instead of hex.")
                        .default_value("false")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    arg!(-c --cols <columns> "Display <columns> octets per line. [default: 16 (-p/--plain: 30)] With -p/--plain, 0 results in one long line of output.")
                        .num_args(1)
                        .value_parser(clap::value_parser!(u16)),
                )
                .arg(
                    arg!( -f --format <format> "Dump format.")
                        .num_args(1)
                        .default_value("hexadecimal")
                        .value_parser(value_parser!(Format)),
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
                    arg!(-s --seek <offset> "Seek to <offset> before dumping.")
                        .num_args(1)
                        .allow_negative_numbers(true)
                        .value_parser(clap::value_parser!(i64)),
                )
                .arg(
                    arg!(-o --display_offset <offset> "Add <offset> to the displayed file position.")
                        .value_parser(clap::value_parser!(usize)),
                )
                .arg(
                    arg!(-l --length <length> "Stop after <length> octets.")
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
                        .default_value("auto")
                        .value_parser(value_parser!(ColorChoice)),
                )
        )
        .subcommand(
            Command::new("generate")
                .about("Generate a source code array.")
                .visible_alias("gen")
                .arg(input_arg.clone())
                .arg(
                    Arg::new("language")
                    .short('L')
                    .long("language")
                    .help("Generate code for this language.")
                    .num_args(1)
                    .default_value("c")
                    .value_parser(value_parser!(Language)),
                )
                .arg(
                    arg!(-C --capitalize "Capitalize the variable name.")
                        .default_value("false")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    arg!(-c --cols <columns> "Print <columns> octets per line. [default: 12] A value of 0 results in one long line of output.")
                        .num_args(1)
                        .value_parser(clap::value_parser!(u16)),
                )
                .arg(
                    arg!(-l --length <length> "Stop after <length> octets.")
                        .num_args(1)
                        .value_parser(clap::value_parser!(usize)),
                )
                .arg(
                    arg!(-n --name <variable_name> "Name of the variable to generate.")
                        .num_args(1)
                        .value_parser(clap::value_parser!(String)),
                )
                .arg(
                    arg!(-s --seek <offset> "Seek to <offset> before generating.")
                        .num_args(1)
                        .allow_negative_numbers(true)
                        .value_parser(clap::value_parser!(i64)),
                )
                .arg(
                    Arg::new("list-languages")
                        .long("list-languages")
                        .help("Lists all supported languages for code generation.")
                        .default_value("false")
                        .action(clap::ArgAction::SetTrue)
                        .exclusive(true)
                )
                .arg(
                    arg!( -v --vector "Generate a vector(dynmaic array) instead of an array if applicable to the language.")
                        .default_value("false")
                        .action(clap::ArgAction::SetTrue),
                )
        )
        .subcommand(
            // Attention: xxd  reverse actually takes the offsets into account and does not blindly read
            // just the hex part
            Command::new("reverse")
                .about("Reverse a hexdump.")
                .visible_alias("rev")
                .arg(input_arg.clone())
                .arg(
                    arg!( -p --plain "Plain text (hex only).")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    arg!(-l --length <length> "Stop after <length> octets.")
                        .num_args(1)
                        .value_parser(clap::value_parser!(usize)),
                )
                .arg(
                    arg!(-s --seek <offset> "Add <offset> to the file positions found in the infile before appplying.")
                        .num_args(1)
                        .allow_negative_numbers(true)
                        .value_parser(clap::value_parser!(i64)),
                )
        )
        .get_matches()
}
