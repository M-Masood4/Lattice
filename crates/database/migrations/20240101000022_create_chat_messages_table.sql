-- Chat messages
CREATE TABLE chat_messages (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  from_user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  to_user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  content TEXT NOT NULL,
  encrypted BOOLEAN DEFAULT TRUE,
  blockchain_hash VARCHAR(255),
  verification_status VARCHAR(20),
  read BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_chat_messages_users ON chat_messages(from_user_id, to_user_id);
CREATE INDEX idx_chat_messages_to_user ON chat_messages(to_user_id, read);
CREATE INDEX idx_chat_messages_created ON chat_messages(created_at);
