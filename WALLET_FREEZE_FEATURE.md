# Wallet Freeze Feature

## Overview
Users can now freeze and unfreeze their temporary wallets to prevent unauthorized outgoing transactions. This is a security feature to protect wallets in case of suspected compromise.

## Features Implemented

### 1. Freeze Wallet
- **Button**: "Freeze" button on each temporary wallet
- **Confirmation**: Requires user confirmation before freezing
- **Effect**: Blocks all outgoing transactions from the wallet
- **Visual Indicator**: Orange "Frozen" badge appears on frozen wallets
- **API Endpoint**: `POST /api/privacy/:user_id/wallets/:wallet_address/freeze`

### 2. Unfreeze Wallet
- **Button**: "Unfreeze" button (orange/warning color) on frozen wallets
- **Security**: Requires password verification to unfreeze
- **Effect**: Restores normal wallet functionality
- **API Endpoint**: `POST /api/privacy/:user_id/wallets/:wallet_address/unfreeze`

### 3. Visual Status
- **Frozen Badge**: Orange badge with "Frozen" text
- **Button State**: Freeze/Unfreeze button toggles based on wallet state
- **Active Wallet**: Can freeze even the active wallet for security

## How It Works

### Freezing a Wallet
1. Click the "Freeze" button on any temporary wallet
2. Confirm the action in the dialog
3. The wallet is immediately frozen
4. A "Frozen" badge appears on the wallet
5. The button changes to "Unfreeze"

### Unfreezing a Wallet
1. Click the "Unfreeze" button on a frozen wallet
2. Enter your password when prompted
3. If password is correct, wallet is unfrozen
4. The "Frozen" badge disappears
5. The button changes back to "Freeze"

## Backend Implementation

### Database Fields
- `is_frozen`: BOOLEAN field in `multi_chain_wallets` table
- `frozen_at`: TIMESTAMP field to track when wallet was frozen

### API Response
The `TemporaryWallet` struct now includes:
```rust
pub struct TemporaryWallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub blockchain: String,
    pub address: String,
    pub temp_tag: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub is_primary: bool,
    pub is_frozen: bool,  // NEW FIELD
}
```

### Security
- Freeze: No authentication required (quick security action)
- Unfreeze: Requires password verification (prevents unauthorized unfreezing)
- The backend validates the user owns the wallet before freezing/unfreezing

## Use Cases

### 1. Suspected Compromise
If you suspect your wallet may be compromised:
1. Immediately freeze the wallet
2. Investigate the issue
3. Transfer funds to a new wallet if needed
4. Unfreeze or delete the wallet

### 2. Temporary Pause
If you want to temporarily stop all transactions:
1. Freeze the wallet
2. Perform your analysis or wait for market conditions
3. Unfreeze when ready to resume trading

### 3. Active Wallet Protection
Even your active wallet can be frozen:
1. Freeze the active wallet for security
2. Set a different wallet as active if needed
3. Unfreeze when you're ready to use it again

## Testing

### Test Freeze
```bash
curl -X POST http://localhost:3000/api/privacy/00000000-0000-0000-0000-000000000001/wallets/{WALLET_ADDRESS}/freeze \
  -H "Content-Type: application/json" \
  -d '{}'
```

### Test Unfreeze
```bash
curl -X POST http://localhost:3000/api/privacy/00000000-0000-0000-0000-000000000001/wallets/{WALLET_ADDRESS}/unfreeze \
  -H "Content-Type: application/json" \
  -d '{"password":"your_password"}'
```

### Check Frozen Status
```bash
curl http://localhost:3000/api/privacy/00000000-0000-0000-0000-000000000001/temporary-wallets | \
  jq '.data[] | {tag: .temp_tag, is_frozen}'
```

## UI Elements

### Badges
- **Active**: Green badge - indicates the current active wallet
- **Frozen**: Orange badge - indicates wallet is frozen
- **Expired**: Red badge - indicates wallet has expired

### Buttons
- **Set as Active**: Blue button - makes wallet the active one
- **Freeze**: Gray button - freezes the wallet
- **Unfreeze**: Orange button - unfreezes the wallet (requires password)
- **Copy Address**: Gray button - copies wallet address
- **Delete**: Gray button - deletes the wallet (coming soon)

## Known Limitations

1. **Password Verification**: The unfreeze function requires a valid user password. For demo purposes, you may need to set up proper authentication.

2. **Blockchain Integration**: The freeze is currently at the application level. It doesn't freeze the wallet on the blockchain itself - it prevents the application from initiating transactions.

3. **2FA Option**: Future enhancement could add 2FA requirement for unfreezing instead of just password.

## Future Enhancements

- [ ] Add 2FA requirement for unfreezing
- [ ] Add freeze reason/notes
- [ ] Add automatic unfreeze after X hours
- [ ] Add freeze history/audit log
- [ ] Add email notification when wallet is frozen/unfrozen
- [ ] Add blockchain-level freeze integration
