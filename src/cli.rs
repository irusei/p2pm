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
    /// Install one or more mods
    Install(InstallArgs),
    /// Update installed mods
    Update(UpdateArgs),
    /// Search for mods
    Search(SearchArgs),
    /// List currently installed mods
    List,
    /// Uninstall one or more mods
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
    /// Package(s) to install
    #[arg(required = true)]
    pub names: Vec<String>,
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
    /// Package(s) to uninstall
    #[arg(required = true)]
    pub names: Vec<String>,
}

#[derive(clap::Args)]
pub struct OpenArgs {
    pub name: String,
}
