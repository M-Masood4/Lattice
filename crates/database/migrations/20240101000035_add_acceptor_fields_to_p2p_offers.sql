-- Add acceptor and conversation fields to p2p_offers table
-- This migration adds support for tracking offer acceptance and linking to chat conversations

ALTER TABLE p2p_offers
ADD COLUMN acceptor_id UUID REFERENCES users(id),
ADD COLUMN accepted_at TIMESTAMP,
ADD COLUMN conversation_id UUID;

-- Add indexes for efficient querying
CREATE INDEX idx_p2p_offers_acceptor ON p2p_offers(acceptor_id);
CREATE INDEX idx_p2p_offers_conversation ON p2p_offers(conversation_id);

-- Add comment for documentation
COMMENT ON COLUMN p2p_offers.acceptor_id IS 'User who accepted this offer';
COMMENT ON COLUMN p2p_offers.accepted_at IS 'Timestamp when offer was accepted';
COMMENT ON COLUMN p2p_offers.conversation_id IS 'Chat conversation ID for this offer';
