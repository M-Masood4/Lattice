-- Create portfolio_snapshots table for tracking portfolio value over time
-- This enables historical performance analysis and gain/loss calculations

CREATE TABLE portfolio_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    total_value_usd DECIMAL(18, 2) NOT NULL,
    snapshot_time TIMESTAMP NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP DEFAULT NOW(),
    
    -- Index for efficient time-series queries
    CONSTRAINT unique_wallet_snapshot UNIQUE (wallet_id, snapshot_time)
);

-- Create index for efficient queries by wallet and time range
CREATE INDEX idx_portfolio_snapshots_wallet_time ON portfolio_snapshots(wallet_id, snapshot_time DESC);

-- Create portfolio_asset_snapshots table for tracking individual asset performance
CREATE TABLE portfolio_asset_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    snapshot_id UUID NOT NULL REFERENCES portfolio_snapshots(id) ON DELETE CASCADE,
    token_mint VARCHAR(44) NOT NULL,
    token_symbol VARCHAR(20) NOT NULL,
    amount DECIMAL(36, 18) NOT NULL,
    value_usd DECIMAL(18, 2),
    created_at TIMESTAMP DEFAULT NOW()
);

-- Create index for efficient queries by snapshot
CREATE INDEX idx_portfolio_asset_snapshots_snapshot ON portfolio_asset_snapshots(snapshot_id);
CREATE INDEX idx_portfolio_asset_snapshots_token ON portfolio_asset_snapshots(snapshot_id, token_mint);
