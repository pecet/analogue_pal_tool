use crate::palette::AsAnsi;
use crate::palette::AsAnsiType;
use crate::palette::{AsAnsiVec, Palette};
use image::io::Reader;
use image::{GenericImage, GenericImageView, Pixel, Rgb};
use log::{debug, info};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Cursor, Write};

pub struct ImageHandler;

impl ImageHandler {
    pub fn color_image(pal_file: &str, input_image: &str, output_image_file: &str) {
        debug!("Opening palette file {}", pal_file);
        let palette = Palette::load(pal_file);
        let palette_colors: HashMap<String, [u8; 3]> = palette.into();
        debug!("Opening image file {}", input_image);
        let image = Reader::open(input_image)
            .unwrap_or_else(|_| panic!("Cannot open image file {}", input_image))
            .decode()
            .unwrap_or_else(|_| panic!("Cannot decode image file {}", input_image));
        info!("Opened image file {}", input_image);
        let mut colors = HashSet::new();
        for pixel in image.pixels() {
            let (_, _, color) = pixel;
            // no need to use alpha channel here
            let color = color.to_rgb().0;
            colors.insert(color);
        }
        debug!("Found {} unique colors in image", colors.len());
        colors.into_iter().enumerate().for_each(|(i, color)| {
            debug!(
                "{}{} {}",
                color.as_ansi(AsAnsiType::ColorValueHex, None),
                color.as_ansi(AsAnsiType::ColorValueDec, None),
                i
            );
        });
        let template = Palette::default();
        debug!(
            "Template palette \n{}",
            template.as_ansi(AsAnsiType::ColorValueDec)
        );
        let template_colors: HashMap<[u8; 3], String> = template.into();
        let mut output_image = image.clone();
        let mut processed = 0_usize;
        let mut skipped = 0_usize;
        for pixel in image.pixels() {
            let (x, y, color) = pixel;
            let color_rgb = &color.to_rgb().0;
            // for some reason colors differ from actually defined
            // so give us some tolerance around that
            let tolerance = 7;
            let result = template_colors.keys().find(
                |c|
                    c[0] <= color_rgb[0].saturating_add(tolerance) &&
                    c[0] >= color_rgb[0].saturating_sub(tolerance)
            );
            if let Some(key) = result {
                let value = template_colors.get(key).unwrap();
                let new_color = Rgb(*palette_colors.get(value).unwrap());
                output_image.put_pixel(x, y, new_color.to_rgba());
                processed += 1;
            } else {
                skipped += 1;
            }
        }
        let mut bytes: Vec<u8> = Vec::new();
        output_image
            .write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)
            .expect("Cannot create output file bytes");
        let mut file = File::create(output_image_file)
            .unwrap_or_else(|_| panic!("Cannot create image file {}", output_image_file));
        file.write_all(&bytes)
            .unwrap_or_else(|_| panic!("Cannot write to image file {}", output_image_file));
        debug!(
            "Processed {} of {} pixels ~{:.2}%",
            processed,
            processed + skipped,
            processed as f32 / (processed + skipped) as f32 * 100.0
        );
        info!("Saved image file {}", output_image_file);
    }
}
