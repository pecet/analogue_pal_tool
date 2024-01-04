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

pub trait AsAnsiVec {
    fn as_ansi(&self) -> ColoredStringVec;
}

pub trait AsAnsi {
    fn as_ansi(&self) -> ColoredString;
}

impl AsAnsi for Color {
    fn as_ansi(&self) -> ColoredString {
        "  ".on_truecolor(self[0], self[1], self[2])
    }
}

impl AsAnsiVec for Colors {
    fn as_ansi(&self) -> ColoredStringVec {
        let mut vec = ColoredStringVec(Vec::new());
        self.iter().for_each(|color| {
            vec.0.push(color.as_ansi())
        });
        vec
    }
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
        // TODO: yes, this is stupid: add fancy macro/fn below or find std Rust fn to do this better
        Self {
            bg: [
                [data[0], data[1], data[2]],
                [data[3], data[4], data[5]],
                [data[6], data[7], data[8]],
                [data[9], data[10], data[11]],
            ],
            obj0: [
                [data[12], data[13], data[14]],
                [data[15], data[16], data[17]],
                [data[18], data[19], data[20]],
                [data[21], data[22], data[23]],
            ],
            obj1: [
                [data[24], data[25], data[26]],
                [data[27], data[28], data[29]],
                [data[30], data[31], data[32]],
                [data[33], data[34], data[35]],
            ],
            window: [
                [data[36], data[37], data[38]],
                [data[39], data[40], data[41]],
                [data[42], data[43], data[44]],
                [data[45], data[46], data[47]],
            ],
            lcd_off: [data[48], data[49], data[50]],
        }
    }

    pub fn as_ansi(&self) -> ColoredStringVec {
        let mut vec = ColoredStringVec(Vec::with_capacity(17));
        vec.0.push("-- Background --\n".white().on_black());
        vec.0.extend(self.bg.as_ansi().0);
        vec.0.push("\n".black().on_black());

        vec.0.push("-- Object 0 --\n".white().on_black());
        vec.0.extend(self.obj0.as_ansi().0);
        vec.0.push("\n".black().on_black());

        vec.0.push("-- Object 1 --\n".white().on_black());
        vec.0.extend(self.obj1.as_ansi().0);
        vec.0.push("\n".black().on_black());

        vec.0.push("-- Window --\n".white().on_black());
        vec.0.extend(self.window.as_ansi().0);
        vec.0.push("\n".black().on_black());

        vec.0.push("-- LCD Off --\n".white().on_black());
        vec.0.push(self.lcd_off.as_ansi());
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