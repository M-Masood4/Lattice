-- Create mesh_price_cache table for storing price data from the mesh network
CREATE TABLE IF NOT EXISTS mesh_price_cache (
    asset VARCHAR(255) PRIMARY KEY,
    price VARCHAR(255) NOT NULL,
    blockchain VARCHAR(50) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    source_node_id UUID NOT NULL,
    change_24h VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_mesh_price_cache_timestamp ON mesh_price_cache(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_mesh_price_cache_blockchain ON mesh_price_cache(blockchain);
