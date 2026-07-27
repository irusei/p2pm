use crate::cli::InstallArgs;
use crate::error::Result;
use crate::repos::aggregator;
use colored::Colorize;

pub async fn run(args: InstallArgs) -> Result<()> {
    let count = args.names.len();
    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!("Installing {} package(s)...", count).bold()
    );

    let mut succeeded = 0;
    let mut failed: Vec<String> = Vec::new();

    for name in &args.names {
        println!(
            "{} {}",
            "[p2pm]".cyan().bold(),
            format!("Installing '{}'", name).bold()
        );

        let mut split = name.split("/");
        let repo_id = split.next();
        let mod_id = split.next();

        if repo_id.is_none() || mod_id.is_none() {
            failed.push(format!(
                "{}: invalid format (expected repo/package_id)",
                name
            ));
            continue;
        }

        match aggregator::install_package(name).await {
            Ok(()) => {
                succeeded += 1;
            }
            Err(e) => {
                failed.push(format!("{}: {}", name, e));
            }
        }
    }

    println!("{} {}", "[p2pm]".cyan().bold(), "Done".green().bold());
    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!("{} succeeded", succeeded).green()
    );
    if !failed.is_empty() {
        println!(
            "{} {}:",
            "[p2pm]".cyan().bold(),
            format!("{} failed", failed.len()).red(),
        );
        for reason in &failed {
            println!("      - {}", reason.dimmed());
        }
    }

    Ok(())
}
