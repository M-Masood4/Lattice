-- Add 2FA support to users table
-- Requirements: 17.5

ALTER TABLE users ADD COLUMN totp_secret VARCHAR(255);
ALTER TABLE users ADD COLUMN totp_enabled BOOLEAN DEFAULT FALSE;
ALTER TABLE users ADD COLUMN totp_verified_at TIMESTAMP;

-- Create index for 2FA lookups
CREATE INDEX idx_users_totp_enabled ON users(totp_enabled);
