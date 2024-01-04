use analogue_pal_tool::palette::{AsAnsiType, Palette, AsAnsiVec};
use fern;
use log::{LevelFilter, debug, info};
use chrono::Local;
use clap::Parser;
use analogue_pal_tool::cli::{Cli, Commands};

fn setup_logging() {
    fern::Dispatch::new()
        // Format the output
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}:{}] {}: {}",
                Local::now().format("[%Y-%m-%d %H:%M:%S]"), // Timestamp format
                record.target(),
                record.line().unwrap_or_default(),
                record.level(),
                message
            ))
        })
        // Set the default logging level
        .level(LevelFilter::Debug)
        // Set the logging level for the `hyper` crate
        .level_for("mastodon_async", LevelFilter::Warn)
        .level_for("rustls", LevelFilter::Warn)
        // Output to stdout
        .chain(std::io::stdout())
        // Output to a log file
        .chain(fern::log_file("output.log")
        .expect("Cannot setup logging to file"))
        // Apply the configuration
        .apply()
        .expect("Cannot setup logging");
}

fn main() {
    setup_logging();
    let cli = Cli::parse();

    let palette = Palette::load(&cli.file_name);

    match cli.command {
        Commands::Display => {
            debug!("Loaded palette:\n{:?}", &palette);
            info!("Palette as ANSI 24-bit colored strings:\n{}", palette.as_ansi(AsAnsiType::JustColor));
        }
    };
}
