use clap::{arg, command, Args, Parser, Subcommand};
use crate::palette::AsAnsiType;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Display palette as ANSI colored string.
    /// Requires 24-bit color support in terminal.
    Display {
        #[clap(short, long, default_value_t, value_enum)]
        display_type: AsAnsiType,
        /// Name / path to .pal file to read
        pal_file_name: String,
    },
    /// Create template pal file which will be used for previews.
    ///
    /// After generating such .pal and loading it on your Analogue Pocket
    /// create screenshots with it, then these screenshots can be used
    /// to preview other palettes.
    CreateTemplatePal {
        #[clap(short, long = "output", required = true)]
        /// Name / path to .pal file to write
        output_pal_file: String,
    }
}