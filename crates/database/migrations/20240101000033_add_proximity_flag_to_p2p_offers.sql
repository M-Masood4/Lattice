-- Add is_proximity_offer flag to p2p_offers table
ALTER TABLE p2p_offers ADD COLUMN IF NOT EXISTS is_proximity_offer BOOLEAN NOT NULL DEFAULT FALSE;

-- Create index for filtering proximity offers
CREATE INDEX IF NOT EXISTS idx_p2p_offers_proximity ON p2p_offers(is_proximity_offer, status, expires_at);
