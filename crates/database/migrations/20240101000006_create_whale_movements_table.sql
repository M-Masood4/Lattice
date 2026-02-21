-- Create whale_movements table
CREATE TABLE whale_movements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    whale_id UUID NOT NULL REFERENCES whales(id) ON DELETE CASCADE,
    transaction_signature VARCHAR(88) UNIQUE NOT NULL,
    movement_type VARCHAR(10) NOT NULL,
    token_mint VARCHAR(44) NOT NULL,
    amount DECIMAL(36, 18) NOT NULL,
    percent_of_position DECIMAL(5, 2),
    detected_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_whale_movements_whale ON whale_movements(whale_id);
CREATE INDEX idx_whale_movements_detected ON whale_movements(detected_at DESC);
CREATE INDEX idx_whale_movements_signature ON whale_movements(transaction_signature);
