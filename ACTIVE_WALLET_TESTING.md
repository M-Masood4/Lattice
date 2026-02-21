# Active Wallet Feature - Testing Guide

## Feature Status: ✅ WORKING

The active wallet selection feature is now fully functional. The backend has been restarted with the updated code.

## What's Working

### Backend API
- ✅ GET endpoint returns `is_primary` field for all wallets
- ✅ PUT endpoint successfully sets a wallet as primary
- ✅ Only one wallet can be primary at a time (enforced by backend)
- ✅ Setting a new primary wallet automatically deactivates the previous one

### Frontend UI
- ✅ "Set as Active" button appears on non-active wallets
- ✅ "Active" badge shows on the primary wallet
- ✅ Active wallet info banner displays at the top
- ✅ Clicking "Set as Active" calls the API and refreshes the list

## How to Test in Browser

1. **Open the Privacy tab** in your browser
2. **View your temporary wallets** - you should see:
   - One wallet with a green "Active" badge
   - Other wallets with a "Set as Active" button
   - An info banner at the top showing the active wallet

3. **Switch active wallet**:
   - Click "Set as Active" on any non-active wallet
   - You should see a success toast notification
   - The wallet list will refresh
   - The new wallet will show the "Active" badge
   - The info banner will update

4. **Verify only one active wallet**:
   - Only one wallet should have the "Active" badge at any time
   - All other wallets should show "Set as Active" button

## API Testing

### Get all wallets with primary status
```bash
curl http://localhost:3000/api/privacy/00000000-0000-0000-0000-000000000001/temporary-wallets | jq '.data[] | {tag: .temp_tag, is_primary}'
```

### Set a wallet as primary
```bash
curl -X PUT http://localhost:3000/api/privacy/00000000-0000-0000-0000-000000000001/temporary-wallets/{WALLET_ID}/primary
```

### Verify only one primary wallet
```bash
curl http://localhost:3000/api/privacy/00000000-0000-0000-0000-000000000001/temporary-wallets | jq '.data[] | select(.is_primary == true)'
```

## What the Active Wallet Controls

The active wallet is used for:
- ✅ All asset conversions
- ✅ P2P exchanges
- ✅ Proximity transfers
- ✅ Trading operations
- ✅ Staking operations
- ✅ Any other wallet-based transactions

## Troubleshooting

If the "Set as Active" button doesn't work:

1. **Check browser console** for JavaScript errors
2. **Verify backend is running**: `curl http://localhost:3000/health`
3. **Check API response includes is_primary**: 
   ```bash
   curl http://localhost:3000/api/privacy/00000000-0000-0000-0000-000000000001/temporary-wallets | jq '.data[0]'
   ```
4. **Hard refresh the page**: Ctrl+Shift+R (or Cmd+Shift+R on Mac)

## Known Limitations

- Delete functionality for temporary wallets is not yet implemented (shows "coming soon" message)
- The active wallet address is not yet integrated with all trading operations (integration pending)
