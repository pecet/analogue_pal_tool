use analogue_pal_tool::palette::{AsAnsiType, Palette, AsAnsiVec};
use fern;
use log::{LevelFilter, debug, info};
use chrono::Local;
use clap::Parser;
use colored::Colorize;
use analogue_pal_tool::cli::{Cli, Commands};

fn setup_logging() {
    fern::Dispatch::new()
        // Format the output
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}:{}] {}: {}",
                Local::now().format("[%Y-%m-%d %H:%M:%S]").to_string().green(), // Timestamp format
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
        .chain(fern::log_file(format!("{}.log", env!("CARGO_PKG_NAME")))
        .expect("Cannot setup logging to file"))
        // Apply the configuration
        .apply()
        .expect("Cannot setup logging");
}

fn main() {
    setup_logging();
    info!("{} [{}] loaded", env!("CARGO_PKG_NAME"), env!("GIT_HASH"));

    let cli = Cli::parse();

    let palette = Palette::load(&cli.file_name);

    match cli.command {
        Commands::Display { display_type } => {
            debug!("Loaded palette:\n{:?}", &palette);
            info!("Palette as ANSI 24-bit colored strings:\n{}", palette.as_ansi(display_type));
        }
    };
}
