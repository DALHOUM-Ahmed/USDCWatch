mod cli;
mod config;
mod database;
mod ethereum;
mod indexer;
mod models;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use config::Config;
use database::Database;
use indexer::Indexer;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    let config = Config::load()?;
    let database = Database::new(&config.database_url).await?;
    
    match cli.command {
        Commands::Index { start_block, latest } => {
            let indexer = Indexer::new(config, database).await?;
            
            let start_block = if latest {
                if start_block.is_some() {
                    eprintln!("Error: Cannot specify both --start-block and --latest");
                    std::process::exit(1);
                }
                let latest_block = indexer.get_ethereum_client().get_latest_block_number().await?;
                let finalized_block = latest_block.saturating_sub(indexer.get_config().finality_blocks);
                println!("Starting from network latest block {} (latest {} minus {} finality blocks)", 
                        finalized_block, latest_block, indexer.get_config().finality_blocks);
                Some(finalized_block)
            } else {
                start_block
            };
            
            indexer.start_indexing(start_block).await?;
        }
        Commands::Query { 
            address, 
            from_block, 
            to_block, 
            limit 
        } => {
            let transfers = database.query_transfers(
                address.as_deref(),
                from_block,
                to_block,
                limit.unwrap_or(100)
            ).await?;
            
            for transfer in transfers {
                println!("{}", serde_json::to_string_pretty(&transfer)?);
            }
        }
        Commands::Stats => {
            let stats = database.get_stats().await?;
            println!("Database Statistics:");
            println!("Total transfers: {}", stats.total_transfers);
            println!("Unique addresses: {}", stats.unique_addresses);
            println!("Latest block: {}", stats.latest_block.unwrap_or(0));
            println!("Earliest block: {}", stats.earliest_block.unwrap_or(0));
        }
    }
    
    Ok(())
}