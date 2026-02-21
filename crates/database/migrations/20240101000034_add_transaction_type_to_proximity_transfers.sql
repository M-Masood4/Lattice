-- Add transaction_type field to proximity_transfers table
ALTER TABLE proximity_transfers ADD COLUMN IF NOT EXISTS transaction_type VARCHAR(20) NOT NULL DEFAULT 'DIRECT_TRANSFER';

-- Create index for filtering by transaction type
CREATE INDEX IF NOT EXISTS idx_proximity_transfers_type ON proximity_transfers(transaction_type, created_at DESC);

-- Add comment explaining the field
COMMENT ON COLUMN proximity_transfers.transaction_type IS 'Type of transaction: DIRECT_TRANSFER or P2P_EXCHANGE';
