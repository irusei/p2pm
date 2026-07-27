use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "p2pm")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Root folder path setup
    Settings(SettingsArgs),
    /// Install a mod
    Install(InstallArgs),
    /// Update installed mods
    Update(UpdateArgs),
    /// Search for mods
    Search(SearchArgs),
    /// List currently installed mods
    List,
    /// Uninstall a mod
    Uninstall(UninstallArgs),
    /// Open a mod folder
    Open(OpenArgs),
    /// Open the game
    OpenGame,
    /// Handle mws-pdmm://deeplink
    Deeplink(DeeplinkArgs),
}

#[derive(clap::Args)]
pub struct SettingsArgs {
    /// Path to PAYDAY 2 folder
    pub path: Option<String>,
}

#[derive(clap::Args)]
pub struct InstallArgs {
    pub name: String,
}

#[derive(clap::Args)]
pub struct UpdateArgs {
    pub package: Option<String>,
}

#[derive(clap::Args)]
pub struct SearchArgs {
    pub query: String,

    #[arg(short, long, default_value = "10")]
    pub limit: usize,
}

#[derive(clap::Args)]
pub struct DeeplinkArgs {
    #[arg(value_name = "URL")]
    pub url: String,
}

#[derive(clap::Args)]
pub struct UninstallArgs {
    pub name: String,
}

#[derive(clap::Args)]
pub struct OpenArgs {
    pub name: String,
}
