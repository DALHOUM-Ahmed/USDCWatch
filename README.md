# Ethereum ERC-20 Transfer Event Indexer

Indexes USDC Transfer events from Ethereum mainnet into SQLite for analysis and querying.

## Overview

Monitors USDC transfers in real-time, handles chain reorgs, prevents duplicates, and provides CLI querying.

## Features

- Real-time block scanning
- USDC Transfer events (0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48)
- Handles chain reorgs automatically
- Prevents duplicates
- Batch processing for performance
- CLI querying with filters

## Setup

Requires Rust 1.70+ and SQLite dev libraries:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install SQLite (Linux only)
sudo apt install libsqlite3-dev  # Ubuntu/Debian
```

Set your RPC endpoint in `.env`:

```bash
ETHEREUM_RPC_URL=https://ethereum.publicnode.com
```

```bash
git clone <repository>
cd rustERC20Indexer
cp .env.example .env
# Edit .env with your RPC URL
cargo run -- index --latest
```

## Commands

### Indexing

```bash
cargo run -- index --latest              # Start from latest block
cargo run -- index --start-block 18500000  # Start from specific block
cargo run -- index                       # Resume from last processed
```

### Querying

```bash
cargo run -- query                       # Recent transfers
cargo run -- query --limit 1000          # More results
cargo run -- query --address 0x742d35... # Specific address
cargo run -- query --from-block 18500000 # Block range
cargo run -- stats                       # Database stats
```

## Database Analysis

```bash
sqlite3 transfers.db
```

### Useful Queries

```sql
-- Top senders by volume
SELECT from_address, COUNT(*), SUM(CAST(value AS REAL)) as total_volume
FROM transfer_events GROUP BY from_address ORDER BY total_volume DESC LIMIT 10;

-- Daily volume
SELECT DATE(timestamp) as date, COUNT(*), SUM(CAST(value AS REAL)) as daily_volume
FROM transfer_events GROUP BY DATE(timestamp) ORDER BY date DESC;

-- Large transfers (>1M USDC)
SELECT transaction_hash, from_address, to_address, CAST(value AS REAL)/1000000 as usdc_amount
FROM transfer_events WHERE CAST(value AS REAL) > 1000000000000 ORDER BY CAST(value AS REAL) DESC;

-- Most active addresses
SELECT address, SUM(transfer_count) as total_transfers FROM (
    SELECT from_address as address, COUNT(*) as transfer_count FROM transfer_events GROUP BY from_address
    UNION ALL
    SELECT to_address as address, COUNT(*) as transfer_count FROM transfer_events GROUP BY to_address
) GROUP BY address ORDER BY total_transfers DESC LIMIT 20;
```

### Export to CSV

```bash
sqlite3 -header -csv transfers.db "SELECT * FROM transfer_events;" > transfers.csv
```

### Clean Database and Start Fresh

```bash
rm transfers.db && sqlite3 transfers.db < setup_database.sql  # Recreates database
```

### Debug

```bash
RUST_LOG=debug cargo run -- index --latest
```

## Output Format

Transfer events as JSON:

```json
{
  "transaction_hash": "0x1234567890abcdef...",
  "from_address": "0x742d35...",
  "to_address": "0xA0b869...",
  "value": "1000000",
  "block_number": 18500000,
  "timestamp": "2023-11-01T12:00:00Z"
}
```

## Database Schema

Two tables: `transfer_events` (the main data) and `processed_blocks` (tracks progress/reorgs).
Prevents duplicates via `(transaction_hash, log_index)` constraint.

## Environment Variables

Configure in `.env`:

- `ETHEREUM_RPC_URL` - Your RPC endpoint
- `DATABASE_URL` - SQLite path (default: `./transfers.db`)
- `BLOCKS_PER_REQUEST` - Batch size (default: 100)
- `FINALITY_BLOCKS` - Confirmation depth (default: 12)
