-- Create user_whale_tracking table
CREATE TABLE user_whale_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    whale_id UUID NOT NULL REFERENCES whales(id) ON DELETE CASCADE,
    token_mint VARCHAR(44) NOT NULL,
    multiplier DECIMAL(10, 2),
    rank INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, whale_id, token_mint)
);

CREATE INDEX idx_user_whale_tracking_user ON user_whale_tracking(user_id);
CREATE INDEX idx_user_whale_tracking_whale ON user_whale_tracking(whale_id);
