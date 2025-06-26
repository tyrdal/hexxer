use clap::{builder::PossibleValue, ValueEnum};
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorChoice {
    Auto,
    Never,
    Always,
}

impl std::fmt::Debug for ColorChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl std::fmt::Display for ColorChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ValueEnum for ColorChoice {
    fn value_variants<'a>() -> &'a [Self] {
        &[ColorChoice::Auto, ColorChoice::Never, ColorChoice::Always]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            ColorChoice::Auto => PossibleValue::new("auto").help("swiftly"),
            ColorChoice::Never => PossibleValue::new("never").help("Crawl slowly but steadily"),
            ColorChoice::Always => PossibleValue::new("always").help(" slowly but steadily"),
        })
    }
}

impl FromStr for ColorChoice {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, false) {
                return Ok(*variant);
            }
        }
        Err(format!("invalid variant: {s}"))
    }
}
