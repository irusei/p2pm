use colored::Colorize;
use semver::Version;
use serde::Deserialize;

use crate::{
    error::P2PMError,
    repos::{repo::SearchEntry, version_utils},
    storage::P2PMPackage,
};

pub const REPO_ID: &str = "mws";

#[derive(Deserialize)]
pub struct MWSFilesData {
    data: Vec<MWSFile>,
}

#[derive(Clone, Deserialize)]
pub struct MWSFile {
    mod_id: u32,
    download_url: String,
    version: String,
}

#[derive(Clone, Deserialize)]
pub struct MWSMetadata {
    name: String,
    desc: String,
    dependencies: Vec<MWSDependency>,
}

#[derive(Clone, Deserialize)]
pub struct MWSDependency {
    mod_id: Option<u32>,
    optional: bool,
}

#[derive(Deserialize)]
pub struct MWSSearchEntry {
    pub id: u32,
    pub name: String,
    pub desc: String,
}

#[derive(Deserialize)]
pub struct MWSSearchResponse {
    data: Vec<MWSSearchEntry>,
}

pub async fn fetch_files_from_mod_id(mod_id: &str) -> Result<Vec<MWSFile>, P2PMError> {
    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!("Fetching file list from ModWorkshop (mod {})", mod_id).bold()
    );
    let mws_api_url = format!("https://api.modworkshop.net/mods/{}/files", mod_id);

    let files = reqwest::get(mws_api_url).await?;

    if !files.status().is_success() {
        return Err(P2PMError::NotFound(mod_id.to_string()));
    }

    let json = files.json::<MWSFilesData>().await?;

    if json.data.is_empty() {
        return Err(P2PMError::NoDownloadsFound(mod_id.to_string()));
    }

    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!("Found {} file(s) for mod {}", json.data.len(), mod_id).bold()
    );

    Ok(json.data)
}

pub async fn fetch_mws_metadata(mod_id: String) -> Result<MWSMetadata, P2PMError> {
    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!("Fetching metadata from ModWorkshop (mod {})", mod_id).bold()
    );
    let mws_api_url = format!("https://api.modworkshop.net/mods/{}", mod_id);

    let files = reqwest::get(mws_api_url).await?;

    if !files.status().is_success() {
        return Err(P2PMError::NotFound(mod_id.to_string()));
    }

    let json = files.json::<MWSMetadata>().await?;

    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!("Found metadata for mod {}", mod_id).bold()
    );

    Ok(json)
}

pub fn get_latest_file_from_files(files: &Vec<MWSFile>) -> Option<MWSFile> {
    if files.len() == 0 {
        return None;
    }

    // compare versions of files and get the newest one
    files
        .iter()
        .max_by(|a, b| {
            version_utils::fix_semver_version(&a.version)
                .unwrap_or_else(|_| Version::new(0, 0, 0))
                .cmp(
                    &version_utils::fix_semver_version(&b.version)
                        .unwrap_or_else(|_| Version::new(0, 0, 0)),
                )
        })
        .map(|f| f.clone())
}

pub async fn fetch_package_data_and_download_url(
    mod_id: &str,
) -> Result<(P2PMPackage, String), P2PMError> {
    let files = fetch_files_from_mod_id(mod_id).await?;
    let latest_file = get_latest_file_from_files(&files);

    if latest_file.is_none() {
        return Err(P2PMError::NoDownloadsFound(mod_id.to_string()));
    }

    let latest_file = latest_file.unwrap();
    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!("Latest version: {}", latest_file.version).bold()
    );

    let package_data = construct_p2pm_package(&latest_file).await?;

    Ok((package_data, latest_file.download_url))
}

async fn construct_p2pm_package(file: &MWSFile) -> Result<P2PMPackage, P2PMError> {
    let metadata = fetch_mws_metadata(file.mod_id.to_string()).await?;

    Ok(P2PMPackage {
        repo_id: REPO_ID.to_string(),
        pkg_id: file.mod_id.to_string(),
        name: metadata.name.clone(),
        desc: metadata.desc.clone(),
        version: version_utils::fix_semver_version(&file.version)
            .unwrap_or_else(|_| Version::new(0, 0, 0)),
        dependencies: metadata
            .dependencies
            .iter()
            .filter(|dependency| !dependency.optional)
            .filter_map(|dependency| dependency.mod_id.map(|id| format!("{}/{}", REPO_ID, id)))
            .collect(),
        pkg_type: None,
    })
}

pub async fn get_latest_version(mod_id: &str) -> Result<Version, P2PMError> {
    let files = fetch_files_from_mod_id(&mod_id).await?;
    let latest_file = get_latest_file_from_files(&files);

    if latest_file.is_none() {
        return Err(P2PMError::NoDownloadsFound(mod_id.to_string()));
    }

    Ok(
        version_utils::fix_semver_version(&latest_file.unwrap().version)
            .unwrap_or_else(|_| Version::new(0, 0, 0)),
    )
}

pub async fn search_mods(query: &str, limit: usize) -> Result<Vec<SearchEntry>, P2PMError> {
    let api_url = "https://api.modworkshop.net/games/1/mods";

    let response = reqwest::Client::new()
        .get(api_url)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "query": query,
            "limit": limit,
            "sort": "downloads"
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(P2PMError::NotFound(query.to_string()));
    }

    let json = response.json::<MWSSearchResponse>().await?;

    let results = json
        .data
        .into_iter()
        .map(|entry| SearchEntry {
            id: format!("{}/{}", REPO_ID, entry.id),
            name: entry.name,
            desc: entry.desc,
        })
        .collect();

    Ok(results)
}
