use anyhow::Result;
use chrono::{DateTime, Utc};
use ethers::prelude::*;
use ethers_core::types::{Filter, Log, H160, H256, U64};
use ethers_providers::{Http, Middleware, Provider};
use std::sync::Arc;

use crate::models::TransferEvent;

const TRANSFER_EVENT_SIGNATURE: &str = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

pub struct EthereumClient {
    provider: Arc<Provider<Http>>,
    usdc_address: H160,
}

impl EthereumClient {
    pub async fn new(rpc_url: &str, usdc_address: &str) -> Result<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let provider = Arc::new(provider);
        let usdc_address: H160 = usdc_address.parse()?;

        Ok(Self {
            provider,
            usdc_address,
        })
    }

    pub async fn get_latest_block_number(&self) -> Result<u64> {
        let block_number = self.provider.get_block_number().await?;
        Ok(block_number.as_u64())
    }

    pub async fn get_block_timestamp(&self, block_number: u64) -> Result<DateTime<Utc>> {
        let block = self
            .provider
            .get_block(BlockId::Number(BlockNumber::Number(U64::from(block_number))))
            .await?
            .ok_or_else(|| anyhow::anyhow!("Block not found"))?;

        let timestamp = DateTime::from_timestamp(block.timestamp.as_u64() as i64, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;

        Ok(timestamp)
    }

    pub async fn get_transfer_events(
        &self,
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<TransferEvent>> {
        let filter = Filter::new()
            .address(self.usdc_address)
            .topic0(H256::from_slice(&hex::decode(&TRANSFER_EVENT_SIGNATURE[2..])?))
            .from_block(BlockNumber::Number(U64::from(from_block)))
            .to_block(BlockNumber::Number(U64::from(to_block)));

        let logs = self.provider.get_logs(&filter).await?;
        let mut events = Vec::new();

        for log in logs {
            if let Some(event) = self.parse_transfer_log(log).await? {
                events.push(event);
            }
        }

        Ok(events)
    }

    async fn parse_transfer_log(&self, log: Log) -> Result<Option<TransferEvent>> {
        if log.topics.len() != 3 {
            return Ok(None);
        }

        let from_address = format!("0x{:x}", H160::from(log.topics[1]));
        let to_address = format!("0x{:x}", H160::from(log.topics[2]));
        let value = U256::from_big_endian(&log.data).to_string();

        let block_number = log
            .block_number
            .ok_or_else(|| anyhow::anyhow!("Missing block number"))?
            .as_u64();

        let block_hash = log
            .block_hash
            .ok_or_else(|| anyhow::anyhow!("Missing block hash"))?;

        let log_index = log
            .log_index
            .ok_or_else(|| anyhow::anyhow!("Missing log index"))?
            .as_u64();

        let transaction_hash = log
            .transaction_hash
            .ok_or_else(|| anyhow::anyhow!("Missing transaction hash"))?;

        let timestamp = self.get_block_timestamp(block_number).await?;
        let id = format!("{}_{}", transaction_hash, log_index);

        Ok(Some(TransferEvent {
            id,
            transaction_hash: format!("0x{:x}", transaction_hash),
            log_index: log_index as i64,
            block_number: block_number as i64,
            block_hash: format!("0x{:x}", block_hash),
            from_address,
            to_address,
            value,
            timestamp,
            created_at: Utc::now(),
        }))
    }

    pub async fn get_block_hash(&self, block_number: u64) -> Result<String> {
        let block = self
            .provider
            .get_block(BlockId::Number(BlockNumber::Number(U64::from(block_number))))
            .await?
            .ok_or_else(|| anyhow::anyhow!("Block not found"))?;

        Ok(format!("0x{:x}", block.hash.unwrap_or_default()))
    }

}