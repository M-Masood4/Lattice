# Stealth Transfers - Quick Start Guide

## Getting Started in 5 Minutes

### Step 1: Access the Dashboard

Open your browser and navigate to:
```
http://localhost:8080
```

### Step 2: Navigate to Stealth View

Click the **"Stealth"** button in the top navigation bar.

### Step 3: Generate Your Stealth Address

1. Click **"Generate New Address"**
2. Your meta-address will appear (format: `stealth:1:...`)
3. Click the copy icon to copy your address
4. Optionally click **"Show QR Code"** to display a scannable QR code

**Share this meta-address with anyone who wants to send you stealth payments!**

---

## Receiving Payments

### Enable Auto-Scanning

1. Click **"Start Auto-Scan"** button
2. The system will automatically scan for incoming payments every 30 seconds
3. Detected payments appear in the "Received Payments" section

### Manual Scanning

1. Click **"Scan Now"** button anytime
2. View detected payments with details:
   - Amount received
   - Stealth address
   - Ephemeral key
   - Viewing tag

### Unshield Received Funds

1. Click **"Unshield to Regular Address"** on any detected payment
2. Enter your destination address
3. Click **"Unshield"**
4. Funds are transferred to your regular wallet

---

## Sending Payments

### Prepare a Stealth Payment

1. Enter the receiver's meta-address (or click QR icon to scan)
2. Enter the amount in SOL
3. Optionally check **"Send via BLE Mesh"** for offline capability
4. Click **"Prepare Payment"**

### Review and Send

1. Review the prepared payment details:
   - Stealth address (one-time address)
   - Ephemeral key (published on-chain)
   - Viewing tag (for efficient scanning)
   - Amount
2. Click **"Send Payment"**

### Payment Status

- **Settled:** Payment sent on-chain (you'll see the transaction signature)
- **Queued:** Payment queued for later (offline mode)

---

## Privacy Features

### Shield Funds (Break Transaction Graph)

**Purpose:** Convert regular funds to stealth address to break on-chain linkage

1. Go to "Shield & Unshield" section
2. Enter amount to shield
3. Click **"Shield"**
4. Your funds are now in a stealth address with no on-chain link to your original wallet

### Unshield Funds (Convert Back)

**Purpose:** Convert stealth funds back to a regular address

1. Enter the stealth address
2. Enter destination address
3. Click **"Unshield"**
4. Funds transferred to regular address

---

## Offline Payments (BLE Mesh)

### Connect to Mesh Network

1. Go to "BLE Mesh Network" section
2. Click **"Connect"**
3. View connected peers and packets relayed

### Send Offline Payment

1. When preparing a payment, check **"Send via BLE Mesh"**
2. Payment is relayed through nearby devices
3. Payment queues automatically if recipient is offline
4. Auto-settles when both parties are online

### Monitor Payment Queue

1. View queued payments in "Payment Queue" section
2. See payment status:
   - **Queued:** Waiting for network
   - **Settling:** Processing on-chain
   - **Settled:** Completed (with signature)
   - **Failed:** Error occurred

---

## QR Code Features

### Share Your Address via QR

1. After generating your stealth address
2. Click **"Show QR Code"**
3. Share the QR code image
4. Others can scan it to get your meta-address

### Scan Someone's QR Code

1. When sending a payment
2. Click the QR icon next to "Receiver Meta-Address"
3. Upload or scan a QR code image
4. Meta-address is automatically filled in

---

## Tips & Best Practices

### For Maximum Privacy

1. **Use Shield/Unshield:** Break transaction graph linkage
2. **Generate New Addresses:** Create multiple stealth addresses for different purposes
3. **Enable Auto-Scan:** Don't miss incoming payments
4. **Use Mesh for Offline:** Send payments without internet connectivity

### For Reliability

1. **Check Payment Queue:** Monitor queued payments regularly
2. **Keep Auto-Scan On:** Ensure you detect payments quickly
3. **Backup Your Keys:** Use the export feature (coming soon)
4. **Monitor Mesh Status:** Check peer count for mesh reliability

### For Performance

1. **Viewing Tags:** Automatically optimized for fast scanning
2. **Batch Settlements:** Queue processes efficiently when >100 entries
3. **Cached Derivations:** Repeated operations are faster

---

## Common Scenarios

### Scenario 1: Private Payment to Friend

1. Friend shares their meta-address QR code
2. You scan the QR code
3. Enter amount and send
4. Friend receives payment at unique stealth address
5. No on-chain linkage between you and friend

### Scenario 2: Offline Market Transaction

1. You're at a market with no internet
2. Vendor shares meta-address
3. You prepare payment with "Send via BLE Mesh" enabled
4. Payment relays through nearby devices
5. Payment queues and settles when you're back online

### Scenario 3: Breaking Transaction History

1. You want to break linkage from old wallet
2. Shield funds from old wallet to stealth address
3. Wait for confirmation
4. Unshield to new wallet
5. No on-chain connection between old and new wallet

---

## Troubleshooting

### "No payments detected"
- Ensure auto-scan is enabled
- Wait 30 seconds for next scan
- Check that sender used your correct meta-address

### "Payment queued (offline)"
- Normal when internet is unavailable
- Payment will auto-settle when connectivity restored
- Check "Payment Queue" section for status

### "Failed to generate stealth address"
- Check API connection (health indicator in top right)
- Refresh the page
- Check browser console for errors

### "Mesh network not connecting"
- BLE mesh requires physical device (iOS/Android)
- Web dashboard shows simulated status
- Check device Bluetooth permissions

---

## API Integration (For Developers)

### Generate Stealth Address
```bash
curl -X POST http://localhost:3000/api/stealth/generate \
  -H "Content-Type: application/json" \
  -d '{"version": 1}'
```

### Prepare Payment
```bash
curl -X POST http://localhost:3000/api/stealth/prepare-payment \
  -H "Content-Type: application/json" \
  -d '{
    "receiver_meta_address": "stealth:1:...",
    "amount_lamports": 1000000000
  }'
```

### Scan for Payments
```bash
curl -X POST http://localhost:3000/api/stealth/scan \
  -H "Content-Type: application/json" \
  -d '{"meta_address": "stealth:1:..."}'
```

See `crates/stealth/API_DOCUMENTATION.md` for complete API reference.

---

## Security Notes

### What's Private
- ✅ Stealth addresses are unlinkable on-chain
- ✅ Viewing keys can't derive spending keys
- ✅ Mesh payloads are encrypted
- ✅ Keys are encrypted at rest

### What's Not Private
- ⚠️ Transaction amounts are visible on-chain
- ⚠️ Transaction timing can be analyzed
- ⚠️ IP addresses may be logged by RPC nodes

### Best Practices
- Use VPN or Tor for additional network privacy
- Vary transaction amounts to avoid patterns
- Use multiple stealth addresses for different purposes
- Don't reuse stealth addresses

---

## Support

### Documentation
- **Full API Docs:** `crates/stealth/API_DOCUMENTATION.md`
- **Integration Guide:** `crates/stealth/INTEGRATION_GUIDE.md`
- **Requirements:** `.kiro/specs/ble-mesh-stealth-transfers/requirements.md`
- **Design:** `.kiro/specs/ble-mesh-stealth-transfers/design.md`

### Getting Help
- Check browser console for errors
- Check API logs: `tail -f api.log`
- Review test results: `cargo test --package stealth`

---

## What's Next?

### Coming Soon
- Mobile app with real BLE mesh
- Multi-chain support (Ethereum, BSC, Polygon)
- Advanced routing algorithms
- Payment request templates
- Transaction history export

### Contribute
- Report issues on GitHub
- Submit feature requests
- Contribute code improvements
- Help with documentation

---

**Enjoy private, offline-capable cryptocurrency payments!** 🔒📱
