use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "subx")]
#[command(about = "Convert between SRT, VTT, TXT, TSV, and JSON transcript formats.")]
pub struct Args {
    /// Path to config TOML (defaults to ./config.toml if present)
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Override log level (trace, debug, info, warn, error)
    #[arg(long)]
    pub log_level: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Convert between formats
    Convert(ConvertCmd),
    /// Print the effective default config as TOML and exit
    PrintDefaultConfig,
}

#[derive(Debug, Parser)]
pub struct ConvertCmd {
    /// Input file path, or '-' for stdin
    pub input: String,

    /// Output file path (optional)
    #[arg(short, long)]
    pub output: Option<String>,

    /// Target format
    #[arg(long, value_enum)]
    pub to: Format,

    /// Force input format (otherwise inferred from extension or content)
    #[arg(long, value_enum)]
    pub from: Option<Format>,

    /// Write to stdout instead of a file
    #[arg(long)]
    pub stdout: bool,

    /// Allow overwriting output file
    #[arg(long)]
    pub overwrite: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Format {
    Srt,
    Vtt,
    Ass,
    Txt,
    Tsv,
    Json,
}

impl Format {
    pub fn extension(self) -> &'static str {
        match self {
            Format::Srt => "srt",
            Format::Vtt => "vtt",
            Format::Ass => "ass",
            Format::Txt => "txt",
            Format::Tsv => "tsv",
            Format::Json => "json",
        }
    }
}
