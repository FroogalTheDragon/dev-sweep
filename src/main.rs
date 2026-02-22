use std::path::PathBuf;
use std::process;

use anyhow::Result;
use clap::Parser;

use dev_sweep::cli::commands::{cmd_clean, cmd_config, cmd_scan, cmd_summary};
use dev_sweep::cli::{Cli, Commands};
use dev_sweep::config::DevSweepConfig;
use dev_sweep::tui::colors::red_bold;

fn main() {
    if let Err(e) = run() {
        eprintln!("  {} {}", red_bold("Error:"), e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = DevSweepConfig::load();

    // CLI flags take precedence over config; config provides defaults.
    let max_depth = cli.max_depth.or(config.max_depth);

    let scan_path = resolve_scan_path(&cli, &config)?;

    match cli.command.unwrap_or(Commands::Scan) {
        Commands::Scan => cmd_scan(
            &scan_path,
            max_depth,
            cli.older_than.as_deref(),
            cli.json,
            &config,
        ),
        Commands::Clean { all, dry_run } => cmd_clean(
            &scan_path,
            max_depth,
            cli.older_than.as_deref(),
            all,
            dry_run,
            cli.json,
            &config,
        ),
        Commands::Summary => cmd_summary(
            &scan_path,
            max_depth,
            cli.older_than.as_deref(),
            cli.json,
            &config,
        ),
        Commands::Config { show, reset } => cmd_config(show, reset),
    }
}

/// Determine the scan path from CLI args, config defaults, or the current directory.
///
/// Priority: CLI `--path` > config `default_roots[0]` > current directory.
fn resolve_scan_path(cli: &Cli, config: &DevSweepConfig) -> Result<PathBuf> {
    let raw = if let Some(ref p) = cli.path {
        p.clone()
    } else if let Some(first) = config.default_roots.first() {
        first.clone()
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };

    let expanded = if raw.starts_with("~") {
        dirs::home_dir()
            .unwrap_or_default()
            .join(raw.strip_prefix("~").unwrap_or(&raw))
    } else {
        raw
    };

    if !expanded.is_dir() {
        anyhow::bail!(
            "Path does not exist or is not a directory: {}",
            expanded.display()
        );
    }

    Ok(expanded)
}
