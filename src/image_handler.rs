use crate::palette::{AsAnsi, AsAnsiType, AsAnsiVec, Color, Palette};

use image::io::Reader;
use image::{DynamicImage, GenericImageView, Pixel};
use log::{debug, info, warn};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Cursor, Write};
use std::process::exit;

use clap::ValueEnum;

use crate::helpers::Helpers;
use crate::png_helper::{PngHelper, PngPalette};
use lazy_static::lazy_static;
use tera::{Context, Tera};

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

    fn palettize_image(template: Palette, image: &DynamicImage) -> Vec<u8> {
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

    fn scale_paletted_image(
        image_array: &[u8],
        width: usize,
        height: usize,
        scale: usize,
    ) -> Vec<u8> {
        if scale == 1 {
            debug!("Passed 1 as scale factor - no scaling necessary");
            return image_array.into();
        } else if scale == 0 {
            panic!("Cannot scale with 0 scale factor!")
        }
        let new_width = width * scale;
        let new_height = height * scale;
        debug!(
            "Scaling paletted image *{} - from {},{} to {},{}",
            scale, width, height, new_width, new_height
        );
        let mut scaled_array = vec![255_u8; new_width * new_height];
        for x in 0..width {
            for y in 0..height {
                let position = y * width + x;
                let color_index = image_array[position];
                // For each pixel of original image we need to create square in new image
                for i in x * scale..x * scale + scale {
                    for j in y * scale..y * scale + scale {
                        let new_position = j * new_width + i;
                        scaled_array[new_position] = color_index;
                    }
                }
            }
        }
        scaled_array
    }

    pub fn color_images(
        pal_file: &str,
        input_images: &Vec<String>,
        output_image_file: &str,
        output_scale: Option<u8>,
        merge: bool,
        max_columns: u8,
        _merge_layout: MergeLayout,
    ) {
        debug!("Opening palette file {}", pal_file);
        let palette = Palette::load(pal_file).unwrap();
        let output_scale = output_scale.unwrap_or(1);

        let template = Palette::default();
        debug!(
            "Template palette loaded \n{}",
            template.as_ansi(AsAnsiType::ColorValueDec)
        );
        let input_images = Helpers::glob_paths(input_images);
        debug!(
            "All input files, including globbed results:\n{:#?}",
            &input_images
        );
        let input_len = input_images.len();
        let (input_width, input_height) = PngHelper::get_size(&input_images[0]);
        let max_columns = max_columns as usize;
        let merged_width: usize = max_columns * input_width as usize * output_scale as usize;
        let no_rows = (input_len as f32 / max_columns as f32).ceil() as usize;
        let merged_height = no_rows * input_height as usize * output_scale as usize;
        let mut merged_image_bytes = if merge {
            Some(vec![255_u8; merged_width * merged_height])
        } else {
            None
        };

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
                let output_image_bytes = {
                    let unscaled = Self::palettize_image(template.clone(), &image);
                    Self::scale_paletted_image(
                        &unscaled,
                        image.width() as usize,
                        image.height() as usize,
                        output_scale as usize,
                    )
                };
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
                    if let Some(merged_image_bytes) = &mut merged_image_bytes {
                        let source_width = (image.width() * output_scale as u32) as usize;
                        let source_height = (image.height() * output_scale as u32) as usize;
                        let y = counter / max_columns;
                        let x = counter % max_columns;
                        let y = y * source_height;
                        let x = x * source_width;
                        PngHelper::copy_from_to(
                            &output_image_bytes,
                            // I like Rust but I don't like constant type conversions, maybe this is my fault though
                            source_width,
                            source_height,
                            merged_image_bytes,
                            merged_width,
                            merged_height,
                            x,
                            y,
                        )
                    } else {
                        // this will never happen
                        panic!("If this happened you should probably play lottery")
                    }
                } else {
                    let pal: PngPalette = palette.clone().into();
                    let pal: [u8; 256 * 3] = pal.into();
                    info!("Saving image file: {}", &output_image_file);
                    PngHelper::save(
                        &output_image_file,
                        image.width() * output_scale as u32,
                        image.height() * output_scale as u32,
                        &pal,
                        &output_image_bytes,
                    );
                }
            });
        if merge {
            if let Some(merged_image_bytes) = &merged_image_bytes {
                let pal: PngPalette = palette.clone().into();
                let pal: [u8; 256 * 3] = pal.into();
                info!("Saving merged image file: {}", &output_image_file);
                PngHelper::save(
                    output_image_file,
                    merged_width as u32,
                    merged_height as u32,
                    &pal,
                    merged_image_bytes,
                );
            } else {
                // this will never happen
                panic!("If this happened you should probably play lottery")
            }
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
