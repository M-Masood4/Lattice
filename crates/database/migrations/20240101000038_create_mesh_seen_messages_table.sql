-- Create mesh_seen_messages table for tracking processed messages
CREATE TABLE IF NOT EXISTS mesh_seen_messages (
    message_id UUID PRIMARY KEY,
    seen_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);

-- Create index on expires_at for efficient cleanup queries
CREATE INDEX IF NOT EXISTS idx_mesh_seen_messages_expires ON mesh_seen_messages(expires_at);
