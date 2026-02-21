-- Add proximity_transfer_id column to blockchain_receipts table to link receipts with proximity transfers
ALTER TABLE blockchain_receipts 
ADD COLUMN IF NOT EXISTS proximity_transfer_id UUID REFERENCES proximity_transfers(id) ON DELETE SET NULL;

-- Create index for finding receipts by proximity transfer
CREATE INDEX IF NOT EXISTS idx_blockchain_receipts_proximity_transfer ON blockchain_receipts(proximity_transfer_id) WHERE proximity_transfer_id IS NOT NULL;
