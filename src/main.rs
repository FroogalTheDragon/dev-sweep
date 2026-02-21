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
    let _config = DevSweepConfig::load();

    let scan_path = cli
        .path
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let scan_path = if scan_path.starts_with("~") {
        dirs::home_dir()
            .unwrap_or_default()
            .join(scan_path.strip_prefix("~").unwrap_or(&scan_path))
    } else {
        scan_path
    };

    if !scan_path.is_dir() {
        anyhow::bail!(
            "Path does not exist or is not a directory: {}",
            scan_path.display()
        );
    }

    match cli.command.unwrap_or(Commands::Scan) {
        Commands::Scan => cmd_scan(
            &scan_path,
            cli.max_depth,
            cli.older_than.as_deref(),
            cli.json,
        ),
        Commands::Clean { all, dry_run } => cmd_clean(
            &scan_path,
            cli.max_depth,
            cli.older_than.as_deref(),
            all,
            dry_run,
            cli.json,
        ),
        Commands::Summary => cmd_summary(
            &scan_path,
            cli.max_depth,
            cli.older_than.as_deref(),
            cli.json,
        ),
        Commands::Config { show, reset } => cmd_config(show, reset),
    }
}
