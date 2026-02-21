# Chat Proximity Network Integration

## Overview
Chat functionality has been restricted to only show users within the proximity network. Users can only see and message peers who are currently discovered via WiFi or Bluetooth proximity discovery.

## Changes Made

### Backend Changes

#### 1. Chat Service (`crates/api/src/chat_service.rs`)
- Added `ProximityContact` struct to represent chat contacts from proximity network
- Added `get_proximity_contacts()` method to retrieve discovered peers as chat contacts
- Added `is_user_in_proximity()` method to check if a user is in the proximity network

#### 2. API Handlers (`crates/api/src/handlers.rs`)
- Added `get_proximity_contacts()` handler to expose proximity contacts via API
- Returns list of discovered peers with their user tags, wallet addresses, discovery methods, and signal strength

#### 3. Routes (`crates/api/src/routes.rs`)
- Added new route: `GET /api/chat/proximity-contacts`
- Returns all users currently in the proximity network

### Frontend Changes

#### 1. Contact Loading (`frontend/app.js`)
- Modified `loadContacts()` to fetch from `/api/chat/proximity-contacts` endpoint
- Replaced mock contacts with real proximity network users
- Shows empty state when no users are discovered

#### 2. Contact Display
- Updated `displayProximityContacts()` to show:
  - User tag
  - Discovery method (WiFi/Bluetooth)
  - Signal strength indicator
- Removed mock contact functionality

#### 3. Contact Selection
- Updated `selectProximityContact()` to use wallet address for identification
- Chat history now loads based on wallet address instead of arbitrary IDs

## API Endpoints

### Get Proximity Contacts
```
GET /api/chat/proximity-contacts
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "peer_id": "peer_abc123",
      "user_tag": "Trader_X7Y9Z2",
      "wallet_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
      "discovery_method": "WiFi",
      "signal_strength": -45,
      "verified": true,
      "last_seen": "2026-02-21T18:00:00Z"
    }
  ],
  "error": null
}
```

## User Flow

1. **Enable Discovery**: User must first enable proximity discovery (WiFi or Bluetooth)
2. **Discover Peers**: System discovers nearby users running the same app
3. **View Contacts**: Chat view shows only discovered peers as available contacts
4. **Select Contact**: User selects a peer from the proximity network
5. **Send Messages**: User can send encrypted messages to selected peer

## Security & Privacy

- **Proximity-Only**: Users can only chat with peers they've physically discovered
- **No Global Directory**: No centralized user directory or contact list
- **Ephemeral Contacts**: Contacts disappear when peers leave proximity range
- **End-to-End Encryption**: All messages are encrypted using AES-256-GCM

## Testing

### Test Proximity Chat Integration
1. Start the application: `./deploy-local.sh`
2. Enable discovery: Navigate to Proximity view and click "Start Discovery"
3. Open chat view: Click on "Chat" in navigation
4. Verify: Contact list shows "No users in proximity network" message
5. When peers are discovered: They will appear in the contact list automatically

### Test API Endpoint
```bash
# Start discovery first
curl -X POST http://localhost:3000/api/proximity/00000000-0000-0000-0000-000000000001/discovery/start \
  -H "Content-Type: application/json" \
  -d '{"method":"WiFi"}'

# Get proximity contacts
curl http://localhost:3000/api/chat/proximity-contacts
```

## Future Enhancements

1. **Real-time Updates**: WebSocket notifications when new peers are discovered
2. **Contact Persistence**: Option to save frequently contacted peers
3. **Group Chats**: Support for multi-peer proximity group conversations
4. **Message History**: Load actual message history between proximity contacts
5. **Presence Indicators**: Show online/offline status for proximity contacts
6. **Distance Estimation**: Show approximate distance based on signal strength

## Notes

- Chat contacts are dynamically updated based on active proximity discovery
- Users must have discovery enabled to see any chat contacts
- Contacts automatically disappear when peers leave proximity range or stop discovery
- The system uses wallet addresses as unique identifiers for chat participants
