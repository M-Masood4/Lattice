-- Create position_modes table for manual/automatic position management
CREATE TABLE position_modes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    asset VARCHAR(50) NOT NULL,
    blockchain VARCHAR(50) NOT NULL DEFAULT 'Solana',
    mode VARCHAR(20) NOT NULL DEFAULT 'manual', -- 'manual' or 'automatic'
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, asset, blockchain)
);

CREATE INDEX idx_position_modes_user ON position_modes(user_id);
CREATE INDEX idx_position_modes_user_asset ON position_modes(user_id, asset);

-- Create manual_orders table for tracking manual trades
CREATE TABLE manual_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    asset VARCHAR(50) NOT NULL,
    blockchain VARCHAR(50) NOT NULL DEFAULT 'Solana',
    action VARCHAR(10) NOT NULL, -- 'BUY' or 'SELL'
    amount DECIMAL(36, 18) NOT NULL,
    price_usd DECIMAL(18, 8),
    total_value_usd DECIMAL(18, 2),
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'executed', 'failed', 'cancelled'
    transaction_hash VARCHAR(255),
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    executed_at TIMESTAMP WITH TIME ZONE,
    cancelled_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_manual_orders_user ON manual_orders(user_id);
CREATE INDEX idx_manual_orders_status ON manual_orders(status);
CREATE INDEX idx_manual_orders_created ON manual_orders(created_at DESC);

-- Create pending_automatic_orders table for tracking automatic orders that need to be cancelled on mode switch
CREATE TABLE pending_automatic_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    asset VARCHAR(50) NOT NULL,
    blockchain VARCHAR(50) NOT NULL DEFAULT 'Solana',
    order_type VARCHAR(50) NOT NULL, -- 'benchmark', 'trim', 'ai_recommendation'
    order_reference_id UUID, -- ID of the benchmark, trim, or recommendation
    action VARCHAR(10) NOT NULL, -- 'BUY' or 'SELL'
    amount DECIMAL(36, 18) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'executed', 'cancelled'
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    executed_at TIMESTAMP WITH TIME ZONE,
    cancelled_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_pending_automatic_orders_user ON pending_automatic_orders(user_id);
CREATE INDEX idx_pending_automatic_orders_asset ON pending_automatic_orders(user_id, asset);
CREATE INDEX idx_pending_automatic_orders_status ON pending_automatic_orders(status);
