-- Create proximity_transfers table for tracking proximity-based P2P transfers
CREATE TABLE IF NOT EXISTS proximity_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sender_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    sender_wallet VARCHAR(44) NOT NULL,
    recipient_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recipient_wallet VARCHAR(44) NOT NULL,
    asset VARCHAR(44) NOT NULL,
    amount DECIMAL(36, 18) NOT NULL,
    transaction_hash VARCHAR(88),
    status VARCHAR(20) NOT NULL,
    discovery_method VARCHAR(20) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    accepted_at TIMESTAMP,
    completed_at TIMESTAMP,
    failed_reason TEXT
);

-- Create indexes for performance
CREATE INDEX idx_proximity_transfers_sender_user ON proximity_transfers(sender_user_id);
CREATE INDEX idx_proximity_transfers_recipient_user ON proximity_transfers(recipient_user_id);
CREATE INDEX idx_proximity_transfers_status ON proximity_transfers(status);
CREATE INDEX idx_proximity_transfers_created_at ON proximity_transfers(created_at DESC);
