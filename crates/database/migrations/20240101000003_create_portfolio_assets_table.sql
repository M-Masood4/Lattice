-- Create portfolio_assets table
CREATE TABLE portfolio_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    token_mint VARCHAR(44) NOT NULL,
    token_symbol VARCHAR(20) NOT NULL,
    amount DECIMAL(36, 18) NOT NULL,
    value_usd DECIMAL(18, 2),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(wallet_id, token_mint)
);

CREATE INDEX idx_portfolio_assets_wallet ON portfolio_assets(wallet_id);
CREATE INDEX idx_portfolio_assets_token_mint ON portfolio_assets(token_mint);
