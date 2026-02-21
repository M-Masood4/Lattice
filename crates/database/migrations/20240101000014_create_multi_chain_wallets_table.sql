-- Multi-chain wallet support
CREATE TABLE multi_chain_wallets (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  blockchain VARCHAR(50) NOT NULL,
  address VARCHAR(255) NOT NULL,
  is_primary BOOLEAN DEFAULT FALSE,
  is_temporary BOOLEAN DEFAULT FALSE,
  temp_tag VARCHAR(50),
  expires_at TIMESTAMP,
  is_frozen BOOLEAN DEFAULT FALSE,
  frozen_at TIMESTAMP,
  created_at TIMESTAMP DEFAULT NOW(),
  UNIQUE(blockchain, address)
);

CREATE INDEX idx_multi_chain_wallets_user ON multi_chain_wallets(user_id);
CREATE INDEX idx_multi_chain_wallets_blockchain ON multi_chain_wallets(blockchain);
CREATE INDEX idx_multi_chain_wallets_frozen ON multi_chain_wallets(is_frozen);
