use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod formats;
mod model;
mod pipeline;

fn main() -> Result<()> {
    let args = cli::Args::parse();

    let cfg = config::Config::load(args.config.as_deref())?;
    config::init_tracing(&cfg.logging, args.log_level.as_deref())?;

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "sub-convtr starting");

    match args.command {
        cli::Command::Convert(cmd) => pipeline::run_convert(cmd, &cfg),
        cli::Command::PrintDefaultConfig => {
            let s = cfg.to_toml_pretty()?;
            print!("{s}");
            Ok(())
        }
    }
}
