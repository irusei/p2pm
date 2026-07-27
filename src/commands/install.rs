use crate::cli::InstallArgs;
use crate::error::Result;
use crate::repos::aggregator;
use colored::Colorize;

pub async fn run(args: InstallArgs) -> Result<()> {
    println!(
        "{} {}",
        "[P2PM]".cyan().bold(),
        format!("Installing '{}'", args.name).bold()
    );

    aggregator::install_package(&args.name).await?;

    println!("{} {}", "[P2PM]".cyan().bold(), "Done".green().bold());
    Ok(())
}
