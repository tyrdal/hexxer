use clap::ValueEnum;
use owo_colors::{CssColors, DynColors };

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Never,
    Always,
}

#[derive(Debug, Copy, Clone)]
pub struct LineColors {
    normal: DynColors,
    alternate: DynColors,
}

impl LineColors {

    pub fn get(&self, is_alternate: bool) -> DynColors
    {
        if is_alternate {
            self.alternate
        } else {
            self.normal
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LineColorConfig {
    pub panel_text: LineColors,
    pub dump_text: LineColors,
    pub nul_char: LineColors,
    pub control_char: LineColors,
    pub undefined_char: LineColors,
}

impl Default for LineColorConfig {
    fn default() -> Self {
        Self {
            panel_text: LineColors {
                normal: DynColors::Css(CssColors::LightBlue),
                alternate: DynColors::Css(CssColors::CadetBlue),
            },
            dump_text: LineColors {
                normal: DynColors::Css(CssColors::LightSteelBlue),
                alternate: DynColors::Css(CssColors::LightSlateGray),
            },
            nul_char: LineColors {
                normal: DynColors::Css(CssColors::DimGray),
                alternate: DynColors::Css(CssColors::DarkGray),
            },
            control_char: LineColors {
                normal: DynColors::Css(CssColors::LawnGreen),
                alternate: DynColors::Css(CssColors::GreenYellow),
            },
            undefined_char: LineColors {
                normal: DynColors::Css(CssColors::LightCoral),
                alternate: DynColors::Css(CssColors::IndianRed),
            },
        }
    }
}
impl LineColorConfig {
    pub fn new() -> Self {
        Self::default()
    }
}
