use crate::error::Result;
use crate::storage;
use colored::Colorize;

pub async fn run() -> Result<()> {
    let installed = storage::get_all_installed_packages().await?;

    if installed.is_empty() {
        println!(
            "{} {}",
            "[P2PM]".cyan().bold(),
            "No packages installed.".dimmed()
        );
        return Ok(());
    }

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

    Ok(())
}
