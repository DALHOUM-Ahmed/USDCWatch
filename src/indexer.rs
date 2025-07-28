use anyhow::Result;
use log::{error, info, warn};
use std::time::Duration;
use tokio::time::sleep;

use crate::{
    config::Config,
    database::Database,
    ethereum::EthereumClient,
};

pub struct Indexer {
    ethereum_client: EthereumClient,
    database: Database,
    config: Config,
}

impl Indexer {
    pub async fn new(config: Config, database: Database) -> Result<Self> {
        let ethereum_client = EthereumClient::new(
            &config.ethereum_rpc_url,
            &config.usdc_contract_address,
        ).await?;

        Ok(Self {
            ethereum_client,
            database,
            config,
        })
    }

    pub fn get_ethereum_client(&self) -> &EthereumClient {
        &self.ethereum_client
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub async fn start_indexing(&self, start_block: Option<u64>) -> Result<()> {
        let mut current_block = match start_block {
            Some(block) => block,
            None => {
                match self.database.get_latest_processed_block().await? {
                    Some(latest) => latest + 1,
                    None => {
                        let latest = self.ethereum_client.get_latest_block_number().await?;
                        latest.saturating_sub(1000)
                    }
                }
            }
        };

        info!("Starting indexer from block {}", current_block);

        loop {
            match self.process_blocks(current_block).await {
                Ok(processed_count) => {
                    if processed_count > 0 {
                        current_block += processed_count;
                        info!("Processed {} blocks, current block: {}", processed_count, current_block);
                    } else {
                        sleep(Duration::from_secs(12)).await;
                    }
                }
                Err(e) => {
                    error!("Error processing blocks: {}", e);
                    sleep(Duration::from_secs(30)).await;
                }
            }
        }
    }

    async fn process_blocks(&self, start_block: u64) -> Result<u64> {
        let latest_block = self.ethereum_client.get_latest_block_number().await?;
        let finalized_block = latest_block.saturating_sub(self.config.finality_blocks);
        
        if start_block > finalized_block {
            return Ok(0);
        }

        let end_block = std::cmp::min(
            start_block + self.config.blocks_per_request - 1,
            finalized_block,
        );

        if let Err(e) = self.check_for_reorg(start_block).await {
            warn!("Reorg check failed: {}", e);
        }

        info!("Processing blocks {} to {}", start_block, end_block);

        let events = self
            .ethereum_client
            .get_transfer_events(start_block, end_block)
            .await?;

        info!("Found {} transfer events", events.len());

        for event in events {
            if let Err(e) = self.database.insert_transfer_event(&event).await {
                error!("Failed to insert transfer event: {}", e);
            }
        }

        for block_num in start_block..=end_block {
            let block_hash = self.ethereum_client.get_block_hash(block_num).await?;
            let timestamp = self.ethereum_client.get_block_timestamp(block_num).await?;
            
            if let Err(e) = self.database.insert_processed_block(block_num, &block_hash, timestamp).await {
                error!("Failed to insert processed block: {}", e);
            }
        }

        Ok(end_block - start_block + 1)
    }

    async fn check_for_reorg(&self, current_block: u64) -> Result<()> {
        if current_block == 0 {
            return Ok(());
        }

        let check_blocks = std::cmp::min(10, current_block);
        let start_check = current_block.saturating_sub(check_blocks);

        for block_num in start_check..current_block {
            if let Ok(actual_hash) = self.ethereum_client.get_block_hash(block_num).await {
                if let Ok(Some(stored_hash)) = self.get_stored_block_hash(block_num).await {
                    if actual_hash != stored_hash {
                        warn!("Reorg detected at block {}", block_num);
                        self.database.handle_reorg(block_num).await?;
                        return Err(anyhow::anyhow!("Reorg detected at block {}", block_num));
                    }
                }
            }
        }

        Ok(())
    }

    async fn get_stored_block_hash(&self, block_number: u64) -> Result<Option<String>> {
        self.database.get_block_hash(block_number).await
    }
}