-- Staking approval requests table
CREATE TABLE staking_approval_requests (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  asset VARCHAR(50) NOT NULL,
  amount DECIMAL(36, 18) NOT NULL,
  provider VARCHAR(50) NOT NULL,
  apy DECIMAL(5, 2) NOT NULL,
  lock_period_days INTEGER NOT NULL,
  status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, approved, rejected, expired
  created_at TIMESTAMP DEFAULT NOW(),
  expires_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_staking_approval_requests_user ON staking_approval_requests(user_id);
CREATE INDEX idx_staking_approval_requests_status ON staking_approval_requests(status);

-- Auto-staking configuration table
CREATE TABLE auto_staking_configs (
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  asset VARCHAR(50) NOT NULL,
  enabled BOOLEAN DEFAULT FALSE,
  minimum_idle_amount DECIMAL(36, 18) NOT NULL DEFAULT 100,
  idle_duration_hours INTEGER NOT NULL DEFAULT 24,
  auto_compound BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW(),
  PRIMARY KEY (user_id, asset)
);

CREATE INDEX idx_auto_staking_configs_enabled ON auto_staking_configs(enabled);
