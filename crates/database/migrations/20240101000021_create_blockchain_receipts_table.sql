-- Blockchain receipts
CREATE TABLE blockchain_receipts (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  payment_id UUID,
  trade_id UUID,
  conversion_id UUID REFERENCES conversions(id),
  amount DECIMAL(18, 2) NOT NULL,
  currency VARCHAR(100) NOT NULL,
  sender VARCHAR(255) NOT NULL,
  recipient VARCHAR(255) NOT NULL,
  blockchain VARCHAR(50) NOT NULL,
  transaction_hash VARCHAR(255) NOT NULL,
  verification_status VARCHAR(20) NOT NULL,
  created_at TIMESTAMP DEFAULT NOW(),
  verified_at TIMESTAMP
);

CREATE INDEX idx_blockchain_receipts_payment ON blockchain_receipts(payment_id);
CREATE INDEX idx_blockchain_receipts_trade ON blockchain_receipts(trade_id);
CREATE INDEX idx_blockchain_receipts_conversion ON blockchain_receipts(conversion_id);
CREATE INDEX idx_blockchain_receipts_created ON blockchain_receipts(created_at);
