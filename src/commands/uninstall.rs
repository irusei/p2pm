use crate::cli::UninstallArgs;
use crate::error::{P2PMError, Result};
use crate::storage;
use colored::Colorize;

pub async fn run(args: UninstallArgs) -> Result<()> {
    let mut split = args.name.split("/");

    let repo_id = split.next();
    let mod_id = split.next();

    if repo_id.is_none() || mod_id.is_none() {
        return Err(color_eyre::eyre::Report::from(P2PMError::NotFound(
            args.name.to_string(),
        )));
    };

    let repo_id = repo_id.unwrap();
    let mod_id = mod_id.unwrap();

    println!(
        "{} {}",
        "[P2PM]".cyan().bold(),
        format!("Uninstalling {}...", args.name).bold()
    );

    storage::uninstall_package(repo_id, mod_id).await?;

    println!("{} {}", "[P2PM]".cyan().bold(), "Done".green().bold());
    Ok(())
}
