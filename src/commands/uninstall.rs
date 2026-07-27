use crate::cli::UninstallArgs;
use crate::error::Result;
use crate::storage;
use colored::Colorize;

pub async fn run(args: UninstallArgs) -> Result<()> {
    let count = args.names.len();
    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!("Uninstalling {} package(s)...", count).bold()
    );

    let mut succeeded = 0;
    let mut failed: Vec<String> = Vec::new();

    for name in &args.names {
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

        let repo_id = repo_id.unwrap();
        let mod_id = mod_id.unwrap();

        println!(
            "{} {}",
            "[p2pm]".cyan().bold(),
            format!("Uninstalling '{}'", name).bold()
        );
        match storage::uninstall_package(repo_id, mod_id).await {
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
