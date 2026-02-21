-- Staking positions
CREATE TABLE staking_positions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  asset VARCHAR(50) NOT NULL,
  amount DECIMAL(36, 18) NOT NULL,
  provider VARCHAR(50) NOT NULL,
  apy DECIMAL(5, 2),
  rewards_earned DECIMAL(36, 18) DEFAULT 0,
  auto_compound BOOLEAN DEFAULT FALSE,
  started_at TIMESTAMP DEFAULT NOW(),
  last_reward_at TIMESTAMP
);

CREATE INDEX idx_staking_positions_user ON staking_positions(user_id);
CREATE INDEX idx_staking_positions_asset ON staking_positions(asset);
