-- Agentic trimming configuration
CREATE TABLE trim_configs (
  user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  enabled BOOLEAN DEFAULT FALSE,
  minimum_profit_percent DECIMAL(5, 2) DEFAULT 20.0,
  trim_percent DECIMAL(5, 2) DEFAULT 25.0,
  max_trims_per_day INTEGER DEFAULT 3,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
