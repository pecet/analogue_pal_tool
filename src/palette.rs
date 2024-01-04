use std::fmt::{Display, Formatter};
use std::fs;
use log::{debug, error, info};
use colored::*;

type Color = [u8; 3];
type Colors = [Color; 4];
#[derive(Debug, Clone, Default)]
pub struct Palette {
    bg: Colors,
    obj0: Colors,
    obj1: Colors,
    window: Colors,
    lcd_off: Color,
}

#[derive(Debug, Copy, Clone)]
pub enum AsAnsiType {
    JustColor,
    ColorNumber,
    ColorValueDec,
    ColorValueInt,
}

pub trait AsAnsiVec {
    fn as_ansi(&self, display_type: AsAnsiType) -> ColoredStringVec;
}

pub trait AsAnsi {
    fn as_ansi(&self, display_type: AsAnsiType) -> ColoredString;
}

impl AsAnsi for Color {
    fn as_ansi(&self, display_type: AsAnsiType) -> ColoredString {
        match display_type {
            AsAnsiType::JustColor => {
                "  ".on_truecolor(self[0], self[1], self[2])
            }
            AsAnsiType::ColorNumber => {
                todo!()
            }
            AsAnsiType::ColorValueDec => {
                todo!()
            }
            AsAnsiType::ColorValueInt => {
                todo!()
            }
        }
    }
}

impl AsAnsiVec for Colors {
    fn as_ansi(&self, display_type: AsAnsiType) -> ColoredStringVec {
        let mut vec = ColoredStringVec(Vec::new());
        self.iter().for_each(|color| {
            vec.0.push(color.as_ansi(display_type))
        });
        vec
    }
}

macro_rules! data_to_array {
    ($data: ident, $start: expr) => {
        $data[$start .. $start + 3].try_into().expect("Cannot convert vec to fixed size array")
    };
}

macro_rules! data_to_multi_array {
    ($data: ident, $start: literal) => {
        [
            data_to_array!($data, $start),
            data_to_array!($data, $start + 3),
            data_to_array!($data, $start + 6),
            data_to_array!($data, $start + 9),
        ]
    };
}
impl Palette {
    /// Load palette from file
    pub fn load(file_name: &str) -> Self {
        debug!("Loading palette from {}", file_name);
        let data = fs::read(file_name).expect("Cannot read palette file");
        if data.len() != 56 {
            panic!("Palette file should have exactly 56 bytes, but it has {} bytes", data.len());
        }
        // check footer
        if data[51] == 0x81 && data[52] == 0x41 && data[53] == 0x50 && data[54] == 0x47 && data[55] == 0x42 {
            debug!("Footer of palette file is correct");
        } else {
            error!("Footer of palette file is incorrect, will try to read anyway")
        }
        info!("Palette from {} loaded", file_name);
        Self {
            bg: data_to_multi_array!(data, 0),
            obj0: data_to_multi_array!(data, 12),
            obj1: data_to_multi_array!(data, 24),
            window: data_to_multi_array!(data, 36),
            lcd_off: data_to_array!(data, 48),
        }
    }
}

impl AsAnsiVec for Palette {
    fn as_ansi(&self, display_type: AsAnsiType) -> ColoredStringVec {
        let mut vec = ColoredStringVec(Vec::with_capacity(17));
        vec.0.push("-- Background --\n".white().on_black());
        vec.0.extend(self.bg.as_ansi(display_type).0);
        vec.0.push("\n".black().on_black());

        vec.0.push("-- Object 0 --\n".white().on_black());
        vec.0.extend(self.obj0.as_ansi(display_type).0);
        vec.0.push("\n".black().on_black());

        vec.0.push("-- Object 1 --\n".white().on_black());
        vec.0.extend(self.obj1.as_ansi(display_type).0);
        vec.0.push("\n".black().on_black());

        vec.0.push("-- Window --\n".white().on_black());
        vec.0.extend(self.window.as_ansi(display_type).0);
        vec.0.push("\n".black().on_black());

        vec.0.push("-- LCD Off --\n".white().on_black());
        vec.0.push(self.lcd_off.as_ansi(display_type));
        vec.0.push("\n".black().on_black());

        vec
    }
}

pub struct ColoredStringVec(pub(self) Vec<ColoredString>);

impl Display for ColoredStringVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(for colored_string in &self.0 {
            write!(f, "{}", colored_string)?;
        })
    }
}