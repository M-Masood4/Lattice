-- Pending trim recommendations
CREATE TABLE pending_trims (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  wallet_id UUID REFERENCES wallets(id) ON DELETE CASCADE,
  token_mint VARCHAR(255) NOT NULL,
  token_symbol VARCHAR(50) NOT NULL,
  amount VARCHAR(255) NOT NULL,
  confidence INTEGER NOT NULL,
  reasoning TEXT NOT NULL,
  suggested_trim_percent DECIMAL(5, 2) NOT NULL,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW(),
  UNIQUE(user_id, token_mint)
);

CREATE INDEX idx_pending_trims_user ON pending_trims(user_id);
CREATE INDEX idx_pending_trims_created ON pending_trims(created_at);
