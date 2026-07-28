use crate::cli::OpenArgs;
use crate::config;
use crate::error::{P2PMError, Result};
use crate::storage;
use colored::Colorize;
use std::process::Command;

pub async fn run(args: OpenArgs) -> Result<()> {
    let mut split = args.name.split("/");

    let repo_id = split.next();
    let mod_id = split.next();

    if repo_id.is_none() || mod_id.is_none() {
        return Err(color_eyre::eyre::Report::from(P2PMError::NotFound(
            args.name.to_string(),
        )));
    };

    let repo_id = repo_id.unwrap();
    let mod_id = mod_id.unwrap();

    // Get installed packages
    let installed = storage::get_all_installed_packages().await?;

    let pkg = installed
        .iter()
        .find(|p| p.repo_id == repo_id && p.pkg_id == mod_id)
        .ok_or_else(|| P2PMError::NotFound(format!("{}/{}", repo_id, mod_id)))?;

    let mod_name = sanitize_filename::sanitize(&pkg.name);

    let game_root = config::load_game_root()
        .map_err(|e| P2PMError::Config(e.to_string()))?
        .ok_or(P2PMError::GameRootNotSet)?;

    let path = match pkg.pkg_type {
        Some(storage::P2PMPackageType::Override) => {
            let mut path = game_root.clone();
            path.push("assets");
            path.push("mod_overrides");
            path.push(&mod_name);
            path
        }
        Some(storage::P2PMPackageType::Mod) => {
            let mut path = game_root.clone();
            path.push("mods");
            path.push(&mod_name);
            path
        }
        Some(storage::P2PMPackageType::Map) => {
            let mut path = game_root.clone();
            path.push("Maps");
            path.push(&mod_name);
            path
        }
        _ => {
            return Err(color_eyre::eyre::Report::from(P2PMError::NotFound(
                format!("Mod folder not found for {}/{}", pkg.repo_id, pkg.pkg_id),
            )));
        }
    };

    if !path.exists() {
        return Err(color_eyre::eyre::Report::from(P2PMError::NotFound(
            format!("Mod folder not found: {}", path.display()),
        )));
    }

    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!("Opening {}...", path.display()).bold()
    );

    let status = Command::new("xdg-open")
        .arg(&path)
        .status()
        .map_err(|e| color_eyre::eyre::eyre!("failed to run xdg-open: {}", e))?;

    if !status.success() {
        return Err(color_eyre::eyre::eyre!(
            "xdg-open exited with non-zero status"
        ));
    }

    Ok(())
}
