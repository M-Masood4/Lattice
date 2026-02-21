-- Add user_tag column to users table for privacy
-- Requirements: 17.1, 20.1, 20.2

ALTER TABLE users ADD COLUMN user_tag VARCHAR(50) UNIQUE;
ALTER TABLE users ADD COLUMN show_email_publicly BOOLEAN DEFAULT FALSE;

-- Create index for user_tag lookups
CREATE INDEX idx_users_user_tag ON users(user_tag);
