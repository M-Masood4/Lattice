-- Create notification_preferences table
CREATE TABLE notification_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    in_app_enabled BOOLEAN DEFAULT TRUE,
    email_enabled BOOLEAN DEFAULT FALSE,
    push_enabled BOOLEAN DEFAULT FALSE,
    frequency VARCHAR(20) DEFAULT 'REALTIME',
    minimum_movement_percent DECIMAL(5, 2) DEFAULT 5.0,
    minimum_confidence INTEGER DEFAULT 70 CHECK (minimum_confidence >= 0 AND minimum_confidence <= 100)
);
