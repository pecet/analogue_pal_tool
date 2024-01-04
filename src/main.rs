use analogue_pal_tool::palette::Palette;
use fern;
use log::LevelFilter;
use chrono::Local;

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
    let palette = Palette::load("dev_assets/SameBoy/Desert.pal");

}
