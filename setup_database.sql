-- Create transfer_events table
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
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_block_number ON transfer_events(block_number);
CREATE INDEX IF NOT EXISTS idx_from_address ON transfer_events(from_address);
CREATE INDEX IF NOT EXISTS idx_to_address ON transfer_events(to_address);
CREATE INDEX IF NOT EXISTS idx_timestamp ON transfer_events(timestamp);

-- Create processed_blocks table
CREATE TABLE IF NOT EXISTS processed_blocks (
    block_number INTEGER PRIMARY KEY,
    block_hash TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    processed_at TEXT NOT NULL
); 