-- Create peer_blocklist table for users to block specific peers
CREATE TABLE IF NOT EXISTS peer_blocklist (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    blocked_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    blocked_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, blocked_user_id)
);

-- Create index for checking if a user is blocked
CREATE INDEX idx_peer_blocklist_user ON peer_blocklist(user_id);
