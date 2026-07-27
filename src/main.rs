mod cli;
mod commands;
mod config;
mod deeplink;
mod error;
mod repos;
mod storage;

use clap::Parser;
use color_eyre::Result;
use colored::Colorize;

fn require_game_root() -> Result<()> {
    match config::load_game_root() {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err(color_eyre::eyre::Report::from(
            crate::error::P2PMError::GameRootNotSet,
        )),
        Err(e) => Err(e.into()),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // Register deeplink handler on first run
    if let Err(e) = deeplink::register_deeplink_handler() {
        eprintln!(
            "{}: Could not register deeplink handler: {}",
            "Warning".red().bold(),
            e
        );
    }

    let args = cli::Cli::parse();

    match args.command {
        cli::Commands::Settings(cmd) => commands::settings::run(cmd).await?,
        cli::Commands::Install(cmd) => {
            require_game_root()?;
            commands::install::run(cmd).await?
        }
        cli::Commands::Update(cmd) => {
            require_game_root()?;
            commands::update::run(cmd).await?
        }
        cli::Commands::Search(cmd) => {
            require_game_root()?;
            commands::search::run(cmd).await?
        }
        cli::Commands::List => {
            require_game_root()?;
            commands::list::run().await?
        }
        cli::Commands::Uninstall(cmd) => {
            require_game_root()?;
            commands::uninstall::run(cmd).await?
        }
        cli::Commands::Deeplink(cmd) => {
            let package_id = deeplink::parse_deeplink(&cmd.url)?;
            println!(
                "{} {}",
                "[P2PM]".cyan().bold(),
                format!("Installing from deeplink: {}", package_id).bold()
            );
            require_game_root()?;
            commands::install::run(cli::InstallArgs { name: package_id }).await?
        }
    }

    Ok(())
}
