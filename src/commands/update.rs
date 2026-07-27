use crate::cli::UpdateArgs;
use crate::error::Result;
use crate::repos::aggregator;

pub async fn run(_args: UpdateArgs) -> Result<()> {
    aggregator::update_all().await?;

    Ok(())
}
