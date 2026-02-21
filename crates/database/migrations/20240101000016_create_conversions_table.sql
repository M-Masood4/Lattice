-- SideShift conversions
CREATE TABLE conversions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  from_asset VARCHAR(50) NOT NULL,
  to_asset VARCHAR(50) NOT NULL,
  from_amount DECIMAL(36, 18) NOT NULL,
  to_amount DECIMAL(36, 18) NOT NULL,
  exchange_rate DECIMAL(18, 8) NOT NULL,
  network_fee DECIMAL(18, 8),
  platform_fee DECIMAL(18, 8),
  provider_fee DECIMAL(18, 8),
  provider VARCHAR(50) NOT NULL, -- SIDESHIFT or JUPITER
  transaction_hash VARCHAR(255),
  status VARCHAR(20) NOT NULL,
  created_at TIMESTAMP DEFAULT NOW(),
  completed_at TIMESTAMP
);

CREATE INDEX idx_conversions_user ON conversions(user_id);
CREATE INDEX idx_conversions_status ON conversions(status);
CREATE INDEX idx_conversions_created ON conversions(created_at);
