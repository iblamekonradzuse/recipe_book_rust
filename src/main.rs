mod htmlscraper;
mod database;
mod cli;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run_cli()
}
