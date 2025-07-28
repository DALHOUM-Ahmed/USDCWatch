use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};

use crate::models::{DatabaseStats, TransferEvent};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        let db = Self { pool };
        db.create_tables().await?;
        Ok(db)
    }

    async fn create_tables(&self) -> Result<()> {
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS transfer_events (
                id TEXT PRIMARY KEY,
                transaction_hash TEXT NOT NULL,
                log_index INTEGER NOT NULL,
                block_number INTEGER NOT NULL,
                block_hash TEXT NOT NULL,
                from_address TEXT NOT NULL,
                to_address TEXT NOT NULL,
                value TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(transaction_hash, log_index)
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            r#"
            CREATE INDEX IF NOT EXISTS idx_block_number ON transfer_events(block_number);
            CREATE INDEX IF NOT EXISTS idx_from_address ON transfer_events(from_address);
            CREATE INDEX IF NOT EXISTS idx_to_address ON transfer_events(to_address);
            CREATE INDEX IF NOT EXISTS idx_timestamp ON transfer_events(timestamp);
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS processed_blocks (
                block_number INTEGER PRIMARY KEY,
                block_hash TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                processed_at TEXT NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_transfer_event(&self, event: &TransferEvent) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO transfer_events 
            (id, transaction_hash, log_index, block_number, block_hash, from_address, to_address, value, timestamp, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            event.id,
            event.transaction_hash,
            event.log_index,
            event.block_number,
            event.block_hash,
            event.from_address,
            event.to_address,
            event.value,
            event.timestamp,
            event.created_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_processed_block(&self, block_number: u64, block_hash: &str, timestamp: DateTime<Utc>) -> Result<()> {
        let block_num = block_number as i64;
        let processed_at = Utc::now();
        
        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO processed_blocks (block_number, block_hash, timestamp, processed_at)
            VALUES (?, ?, ?, ?)
            "#,
            block_num,
            block_hash,
            timestamp,
            processed_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_latest_processed_block(&self) -> Result<Option<u64>> {
        let row = sqlx::query!("SELECT MAX(block_number) as max_block FROM processed_blocks")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.max_block.map(|b| b as u64))
    }

    pub async fn query_transfers(
        &self,
        address: Option<&str>,
        from_block: Option<u64>,
        to_block: Option<u64>,
        limit: i64,
    ) -> Result<Vec<TransferEvent>> {
        let mut query = "SELECT * FROM transfer_events WHERE 1=1".to_string();
        let mut conditions = Vec::new();

        if let Some(addr) = address {
            conditions.push(format!("(from_address = '{}' OR to_address = '{}')", addr, addr));
        }

        if let Some(from) = from_block {
            conditions.push(format!("block_number >= {}", from));
        }

        if let Some(to) = to_block {
            conditions.push(format!("block_number <= {}", to));
        }

        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY block_number DESC, log_index ASC");
        query.push_str(&format!(" LIMIT {}", limit));

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        
        let mut transfers = Vec::new();
        for row in rows {
            transfers.push(TransferEvent {
                id: row.get("id"),
                transaction_hash: row.get("transaction_hash"),
                log_index: row.get("log_index"),
                block_number: row.get("block_number"),
                block_hash: row.get("block_hash"),
                from_address: row.get("from_address"),
                to_address: row.get("to_address"),
                value: row.get("value"),
                timestamp: row.get::<String, _>("timestamp").parse()?,
                created_at: row.get::<String, _>("created_at").parse()?,
            });
        }

        Ok(transfers)
    }

    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let total_transfers_row = sqlx::query!("SELECT COUNT(*) as count FROM transfer_events")
            .fetch_one(&self.pool)
            .await?;

        let unique_addresses_row = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT address) as count FROM (
                SELECT from_address as address FROM transfer_events
                UNION
                SELECT to_address as address FROM transfer_events
            )
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let block_stats_row = sqlx::query!(
            "SELECT MIN(block_number) as min_block, MAX(block_number) as max_block FROM transfer_events"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseStats {
            total_transfers: total_transfers_row.count as i64,
            unique_addresses: unique_addresses_row.count as i64,
            latest_block: block_stats_row.max_block.map(|b| b as i64),
            earliest_block: block_stats_row.min_block.map(|b| b as i64),
        })
    }

    pub async fn handle_reorg(&self, invalid_block: u64) -> Result<()> {
        let invalid_block_i64 = invalid_block as i64;
        
        sqlx::query!(
            "DELETE FROM transfer_events WHERE block_number >= ?",
            invalid_block_i64
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            "DELETE FROM processed_blocks WHERE block_number >= ?",
            invalid_block_i64
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_block_hash(&self, block_number: u64) -> Result<Option<String>> {
        let block_num = block_number as i64;
        
        let row = sqlx::query!(
            "SELECT block_hash FROM processed_blocks WHERE block_number = ?",
            block_num
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.block_hash))
    }
}