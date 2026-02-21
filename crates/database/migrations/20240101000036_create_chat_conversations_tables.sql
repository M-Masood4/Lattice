-- Create chat conversations and participants tables
-- This migration adds support for group conversations and linking conversations to P2P offers

-- Chat conversations table
CREATE TABLE IF NOT EXISTS chat_conversations (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  offer_id UUID REFERENCES p2p_offers(id),
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- Chat participants table (many-to-many relationship)
CREATE TABLE IF NOT EXISTS chat_participants (
  conversation_id UUID REFERENCES chat_conversations(id) ON DELETE CASCADE,
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  joined_at TIMESTAMP DEFAULT NOW(),
  PRIMARY KEY (conversation_id, user_id)
);

-- Add indexes for efficient querying
CREATE INDEX idx_chat_conversations_offer ON chat_conversations(offer_id);
CREATE INDEX idx_chat_participants_user ON chat_participants(user_id);
CREATE INDEX idx_chat_participants_conversation ON chat_participants(conversation_id);

-- Add comments for documentation
COMMENT ON TABLE chat_conversations IS 'Chat conversations between users, optionally linked to P2P offers';
COMMENT ON TABLE chat_participants IS 'Users participating in chat conversations';
COMMENT ON COLUMN chat_conversations.offer_id IS 'Optional reference to P2P offer that initiated this conversation';
