use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::cleaner::clean_projects;
use crate::config::DevSweepConfig;
use crate::scanner::{ScannedProject, scan_directory};
use crate::tui::colors::{blue, cyan, dim, green, red_bold, yellow_bold};
use crate::tui::display::{confirm, multi_select, print_clean_summary, print_results_table};
use crate::util::{format_bytes, parse_age};

// ── Commands ────────────────────────────────────────────────────────────────

pub fn cmd_scan(
    path: &Path,
    max_depth: Option<usize>,
    older_than: Option<&str>,
    json: bool,
    config: &DevSweepConfig,
) -> Result<()> {
    let mut projects = scan_directory(path, max_depth, config)?;
    filter_by_age(&mut projects, older_than)?;
    sort_by_size(&mut projects);

    if json {
        println!("{}", serde_json::to_string_pretty(&projects)?);
    } else {
        print_results_table(&projects);
    }

    Ok(())
}

pub fn cmd_clean(
    path: &Path,
    max_depth: Option<usize>,
    older_than: Option<&str>,
    all: bool,
    dry_run: bool,
    json: bool,
    config: &DevSweepConfig,
) -> Result<()> {
    let mut projects = scan_directory(path, max_depth, config)?;
    filter_by_age(&mut projects, older_than)?;
    sort_by_size(&mut projects);

    if projects.is_empty() {
        println!(
            "\n  {} No projects with cleanable artifacts found.\n",
            blue("ℹ")
        );
        return Ok(());
    }

    print_results_table(&projects);

    let selected_projects: Vec<&ScannedProject> = if all {
        if !dry_run {
            let total: u64 = projects.iter().map(|p| p.total_cleanable_bytes).sum();
            let confirmed = confirm(&format!(
                "Clean ALL {} projects? This will free {} and cannot be undone!",
                projects.len(),
                format_bytes(total),
            ))?;

            if !confirmed {
                println!("  {} Aborted.\n", red_bold("✗"));
                return Ok(());
            }
        }
        projects.iter().collect()
    } else {
        let items: Vec<String> = projects
            .iter()
            .map(|p| {
                format!(
                    "{} ({}) — {} [{}]",
                    p.name,
                    p.kind,
                    format_bytes(p.total_cleanable_bytes),
                    p.clean_targets
                        .iter()
                        .map(|t| t.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect();

        let selections = multi_select("Select projects to clean:", &items)?;

        if selections.is_empty() {
            println!("  {} Nothing selected.\n", blue("ℹ"));
            return Ok(());
        }

        if !dry_run {
            let sel_total: u64 = selections
                .iter()
                .map(|&i| projects[i].total_cleanable_bytes)
                .sum();
            let confirmed = confirm(&format!(
                "Clean {} projects? This will free {}.",
                selections.len(),
                format_bytes(sel_total),
            ))?;
            if !confirmed {
                println!("  {} Aborted.\n", red_bold("✗"));
                return Ok(());
            }
        }

        selections.iter().map(|&i| &projects[i]).collect()
    };

    let action = if dry_run { "Would clean" } else { "Cleaning" };
    println!(
        "\n  {} {} {} projects...\n",
        dim("→"),
        action,
        cyan(&selected_projects.len().to_string()),
    );

    let results = clean_projects(&selected_projects, dry_run);

    if json {
        let summary = serde_json::json!({
            "dry_run": dry_run,
            "projects_cleaned": results.len(),
            "total_bytes_freed": results.iter().map(|r| r.bytes_freed).sum::<u64>(),
            "errors": results.iter().flat_map(|r| r.errors.clone()).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        print_clean_summary(&results, dry_run);
    }

    Ok(())
}

pub fn cmd_summary(
    path: &Path,
    max_depth: Option<usize>,
    older_than: Option<&str>,
    json: bool,
    config: &DevSweepConfig,
) -> Result<()> {
    let mut projects = scan_directory(path, max_depth, config)?;
    filter_by_age(&mut projects, older_than)?;

    let total_bytes: u64 = projects.iter().map(|p| p.total_cleanable_bytes).sum();
    let total_projects = projects.len();

    let mut by_kind: HashMap<String, (usize, u64)> = HashMap::new();
    for p in &projects {
        let entry = by_kind.entry(p.kind.to_string()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += p.total_cleanable_bytes;
    }

    if json {
        let summary = serde_json::json!({
            "total_projects": total_projects,
            "total_reclaimable_bytes": total_bytes,
            "total_reclaimable_human": format_bytes(total_bytes),
            "by_kind": by_kind.iter().map(|(k, (count, bytes))| {
                serde_json::json!({
                    "kind": k,
                    "projects": count,
                    "reclaimable_bytes": bytes,
                    "reclaimable_human": format_bytes(*bytes),
                })
            }).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        println!("\n  📊 dev-sweep summary for {}\n", path.display());
        println!(
            "  Total projects:     {}",
            cyan(&total_projects.to_string())
        );
        println!(
            "  Reclaimable space:  {}",
            yellow_bold(&format_bytes(total_bytes))
        );
        println!();

        if !by_kind.is_empty() {
            println!("  {}", dim("By project type:"));

            let mut sorted: Vec<_> = by_kind.iter().collect();
            sorted.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));

            for (kind, (count, bytes)) in sorted {
                println!(
                    "    {:>12}  {} projects, {}",
                    kind,
                    cyan(&count.to_string()),
                    yellow_bold(&format_bytes(*bytes)),
                );
            }
            println!();
        }
    }

    Ok(())
}

pub fn cmd_config(show: bool, reset: bool) -> Result<()> {
    if reset {
        let config = DevSweepConfig::default();
        config.save()?;
        println!("  {} Config reset to defaults.", green("✓"));
        println!(
            "  {} {}",
            dim("→"),
            DevSweepConfig::config_path().display()
        );
        return Ok(());
    }

    if show {
        let config = DevSweepConfig::load();
        println!("{}", serde_json::to_string_pretty(&config)?);
        return Ok(());
    }

    let config_path = DevSweepConfig::config_path();
    println!("\n  ⚙ dev-sweep configuration\n");
    println!("  Config file: {}", config_path.display());
    println!(
        "  Exists:      {}",
        if config_path.exists() {
            green("yes")
        } else {
            dim("no (using defaults)")
        }
    );

    let config = DevSweepConfig::load();
    println!("\n{}", serde_json::to_string_pretty(&config)?);
    println!(
        "\n  {} Use {} or {} to manage.\n",
        dim("→"),
        green("--show"),
        green("--reset")
    );

    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn sort_by_size(projects: &mut [ScannedProject]) {
    projects.sort_unstable_by_key(|p| std::cmp::Reverse(p.total_cleanable_bytes));
}

fn filter_by_age(projects: &mut Vec<ScannedProject>, older_than: Option<&str>) -> Result<()> {
    if let Some(age_str) = older_than {
        let duration = parse_age(age_str)?;
        let cutoff = chrono::Local::now() - duration;
        projects.retain(|p| p.last_modified < cutoff);
    }
    Ok(())
}
