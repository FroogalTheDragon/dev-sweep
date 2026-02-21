use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// CLI argument definitions for dev-sweep.
#[derive(Parser)]
#[command(
    name = "dev-sweep",
    about = "🧹 Find and clean build artifacts & dependency caches across all your dev projects",
    long_about = "dev-sweep scans your filesystem for developer projects and identifies \
                  reclaimable disk space from build artifacts, dependency caches, and \
                  generated files. It supports 17+ project types including Rust, Node.js, \
                  Python, Java, .NET, Go, and more.",
    version,
    author = "Mark Waid Jr"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Directory to scan (defaults to current directory)
    #[arg(global = true)]
    pub path: Option<PathBuf>,

    /// Maximum directory depth to scan
    #[arg(short = 'd', long, global = true)]
    pub max_depth: Option<usize>,

    /// Only show projects older than this (e.g. "30d", "3m", "1y")
    #[arg(short, long, global = true)]
    pub older_than: Option<String>,

    /// Output results as JSON
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan for projects and show what can be cleaned (default)
    Scan,
    /// Interactively select and clean projects
    Clean {
        /// Clean all found projects without prompting
        #[arg(short, long)]
        all: bool,
        /// Show what would be cleaned without actually deleting
        #[arg(long)]
        dry_run: bool,
    },
    /// Show a quick summary of reclaimable space
    Summary,
    /// Manage dev-sweep configuration
    Config {
        /// Show the current config
        #[arg(long)]
        show: bool,
        /// Reset config to defaults
        #[arg(long)]
        reset: bool,
    },
}
