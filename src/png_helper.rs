use std::fs::File;
use std::io::BufWriter;
use std::ops::Index;
use std::path::Path;
use itertools::Itertools;
use png;
use crate::palette::Color;

pub struct Palette {
    pal: [u8; 256 * 3],
    index: usize,
}
impl From<Palette> for [u8; 256 * 3] {
    fn from(value: Palette) -> Self {
        value.pal
    }
}
impl Palette {
    fn new() -> Self {
        Self {
            pal: [255; 256 * 3],
            index: 0,
        }
    }
    fn push(&mut self, color: Color) -> bool {
        if self.index < 256 {
            self.pal[self.index * 3] = color[0];
            self.pal[self.index * 3 + 1] = color[1];
            self.pal[self.index * 3 + 2] = color[2];
            self.index += 1;
            return true
        }
        false
    }
    fn index_of(&self, color: Color) -> Option<usize> {
        let pos = self.pal.chunks_exact(3).map(|c| {
            let rgb: [u8; 3] = c.try_into().expect("Cannot convert color chunk");
            rgb
        }).find_position(|c| {
            c == &color[..]
        });
        match pos {
            None => None,
            Some((index, _)) => Some(index)
        }
    }
}

pub struct PngHelper;

impl PngHelper {
    pub fn save(file_name: &str,
                    width: u32,
                    height: u32,
                    palette: &[u8],
                    data: &[u8],
    ) {
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
        writer.write_image_data(&data).unwrap(); // save
    }
}