-- Wallet verifications
CREATE TABLE wallet_verifications (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  wallet_address VARCHAR(255) NOT NULL,
  blockchain VARCHAR(50) NOT NULL,
  challenge_message TEXT NOT NULL,
  signature TEXT NOT NULL,
  verified BOOLEAN DEFAULT TRUE,
  verified_at TIMESTAMP DEFAULT NOW(),
  UNIQUE(user_id, wallet_address, blockchain)
);

CREATE INDEX idx_wallet_verifications_user ON wallet_verifications(user_id);
CREATE INDEX idx_wallet_verifications_wallet ON wallet_verifications(wallet_address, blockchain);
