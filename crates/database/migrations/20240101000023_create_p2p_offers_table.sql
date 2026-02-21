-- P2P offers
CREATE TABLE p2p_offers (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  offer_type VARCHAR(10) NOT NULL, -- BUY or SELL
  from_asset VARCHAR(50) NOT NULL,
  to_asset VARCHAR(50) NOT NULL,
  from_amount DECIMAL(36, 18) NOT NULL,
  to_amount DECIMAL(36, 18) NOT NULL,
  price DECIMAL(18, 8) NOT NULL,
  status VARCHAR(20) NOT NULL,
  escrow_tx_hash VARCHAR(255),
  matched_with_offer_id UUID,
  created_at TIMESTAMP DEFAULT NOW(),
  expires_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_p2p_offers_status ON p2p_offers(status, expires_at);
CREATE INDEX idx_p2p_offers_user ON p2p_offers(user_id);
CREATE INDEX idx_p2p_offers_assets ON p2p_offers(from_asset, to_asset, status);
