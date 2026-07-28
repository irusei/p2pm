use std::io::Cursor;
use std::path::PathBuf;

use bytes::Bytes;
use color_eyre::owo_colors::OwoColorize;
use compress_tools::{ArchiveContents, ArchiveIteratorBuilder};
use regex::Regex;
use semver::Version;
use tokio::fs;

use crate::{config, error::P2PMError, repos::aggregator};

pub fn get_mod_type(zip_bytes: &[u8]) -> Option<P2PMPackageType> {
    let mut should_read = false;
    let mut chunks: Vec<u8> = vec![];
    let cursor = Cursor::new(zip_bytes.to_vec());
    if let Ok(mut iter) = ArchiveIteratorBuilder::new(cursor).build() {
        loop {
            match iter.next() {
                Some(ArchiveContents::StartOfEntry(name, _)) => {
                    if name.ends_with("/mod.txt") || name == "mod.txt" {
                        return Some(P2PMPackageType::Mod);
                    }
                    if name.ends_with("/main.xml") || name == "main.xml" {
                        // read main.xml to determine between Map and Mod
                        should_read = true;
                    }
                }
                Some(ArchiveContents::DataChunk(chunk)) => {
                    if should_read {
                        chunks.extend_from_slice(&chunk);
                    }
                }
                Some(ArchiveContents::EndOfEntry) => {
                    if should_read {
                        break;
                    }
                }
                Some(ArchiveContents::Err(_)) | None => break,
            }
        }
    }

    if should_read {
        if let Ok(content) = String::from_utf8(chunks) {
            if Regex::new(r"<Mod\b").is_ok_and(|re| re.is_match(&content)) {
                return Some(P2PMPackageType::Mod);
            }
            if Regex::new(r"<table\b").is_ok_and(|re| re.is_match(&content)) {
                return Some(P2PMPackageType::Map);
            }
        }
        return None;
    }
    Some(P2PMPackageType::Override)
}

pub fn read_mod_txt(zip_bytes: &[u8]) -> Vec<u8> {
    let cursor = Cursor::new(zip_bytes.to_vec());
    if let Ok(mut iter) = ArchiveIteratorBuilder::new(cursor).build() {
        let mut should_append = false;
        let mut chunks: Vec<u8> = vec![];
        loop {
            match iter.next() {
                Some(ArchiveContents::StartOfEntry(name, _)) => {
                    if name.ends_with("/mod.txt") || name == "mod.txt" {
                        should_append = true;
                    }
                }
                Some(ArchiveContents::DataChunk(chunk)) => {
                    if should_append {
                        chunks.extend_from_slice(&chunk);
                    }
                }
                Some(ArchiveContents::EndOfEntry) => {
                    if should_append {
                        return chunks;
                    }
                }
                Some(ArchiveContents::Err(_)) | None => break,
            }
        }
    }
    vec![]
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum P2PMPackageType {
    Mod,
    Override,
    Map,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct P2PMPackage {
    pub repo_id: String,
    pub pkg_id: String,
    pub name: String,
    pub desc: String,
    pub version: Version,
    pub dependencies: Vec<String>,
    pub pkg_type: Option<P2PMPackageType>,
}

pub async fn get_installed_version(
    repo_id: &str,
    pkg_id: &str,
) -> Result<Option<Version>, P2PMError> {
    let game_root = config::load_game_root()
        .map_err(|e| P2PMError::Config(e.to_string()))?
        .ok_or(P2PMError::GameRootNotSet)?;

    let json_path = game_root.join("mods").join("p2pm_mods.json");

    if !json_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&json_path).await?;
    let installed: Vec<P2PMPackage> = serde_json::from_str(&content)
        .map_err(|e| P2PMError::Config(format!("Failed to parse p2pm_mods.json: {e}")))?;

    for pkg in installed {
        if pkg.repo_id == repo_id && pkg.pkg_id == pkg_id {
            return Ok(Some(pkg.version));
        }
    }

    Ok(None)
}

#[derive(Debug, Clone)]
pub struct UntrackedMod {
    pub name: String,
    pub path: PathBuf,
    pub mod_type: P2PMPackageType,
}

pub async fn get_untracked_mods() -> Result<Vec<UntrackedMod>, P2PMError> {
    let game_root = config::load_game_root()
        .map_err(|e| P2PMError::Config(e.to_string()))?
        .ok_or(P2PMError::GameRootNotSet)?;

    let installed = get_all_installed_packages().await?;
    let tracked_names: std::collections::HashSet<String> = installed
        .iter()
        .map(|p| sanitize_filename::sanitize(&p.name))
        .collect();

    let mut untracked: Vec<UntrackedMod> = Vec::new();

    // Scan mods folder
    let mods_path = game_root.join("mods");
    if mods_path.is_dir() {
        if let Ok(mut entries) = fs::read_dir(&mods_path).await {
            while let Some(entry) = entries.next_entry().await.ok().flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let path = entry.path();
                if path.is_dir() && !tracked_names.contains(&name) && name != "base" {
                    if path.join("mod.txt").exists() {
                        untracked.push(UntrackedMod {
                            name,
                            path,
                            mod_type: P2PMPackageType::Mod,
                        });
                    }
                }
            }
        }
    }

    // Scan mod_overrides folder
    let overrides_path = game_root.join("assets").join("mod_overrides");
    if overrides_path.is_dir() {
        if let Ok(mut entries) = fs::read_dir(&overrides_path).await {
            while let Some(entry) = entries.next_entry().await.ok().flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let path = entry.path();
                if path.is_dir() && !tracked_names.contains(&name) {
                    untracked.push(UntrackedMod {
                        name,
                        path,
                        mod_type: P2PMPackageType::Override,
                    });
                }
            }
        }
    }

    // Scan Maps folder
    let maps_path = game_root.join("Maps");
    if maps_path.is_dir() {
        if let Ok(mut entries) = fs::read_dir(&maps_path).await {
            while let Some(entry) = entries.next_entry().await.ok().flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let path = entry.path();
                if path.is_dir() && !tracked_names.contains(&name) {
                    untracked.push(UntrackedMod {
                        name,
                        path,
                        mod_type: P2PMPackageType::Map,
                    });
                }
            }
        }
    }

    Ok(untracked)
}

pub async fn get_all_installed_packages() -> Result<Vec<P2PMPackage>, P2PMError> {
    let game_root = config::load_game_root()
        .map_err(|e| P2PMError::Config(e.to_string()))?
        .ok_or(P2PMError::GameRootNotSet)?;

    let json_path = game_root.join("mods").join("p2pm_mods.json");

    if !json_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&json_path).await?;
    let installed: Vec<P2PMPackage> = serde_json::from_str(&content)
        .map_err(|e| P2PMError::Config(format!("Failed to parse p2pm_mods.json: {e}")))?;

    Ok(installed)
}

pub async fn uninstall_package(repo_id: &str, pkg_id: &str) -> Result<(), P2PMError> {
    let game_root = config::load_game_root()
        .map_err(|e| P2PMError::Config(e.to_string()))?
        .ok_or(P2PMError::GameRootNotSet)?;

    let json_path = game_root.join("mods").join("p2pm_mods.json");

    if !json_path.exists() {
        return Err(P2PMError::NotFound(format!("{}/{}", repo_id, pkg_id)));
    }

    let content = fs::read_to_string(&json_path).await?;
    let mut installed: Vec<P2PMPackage> = serde_json::from_str(&content)
        .map_err(|e| P2PMError::Config(format!("Failed to parse p2pm_mods.json: {e}")))?;

    let (mod_name, mod_type) = installed
        .iter()
        .find(|p| p.repo_id == repo_id && p.pkg_id == pkg_id)
        .map(|p| (p.name.clone(), p.pkg_type.clone()))
        .unwrap_or_else(|| (String::new(), None));

    // Make sure it actually existed ig
    let original_len = installed.len();
    installed.retain(|p| !(p.repo_id == repo_id && p.pkg_id == pkg_id));

    if installed.len() == original_len {
        return Err(P2PMError::NotFound(format!("{}/{}", repo_id, pkg_id)));
    }

    // Write back
    let content = serde_json::to_string_pretty(&installed)?;
    fs::write(&json_path, content).await?;

    // Remove mod files from game directory
    let mod_name = sanitize_filename::sanitize(&mod_name);

    match mod_type {
        Some(P2PMPackageType::Mod) => {
            let mut mods_path = game_root.clone();
            mods_path.push("mods");
            mods_path.push(&mod_name);
            if mods_path.exists() {
                fs::remove_dir_all(&mods_path).await?;
            }
        }
        Some(P2PMPackageType::Override) => {
            let mut overrides_path = game_root.clone();
            overrides_path.push("assets");
            overrides_path.push("mod_overrides");
            overrides_path.push(&mod_name);
            if overrides_path.exists() {
                fs::remove_dir_all(&overrides_path).await?;
            }
        }
        Some(P2PMPackageType::Map) => {
            let mut maps_path = game_root.clone();
            maps_path.push("Maps");
            maps_path.push(&mod_name);
            if maps_path.exists() {
                fs::remove_dir_all(&maps_path).await?;
            }
        }
        _ => {}
    }

    Ok(())
}

pub async fn save_installed_packages(
    game_root: &PathBuf,
    package: &P2PMPackage,
) -> Result<(), P2PMError> {
    let mut mods_path = game_root.clone();
    mods_path.push("mods");
    let json_path = mods_path.join("p2pm_mods.json");

    // load existing packages or start empty
    let mut installed: Vec<P2PMPackage> = if json_path.exists() {
        let content = fs::read_to_string(&json_path).await?;
        serde_json::from_str(&content)
            .map_err(|e| P2PMError::Config(format!("Failed to parse p2pm_mods.json: {e}")))?
    } else {
        Vec::new()
    };

    // Check if package already exists, update if so
    let mut found = false;
    for item in installed.iter_mut() {
        if item.repo_id == package.repo_id && item.pkg_id == package.pkg_id {
            *item = package.clone();
            found = true;
            break;
        }
    }
    if !found {
        installed.push(package.clone());
    }

    // Write back
    let content = serde_json::to_string_pretty(&installed)?;
    fs::write(&json_path, content).await?;

    Ok(())
}

async fn create_missing_paths(game_root: &PathBuf) -> Result<(), P2PMError> {
    if !game_root.exists() {
        return Err(P2PMError::GameRootNotSet);
    }

    let mut mod_overrides = game_root.clone();
    mod_overrides.push("assets");
    mod_overrides.push("mod_overrides");

    fs::create_dir_all(&mod_overrides)
        .await
        .map_err(|_| P2PMError::Permission(mod_overrides.display().to_string()))?;

    let mut mods_folder = game_root.clone();
    mods_folder.push("mods");

    fs::create_dir_all(&mods_folder)
        .await
        .map_err(|_| P2PMError::Permission(mods_folder.display().to_string()))?;

    let mut maps_folder = game_root.clone();
    maps_folder.push("Maps");

    fs::create_dir_all(&maps_folder)
        .await
        .map_err(|_| P2PMError::Permission(maps_folder.display().to_string()))?;

    Ok(())
}

pub async fn install_mod_from_zip(
    mod_data: P2PMPackage,
    zip_bytes: Bytes,
) -> Result<(), P2PMError> {
    let game_root = config::load_game_root().unwrap().unwrap();
    create_missing_paths(&game_root).await?;

    // Resolve dependencies
    // TODO: fix circular dependencies
    for dependency in &mod_data.dependencies {
        Box::pin(aggregator::install_package(&dependency)).await?;
    }

    let pkg_type = get_mod_type(&zip_bytes);

    match pkg_type {
        Some(P2PMPackageType::Mod) => {
            println!("{} {}", "[p2pm]".cyan().bold(), "Installing as mod".bold());
            install_as(&game_root, &mod_data, &zip_bytes, "mods").await?;
        }
        Some(P2PMPackageType::Override) => {
            println!(
                "{} {}",
                "[p2pm]".cyan().bold(),
                "Installing as override".bold()
            );
            install_as_override(&game_root, &mod_data, &zip_bytes).await?;
        }
        Some(P2PMPackageType::Map) => {
            println!("{} {}", "[p2pm]".cyan().bold(), "Installing as map".bold());
            install_as(&game_root, &mod_data, &zip_bytes, "Maps").await?;
        }
        None => return Err(P2PMError::UnknownType(mod_data.name)),
    }

    // Save to p2pm_mods.json
    let mut new_mod_data = mod_data.clone();
    new_mod_data.pkg_type = pkg_type;

    save_installed_packages(&game_root, &new_mod_data).await?;

    Ok(())
}

pub async fn install_as(
    game_root: &PathBuf,
    mod_data: &P2PMPackage,
    zip_bytes: &Bytes,
    subfolder: &str,
) -> Result<(), P2PMError> {
    let mut install_folder = game_root.clone();
    install_folder.push(subfolder);

    // check if folder already exists
    install_folder.push(sanitize_filename::sanitize(&mod_data.name));

    if install_folder.exists() {
        fs::remove_dir_all(&install_folder).await?;
    }

    let mut base_folder: Option<String> = None;
    let mod_locator_cursor = Cursor::new(zip_bytes.to_vec());

    if let Ok(mut iter) = ArchiveIteratorBuilder::new(mod_locator_cursor).build() {
        loop {
            match iter.next_header() {
                Some(ArchiveContents::StartOfEntry(name, _)) => {
                    if let Some(mod_root_folder) = name.strip_suffix("mod.txt") {
                        base_folder = Some(mod_root_folder.to_string());
                        break;
                    } else if let Some(mod_root_folder) = name.strip_suffix("main.xml") {
                        base_folder = Some(mod_root_folder.to_string());
                        break;
                    }
                }
                Some(ArchiveContents::DataChunk(_)) => {}
                Some(ArchiveContents::EndOfEntry) => {}
                Some(ArchiveContents::Err(_)) | None => break,
            }
        }
    }

    if let Some(base_folder) = base_folder {
        let cursor = Cursor::new(zip_bytes.to_vec());
        if let Ok(mut iter) = ArchiveIteratorBuilder::new(cursor).build() {
            let mut dest_file_name: Option<String> = None;
            let mut dest_chunk: Vec<u8> = vec![];

            loop {
                match iter.next() {
                    Some(ArchiveContents::StartOfEntry(name, _)) => {
                        dest_chunk.clear();
                        if &base_folder != "" {
                            if name.contains(&base_folder) && !name.ends_with("/") {
                                if let Some((_, rest)) = name.split_once(&base_folder) {
                                    dest_file_name =
                                        Some(format!("{}/{}", install_folder.display(), rest));
                                    println!(
                                        "{} {}",
                                        " ".repeat(8),
                                        dest_file_name.clone().unwrap().dimmed()
                                    );
                                }
                            }
                        }
                    }
                    Some(ArchiveContents::DataChunk(chunk)) => {
                        if dest_file_name.is_some() {
                            dest_chunk.extend_from_slice(&chunk);
                        }
                    }
                    Some(ArchiveContents::EndOfEntry) => {
                        // only extract if dest_file_name is set
                        if let Some(destination) = dest_file_name.take() {
                            let path = PathBuf::from(destination);
                            if let Some(parent) = path.parent() {
                                if let Err(e) = fs::create_dir_all(parent).await {
                                    println!(
                                        "{} {}",
                                        " ".repeat(36),
                                        format!("Failed creating {}: {}", parent.display(), e)
                                            .red()
                                            .bold()
                                    );
                                    continue;
                                }
                            }

                            match fs::write(&path, &dest_chunk).await {
                                Ok(_) => {}
                                Err(_) => {
                                    println!(
                                        "{} {}",
                                        " ".repeat(36),
                                        format!("IMPARTIAL WRITE: {}", &path.display())
                                            .yellow()
                                            .bold()
                                    )
                                }
                            }

                            dest_chunk.clear();
                        }
                    }
                    Some(ArchiveContents::Err(_)) | None => break,
                }
            }
        }
    }
    Ok(())
}

pub async fn install_as_override(
    game_root: &PathBuf,
    mod_data: &P2PMPackage,
    zip_bytes: &Bytes,
) -> Result<(), P2PMError> {
    let override_asset_folders = vec![
        "anims",
        "core",
        "effects",
        "environments",
        "fonts",
        "gamedata",
        "guis",
        "levels",
        "lib",
        "movies",
        "physic_effects",
        "settings",
        "shaders",
        "soundbanks",
        "strings",
        "units",
    ];

    let mut mod_name_folder = game_root.clone();
    mod_name_folder.push("assets");
    mod_name_folder.push("mod_overrides");

    // check if folder already exists
    mod_name_folder.push(sanitize_filename::sanitize(&mod_data.name));

    if mod_name_folder.exists() {
        fs::remove_dir_all(&mod_name_folder).await?;
    }

    let cursor = Cursor::new(zip_bytes.to_vec());
    if let Ok(mut iter) = ArchiveIteratorBuilder::new(cursor).build() {
        let mut dest_file_name: Option<String> = None;
        let mut dest_chunk: Vec<u8> = vec![];

        loop {
            match iter.next() {
                Some(ArchiveContents::StartOfEntry(name, _)) => {
                    dest_chunk.clear();
                    for folder in &override_asset_folders {
                        let asset_folder = &format!("/{}/", folder);
                        if name.contains(asset_folder) && !name.ends_with("/") {
                            if let Some((_, rest)) = name.split_once(asset_folder) {
                                dest_file_name = Some(format!(
                                    "{}{}{}",
                                    mod_name_folder.display(),
                                    asset_folder,
                                    rest
                                ));
                                println!(
                                    "{} {}",
                                    " ".repeat(8),
                                    dest_file_name.as_ref().unwrap().dimmed()
                                );
                            }
                            break;
                        }
                    }
                }
                Some(ArchiveContents::DataChunk(chunk)) => {
                    if dest_file_name.is_some() {
                        dest_chunk.extend_from_slice(&chunk);
                    }
                }
                Some(ArchiveContents::EndOfEntry) => {
                    // only extract if dest_file_name is set
                    if let Some(destination) = dest_file_name.take() {
                        let path = PathBuf::from(&destination);
                        if let Some(parent) = path.parent() {
                            if let Err(e) = fs::create_dir_all(parent).await {
                                println!(
                                    "{} {}",
                                    " ".repeat(36),
                                    format!("Failed creating {}: {}", parent.display(), e)
                                        .red()
                                        .bold()
                                );
                                continue;
                            }
                        }

                        match fs::write(&path, &dest_chunk).await {
                            Ok(_) => {}
                            Err(_) => {
                                println!(
                                    "{} {}",
                                    " ".repeat(36),
                                    format!("IMPARTIAL WRITE: {}", &path.display())
                                        .yellow()
                                        .bold()
                                )
                            }
                        }

                        dest_chunk.clear();
                    }
                }
                Some(ArchiveContents::Err(_)) | None => break,
            }
        }
    }

    Ok(())
}
