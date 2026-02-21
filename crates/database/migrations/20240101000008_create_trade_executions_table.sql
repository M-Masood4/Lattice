-- Create trade_executions table
CREATE TABLE trade_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recommendation_id UUID REFERENCES recommendations(id),
    transaction_signature VARCHAR(88) UNIQUE NOT NULL,
    action VARCHAR(10) NOT NULL,
    token_mint VARCHAR(44) NOT NULL,
    amount DECIMAL(36, 18) NOT NULL,
    price_usd DECIMAL(18, 8),
    total_value_usd DECIMAL(18, 2),
    status VARCHAR(20) NOT NULL,
    executed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    confirmed_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_trade_executions_user ON trade_executions(user_id);
CREATE INDEX idx_trade_executions_recommendation ON trade_executions(recommendation_id);
CREATE INDEX idx_trade_executions_executed ON trade_executions(executed_at DESC);
