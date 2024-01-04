use clap::{arg, command, Args, Parser, Subcommand};

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
    Display,
}