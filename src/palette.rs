use clap::ValueEnum;
use colored::*;
use log::{debug, error};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::{fs, io};

use std::fs::File;
use std::io::Write;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cannot convert from Palette to Vec<u8>")]
    GenericConversionError,
    #[error("Invalid palette data size, must be exactly 56 bytes, but is {0}")]
    InvalidSize(usize),
    #[error("Incorrect footer")]
    IncorrectFooter,
    #[error("Error while reading file {0}")]
    IoError(#[from] io::Error),
}

pub type Color = [u8; 3];
pub type Colors = [Color; 4];
#[derive(Debug, Clone)]
pub struct Palette {
    bg: Colors,     // 3 * 4 bytes
    obj0: Colors,   // 3 * 4 bytes
    obj1: Colors,   // 3 * 4 bytes
    window: Colors, // 3 * 4 bytes
    lcd_off: Color, // 3 bytes
}

impl From<Palette> for Vec<u8> {
    fn from(value: Palette) -> Self {
        let all_colors: Vec<Colors> = vec![value.bg, value.obj0, value.obj1, value.window];
        let mut all_color: Vec<Color> = all_colors.into_iter().flatten().collect();
        all_color.push(value.lcd_off);
        let all_u8: Vec<u8> = all_color.into_iter().flatten().collect();
        all_u8
    }
}

macro_rules! data_to_array {
    ($data: ident, $start: expr) => {
        $data[$start..$start + 3]
            .try_into()
            .expect("Cannot convert vec to fixed size array")
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

impl TryFrom<Vec<u8>> for Palette {
    type Error = Error;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() != 56 {
            return Err(Error::InvalidSize(value.len()));
        }
        if value[51..56] != [0x81, 0x41, 0x50, 0x47, 0x42] {
            return Err(Error::IncorrectFooter);
        }
        Ok(Self {
            bg: data_to_multi_array!(value, 0),
            obj0: data_to_multi_array!(value, 12),
            obj1: data_to_multi_array!(value, 24),
            window: data_to_multi_array!(value, 36),
            lcd_off: data_to_array!(value, 48),
        })
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            bg: [[0, 0, 0], [32, 32, 32], [64, 64, 64], [128, 128, 128]],
            obj0: [[0, 255, 0], [32, 255, 32], [64, 255, 64], [128, 255, 128]],
            obj1: [[0, 0, 255], [32, 32, 255], [64, 64, 255], [128, 128, 255]],
            window: [[255, 0, 0], [255, 32, 32], [255, 64, 64], [255, 128, 128]],
            lcd_off: [255, 0, 255],
        }
    }
}

#[derive(Debug, Copy, Clone, Default, ValueEnum)]
pub enum AsAnsiType {
    JustColor,
    ColorNumber,
    ColorValueDec,
    #[default]
    ColorValueHex,
}

pub trait AsAnsiVec {
    fn as_ansi(&self, display_type: AsAnsiType) -> ColoredStringVec;
}

pub trait AsAnsi {
    fn as_ansi(&self, display_type: AsAnsiType, text: Option<String>) -> ColoredString;
}

pub trait ColorExt {
    fn contrast_color(&self) -> Color;
}

impl ColorExt for Color {
    /// Get white or black color, depending of which will be better visible by user.
    ///
    /// Based on https://stackoverflow.com/a/1855903
    fn contrast_color(&self) -> Color {
        let luminance =
            (0.299 * self[0] as f32) + (0.586 * self[1] as f32) + (0.114 * self[2] as f32);
        let luminance = luminance / 255.0;
        let value = if luminance > 0.5 { 0 } else { 255 };
        [value, value, value]
    }
}

impl AsAnsi for Color {
    fn as_ansi(&self, display_type: AsAnsiType, text: Option<String>) -> ColoredString {
        match display_type {
            AsAnsiType::JustColor => "  ".on_truecolor(self[0], self[1], self[2]),
            AsAnsiType::ColorNumber => {
                let contrast_color = self.contrast_color();
                format!("  {}  ", &text.expect("Provide text"))
                    .on_truecolor(self[0], self[1], self[2])
                    .truecolor(contrast_color[0], contrast_color[1], contrast_color[2])
            }
            AsAnsiType::ColorValueDec => {
                let contrast_color = self.contrast_color();
                let mut padding = String::new();
                self.iter().for_each(|value| {
                    if *value < 100 {
                        padding += " ";
                    }
                    if *value < 10 {
                        padding += " ";
                    }
                });
                format!("  [{}, {}, {}]  {}", self[0], self[1], self[2], padding)
                    .on_truecolor(self[0], self[1], self[2])
                    .truecolor(contrast_color[0], contrast_color[1], contrast_color[2])
            }
            AsAnsiType::ColorValueHex => {
                let contrast_color = self.contrast_color();
                format!("  #{:02x}{:02x}{:02x}  ", self[0], self[1], self[2])
                    .on_truecolor(self[0], self[1], self[2])
                    .truecolor(contrast_color[0], contrast_color[1], contrast_color[2])
            }
        }
    }
}

impl AsAnsiVec for Colors {
    fn as_ansi(&self, display_type: AsAnsiType) -> ColoredStringVec {
        let mut vec = ColoredStringVec(Vec::new());
        self.iter().enumerate().for_each(|(i, color)| {
            if let AsAnsiType::ColorNumber = display_type {
                vec.0.push(color.as_ansi(display_type, Some(i.to_string())))
            } else {
                vec.0.push(color.as_ansi(display_type, None))
            }
        });
        vec
    }
}

impl Palette {
    /// Load palette from file
    pub fn load(file_name: &str) -> Result<Self, Error> {
        debug!("Loading palette from {}", file_name);
        let data = fs::read(file_name)?;
        data.try_into()
    }

    pub fn save(&self, file_name: &str) {
        debug!("Saving palette to {}", file_name);
        let mut file =
            File::create(file_name).unwrap_or_else(|_| panic!("Cannot create file {}", file_name));

        let data: Vec<u8> = self.clone().into();
        file.write_all(&data)
            .unwrap_or_else(|_| panic!("Cannot write 'data' to {}", file_name));

        let footer: Vec<u8> = vec![0x81, 0x41, 0x50, 0x47, 0x42];
        file.write_all(&footer)
            .unwrap_or_else(|_| panic!("Cannot write 'footer' to {}", file_name));
    }
}

/// Converts to hashset, only include UNIQUE colors (!)
/// This is why colors for template / default must be unique also
impl From<Palette> for HashSet<Color> {
    fn from(value: Palette) -> Self {
        let mut color_set = HashSet::new();
        value
            .bg
            .into_iter()
            .chain(value.obj0)
            .chain(value.obj1)
            .chain(value.window)
            .for_each(|color| {
                color_set.insert(color);
            });
        color_set.insert(value.lcd_off);
        color_set
    }
}

impl From<Palette> for HashMap<Color, String> {
    fn from(value: Palette) -> Self {
        let mut color_map = HashMap::new();
        value.bg.into_iter().enumerate().for_each(|(i, color)| {
            color_map.insert(color, format!("bg_{}", i));
        });
        value.obj0.into_iter().enumerate().for_each(|(i, color)| {
            color_map.insert(color, format!("obj0_{}", i));
        });
        value.obj1.into_iter().enumerate().for_each(|(i, color)| {
            color_map.insert(color, format!("obj1_{}", i));
        });
        value.window.into_iter().enumerate().for_each(|(i, color)| {
            color_map.insert(color, format!("window_{}", i));
        });
        color_map.insert(value.lcd_off, "lcd_off".to_string());
        color_map
    }
}

// TODO: somehow make above implementation and below one share code
// maybe put values to pairs first, and then try to do that?
// or make macro, whatever you do this is duplication sucks
impl From<Palette> for HashMap<String, Color> {
    fn from(value: Palette) -> Self {
        let mut color_map = HashMap::new();
        value.bg.into_iter().enumerate().for_each(|(i, color)| {
            color_map.insert(format!("bg_{}", i), color);
        });
        value.obj0.into_iter().enumerate().for_each(|(i, color)| {
            color_map.insert(format!("obj0_{}", i), color);
        });
        value.obj1.into_iter().enumerate().for_each(|(i, color)| {
            color_map.insert(format!("obj1_{}", i), color);
        });
        value.window.into_iter().enumerate().for_each(|(i, color)| {
            color_map.insert(format!("window_{}", i), color);
        });
        color_map.insert("lcd_off".to_string(), value.lcd_off);
        color_map
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
        if let AsAnsiType::ColorNumber = display_type {
            vec.0
                .push(self.lcd_off.as_ansi(display_type, Some(0.to_string())));
        } else {
            vec.0.push(self.lcd_off.as_ansi(display_type, None));
        }
        vec.0.push("\n".black().on_black());

        vec
    }
}

pub struct ColoredStringVec(pub(self) Vec<ColoredString>);

impl Display for ColoredStringVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for colored_string in &self.0 {
            write!(f, "{}", colored_string)?;
        }
        Ok(())
    }
}
