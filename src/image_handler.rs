use crate::palette::{AsAnsi, AsAnsiType, AsAnsiVec, Color, Palette};
use image::imageops::FilterType;
use image::io::Reader;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Pixel, Rgb, Rgba};
use log::{debug, info, warn};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Cursor, Write};
use std::process::exit;

use clap::ValueEnum;

use crate::helpers::Helpers;
use lazy_static::lazy_static;
use tera::{Context, Tera};
use crate::png_helper::{PngHelper, PngPalette};

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Template parsing error(s): {}", e);
                exit(1);
            }
        };
        tera.autoescape_on(vec![]);
        tera
    };
}

#[derive(Debug, Copy, Clone, Default, ValueEnum)]
pub enum MergeLayout {
    #[default]
    #[clap(alias = "h")]
    Horizontal,
    #[clap(alias = "v")]
    Vertical,
}

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

    fn palettize_image (
        template: Palette,
        image: &DynamicImage,
        output_scale: u8,
    ) -> Vec<u8> {
        let colors = Self::find_unique_colors(image);
        let percentage_of_colors = colors.len() as f32 / Self::ALMOST_ALL_COLORS as f32 * 100.0;
        if percentage_of_colors >= 100.0 {
            info!("All colors from palette (except lcd_off) have representation in source image");
        } else {
            warn!("Only ~{:.2}% of colors have representation in source image ({} of {}; not counting lcd_off)", percentage_of_colors, colors.len(), Self::ALMOST_ALL_COLORS)
        }
        let template: PngPalette = template.into();
        let (width, height) = (image.width() as usize, image.height() as usize);
        let mut image_buffer = vec![255_u8; width * height];

        let mut position = 0_usize;
        for (_, _, color) in image.pixels() {
            let color = color.to_rgb().0;
            let color_index = template.index_of_with_tolerance(color, 8);
            // We just store color index in Vector, because this is how paletted images really work
            // Because we will be supplying different palette when saving - this will colorize our image
            // Much faster than previously used here PNG RGBA and manually putting whole RGBA pixels
            // However we will be needed to implement scaling and merging ourselves - can we do it?
            if let Some(color_index) = color_index {
                image_buffer[position] = color_index as u8; // palette index will never exceed u8 size
            }
            position += 1;
        }
        image_buffer
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
        max_columns: u8,
        merge_layout: MergeLayout,
    ) {
        debug!("Opening palette file {}", pal_file);
        let palette = Palette::load(pal_file).unwrap();

        let template = Palette::default();
        debug!(
            "Template palette loaded \n{}",
            template.as_ansi(AsAnsiType::ColorValueDec)
        );
        let input_len = input_images.len();
        let mut images_to_merge: Vec<DynamicImage> = Vec::with_capacity(if merge {
            input_images.len()
        } else {
            // So according to docs this will not allocate vector
            // which is what we want, so we can avoid wrapping this vector in Option
            0
        });
        let input_images = Helpers::glob_paths(input_images);
        debug!(
            "All input files, including globbed results:\n{:#?}",
            &input_images
        );
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
                let output_image_bytes = Self::palettize_image(template.clone(), &image, 1);
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
                    //images_to_merge.push(output_image);
                } else {
                    let pal: PngPalette = palette.clone().into();
                    let pal: [u8; 256 * 3] = pal.into();
                    PngHelper::save(
                        &output_image_file,
                        image.width(),
                        image.height(),
                        &pal,
                        &output_image_bytes
                    );
                }
            });
        if merge {
            let max_columns = max_columns as usize;
            let mut width: u32 = images_to_merge
                .iter()
                .take(max_columns)
                .map(|m| m.width())
                .sum();
            let mut height: u32 = images_to_merge
                .chunks(max_columns)
                .map(|chunk| chunk.iter().map(|image| image.height()).max().unwrap())
                .sum();
            if let MergeLayout::Vertical = merge_layout {
                (width, height) = (height, width);
            }
            info!("Merging images - may take a while");
            debug!(
                "Merged image size will be: width = {}, height = {}",
                width, height
            );
            let mut merged_image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);
            images_to_merge
                .chunks(max_columns)
                .enumerate()
                .for_each(|(i, chunk)| {
                    chunk.iter().enumerate().for_each(|(j, image)| {
                        let (x, y) = match merge_layout {
                            MergeLayout::Horizontal => (
                                (j as u32 * image.width()) as i64,
                                (i as u32 * image.height()) as i64,
                            ),
                            MergeLayout::Vertical => (
                                (i as u32 * image.width()) as i64,
                                (j as u32 * image.height()) as i64,
                            ),
                        };
                        image::imageops::overlay(&mut merged_image, image, x, y);
                    });
                });
            Self::save_image(&DynamicImage::ImageRgba8(merged_image), output_image_file);
        }
    }

    pub fn use_palettes_to_color_images(
        pal_files: &Vec<String>,
        input_images: &Vec<String>,
        output_image_file: &str,
        output_scale: Option<u8>,
        merge: bool,
        max_columns: u8,
        merge_layout: MergeLayout,
        generate_html: bool,
    ) {
        let pal_files = Helpers::glob_paths(pal_files);
        if pal_files.len() == 1 {
            return Self::color_images(
                &pal_files[0],
                input_images,
                output_image_file,
                output_scale,
                merge,
                max_columns,
                merge_layout,
            );
        }
        let pal_images: Vec<_> = pal_files
            .par_iter()
            .map(|pal| {
                let pal_name_escaped = pal.replace('/', "$");
                let output_image_file =
                    output_image_file.replace(".png", &format!("{}.png", pal_name_escaped));
                Self::color_images(
                    pal,
                    input_images,
                    &output_image_file,
                    output_scale,
                    merge,
                    max_columns,
                    merge_layout,
                );
                (pal.clone(), output_image_file)
            })
            .collect();
        if generate_html {
            let mut context = Context::new();
            context.insert("version", env!("GIT_HASH_SHORT"));
            let mut palletes: Vec<HashMap<_, _>> = Vec::new();
            // TODO: obviously un-hardcode this
            let html_file = "output.html";
            info!("Generating HTML file '{html_file}'...");
            debug!("Output images = {pal_images:#?}");
            pal_images.iter().for_each(|(pal, image)| {
                let pal_name = if let Some(last_slash) = pal.rfind('/') {
                    &pal[last_slash + 1..pal.len()]
                } else {
                    pal
                };
                palletes.push(HashMap::from([
                    ("name", pal_name),
                    ("path", pal),
                    ("image", image),
                ]));
            });
            context.insert("palettes", &palletes);
            let rendered = TEMPLATES.render("index.html", &context).unwrap();
            std::fs::write(html_file, rendered).expect("Cannot create HTML file");
            info!("Created HTML file '{html_file}'")
        }
    }
}
