use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ethereum-erc20-indexer")]
#[command(about = "A service to index ERC-20 Transfer events from Ethereum")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Index {
        #[arg(short, long)]
        start_block: Option<u64>,
        #[arg(long)]
        latest: bool,
    },
    Query {
        #[arg(short, long)]
        address: Option<String>,
        #[arg(long)]
        from_block: Option<u64>,
        #[arg(long)]
        to_block: Option<u64>,
        #[arg(short, long)]
        limit: Option<i64>,
    },
    Stats,
}