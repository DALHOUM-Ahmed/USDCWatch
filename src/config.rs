use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub ethereum_rpc_url: String,
    pub database_url: String,
    pub usdc_contract_address: String,
    pub blocks_per_request: u64,
    pub finality_blocks: u64,
}

impl Config {
    pub fn load() -> Result<Self> {
        let _ = dotenvy::dotenv();
        let config = Config {
            ethereum_rpc_url: std::env::var("ETHEREUM_RPC_URL")
                .unwrap_or_else(|_| "https://ethereum.publicnode.com".to_string()),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./transfers.db".to_string()),
            usdc_contract_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            blocks_per_request: std::env::var("BLOCKS_PER_REQUEST")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            finality_blocks: std::env::var("FINALITY_BLOCKS")
                .unwrap_or_else(|_| "12".to_string())
                .parse()
                .unwrap_or(12),
        };
        
        Ok(config)
    }
}