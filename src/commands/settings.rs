use crate::cli::SettingsArgs;
use crate::config;
use crate::error::Result;
use colored::Colorize;

pub async fn run(args: SettingsArgs) -> Result<()> {
    match args.path {
        Some(path) => {
            config::save_game_root(&path)?;
            println!(
                "{} {}",
                "[P2PM]".cyan().bold(),
                format!("PAYDAY 2 root folder set to: {}", path).bold()
            );
        }
        None => match config::load_game_root() {
            Ok(Some(path)) => {
                println!(
                    "{} {}",
                    "[P2PM]".cyan().bold(),
                    format!("Current PAYDAY 2 root folder: {}", path.display()).bold()
                );
            }
            Ok(None) => {
                println!(
                    "{} {}",
                    "[P2PM]".cyan().bold(),
                    "No game root folder configured.".dimmed()
                );
                println!(
                    "{} {}",
                    " ".repeat(8),
                    "Usage: p2pm settings <path-to-pd2-folder>".dimmed()
                );
            }
            Err(e) => {
                eprintln!("{}: {}", "Error".red().bold(), e);
            }
        },
    }

    Ok(())
}
