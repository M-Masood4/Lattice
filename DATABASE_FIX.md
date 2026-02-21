# Database Fix - P2P Exchange Enhancements

## Issue
When trying to create a P2P offer, users were getting the error:
```
Failed to create offer: Database error: Failed to create offer: db error
```

## Root Cause
1. Database migrations for P2P exchange enhancements were not applied
2. Missing columns: `acceptor_id`, `accepted_at`, `conversation_id` in `p2p_offers` table
3. Missing tables: `chat_conversations` and `chat_participants`
4. Frontend was missing `is_proximity_offer` field in the payload

## Solution Applied

### 1. Updated Database Connection
Changed `.env` file to use correct PostgreSQL user:
```env
DATABASE_URL=postgresql://nright@localhost:5432/solana_whale_tracker
```

### 2. Applied Database Migrations

**Added columns to p2p_offers table:**
```sql
ALTER TABLE p2p_offers
ADD COLUMN IF NOT EXISTS acceptor_id UUID REFERENCES users(id),
ADD COLUMN IF NOT EXISTS accepted_at TIMESTAMP,
ADD COLUMN IF NOT EXISTS conversation_id UUID;

CREATE INDEX IF NOT EXISTS idx_p2p_offers_acceptor ON p2p_offers(acceptor_id);
CREATE INDEX IF NOT EXISTS idx_p2p_offers_conversation ON p2p_offers(conversation_id);
```

**Created chat_conversations table:**
```sql
CREATE TABLE IF NOT EXISTS chat_conversations (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  offer_id UUID REFERENCES p2p_offers(id),
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);
```

**Created chat_participants table:**
```sql
CREATE TABLE IF NOT EXISTS chat_participants (
  conversation_id UUID REFERENCES chat_conversations(id) ON DELETE CASCADE,
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  joined_at TIMESTAMP DEFAULT NOW(),
  PRIMARY KEY (conversation_id, user_id)
);
```

**Added indexes:**
```sql
CREATE INDEX IF NOT EXISTS idx_chat_conversations_offer ON chat_conversations(offer_id);
CREATE INDEX IF NOT EXISTS idx_chat_participants_user ON chat_participants(user_id);
CREATE INDEX IF NOT EXISTS idx_chat_participants_conversation ON chat_participants(conversation_id);
```

### 3. Fixed Frontend Code
Added missing `is_proximity_offer` field to the offer creation payload in `frontend/app.js`:
```javascript
const payload = {
    offer_type: offerType,
    from_asset: fromAsset,
    to_asset: toAsset,
    from_amount: fromAmount.toString(),
    to_amount: toAmount.toString(),
    price: price.toString(),
    is_proximity_offer: false  // Added this field
};
```

### 4. Restarted Backend
Restarted the API server with the correct DATABASE_URL environment variable.

## Verification

### Test Offer Creation
```bash
curl -X POST http://localhost:3000/api/p2p/00000000-0000-0000-0000-000000000001/offers \
  -H "Content-Type: application/json" \
  -d '{
    "offer_type": "SELL",
    "from_asset": "SOL",
    "to_asset": "USDC",
    "from_amount": "10",
    "to_amount": "100",
    "price": "10",
    "is_proximity_offer": false
  }'
```

**Expected Response:**
```json
{
    "success": true,
    "data": {
        "id": "9cab0d06-5108-4c6b-bc5a-6183a48c0062",
        "user_id": "00000000-0000-0000-0000-000000000001",
        "offer_type": "Sell",
        "from_asset": "SOL",
        "to_asset": "USDC",
        "from_amount": 10.0,
        "to_amount": 100.0,
        "price": 10.0,
        "status": "Active",
        "acceptor_id": null,
        "accepted_at": null,
        "conversation_id": null,
        "created_at": "2026-02-21T20:31:34.321063Z",
        "expires_at": "2026-02-22T20:31:34.321063Z"
    }
}
```

### Verify Database Schema
```bash
psql postgresql://nright@localhost:5432/solana_whale_tracker -c "\d p2p_offers"
```

Should show the new columns:
- `acceptor_id` (UUID)
- `accepted_at` (TIMESTAMP)
- `conversation_id` (UUID)

## Status
✅ **FIXED** - Database is now properly configured and P2P offers can be created successfully.

## Next Steps
1. Test creating offers through the frontend UI at http://localhost:8080
2. Test accepting offers
3. Verify chat conversation creation on offer acceptance
4. Test marketplace view showing all offers

## Files Modified
- `.env` - Updated DATABASE_URL
- `frontend/app.js` - Added `is_proximity_offer` field
- Database schema - Applied migrations manually

---

**Fixed on:** February 21, 2026  
**Issue Duration:** ~10 minutes  
**Resolution:** Database migrations applied + frontend fix
