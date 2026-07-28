use std::sync::LazyLock;

use crate::{
    error::P2PMError,
    repos::{repo::SearchEntry, version_utils::fix_semver_version},
    storage::{self, P2PMPackage},
};
use colored::Colorize;
use regex::Regex;
use semver::Version;
use serde::Deserialize;
use strsim::jaro_winkler;

pub const REPO_ID: &str = "z77";

#[derive(Deserialize, Clone)]
pub struct Z77Mod {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub download_url: String,
    pub dependencies: Vec<String>,
}

static MODS: LazyLock<Vec<Z77Mod>> = LazyLock::new(|| {
    let json = include_str!("z77_mods.json");
    serde_json::from_str(json).expect("Failed to parse z77_mods.json")
});

pub async fn fetch_package_data_and_download_url(
    mod_id: &str,
) -> Result<(P2PMPackage, String), P2PMError> {
    let z77_mod = &*MODS
        .iter()
        .find(|m| m.id == mod_id)
        .ok_or_else(|| P2PMError::NotFound(format!("{}/{}", REPO_ID, mod_id)))?;

    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!("Found mod: {}", z77_mod.name).bold()
    );

    let package = P2PMPackage {
        repo_id: REPO_ID.to_string(),
        pkg_id: mod_id.to_string(),
        name: z77_mod.name.clone(),
        desc: z77_mod.desc.clone(),
        version: get_latest_version(&mod_id).await?,
        dependencies: z77_mod.dependencies.clone(),
        pkg_type: None,
    };

    Ok((package, z77_mod.download_url.clone()))
}

pub async fn get_latest_version(mod_id: &str) -> Result<Version, P2PMError> {
    let z77_mod = &*MODS
        .iter()
        .find(|m| m.id == mod_id)
        .ok_or_else(|| P2PMError::NotFound(format!("{}/{}", REPO_ID, mod_id)))?;

    // fetch zip_bytes of mod
    let zip_bytes = reqwest::get(&z77_mod.download_url).await?.bytes().await?;

    // read mod.txt
    let mod_txt_bytes = storage::read_mod_txt(&zip_bytes);

    // can't read the json for version, so must use regex
    if mod_txt_bytes.len() > 0 {
        let mod_txt_utf8 = String::from_utf8(mod_txt_bytes);
        if let Ok(mod_txt_utf8) = mod_txt_utf8 {
            let re = Regex::new(r#""version"\s*:\s*"([^"]+)"#);
            if let Ok(re) = re {
                let version = re
                    .captures(&mod_txt_utf8)
                    .and_then(|c| c.get(1))
                    .map(|m| {
                        fix_semver_version(m.as_str()).unwrap_or_else(|_| Version::new(0, 0, 0))
                    })
                    .unwrap_or_else(|| Version::new(0, 0, 0));

                return Ok(version);
            }
        }
    }

    Ok(Version::new(0, 0, 0))
}

pub fn search_mods(query: &str, limit: usize) -> Result<Vec<super::SearchEntry>, P2PMError> {
    let mut results: Vec<SearchEntry> = MODS
        .iter()
        .filter_map(|m| {
            let score = jaro_winkler(&m.name.to_lowercase(), &query.to_lowercase());
            if score >= 0.8 {
                Some(SearchEntry {
                    id: format!("{}/{}", REPO_ID, m.id.clone()),
                    name: m.name.clone(),
                    desc: m.desc.clone(),
                })
            } else {
                None
            }
        })
        .collect();

    results.truncate(limit);
    Ok(results)
}
