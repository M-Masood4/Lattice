-- P2P exchanges
CREATE TABLE p2p_exchanges (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  buyer_offer_id UUID REFERENCES p2p_offers(id),
  seller_offer_id UUID REFERENCES p2p_offers(id),
  buyer_user_id UUID REFERENCES users(id),
  seller_user_id UUID REFERENCES users(id),
  asset VARCHAR(50) NOT NULL,
  amount DECIMAL(36, 18) NOT NULL,
  price DECIMAL(18, 8) NOT NULL,
  platform_fee DECIMAL(18, 8) NOT NULL,
  transaction_hash VARCHAR(255),
  status VARCHAR(20) NOT NULL,
  executed_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_p2p_exchanges_users ON p2p_exchanges(buyer_user_id, seller_user_id);
CREATE INDEX idx_p2p_exchanges_offers ON p2p_exchanges(buyer_offer_id, seller_offer_id);
CREATE INDEX idx_p2p_exchanges_executed ON p2p_exchanges(executed_at);
