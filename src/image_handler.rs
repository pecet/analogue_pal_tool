use crate::palette::AsAnsi;
use crate::palette::AsAnsiType;
use crate::palette::{AsAnsiVec, Palette};
use image::io::Reader;
use image::{GenericImage, GenericImageView, Pixel, Rgb};
use log::{debug, info, warn};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Cursor, Write};
use image::imageops::FilterType;

pub struct ImageHandler;

impl ImageHandler {
    /// Full palette contains that number of colors
    const ALL_COLORS: usize = 17;

    /// For some reason colors found on screenshots
    /// so give us some tolerance around that
    const TEMPLATE_TOLERANCE_UPPER: u8 = 8;
    const TEMPLATE_TOLERANCE_LOWER: u8 = 8;
    pub fn color_image(pal_file: &str, input_image: &str, output_image_file: &str, output_scale: Option<u8>) {
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
        let percentage_of_colors = (colors.len() as f32 / (Self::ALL_COLORS - 1) as f32) * 100.0;
        if percentage_of_colors >= 100.0 {
            info!("All colors from palette (except lcd_off) have representation in source image");
        } else {
            warn!("Only ~{:.2}% of colors have representation in source image ({} of {}; not counting lcd_off)", percentage_of_colors, colors.len(), Self::ALL_COLORS - 1)
        }

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
            let result = template_colors.keys().find(
                |c|
                    c[0] <= color_rgb[0].saturating_add(Self::TEMPLATE_TOLERANCE_UPPER) &&
                    c[0] >= color_rgb[0].saturating_sub(Self::TEMPLATE_TOLERANCE_LOWER)
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
        let mut output_image = if let Some(scale) = output_scale {
            output_image.resize(output_image.width() * scale as u32, output_image.height() * scale as u32, FilterType::Nearest)
        } else {
            output_image
        };
        let mut bytes: Vec<u8> = Vec::new();
        output_image
            .write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)
            .expect("Cannot create output file bytes");
        let mut file = File::create(output_image_file)
            .unwrap_or_else(|_| panic!("Cannot create image file {}", output_image_file));
        file.write_all(&bytes)
            .unwrap_or_else(|_| panic!("Cannot write to image file {}", output_image_file));
        let percentage = processed as f32 / (processed + skipped) as f32 * 100.0;
        debug!(
            "Processed {} of {} pixels ~{:.2}%",
            processed,
            processed + skipped,
            percentage
        );
        if percentage >= 100.0 {
            info!("Successfully colorized all pixels");
        } else {
            warn!("Not all pixels were colorized (~{:.2}%), maybe you used incorrect template .pal file on Analogue Pocket..?", percentage)
        }
        info!("Saved image file {}", output_image_file);
    }
}
