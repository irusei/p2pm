use crate::cli::SearchArgs;
use crate::error::Result;
use crate::repos::aggregator;
use colored::Colorize;

pub async fn run(args: SearchArgs) -> Result<()> {
    println!(
        "{} {}",
        "[p2pm]".cyan().bold(),
        format!("Searching for '{}'...", args.query).bold()
    );

    let results = aggregator::search_mods(&args.query, args.limit).await?;

    if results.is_empty() {
        println!(
            "{} {}",
            "[p2pm]".cyan().bold(),
            "No results found.".dimmed()
        );
        return Ok(());
    }

    for result in &results {
        let pkg_ref = format!("{} ({})", result.id, result.name);

        // Trim
        let desc: String = result.desc.chars().filter(|c| *c != '\n').collect();
        let desc = if desc.len() > 100 {
            format!("{}...", &desc[..97])
        } else {
            desc
        };

        println!("{}  {} [{}]", "*".cyan(), pkg_ref.bold(), "Search".green());
        println!("{}      Description:   {}", " ".repeat(14), desc.dimmed());
        println!();
    }

    Ok(())
}
