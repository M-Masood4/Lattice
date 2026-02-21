-- Voice commands log
CREATE TABLE voice_commands (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  transcribed_text TEXT NOT NULL,
  command_type VARCHAR(50) NOT NULL,
  parameters JSONB,
  executed BOOLEAN DEFAULT FALSE,
  result TEXT,
  created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_voice_commands_user ON voice_commands(user_id);
CREATE INDEX idx_voice_commands_created ON voice_commands(created_at);
