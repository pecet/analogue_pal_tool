use crate::image_handler::MergeLayout;
use crate::palette::AsAnsiType;
use clap::{command, Parser, Subcommand, ValueEnum};
use log::LevelFilter;
use rayon::prelude::*;

/// We need this so we can implement ValueEnum for foreign type LevelFilter
///
/// So this is basically copy of that
#[derive(Debug, Copy, Clone, Default, ValueEnum)]
pub enum MyLevelFilter {
    /// A level lower than all log levels.
    Off,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    #[default]
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}

macro_rules! __mlf_internal {
    ($level: ident, $current: ident) => {
        if let MyLevelFilter::$level = $current {
            return LevelFilter::$level;
        }
    };
}

macro_rules! mlf {
    ($current: ident, $($args: ident),*) => {
        $(
            __mlf_internal!($args, $current);
        )*
    }
}

impl From<MyLevelFilter> for LevelFilter {
    fn from(value: MyLevelFilter) -> Self {
        mlf!(value, Off, Error, Warn, Info, Debug, Trace);
        LevelFilter::Off
    }
}

#[derive(Parser, Debug)]
#[command(author, version = env!("GIT_HASH_SHORT"), about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[clap(short='z', long="log_level", value_enum, default_value_t)]
    pub log_level: MyLevelFilter,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Display palette as ANSI colored string.
    /// Requires 24-bit color support in terminal.
    #[clap(aliases = ["d", "show"])]
    Display {
        #[clap(short, long, default_value_t, value_enum)]
        display_type: AsAnsiType,
        /// Name / path to .pal file to read
        pal_file_name: String,
    },
    /// Create template pal file which will be used for taking screenshots for colorization.
    ///
    /// After generating such .pal and loading it on your Analogue Pocket
    /// create screenshots with it, then these screenshots can be used
    /// to preview other palettes.
    #[clap(aliases = ["t", "template"])]
    CreateTemplatePal {
        #[clap(short, long = "output", required = true)]
        /// Name / path to .pal file to write
        output_pal_file: String,
    },
    /// Colorize input screenshot .png file using palette provided and save as new image file
    #[clap(aliases = ["c", "color-image", "color", "colorize"])]
    ColorizeImage {
        /// Name / path to .pal file(s) to read
        ///
        /// Glob patterns may be used e.g: *.pal palettes/**/*.pal
        #[clap(short = 'p', long = "pal", alias = "palette", required = true)]
        pal_file_name: Vec<String>,
        /// Name / path to input screenshot(s) .png file(s) to read
        ///
        /// Screenshot(s) MUST be created using palette generated by create-template-pal
        ///
        /// Glob patterns may be used e.g: *.png screenshots/**/*.png
        #[clap(required = true)]
        input_image_files: Vec<String>,
        /// Name / path to .png file to write
        ///
        /// If multiple input images are provided and --merge is not, then output will be used as a prefix
        /// and images with counter
        /// E.g. for 'out.png':
        /// out000.png out001.png etc.
        ///
        /// If multiple pal files are provided then pal file path will be included, with '$' replacing '/'
        /// E.g. for 'out.png', and '1.pal' '2.pal' 'directory/3.pal':
        /// out1.pal.png out2.pal.png outdirectory$3.pal.png
        #[clap(short, long = "output", required = true, verbatim_doc_comment)]
        output_image_file: String,
        /// Scale factor to apply for output image, only integer values are supported
        ///
        /// If not supplied no scaling is applied
        #[clap(short = 's', long = "scale")]
        scale: Option<u8>,
        /// Merge multiple images into one output image
        #[clap(short = 'm', long = "merge")]
        merge: bool,
        /// Merge: maximum columns to use
        #[clap(short = 'k', long = "columns", default_value_t = 4)]
        max_columns: u8,
        /// Merge: layout to use while merging
        #[clap(short = 'l', long = "layout", default_value_t, value_enum)]
        merge_layout: MergeLayout,
        /// Generate HTML file for image previews
        #[clap(short = 't', long = "html", default_value_t = false)]
        generate_html: bool,
    },
}
