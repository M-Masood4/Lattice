-- Trim executions
CREATE TABLE trim_executions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  position_id UUID NOT NULL,
  asset VARCHAR(50) NOT NULL,
  amount_sold DECIMAL(36, 18) NOT NULL,
  price_usd DECIMAL(18, 8) NOT NULL,
  profit_realized DECIMAL(18, 2) NOT NULL,
  confidence INTEGER NOT NULL,
  reasoning TEXT NOT NULL,
  transaction_hash VARCHAR(255),
  executed_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_trim_executions_user ON trim_executions(user_id);
CREATE INDEX idx_trim_executions_executed ON trim_executions(executed_at);
