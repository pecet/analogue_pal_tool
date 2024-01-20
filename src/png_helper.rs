use std::fs::File;
use std::io::BufWriter;

use crate::palette::{Color, Palette};
use itertools::Itertools;
use png;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Array referenced is too big")]
    ArrayTooBig,
}

pub struct PngPalette {
    pal: [u8; PngPalette::SIZE],
    index: usize,
}
impl From<PngPalette> for [u8; PngPalette::SIZE] {
    fn from(value: PngPalette) -> Self {
        value.pal
    }
}
impl Default for PngPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<&[u8]> for PngPalette {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() >= PngPalette::SIZE {
            return Err(Error::ArrayTooBig);
        }
        let mut array: [u8; PngPalette::SIZE] = [255; PngPalette::SIZE];
        value.iter().enumerate().for_each(|(i, v)| {
            array[i] = *v;
        });
        Ok(Self {
            pal: array,
            index: 0,
        })
    }
}

impl From<Palette> for PngPalette {
    fn from(value: Palette) -> Self {
        let pal_array: Vec<u8> = value.into();
        // Can unwrap, will contain exactly 17 * 3 bytes as underlying type
        let pal_array: [u8; 17 * 3] = pal_array.try_into().unwrap();
        // (&pal_array) = &[u8; 17 * 3] not &[u8], hence specify this directly here
        let pal_array_ref: &[u8] = &pal_array;
        // Similarly here we can unwrap for the same reason as above
        pal_array_ref.try_into().unwrap()
    }
}

impl PngPalette {
    const SIZE: usize = 256 * 3;
    pub fn new() -> Self {
        Self {
            pal: [255; Self::SIZE],
            index: 0,
        }
    }

    pub fn push(&mut self, color: Color) -> bool {
        let value = self.set(self.index, color);
        if value {
            self.index += 1;
        }
        value
    }
    pub fn set(&mut self, index: usize, color: Color) -> bool {
        if index < 256 {
            self.pal[index * 3] = color[0];
            self.pal[index * 3 + 1] = color[1];
            self.pal[index * 3 + 2] = color[2];
            return true;
        }
        false
    }
    pub fn index_of(&self, color: Color) -> Option<usize> {
        let pos = self
            .pal
            .chunks_exact(3)
            .map(|c| {
                let rgb: [u8; 3] = c.try_into().expect("Cannot convert color chunk");
                rgb
            })
            .find_position(|c| c == &color[..]);
        pos.map(|(index, _)| index)
    }

    pub fn index_of_with_tolerance(&self, color: Color, tolerance: u8) -> Option<usize> {
        let pos = self
            .pal
            .chunks_exact(3)
            .map(|c| {
                let rgb: Color = c.try_into().expect("Cannot convert color chunk");
                rgb
            })
            .find_position(|c| {
                c.iter().enumerate().all(|(i, x)| {
                    (color[i].saturating_sub(tolerance)..=color[i].saturating_add(tolerance))
                        .contains(x)
                    //*x <= color[i].saturating_add(tolerance) && *x >= color[i].saturating_sub(tolerance)
                })
            });
        pos.map(|(index, _)| index)
    }
}

pub struct PngHelper;

impl PngHelper {
    pub fn save(file_name: &str, width: u32, height: u32, palette: &[u8], data: &[u8]) {
        let file = File::create(file_name).expect("Cannot create .png file");
        let writer = BufWriter::new(file);
        let mut encoder = png::Encoder::new(writer, width, height);
        encoder.set_color(png::ColorType::Indexed);
        encoder.set_depth(png::BitDepth::Eight);
        // These two values are copied directly from png crate docs,
        // so they must be safe defaults right?
        encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
        let source_chromaticities = png::SourceChromaticities::new(
            (0.31270, 0.32900),
            (0.64000, 0.33000),
            (0.30000, 0.60000),
            (0.15000, 0.06000),
        );
        encoder.set_source_chromaticities(source_chromaticities);
        encoder.set_palette(palette);
        let mut writer = encoder.write_header().unwrap();
        // write sequence of palette indexes
        writer.write_image_data(data).unwrap(); // save
    }
}
