use crate::palette::{AsAnsi, AsAnsiType, AsAnsiVec, Color, Palette};
use image::imageops::FilterType;
use image::io::Reader;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Pixel, Rgb, Rgba};
use log::{debug, error, info, warn};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Cursor, Write};
use std::process::exit;
use glob::{glob, Paths};
use itertools::Itertools;
use crate::helpers::Helpers;

pub struct ImageHandler;

impl ImageHandler {
    /// Full palette contains almost that number of colors.
    /// It actually contains +1 because we do not count lcd_off here
    const ALMOST_ALL_COLORS: usize = 16;

    /// For some reason colors found on screenshots so give us some tolerance around that
    const TEMPLATE_TOLERANCE_UPPER: u8 = 8;
    const TEMPLATE_TOLERANCE_LOWER: u8 = 8;

    fn find_unique_colors(image: &DynamicImage) -> HashSet<Color> {
        let mut colors = HashSet::new();
        for pixel in image.pixels() {
            let (_, _, color) = pixel;
            // no need to use alpha channel here
            let color = color.to_rgb().0;
            colors.insert(color);
        }
        debug!("Found {} unique colors in image", colors.len());
        colors
    }

    fn color_image(
        palette_colors: &HashMap<String, Color>,
        template_colors: &HashMap<Color, String>,
        image: &DynamicImage,
        output_scale: u8,
    ) -> DynamicImage {
        let colors = Self::find_unique_colors(image);
        let percentage_of_colors = colors.len() as f32 / Self::ALMOST_ALL_COLORS as f32 * 100.0;
        if percentage_of_colors >= 100.0 {
            info!("All colors from palette (except lcd_off) have representation in source image");
        } else {
            warn!("Only ~{:.2}% of colors have representation in source image ({} of {}; not counting lcd_off)", percentage_of_colors, colors.len(), Self::ALMOST_ALL_COLORS)
        }

        colors.into_iter().enumerate().for_each(|(i, color)| {
            debug!(
                "{}{} {}",
                color.as_ansi(AsAnsiType::ColorValueHex, None),
                color.as_ansi(AsAnsiType::ColorValueDec, None),
                i
            );
        });
        let mut output_image = image.clone();
        let mut processed = 0_usize;
        let mut skipped = 0_usize;
        for pixel in image.pixels() {
            let (x, y, color) = pixel;
            let color_rgb = &color.to_rgb().0;
            let result = template_colors.keys().find(|color| {
                color.iter().enumerate().all(|(i, c)| {
                    *c <= color_rgb[i].saturating_add(Self::TEMPLATE_TOLERANCE_UPPER)
                        && *c >= color_rgb[i].saturating_sub(Self::TEMPLATE_TOLERANCE_LOWER)
                })
            });
            if let Some(key) = result {
                let value = template_colors.get(key).unwrap();
                let new_color = Rgb(*palette_colors.get(value).unwrap());
                output_image.put_pixel(x, y, new_color.to_rgba());
                processed += 1;
            } else {
                skipped += 1;
            }
        }
        let scale = output_scale;
        if !(1..=20).contains(&scale) {
            panic!("Scale must be between 1 and 20");
        }
        // no need to check if scale = 1 here as DynamicImage::resize
        // already just returns original image if this is true
        let output_image = output_image.resize(
            output_image.width() * scale as u32,
            output_image.height() * scale as u32,
            FilterType::Nearest,
        );
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
        output_image
    }

    fn save_image(image: &DynamicImage, image_path: &str) {
        let mut bytes: Vec<u8> = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)
            .expect("Cannot create output file bytes");
        let mut file = File::create(image_path)
            .unwrap_or_else(|_| panic!("Cannot create image file {}", &image_path));
        file.write_all(&bytes)
            .unwrap_or_else(|_| panic!("Cannot write to image file {}", &image_path));
        info!("Saved image file {}", image_path);
    }

    pub fn color_images(
        pal_file: &str,
        input_images: &Vec<String>,
        output_image_file: &str,
        output_scale: Option<u8>,
        merge: bool,
    ) {
        debug!("Opening palette file {}", pal_file);
        let palette = Palette::load(pal_file);
        let palette_colors: HashMap<String, Color> = palette.into();

        let template = Palette::default();
        debug!(
            "Template palette loaded \n{}",
            template.as_ansi(AsAnsiType::ColorValueDec)
        );
        let template_colors: HashMap<Color, String> = template.into();
        let input_len = input_images.len();
        let mut images_to_merge: Vec<DynamicImage> = Vec::with_capacity(
            if merge {
                input_images.len()
            } else {
                // So according to docs this will not allocate vector
                // which is what we want, so we can avoid wrapping this vector in Option
                0
            }
        );
        let input_images = Helpers::glob_paths(&input_images);
        debug!("All input files, including globbed results:\n{:#?}", &input_images);
        input_images
            .iter()
            .enumerate()
            .for_each(|(counter, input_image)| {
                debug!("Opening image file {}", input_image);
                let image = Reader::open(input_image)
                    .unwrap_or_else(|_| panic!("Cannot open image file {}", input_image))
                    .decode()
                    .unwrap_or_else(|_| panic!("Cannot decode image file {}", input_image));
                info!("Opened image file {}", input_image);
                let output_image = Self::color_image(
                    &palette_colors,
                    &template_colors,
                    &image,
                    output_scale.unwrap_or(1),
                );
                let output_image_file = if output_image_file.to_lowercase().ends_with(".png") {
                    output_image_file.to_string()
                } else {
                    format!("{}.png", output_image_file)
                };
                let output_image_file = if input_len > 1 && !merge {
                    // TODO: This will not work correctly in edge case when user will use e.g. 'test.png.png' 🤷
                    // We should only replace last match
                    output_image_file
                        .to_lowercase()
                        .replace(".png", &format!("{:03}.png", counter))
                } else {
                    output_image_file
                };
                if merge {
                    images_to_merge.push(output_image);
                } else {
                    Self::save_image(&output_image, &output_image_file);
                }
            });
        if merge {
            let width: u32 = images_to_merge.iter().map(|m| m.width()).sum();
            let height: u32 = images_to_merge.iter().map(|m| m.height()).max().unwrap();
            debug!("Merged image size will be: width = {}, height = {}", width, height);
            let mut merged_image: ImageBuffer::<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);
            images_to_merge.iter().enumerate().for_each(|(i, image)|{
                image::imageops::overlay(&mut merged_image, image, (i as u32 * image.width()) as i64, 0);
            });
            Self::save_image(&DynamicImage::ImageRgba8(merged_image), &output_image_file);
        }
    }
}
