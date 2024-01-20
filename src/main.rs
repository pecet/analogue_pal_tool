use std::fs;
use std::path::Path;

use analogue_pal_tool::palette::{AsAnsiVec, Palette};

use analogue_pal_tool::cli::{Cli, ColorizeImage, Commands, CreateTemplatePal, Display};
use analogue_pal_tool::image_handler::ImageHandler;
use chrono::Local;
use clap::Parser;
use colored::Colorize;

use log::{debug, info, warn, LevelFilter};

fn setup_logging(level: LevelFilter) {
    fern::Dispatch::new()
        // Format the output
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}:{}] {}: {}",
                Local::now()
                    .format("[%Y-%m-%d %H:%M:%S]")
                    .to_string()
                    .green(), // Timestamp format
                record.target().to_string().cyan(),
                record.line().unwrap_or_default().to_string().yellow(),
                record.level(),
                message
            ))
        })
        // Set the default logging level
        .level(level)
        // Output to stdout
        .chain(std::io::stdout())
        // Output to a log file
        .chain(
            fern::log_file(format!("{}.log", env!("CARGO_PKG_NAME")))
                .expect("Cannot setup logging to file"),
        )
        // Apply the configuration
        .apply()
        .expect("Cannot setup logging");
}

fn main() {
    let cli = Cli::parse();
    setup_logging(cli.log_level.into());
    info!("{} [{}] loaded", env!("CARGO_PKG_NAME"), env!("GIT_HASH"));
    match cli.command {
        Commands::Display(Display { display_type, pal_file_name }) => {
            let palette = Palette::load(&pal_file_name)
                .unwrap_or_else(|err| panic!("Cannot load palette: {err}"));
            debug!("Loaded palette:\n{:?}", &palette);
            info!(
                "Palette as ANSI 24-bit colored strings:\n{}",
                palette.as_ansi(display_type)
            );
        }
        Commands::CreateTemplatePal(CreateTemplatePal { output_pal_file }) => {
            let palette = Palette::default();
            palette.save(&output_pal_file);
        }
        Commands::ColorizeImage(ColorizeImage { pal_file_name, input_image_files, output_image_file, scale, merge, max_columns, merge_layout, generate_html }) => {
            if let Some(last_slash) = &output_image_file.rfind('/') {
                let output_dir = &output_image_file[0..*last_slash];
                if !Path::new(output_dir).exists() {
                    warn!("Directory '{output_dir}' does not exists, it will be created");
                    fs::create_dir_all(output_dir).expect("Cannot create directory");
                }
            }
            ImageHandler::use_palettes_to_color_images(
                &pal_file_name,
                &input_image_files,
                &output_image_file,
                scale,
                merge,
                max_columns,
                merge_layout,
                generate_html,
            );
        }
    };
}
