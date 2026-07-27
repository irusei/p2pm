use bytes::Bytes;
use colored::Colorize;
use semver::Version;

use crate::{
    error::P2PMError,
    repos::mws::{self},
    storage,
};

pub async fn fetch_zip(download_url: &str) -> Result<Bytes, P2PMError> {
    // fetch zip
    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!("Downloading from {}", download_url).bold()
    );
    let zip_request = reqwest::get(download_url).await?;
    let zip_bytes = zip_request.bytes().await?;
    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!("Downloaded {} bytes", zip_bytes.len()).bold()
    );

    Ok(zip_bytes)
}

pub async fn install_package(package_id: &str) -> Result<(), P2PMError> {
    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!("Parsing package ID '{}'", package_id).bold()
    );

    // package_id must be in repo/package_id format
    let mut split = package_id.split("/");

    let repo_id = split.next();
    let mod_id = split.next();

    if repo_id.is_none() || mod_id.is_none() {
        return Err(P2PMError::NotFound(package_id.to_string()));
    };

    let repo_id = repo_id.unwrap();
    let mod_id = mod_id.unwrap();

    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!("Using repo: {}", repo_id).bold()
    );

    let (package_data, dl_url) = match repo_id {
        mws::REPO_ID => {
            println!(
                "{} {}",
                "[p2pm]".cyan().bold(),
                "Fetching mod data from ModWorkshop".bold()
            );
            mws::fetch_package_data_and_download_url(&mod_id).await?
        }
        _ => return Err(P2PMError::RepoNotFound(repo_id.to_string())),
    };

    let installed_version = storage::get_installed_version(repo_id, mod_id).await?;
    if let Some(version) = installed_version {
        if version.eq(&package_data.version) {
            println!(
                "{} {}",
                "[p2pm]".cyan().bold(),
                format!(
                    "Version v{} of mod {} is already installed, skipping",
                    package_data.version, package_data.name
                )
                .dimmed()
            );
            return Ok(());
        }
    }
    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!(
            "Downloading mod: {} v{}",
            package_data.name, package_data.version
        )
        .bold()
    );

    let zip_bytes = fetch_zip(&dl_url).await?;
    storage::install_mod_from_zip(package_data, zip_bytes).await?;

    Ok(())
}

pub async fn update_all() -> Result<(), P2PMError> {
    let installed = storage::get_all_installed_packages().await?;

    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        "Checking for updates...".bold()
    );

    let mut updated = 0;

    for package in installed {
        let latest_version = match package.repo_id.as_str() {
            mws::REPO_ID => mws::get_latest_version(&package.pkg_id).await?,
            _ => Version::new(0, 0, 0),
        };

        if latest_version == package.version {
            println!(
                "{} {}",
                "[p2pm]".cyan().bold(),
                format!("{} v{} is up to date", package.name, package.version).dimmed()
            );
            continue;
        }

        println!(
            "{} {}",
            "[p2pm]".cyan().bold(),
            format!(
                "Updating {} v{} -> v{}",
                package.name, package.version, latest_version
            )
            .bold()
        );

        install_package(&format!("{}/{}", package.repo_id, package.pkg_id)).await?;
        updated += 1;
    }

    if updated > 0 {
        println!(
            "{} {}",
            "[p2pm]".cyan().bold(),
            format!("{} package(s) updated", updated).green().bold()
        );
    } else {
        println!(
            "{} {}",
            "[p2pm]".cyan().bold(),
            "All packages are up to date".green().bold()
        );
    }

    Ok(())
}
