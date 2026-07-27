use std::path::PathBuf;

use crate::error::{P2PMError, Result};

const CONFIG_DIR: &str = ".config/p2pm";
const CONFIG_FILE: &str = "settings.json";

fn config_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(CONFIG_DIR).join(CONFIG_FILE)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Settings {
    game_root: Option<String>,
}

pub fn load_game_root() -> Result<Option<PathBuf>> {
    let path = config_path();
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)?;
    let settings: Settings = serde_json::from_str(&content)
        .map_err(|_| P2PMError::Config("Invalid settings file".to_string()))?;

    match settings.game_root {
        Some(path_str) => {
            let p = PathBuf::from(path_str);
            if !p.is_dir() {
                return Err(color_eyre::eyre::Report::from(P2PMError::Config(format!(
                    "Game root path '{}' does not exist or is not a directory",
                    p.display()
                ))));
            }
            Ok(Some(p))
        }
        None => Ok(None),
    }
}

pub fn save_game_root(path: &str) -> Result<()> {
    let path_buf = PathBuf::from(path);
    if !path_buf.is_dir() {
        return Err(color_eyre::eyre::Report::from(P2PMError::Config(format!(
            "'{}' is not a valid directory",
            path_buf.display()
        ))));
    }

    let config_file = config_path();
    let config_dir = config_file.parent().unwrap();
    std::fs::create_dir_all(config_dir)?;

    let settings = Settings {
        game_root: Some(path.to_string()),
    };

    let content = serde_json::to_string_pretty(&settings)?;
    std::fs::write(config_path(), content)?;

    Ok(())
}
