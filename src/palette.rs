use std::fs;
use log::{debug, info};

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

impl Palette {
    pub fn load(file_name: &str) -> Self {
        debug!("Loading palette from {}", file_name);
        let data = fs::read(file_name).expect("Cannot read palette file");
        info!("Palette from {} loaded", file_name);

        todo!()
    }
}
