-- Create discovery_sessions table for tracking active discovery sessions
CREATE TABLE IF NOT EXISTS discovery_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    discovery_method VARCHAR(20) NOT NULL,
    started_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP NOT NULL,
    ended_at TIMESTAMP,
    auto_extend BOOLEAN DEFAULT FALSE
);

-- Create index for finding active sessions by user
CREATE INDEX idx_discovery_sessions_user_active ON discovery_sessions(user_id, ended_at) WHERE ended_at IS NULL;
