use crate::config;
use crate::error::Result;
use crate::storage;
use colored::Colorize;
use tokio::fs;

pub async fn run() -> Result<()> {
    let installed = storage::get_all_installed_packages().await?;
    let game_root = config::load_game_root().ok().flatten();

    let tracked_names: std::collections::HashSet<String> = installed
        .iter()
        .map(|p| sanitize_filename::sanitize(&p.name))
        .collect();

    // scan for untracked mods
    let mut untracked: Vec<String> = Vec::new();
    if let Some(ref root) = game_root {
        // Scan mods folder
        let mods_path = root.join("mods");
        if mods_path.is_dir() {
            let mut entries = fs::read_dir(&mods_path).await.ok();
            while let Some(ref mut entries) = entries {
                if let Some(entry) = entries.next_entry().await.ok().flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let path = entry.path();
                    if path.is_dir() && !tracked_names.contains(&name) && name != "base" {
                        // Check if it has mod.txt
                        if path.join("mod.txt").exists() {
                            untracked.push(name);
                        }
                    }
                } else {
                    break;
                }
            }
        }

        // Scan mod_overrides folder
        let overrides_path = root.join("assets").join("mod_overrides");
        if overrides_path.is_dir() {
            let mut entries = fs::read_dir(&overrides_path).await.ok();
            while let Some(ref mut entries) = entries {
                if let Some(entry) = entries.next_entry().await.ok().flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let path = entry.path();
                    if path.is_dir() && !tracked_names.contains(&name) {
                        untracked.push(name);
                    }
                } else {
                    break;
                }
            }
        }
    }

    if installed.is_empty() && untracked.is_empty() {
        println!(
            "{} {}",
            "[p2pm]".cyan().bold(),
            "No packages installed.".dimmed()
        );
        return Ok(());
    }

    if !installed.is_empty() {
        println!("{}", "Installed (tracked by p2pm):".bold().cyan());
        for pkg in &installed {
            let pkg_ref = format!("{}/{} ({})", pkg.repo_id, pkg.pkg_id, pkg.name);
            let version_str = pkg.version.to_string();

            // Trim
            let desc: String = pkg.desc.chars().filter(|c| *c != '\n').collect();
            let desc = if desc.len() > 100 {
                format!("{}...", &desc[..97])
            } else {
                desc
            };

            println!(
                "{}  {} [{}]",
                "*".cyan(),
                pkg_ref.bold(),
                "Installed".green()
            );
            println!(
                "{}      Latest version installed: {}",
                " ".repeat(14),
                version_str.blue()
            );
            println!("{}      Description:   {}", " ".repeat(14), desc.dimmed());
            if !pkg.dependencies.is_empty() {
                println!(
                    "{}      Dependencies:  {}",
                    " ".repeat(14),
                    pkg.dependencies.join(", ").dimmed()
                );
            }
            println!();
        }
    }

    if !untracked.is_empty() {
        println!("{}", "Untracked (not managed by p2pm):".bold().yellow());
        for name in &untracked {
            println!("{}  {}", "-".yellow(), name.bold());
        }
        println!();
    }

    Ok(())
}
