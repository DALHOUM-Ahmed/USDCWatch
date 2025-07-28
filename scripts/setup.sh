#!/bin/bash

set -e

echo "Setting up Ethereum ERC-20 Indexer..."

if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo not found. Please install Rust first."
    echo "Visit: https://rustup.rs/"
    exit 1
fi

if ! command -v sqlite3 &> /dev/null; then
    echo "Warning: SQLite3 CLI not found. Install for database inspection."
fi

echo "Building project..."
cargo build --release

echo "Creating environment file..."
if [ ! -f .env ]; then
    cp .env.example .env
    echo "Created .env file. Please update with your Ethereum RPC URL."
else
    echo ".env file already exists."
fi

echo "Setup complete!"
echo ""
echo "Next steps:"
echo "1. Edit .env file with your Ethereum RPC URL"
echo "2. Run: cargo run -- index --start-block <block_number>"
echo "3. Query data: cargo run -- query --limit 10"