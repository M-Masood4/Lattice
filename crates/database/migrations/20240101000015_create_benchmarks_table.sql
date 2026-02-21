-- Price benchmarks
CREATE TABLE benchmarks (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  asset VARCHAR(50) NOT NULL,
  blockchain VARCHAR(50) NOT NULL,
  target_price DECIMAL(18, 8) NOT NULL CHECK (target_price > 0),
  trigger_type VARCHAR(20) NOT NULL, -- ABOVE or BELOW
  action_type VARCHAR(20) NOT NULL, -- ALERT or EXECUTE
  trade_action VARCHAR(10), -- BUY or SELL (if EXECUTE)
  trade_amount DECIMAL(36, 18),
  is_active BOOLEAN DEFAULT TRUE,
  triggered_at TIMESTAMP WITH TIME ZONE,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_benchmarks_user_active ON benchmarks(user_id, is_active);
CREATE INDEX idx_benchmarks_asset ON benchmarks(asset, is_active);
