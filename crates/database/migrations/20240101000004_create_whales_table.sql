-- Create whales table
CREATE TABLE whales (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    address VARCHAR(44) UNIQUE NOT NULL,
    total_value_usd DECIMAL(18, 2),
    first_detected TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_checked TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_whales_address ON whales(address);
CREATE INDEX idx_whales_last_checked ON whales(last_checked);
