-- Identity verifications
CREATE TABLE identity_verifications (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  verification_level INTEGER NOT NULL CHECK (verification_level IN (0, 1, 2)),
  status VARCHAR(20) NOT NULL,
  provider VARCHAR(50),
  provider_request_id VARCHAR(255),
  submitted_at TIMESTAMP DEFAULT NOW(),
  processed_at TIMESTAMP,
  rejection_reason TEXT
);

CREATE INDEX idx_identity_verifications_user ON identity_verifications(user_id);
CREATE INDEX idx_identity_verifications_status ON identity_verifications(status);
