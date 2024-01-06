use clap::ValueEnum;
use colored::*;
use log::{debug, error, info};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::File;
use std::io::Write;

pub type Color = [u8; 3];
pub type Colors = [Color; 4];
#[derive(Debug, Clone)]
pub struct Palette {
    bg: Colors,
    obj0: Colors,
    obj1: Colors,
    window: Colors,
    lcd_off: Color,
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
impl Palette {
    /// Load palette from file
    pub fn load(file_name: &str) -> Self {
        debug!("Loading palette from {}", file_name);
        let data = fs::read(file_name).expect("Cannot read palette file");
        if data.len() != 56 {
            panic!(
                "Palette file should have exactly 56 bytes, but it has {} bytes",
                data.len()
            );
        }
        // check footer
        if data[51] == 0x81
            && data[52] == 0x41
            && data[53] == 0x50
            && data[54] == 0x47
            && data[55] == 0x42
        {
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

    pub fn save(&self, file_name: &str) {
        debug!("Saving palette to {}", file_name);
        let mut file =
            File::create(file_name).unwrap_or_else(|_| panic!("Cannot create file {}", file_name));

        // TODO: macro this
        let data: Vec<u8> = self.bg.into_iter().flatten().collect();
        file.write_all(&data)
            .unwrap_or_else(|_| panic!("Cannot write 'bg' to {}", file_name));

        let data: Vec<u8> = self.obj0.into_iter().flatten().collect();
        file.write_all(&data)
            .unwrap_or_else(|_| panic!("Cannot write 'obj0' to {}", file_name));

        let data: Vec<u8> = self.obj1.into_iter().flatten().collect();
        file.write_all(&data)
            .unwrap_or_else(|_| panic!("Cannot write 'obj1' to {}", file_name));

        let data: Vec<u8> = self.window.into_iter().flatten().collect();
        file.write_all(&data)
            .unwrap_or_else(|_| panic!("Cannot write 'window' to {}", file_name));

        let data: Vec<u8> = self.lcd_off.to_vec();
        file.write_all(&data)
            .unwrap_or_else(|_| panic!("Cannot write 'lcd_off' to {}", file_name));

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
