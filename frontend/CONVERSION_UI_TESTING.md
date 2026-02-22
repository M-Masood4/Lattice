# Conversion UI Testing Guide

## Quick Start Testing

### 1. Start the Application
```bash
# Start the backend (from project root)
cargo run --bin api

# Open frontend in browser
open frontend/index.html
# or
python3 -m http.server 8080 --directory frontend
```

### 2. Navigate to Convert View
- Click the "Convert" button in the navigation bar
- The conversion interface should load

## Test Cases

### Test Case 1: Asset Selection
**Steps:**
1. Select "SOL" from the "From" dropdown
2. Select "USDC" from the "To" dropdown
3. Verify "Get Quote" button becomes enabled

**Expected Result:**
- Both dropdowns show selected assets
- Get Quote button is enabled
- No error messages displayed

### Test Case 2: Exact Input Mode (Default)
**Steps:**
1. Select SOL → USDC
2. Enter "10" in the From amount field
3. Click "Get Quote"
4. Wait for quote to load

**Expected Result:**
- Loading overlay appears
- Quote details section appears with:
  - Exchange rate (e.g., "1 SOL = 200 USDC")
  - Network fee
  - Platform fee
  - Provider fee
  - Total fees
  - Final amount to receive
- Quote expiration countdown starts
- "Convert" button appears and is enabled
- To amount field is populated (read-only)

### Test Case 3: Exact Output Mode
**Steps:**
1. Click "Exact Output" mode button
2. Select ETH → BTC
3. Enter "0.1" in the To amount field (desired output)
4. Click "Get Quote"

**Expected Result:**
- Mode button shows "Exact Output" as active
- From amount field becomes read-only
- To amount field is editable
- Quote shows required input amount in From field
- All fee details displayed correctly

### Test Case 4: Mode Switching
**Steps:**
1. Start in Exact Input mode with SOL → USDC
2. Enter "5" in From amount
3. Switch to Exact Output mode
4. Verify From amount is now read-only
5. Switch back to Exact Input mode

**Expected Result:**
- Input fields toggle readonly state correctly
- Active mode button updates visually
- Quote is cleared when switching modes
- No errors occur

### Test Case 5: Asset Swap
**Steps:**
1. Select SOL → USDC
2. Enter "10" in From amount
3. Click the swap button (circular arrow icon)

**Expected Result:**
- Assets swap positions (now USDC → SOL)
- Swap button rotates 180 degrees
- Quote is cleared
- Amount fields are cleared or swapped appropriately

### Test Case 6: Quote Expiration
**Steps:**
1. Get a quote for any asset pair
2. Wait for the countdown timer to reach 0

**Expected Result:**
- Timer counts down from initial value
- When timer reaches 0:
  - "Quote expired" message appears
  - "Convert" button becomes disabled
  - Warning toast notification appears
  - User must get a new quote

### Test Case 7: Conversion Execution
**Steps:**
1. Get a valid quote for SOL → USDC
2. Click "Convert" button
3. Confirm in the browser dialog

**Expected Result:**
- Confirmation dialog appears
- Loading overlay shows during execution
- Success toast notification appears
- Form is cleared
- Conversion history refreshes automatically
- New conversion appears at top of history

### Test Case 8: Conversion History Display
**Steps:**
1. Navigate to Convert view
2. Scroll to "Conversion History" section
3. Click "Refresh" button

**Expected Result:**
- History displays past conversions
- Each item shows:
  - Asset pair (FROM → TO)
  - Amounts
  - Timestamp
  - Status (completed/pending/failed)
  - Exchange rate
  - P/L percentage with color coding
- Status indicated by border color:
  - Green = completed
  - Yellow = pending
  - Red = failed

### Test Case 9: P/L Calculation
**Steps:**
1. View conversion history
2. Check P/L percentages

**Expected Result:**
- Positive P/L shown in green with "+" prefix
- Negative P/L shown in red with "-" prefix
- Percentage calculated based on exchange rate difference

### Test Case 10: Input Validation
**Steps:**
1. Try to get quote without selecting assets
2. Try to get quote with zero amount
3. Try to get quote with negative amount
4. Try to select same asset for both From and To

**Expected Result:**
- Get Quote button remains disabled for invalid inputs
- Error messages displayed for invalid states
- Cannot proceed with invalid configuration

### Test Case 11: Error Handling
**Steps:**
1. Disconnect from API (stop backend)
2. Try to get a quote
3. Observe error handling

**Expected Result:**
- Error message displayed in red
- Error toast notification appears
- Mock data may be shown for demo purposes
- Application doesn't crash

### Test Case 12: Responsive Design
**Steps:**
1. Resize browser window to mobile size (< 768px)
2. Test all functionality

**Expected Result:**
- Layout adapts to mobile view
- Asset inputs stack vertically
- All buttons remain accessible
- History items display properly
- No horizontal scrolling required

## API Integration Testing

### Test with Real Backend
If backend is running at `http://localhost:3000`:

1. **Get Quote Endpoint:**
```bash
curl -X POST http://localhost:3000/api/conversions/quote \
  -H "Content-Type: application/json" \
  -d '{
    "from_asset": "SOL",
    "to_asset": "USDC",
    "amount": "10",
    "amount_type": "from"
  }'
```

2. **Execute Conversion Endpoint:**
```bash
curl -X POST http://localhost:3000/api/conversions/demo-user-id/execute \
  -H "Content-Type: application/json" \
  -d '{
    "quote_id": "test-quote-id",
    "from_asset": "SOL",
    "to_asset": "USDC",
    "from_amount": "10",
    "to_amount": "2000",
    "exchange_rate": "200",
    "network_fee": "0.01",
    "platform_fee": "0.05",
    "provider_fee": "0.02",
    "provider": "SideShift",
    "expires_at": "2026-02-20T12:00:00Z",
    "settle_address": "test-address",
    "refund_address": "test-address"
  }'
```

3. **Get History Endpoint:**
```bash
curl http://localhost:3000/api/conversions/demo-user-id/history
```

## Mock Data Testing

When API is unavailable, the UI falls back to mock data:
- Mock conversion history with 3 sample conversions
- Demonstrates all status types
- Shows realistic amounts and rates

## Performance Testing

### Load Time
- Conversion view should load instantly
- No lag when switching to Convert tab

### Quote Fetching
- Quote should return within 2-3 seconds
- Loading overlay should be visible during fetch

### History Loading
- History should load within 1-2 seconds
- Smooth scrolling for long history lists

## Browser Compatibility

Test in:
- Chrome/Edge (Chromium)
- Firefox
- Safari
- Mobile browsers (iOS Safari, Chrome Mobile)

## Known Limitations

1. **User Authentication**: Currently uses demo user ID
2. **Balance Checking**: Not implemented in UI (backend may validate)
3. **Jupiter Fallback**: Not fully implemented in backend
4. **Real-time Price Updates**: Requires manual quote refresh
5. **Slippage Protection**: Not implemented

## Troubleshooting

### Quote Not Loading
- Check browser console for errors
- Verify backend is running
- Check API URL in Settings view
- Verify network connectivity

### Convert Button Disabled
- Check if quote has expired
- Verify assets are selected
- Ensure amount is positive
- Check for error messages

### History Not Showing
- Verify wallet is connected
- Check if user has any conversions
- Try clicking Refresh button
- Check browser console for errors

## Success Criteria

✅ All asset selections work correctly
✅ Both exact input and exact output modes function
✅ Quote fetching displays all required information
✅ Fee breakdown is clear and accurate
✅ Quote expiration timer works correctly
✅ Conversion execution completes successfully
✅ History displays with correct P/L calculations
✅ Error handling is graceful
✅ Responsive design works on mobile
✅ No console errors during normal operation
