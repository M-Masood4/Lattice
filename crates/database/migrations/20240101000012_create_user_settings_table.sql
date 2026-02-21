-- Create user_settings table
CREATE TABLE user_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    auto_trader_enabled BOOLEAN DEFAULT FALSE,
    max_trade_percentage DECIMAL(5, 2) DEFAULT 5.0 CHECK (max_trade_percentage > 0 AND max_trade_percentage <= 100),
    max_daily_trades INTEGER DEFAULT 10 CHECK (max_daily_trades > 0),
    stop_loss_percentage DECIMAL(5, 2) DEFAULT 10.0 CHECK (stop_loss_percentage > 0 AND stop_loss_percentage <= 100),
    risk_tolerance VARCHAR(10) DEFAULT 'MEDIUM',
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
