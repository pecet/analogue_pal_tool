use clap::{arg, command, Args, Parser, Subcommand};
use crate::palette::AsAnsiType;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Name / path to .pal file to use
    pub file_name: String,
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
    },
}