-- Add user_tag column to users table for privacy features
ALTER TABLE users ADD COLUMN user_tag VARCHAR(50) UNIQUE;
ALTER TABLE users ADD COLUMN show_email_publicly BOOLEAN DEFAULT FALSE;

-- Create index for user tag lookups
CREATE INDEX idx_users_user_tag ON users(user_tag);
