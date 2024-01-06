use analogue_pal_tool::palette::{AsAnsiVec, Palette};

use analogue_pal_tool::cli::{Cli, Commands};
use analogue_pal_tool::image_handler::ImageHandler;
use chrono::Local;
use clap::Parser;
use colored::Colorize;
use log::{debug, info, LevelFilter};

fn setup_logging() {
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
        .level(LevelFilter::Debug)
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
    setup_logging();
    let cli = Cli::parse();
    info!("{} [{}] loaded", env!("CARGO_PKG_NAME"), env!("GIT_HASH"));
    match cli.command {
        Commands::Display {
            display_type,
            pal_file_name,
        } => {
            let palette = Palette::load(&pal_file_name);
            debug!("Loaded palette:\n{:?}", &palette);
            info!(
                "Palette as ANSI 24-bit colored strings:\n{}",
                palette.as_ansi(display_type)
            );
        }
        Commands::CreateTemplatePal { output_pal_file } => {
            let palette = Palette::default();
            palette.save(&output_pal_file);
        }
        Commands::ColorizeImage {
            pal_file_name,
            input_image_files,
            output_image_file,
            scale,
            merge, max_columns, merge_layout,
        } => {
            ImageHandler::color_images(
                &pal_file_name,
                &input_image_files,
                &output_image_file,
                scale,
                merge, max_columns, merge_layout,
            );
        }
    };
}
