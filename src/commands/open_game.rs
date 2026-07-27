use crate::error::Result;
use colored::Colorize;
use std::process::Command;

pub async fn run() -> Result<()> {
    println!(
        "{} {}",
        "[P2PM]".cyan().bold(),
        "Launching PAYDAY 2...".bold()
    );

    let status = Command::new("xdg-open")
        .arg("steam://run/218620")
        .status()
        .map_err(|e| color_eyre::eyre::eyre!("failed to run xdg-open: {}", e))?;

    if !status.success() {
        return Err(color_eyre::eyre::eyre!(
            "xdg-open exited with non-zero status"
        ));
    }

    Ok(())
}
