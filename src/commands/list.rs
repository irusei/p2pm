use crate::error::Result;
use crate::storage;
use colored::Colorize;

pub async fn run() -> Result<()> {
    let installed = storage::get_all_installed_packages().await?;
    let untracked = storage::get_untracked_mods().await?;

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
        for mod_info in &untracked {
            let type_str = match mod_info.mod_type {
                storage::P2PMPackageType::Mod => "mod",
                storage::P2PMPackageType::Override => "override",
                _ => "unknown",
            };
            println!(
                "{}  {} [{}]",
                "-".yellow(),
                mod_info.name.bold(),
                type_str.dimmed()
            );
        }
        println!();
    }

    Ok(())
}
