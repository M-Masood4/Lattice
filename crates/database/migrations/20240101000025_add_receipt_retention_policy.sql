-- Add receipt retention policy
-- Receipts must be retained for at least 7 years for tax compliance

-- Add a check constraint to prevent deletion of receipts less than 7 years old
-- This is implemented via a trigger since PostgreSQL doesn't support check constraints on DELETE

-- Add archived flag to track archived receipts
ALTER TABLE blockchain_receipts ADD COLUMN IF NOT EXISTS archived BOOLEAN DEFAULT FALSE;
ALTER TABLE blockchain_receipts ADD COLUMN IF NOT EXISTS archived_at TIMESTAMP;

-- Create index on created_at for efficient archival queries
CREATE INDEX IF NOT EXISTS idx_blockchain_receipts_created_archived 
ON blockchain_receipts(created_at, archived);

-- Create a function to prevent deletion of receipts less than 7 years old
CREATE OR REPLACE FUNCTION prevent_recent_receipt_deletion()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.created_at > NOW() - INTERVAL '7 years' THEN
        RAISE EXCEPTION 'Cannot delete receipt created less than 7 years ago (created: %, retention until: %)', 
            OLD.created_at, 
            OLD.created_at + INTERVAL '7 years';
    END IF;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to enforce retention policy
DROP TRIGGER IF EXISTS enforce_receipt_retention ON blockchain_receipts;
CREATE TRIGGER enforce_receipt_retention
    BEFORE DELETE ON blockchain_receipts
    FOR EACH ROW
    EXECUTE FUNCTION prevent_recent_receipt_deletion();

-- Add comment explaining the retention policy
COMMENT ON COLUMN blockchain_receipts.archived IS 'Flag indicating receipt has been archived for long-term storage';
COMMENT ON COLUMN blockchain_receipts.archived_at IS 'Timestamp when receipt was archived';
COMMENT ON TRIGGER enforce_receipt_retention ON blockchain_receipts IS 'Prevents deletion of receipts less than 7 years old for tax compliance';
