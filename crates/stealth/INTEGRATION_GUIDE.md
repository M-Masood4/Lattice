# Stealth Address Integration Guide

## Overview

This guide explains how to integrate the stealth address system with the existing crypto trading platform, including platform-specific setup for iOS and Android, API endpoints, and WebSocket events.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Platform Integration](#platform-integration)
3. [iOS Setup](#ios-setup)
4. [Android Setup](#android-setup)
5. [API Endpoints](#api-endpoints)
6. [WebSocket Events](#websocket-events)
7. [Database Schema](#database-schema)
8. [Service Integration](#service-integration)
9. [Testing](#testing)
10. [Deployment](#deployment)

---

## Architecture Overview

### Component Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend (React)                      │
│  • Stealth wallet UI                                     │
│  • QR code scanner/generator                             │
│  • Payment queue status                                  │
└────────────────────┬────────────────────────────────────┘
                     │ REST API / WebSocket
┌────────────────────▼────────────────────────────────────┐
│              API Layer (crates/api)                      │
│  • Stealth handlers                                      │
│  • WebSocket service                                     │
│  • Authentication                                        │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│         Stealth Wallet Service (crates/stealth)          │
│  • Wallet manager                                        │
│  • Payment queue                                         │
│  • Scanner                                               │
└────────────────────┬────────────────────────────────────┘
                     │
        ┌────────────┴────────────┐
        │                         │
┌───────▼──────┐         ┌────────▼────────┐
│  BLE Mesh    │         │    Blockchain   │
│  (crates/    │         │    (crates/     │
│  ble-mesh)   │         │    blockchain)  │
└──────────────┘         └─────────────────┘
```

### Data Flow

1. **Payment Initiation**: Frontend → API → Wallet Manager
2. **Stealth Address Generation**: Wallet Manager → Generator → Crypto
3. **Blockchain Submission**: Wallet Manager → Blockchain Client → Solana
4. **Scanning**: Scanner → Blockchain Client → Solana
5. **Notifications**: Scanner → WebSocket Service → Frontend

---

## Platform Integration

### Existing Platform Services

The stealth address system integrates with these existing services:


#### 1. Blockchain Client (`crates/blockchain/src/client.rs`)

```rust
// Add stealth transaction methods
impl SolanaClient {
    pub async fn send_stealth_transaction(
        &self,
        stealth_address: &Pubkey,
        amount: u64,
        ephemeral_public_key: &Pubkey,
        viewing_tag: &[u8; 4],
        payer: &Keypair,
    ) -> Result<Signature> {
        // Create transaction with stealth metadata
        let instruction = create_stealth_payment_instruction(
            stealth_address,
            amount,
            ephemeral_public_key,
            viewing_tag,
        );
        
        // Use existing retry and circuit breaker logic
        self.send_transaction_with_retry(&[instruction], payer).await
    }
}
```

#### 2. Wallet Service (`crates/api/src/wallet_service.rs`)

```rust
// Add stealth wallet methods
impl WalletService {
    pub async fn create_stealth_wallet(&self, user_id: &str) -> Result<StealthKeyPair> {
        let keypair = StealthKeyPair::generate_standard()?;
        
        // Store in secure storage
        self.storage.store_keypair(&keypair).await?;
        
        // Store meta-address in database
        self.db.store_stealth_meta_address(user_id, &keypair.to_meta_address()).await?;
        
        Ok(keypair)
    }
    
    pub async fn get_stealth_wallet(&self, user_id: &str) -> Result<StealthKeyPair> {
        self.storage.load_keypair().await?
            .ok_or_else(|| Error::WalletNotFound)
    }
}
```

#### 3. WebSocket Service (`crates/api/src/websocket_service.rs`)

```rust
// Add stealth payment events
impl WebSocketService {
    pub async fn emit_stealth_payment_detected(
        &self,
        user_id: &str,
        payment: &DetectedPayment,
    ) {
        let event = json!({
            "type": "stealth_payment_detected",
            "data": {
                "stealth_address": payment.stealth_address.to_string(),
                "amount": payment.amount,
                "ephemeral_public_key": payment.ephemeral_public_key.to_string(),
                "slot": payment.slot,
                "signature": payment.signature.to_string(),
            }
        });
        
        self.send_to_user(user_id, event).await;
    }
    
    pub async fn emit_payment_queued(&self, user_id: &str, payment_id: &str) {
        let event = json!({
            "type": "payment_queued",
            "data": {
                "payment_id": payment_id,
                "status": "queued",
            }
        });
        
        self.send_to_user(user_id, event).await;
    }
    
    pub async fn emit_payment_settled(
        &self,
        user_id: &str,
        payment_id: &str,
        signature: &Signature,
    ) {
        let event = json!({
            "type": "payment_settled",
            "data": {
                "payment_id": payment_id,
                "status": "settled",
                "signature": signature.to_string(),
            }
        });
        
        self.send_to_user(user_id, event).await;
    }
}
```

---

## iOS Setup

### Prerequisites

- Xcode 14.0+
- iOS 13.0+ deployment target
- CocoaPods or Swift Package Manager

### 1. Add Permissions to Info.plist

```xml
<key>NSBluetoothAlwaysUsageDescription</key>
<string>This app uses Bluetooth to send payments through mesh network</string>

<key>NSBluetoothPeripheralUsageDescription</key>
<string>This app uses Bluetooth to receive payments through mesh network</string>

<key>NSCameraUsageDescription</key>
<string>This app uses the camera to scan QR codes for payment addresses</string>
```

### 2. Configure Rust Build

Add to `Cargo.toml`:

```toml
[lib]
crate-type = ["staticlib", "cdylib"]

[target.'cfg(target_os = "ios")'.dependencies]
security-framework = "2.9"  # For Keychain access
```

### 3. Build for iOS

```bash
# Install iOS targets
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios  # For simulator

# Build
cargo build --target aarch64-apple-ios --release
cargo build --target x86_64-apple-ios --release  # Simulator
```

### 4. iOS Bridge Code

Create `ios/StealthWalletBridge.swift`:

```swift
import Foundation

class StealthWalletBridge {
    // Call Rust functions via FFI
    func generateKeyPair() -> String {
        let result = stealth_generate_keypair()
        return String(cString: result)
    }
    
    func preparePayment(metaAddress: String, amount: UInt64) -> String {
        let result = stealth_prepare_payment(metaAddress, amount)
        return String(cString: result)
    }
    
    func scanForPayments() -> [DetectedPayment] {
        let result = stealth_scan_payments()
        // Parse JSON result
        return parsePayments(result)
    }
}
```

### 5. iOS Secure Storage

The `IosKeychainStorage` implementation uses iOS Keychain:

```rust
// crates/stealth/src/storage/platform/ios.rs
use security_framework::keychain::{SecKeychain, SecKeychainItem};

pub struct IosKeychainStorage;

impl SecureStorage for IosKeychainStorage {
    async fn store_keypair(&self, keypair: &StealthKeyPair) -> StealthResult<()> {
        // Store in iOS Keychain with kSecAttrAccessibleWhenUnlockedThisDeviceOnly
        let encrypted = keypair.export_encrypted(DEVICE_KEY)?;
        
        SecKeychain::default()
            .add_generic_password("stealth_keypair", &encrypted)
            .map_err(|e| StealthError::StorageError(e.to_string()))?;
        
        Ok(())
    }
}
```

---

## Android Setup

### Prerequisites

- Android Studio Arctic Fox+
- Android SDK 23+ (Android 6.0+)
- NDK r23+

### 1. Add Permissions to AndroidManifest.xml

```xml
<uses-permission android:name="android.permission.BLUETOOTH" />
<uses-permission android:name="android.permission.BLUETOOTH_ADMIN" />
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION" />
<uses-permission android:name="android.permission.CAMERA" />

<!-- Android 12+ -->
<uses-permission android:name="android.permission.BLUETOOTH_SCAN" />
<uses-permission android:name="android.permission.BLUETOOTH_CONNECT" />
<uses-permission android:name="android.permission.BLUETOOTH_ADVERTISE" />
```

### 2. Configure Rust Build

Add to `Cargo.toml`:

```toml
[target.'cfg(target_os = "android")'.dependencies]
jni = "0.21"  # For JNI bindings
```

### 3. Build for Android

```bash
# Install Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android

# Build
cargo ndk --target aarch64-linux-android --platform 23 build --release
```

### 4. Android Bridge Code

Create `android/StealthWalletBridge.kt`:

```kotlin
class StealthWalletBridge {
    companion object {
        init {
            System.loadLibrary("stealth")
        }
    }
    
    external fun generateKeyPair(): String
    external fun preparePayment(metaAddress: String, amount: Long): String
    external fun scanForPayments(): String
    
    fun parsePayments(json: String): List<DetectedPayment> {
        // Parse JSON result
        return Gson().fromJson(json, Array<DetectedPayment>::class.java).toList()
    }
}
```

### 5. Request Runtime Permissions

```kotlin
class MainActivity : AppCompatActivity() {
    private val PERMISSION_REQUEST_CODE = 1
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Request Bluetooth and location permissions
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            requestPermissions(
                arrayOf(
                    Manifest.permission.BLUETOOTH_SCAN,
                    Manifest.permission.BLUETOOTH_CONNECT,
                    Manifest.permission.BLUETOOTH_ADVERTISE,
                    Manifest.permission.ACCESS_FINE_LOCATION
                ),
                PERMISSION_REQUEST_CODE
            )
        }
    }
}
```

### 6. Android Secure Storage

The `AndroidKeystoreStorage` implementation uses Android Keystore:

```rust
// crates/stealth/src/storage/platform/android.rs
use jni::JNIEnv;
use jni::objects::JObject;

pub struct AndroidKeystoreStorage {
    jni_env: JNIEnv<'static>,
}

impl SecureStorage for AndroidKeystoreStorage {
    async fn store_keypair(&self, keypair: &StealthKeyPair) -> StealthResult<()> {
        // Store in Android Keystore
        let encrypted = keypair.export_encrypted(DEVICE_KEY)?;
        
        // Call Android Keystore via JNI
        self.jni_env.call_method(
            keystore_obj,
            "setEntry",
            "(Ljava/lang/String;[B)V",
            &[JValue::from("stealth_keypair"), JValue::from(&encrypted)]
        ).map_err(|e| StealthError::StorageError(e.to_string()))?;
        
        Ok(())
    }
}
```


---

## API Endpoints

### Base URL

```
Production: https://api.yourplatform.com
Development: http://localhost:8080
```

### Authentication

All endpoints require JWT authentication:

```
Authorization: Bearer <jwt_token>
```

### Endpoints

#### 1. Generate Stealth Wallet

**POST** `/api/stealth/generate`

Generate a new stealth key pair for the authenticated user.

**Request:**
```json
{
  "version": 1  // 1 = standard, 2 = hybrid post-quantum
}
```

**Response:**
```json
{
  "meta_address": "stealth:1:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp...",
  "spending_public_key": "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp...",
  "viewing_public_key": "7cvkjYAkUYs4W8XcXsHNrdKieRvoBd7Y...",
  "qr_code": "data:image/png;base64,iVBORw0KGgoAAAANS..."
}
```

**Implementation:**
```rust
// crates/api/src/handlers.rs
pub async fn generate_stealth_wallet(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(req): Json<GenerateWalletRequest>,
) -> Result<Json<GenerateWalletResponse>> {
    let keypair = if req.version == 2 {
        HybridStealthKeyPair::generate_hybrid()?.into()
    } else {
        StealthKeyPair::generate_standard()?
    };
    
    let meta_address = keypair.to_meta_address();
    let qr_code = QrCodeHandler::encode_meta_address(&meta_address)?;
    
    // Store in database
    state.wallet_service.create_stealth_wallet(&user.id, keypair).await?;
    
    Ok(Json(GenerateWalletResponse {
        meta_address,
        spending_public_key: keypair.spending_public_key().to_string(),
        viewing_public_key: keypair.viewing_public_key().to_string(),
        qr_code: format!("data:image/png;base64,{}", base64::encode(qr_code)),
    }))
}
```

#### 2. Prepare Stealth Payment

**POST** `/api/stealth/prepare-payment`

Prepare a stealth payment to a receiver.

**Request:**
```json
{
  "receiver_meta_address": "stealth:1:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp...",
  "amount": 1000000000
}
```

**Response:**
```json
{
  "payment_id": "550e8400-e29b-41d4-a716-446655440000",
  "stealth_address": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
  "ephemeral_public_key": "3kVK9qsEPFdqw7oWNW3b9wHP3BqKDWHDqSMqCiRmxnmY",
  "viewing_tag": [12, 34, 56, 78],
  "amount": 1000000000
}
```

**Implementation:**
```rust
pub async fn prepare_stealth_payment(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(req): Json<PreparePaymentRequest>,
) -> Result<Json<PreparePaymentResponse>> {
    let wallet_manager = state.get_wallet_manager(&user.id).await?;
    
    let prepared = wallet_manager.prepare_payment(
        &req.receiver_meta_address,
        req.amount,
    )?;
    
    let payment_id = Uuid::new_v4();
    
    Ok(Json(PreparePaymentResponse {
        payment_id: payment_id.to_string(),
        stealth_address: prepared.stealth_address.to_string(),
        ephemeral_public_key: prepared.ephemeral_public_key.to_string(),
        viewing_tag: prepared.viewing_tag,
        amount: prepared.amount,
    }))
}
```

#### 3. Send Stealth Payment

**POST** `/api/stealth/send`

Send or queue a prepared stealth payment.

**Request:**
```json
{
  "payment_id": "550e8400-e29b-41d4-a716-446655440000",
  "stealth_address": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
  "ephemeral_public_key": "3kVK9qsEPFdqw7oWNW3b9wHP3BqKDWHDqSMqCiRmxnmY",
  "viewing_tag": [12, 34, 56, 78],
  "amount": 1000000000
}
```

**Response:**
```json
{
  "status": "settled",  // or "queued"
  "signature": "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4yhzQKDoLfgEdMc7w5Uh7yy9N",
  "payment_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Implementation:**
```rust
pub async fn send_stealth_payment(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(req): Json<SendPaymentRequest>,
) -> Result<Json<SendPaymentResponse>> {
    let mut wallet_manager = state.get_wallet_manager(&user.id).await?;
    
    let prepared = PreparedPayment {
        stealth_address: Pubkey::from_str(&req.stealth_address)?,
        amount: req.amount,
        ephemeral_public_key: Pubkey::from_str(&req.ephemeral_public_key)?,
        viewing_tag: req.viewing_tag,
    };
    
    let status = wallet_manager.send_payment(prepared).await?;
    
    // Emit WebSocket event
    match &status {
        PaymentStatus::Queued => {
            state.ws_service.emit_payment_queued(&user.id, &req.payment_id).await;
        }
        PaymentStatus::Settled(sig) => {
            state.ws_service.emit_payment_settled(&user.id, &req.payment_id, sig).await;
        }
        _ => {}
    }
    
    Ok(Json(SendPaymentResponse {
        status: format!("{:?}", status),
        signature: match status {
            PaymentStatus::Settled(sig) => Some(sig.to_string()),
            _ => None,
        },
        payment_id: req.payment_id,
    }))
}
```

#### 4. Scan for Incoming Payments

**GET** `/api/stealth/scan`

Scan blockchain for incoming stealth payments.

**Query Parameters:**
- `from_slot` (optional): Starting slot
- `to_slot` (optional): Ending slot

**Response:**
```json
{
  "payments": [
    {
      "stealth_address": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
      "amount": 1000000000,
      "ephemeral_public_key": "3kVK9qsEPFdqw7oWNW3b9wHP3BqKDWHDqSMqCiRmxnmY",
      "viewing_tag": [12, 34, 56, 78],
      "slot": 123456789,
      "signature": "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4yhzQKDoLfgEdMc7w5Uh7yy9N"
    }
  ],
  "last_scanned_slot": 123456789
}
```

**Implementation:**
```rust
pub async fn scan_stealth_payments(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Query(params): Query<ScanParams>,
) -> Result<Json<ScanResponse>> {
    let mut wallet_manager = state.get_wallet_manager(&user.id).await?;
    
    let payments = wallet_manager.scan_incoming().await?;
    
    // Emit WebSocket events for new payments
    for payment in &payments {
        state.ws_service.emit_stealth_payment_detected(&user.id, payment).await;
    }
    
    Ok(Json(ScanResponse {
        payments: payments.into_iter().map(|p| PaymentDto::from(p)).collect(),
        last_scanned_slot: state.get_last_scanned_slot(&user.id).await?,
    }))
}
```

#### 5. Shield Funds

**POST** `/api/stealth/shield`

Convert regular funds to stealth address.

**Request:**
```json
{
  "amount": 5000000000,
  "source_wallet": "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp..."
}
```

**Response:**
```json
{
  "signature": "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4yhzQKDoLfgEdMc7w5Uh7yy9N",
  "stealth_address": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin"
}
```

#### 6. Unshield Funds

**POST** `/api/stealth/unshield`

Convert stealth funds to regular address.

**Request:**
```json
{
  "stealth_address": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
  "ephemeral_public_key": "3kVK9qsEPFdqw7oWNW3b9wHP3BqKDWHDqSMqCiRmxnmY",
  "destination": "7cvkjYAkUYs4W8XcXsHNrdKieRvoBd7Y..."
}
```

**Response:**
```json
{
  "signature": "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4yhzQKDoLfgEdMc7w5Uh7yy9N"
}
```

#### 7. Get Payment Queue Status

**GET** `/api/stealth/queue`

Get status of queued payments.

**Response:**
```json
{
  "queued_payments": [
    {
      "payment_id": "550e8400-e29b-41d4-a716-446655440000",
      "status": "queued",
      "amount": 1000000000,
      "created_at": "2024-01-15T10:30:00Z",
      "retry_count": 0
    }
  ],
  "total_queued": 1
}
```


---

## WebSocket Events

### Connection

```javascript
const ws = new WebSocket('wss://api.yourplatform.com/ws');

ws.onopen = () => {
  // Authenticate
  ws.send(JSON.stringify({
    type: 'auth',
    token: jwt_token
  }));
};
```

### Event Types

#### 1. stealth_payment_detected

Emitted when a new stealth payment is detected during scanning.

```json
{
  "type": "stealth_payment_detected",
  "data": {
    "stealth_address": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
    "amount": 1000000000,
    "ephemeral_public_key": "3kVK9qsEPFdqw7oWNW3b9wHP3BqKDWHDqSMqCiRmxnmY",
    "slot": 123456789,
    "signature": "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4yhzQKDoLfgEdMc7w5Uh7yy9N"
  }
}
```

**Frontend Handler:**
```javascript
ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  
  if (message.type === 'stealth_payment_detected') {
    const { amount, stealth_address } = message.data;
    showNotification(`Received ${amount / 1e9} SOL at stealth address`);
    updateBalance();
  }
};
```

#### 2. payment_queued

Emitted when a payment is queued (offline mode).

```json
{
  "type": "payment_queued",
  "data": {
    "payment_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "queued"
  }
}
```

**Frontend Handler:**
```javascript
if (message.type === 'payment_queued') {
  const { payment_id } = message.data;
  showNotification('Payment queued - will settle when online');
  updatePaymentStatus(payment_id, 'queued');
}
```

#### 3. payment_settled

Emitted when a queued payment is successfully settled on-chain.

```json
{
  "type": "payment_settled",
  "data": {
    "payment_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "settled",
    "signature": "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4yhzQKDoLfgEdMc7w5Uh7yy9N"
  }
}
```

**Frontend Handler:**
```javascript
if (message.type === 'payment_settled') {
  const { payment_id, signature } = message.data;
  showNotification('Payment settled successfully');
  updatePaymentStatus(payment_id, 'settled', signature);
}
```

#### 4. payment_failed

Emitted when a payment fails after maximum retry attempts.

```json
{
  "type": "payment_failed",
  "data": {
    "payment_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "failed",
    "error": "Insufficient funds"
  }
}
```

**Frontend Handler:**
```javascript
if (message.type === 'payment_failed') {
  const { payment_id, error } = message.data;
  showError(`Payment failed: ${error}`);
  updatePaymentStatus(payment_id, 'failed');
}
```

---

## Database Schema

### Tables

#### stealth_wallets

```sql
CREATE TABLE stealth_wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL UNIQUE,
    meta_address TEXT NOT NULL,
    spending_public_key VARCHAR(44) NOT NULL,
    viewing_public_key VARCHAR(44) NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    INDEX idx_user_id (user_id),
    INDEX idx_meta_address (meta_address)
);
```

#### stealth_payments

```sql
CREATE TABLE stealth_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    payment_id VARCHAR(255) NOT NULL UNIQUE,
    stealth_address VARCHAR(44) NOT NULL,
    amount BIGINT NOT NULL,
    ephemeral_public_key VARCHAR(44) NOT NULL,
    viewing_tag BYTEA NOT NULL,
    status VARCHAR(20) NOT NULL,  -- queued, settling, settled, failed
    signature VARCHAR(88),
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    INDEX idx_user_id (user_id),
    INDEX idx_payment_id (payment_id),
    INDEX idx_status (status),
    INDEX idx_created_at (created_at)
);
```

#### stealth_scan_index

```sql
CREATE TABLE stealth_scan_index (
    user_id VARCHAR(255) PRIMARY KEY,
    last_scanned_slot BIGINT NOT NULL,
    last_scan_time TIMESTAMP NOT NULL DEFAULT NOW(),
    
    INDEX idx_last_scan_time (last_scan_time)
);
```

#### detected_payments

```sql
CREATE TABLE detected_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    stealth_address VARCHAR(44) NOT NULL,
    amount BIGINT NOT NULL,
    ephemeral_public_key VARCHAR(44) NOT NULL,
    viewing_tag BYTEA NOT NULL,
    slot BIGINT NOT NULL,
    signature VARCHAR(88) NOT NULL UNIQUE,
    spent BOOLEAN NOT NULL DEFAULT FALSE,
    detected_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    INDEX idx_user_id (user_id),
    INDEX idx_stealth_address (stealth_address),
    INDEX idx_signature (signature),
    INDEX idx_spent (spent)
);
```

### Migrations

```rust
// crates/database/migrations/YYYYMMDD_create_stealth_tables.sql

-- Up
CREATE TABLE stealth_wallets (...);
CREATE TABLE stealth_payments (...);
CREATE TABLE stealth_scan_index (...);
CREATE TABLE detected_payments (...);

-- Down
DROP TABLE detected_payments;
DROP TABLE stealth_scan_index;
DROP TABLE stealth_payments;
DROP TABLE stealth_wallets;
```

---

## Service Integration

### 1. Initialize Stealth Services

```rust
// crates/api/src/main.rs

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize blockchain client
    let blockchain_client = Arc::new(SolanaClient::new(rpc_url)?);
    
    // Initialize secure storage
    #[cfg(target_os = "ios")]
    let storage = Arc::new(IosKeychainStorage::new());
    
    #[cfg(target_os = "android")]
    let storage = Arc::new(AndroidKeystoreStorage::new());
    
    // Initialize stealth services
    let stealth_service = Arc::new(StealthService::new(
        blockchain_client.clone(),
        storage.clone(),
    ));
    
    // Initialize BLE mesh (if available)
    let mesh_service = Arc::new(MeshService::new());
    
    // Add to app state
    let app_state = AppState {
        blockchain_client,
        stealth_service,
        mesh_service,
        // ... other services
    };
    
    // Start background tasks
    tokio::spawn(start_payment_queue_processor(app_state.clone()));
    tokio::spawn(start_periodic_scanner(app_state.clone()));
    
    // Start API server
    let app = Router::new()
        .route("/api/stealth/generate", post(generate_stealth_wallet))
        .route("/api/stealth/prepare-payment", post(prepare_stealth_payment))
        .route("/api/stealth/send", post(send_stealth_payment))
        .route("/api/stealth/scan", get(scan_stealth_payments))
        .route("/api/stealth/shield", post(shield_funds))
        .route("/api/stealth/unshield", post(unshield_funds))
        .route("/api/stealth/queue", get(get_payment_queue))
        .layer(Extension(app_state));
    
    axum::Server::bind(&"0.0.0.0:8080".parse()?)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}
```

### 2. Background Tasks

#### Payment Queue Processor

```rust
async fn start_payment_queue_processor(state: AppState) {
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        
        // Get all users with queued payments
        let users = state.db.get_users_with_queued_payments().await.unwrap_or_default();
        
        for user_id in users {
            if let Ok(mut wallet_manager) = state.get_wallet_manager(&user_id).await {
                if let Ok(mut queue) = state.get_payment_queue(&user_id).await {
                    match queue.process_queue().await {
                        Ok(results) => {
                            for result in results {
                                // Emit WebSocket events
                                match result.status {
                                    PaymentStatus::Settled(sig) => {
                                        state.ws_service.emit_payment_settled(
                                            &user_id,
                                            &result.payment_id.to_string(),
                                            &sig
                                        ).await;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Queue processing failed for user {}: {}", user_id, e);
                        }
                    }
                }
            }
        }
    }
}
```

#### Periodic Scanner

```rust
async fn start_periodic_scanner(state: AppState) {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        
        // Get all users with stealth wallets
        let users = state.db.get_users_with_stealth_wallets().await.unwrap_or_default();
        
        for user_id in users {
            if let Ok(mut wallet_manager) = state.get_wallet_manager(&user_id).await {
                match wallet_manager.scan_incoming().await {
                    Ok(payments) => {
                        for payment in payments {
                            // Store in database
                            state.db.store_detected_payment(&user_id, &payment).await.ok();
                            
                            // Emit WebSocket event
                            state.ws_service.emit_stealth_payment_detected(
                                &user_id,
                                &payment
                            ).await;
                        }
                    }
                    Err(e) => {
                        log::error!("Scanning failed for user {}: {}", user_id, e);
                    }
                }
            }
        }
    }
}
```


---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stealth_wallet_integration() {
        let blockchain_client = Arc::new(MockSolanaClient::new());
        let storage = Arc::new(MockStorage::new());
        
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let wallet_manager = StealthWalletManager::new(
            keypair,
            blockchain_client,
            storage,
        );
        
        let meta_address = wallet_manager.get_meta_address();
        assert!(meta_address.starts_with("stealth:1:"));
    }

    #[tokio::test]
    async fn test_api_endpoint_generate_wallet() {
        let app = create_test_app().await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/stealth/generate")
                    .header("Authorization", "Bearer test_token")
                    .body(Body::from(r#"{"version": 1}"#))
                    .unwrap()
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_payment_flow() {
    // Setup
    let receiver_keypair = StealthKeyPair::generate_standard().unwrap();
    let receiver_meta = receiver_keypair.to_meta_address();
    
    // Prepare payment
    let generator = StealthAddressGenerator::new();
    let output = generator.generate_stealth_address(&receiver_meta, None).await.unwrap();
    
    // Send payment (simulated)
    let blockchain_client = Arc::new(TestSolanaClient::new());
    blockchain_client.send_stealth_transaction(
        &output.stealth_address,
        1_000_000_000,
        &output.ephemeral_public_key,
        &output.viewing_tag,
        &payer_keypair,
    ).await.unwrap();
    
    // Scan for payment
    let scanner = StealthScanner::new(&receiver_keypair, blockchain_client);
    let payments = scanner.scan_for_payments(None, None).await.unwrap();
    
    assert_eq!(payments.len(), 1);
    assert_eq!(payments[0].amount, 1_000_000_000);
}
```

### Platform-Specific Tests

#### iOS Tests

```swift
// ios/Tests/StealthWalletTests.swift
import XCTest

class StealthWalletTests: XCTestCase {
    func testGenerateKeyPair() {
        let bridge = StealthWalletBridge()
        let metaAddress = bridge.generateKeyPair()
        
        XCTAssertTrue(metaAddress.hasPrefix("stealth:1:"))
    }
    
    func testKeychainStorage() {
        let storage = IosKeychainStorage()
        // Test keychain operations
    }
}
```

#### Android Tests

```kotlin
// android/src/test/kotlin/StealthWalletTest.kt
class StealthWalletTest {
    @Test
    fun testGenerateKeyPair() {
        val bridge = StealthWalletBridge()
        val metaAddress = bridge.generateKeyPair()
        
        assertTrue(metaAddress.startsWith("stealth:1:"))
    }
    
    @Test
    fun testKeystoreStorage() {
        val storage = AndroidKeystoreStorage()
        // Test keystore operations
    }
}
```

---

## Deployment

### Environment Variables

```bash
# .env
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_WS_URL=wss://api.mainnet-beta.solana.com
DATABASE_URL=postgresql://user:pass@localhost/stealth_db
REDIS_URL=redis://localhost:6379
JWT_SECRET=your_jwt_secret_here

# BLE Mesh (optional)
BLE_MESH_ENABLED=true
BLE_MESH_TTL=5
BLE_MESH_MAX_PEERS=10

# Payment Queue
PAYMENT_QUEUE_MAX_SIZE=1000
PAYMENT_QUEUE_RETRY_LIMIT=5
PAYMENT_QUEUE_PROCESS_INTERVAL=30

# Scanning
SCAN_INTERVAL=60
SCAN_BATCH_SIZE=1000
```

### Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Build for production
RUN cargo build --release --package api

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/api /usr/local/bin/api

EXPOSE 8080

CMD ["api"]
```

```yaml
# docker-compose.yml
version: '3.8'

services:
  api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - SOLANA_RPC_URL=${SOLANA_RPC_URL}
      - DATABASE_URL=${DATABASE_URL}
      - REDIS_URL=redis://redis:6379
    depends_on:
      - postgres
      - redis
    restart: unless-stopped

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=stealth_db
      - POSTGRES_USER=stealth_user
      - POSTGRES_PASSWORD=${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    restart: unless-stopped

volumes:
  postgres_data:
```

### Kubernetes Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: stealth-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: stealth-api
  template:
    metadata:
      labels:
        app: stealth-api
    spec:
      containers:
      - name: api
        image: yourregistry/stealth-api:latest
        ports:
        - containerPort: 8080
        env:
        - name: SOLANA_RPC_URL
          valueFrom:
            secretKeyRef:
              name: stealth-secrets
              key: solana-rpc-url
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: stealth-secrets
              key: database-url
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: stealth-api
spec:
  selector:
    app: stealth-api
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
```

### Database Migrations

```bash
# Run migrations
diesel migration run --database-url $DATABASE_URL

# Or using sqlx
sqlx migrate run --database-url $DATABASE_URL
```

### Monitoring

```rust
// Add Prometheus metrics
use prometheus::{Counter, Histogram, Registry};

lazy_static! {
    static ref STEALTH_PAYMENTS_TOTAL: Counter = Counter::new(
        "stealth_payments_total",
        "Total number of stealth payments"
    ).unwrap();
    
    static ref STEALTH_SCAN_DURATION: Histogram = Histogram::new(
        "stealth_scan_duration_seconds",
        "Duration of blockchain scanning"
    ).unwrap();
    
    static ref PAYMENT_QUEUE_SIZE: Gauge = Gauge::new(
        "payment_queue_size",
        "Current payment queue size"
    ).unwrap();
}

// Expose metrics endpoint
pub async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    encoder.encode_to_string(&metric_families).unwrap()
}
```

---

## Security Considerations

### 1. Key Storage

- **iOS**: Keys stored in iOS Keychain with `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`
- **Android**: Keys stored in Android Keystore with hardware-backed encryption
- **Never** store keys in plaintext or application preferences

### 2. API Security

```rust
// Rate limiting
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

let governor_conf = Box::new(
    GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(20)
        .finish()
        .unwrap()
);

let app = Router::new()
    .route("/api/stealth/*", /* handlers */)
    .layer(GovernorLayer { config: governor_conf });
```

### 3. Input Validation

```rust
// Validate meta-address format
fn validate_meta_address(addr: &str) -> Result<()> {
    if !addr.starts_with("stealth:") {
        return Err(Error::InvalidMetaAddress);
    }
    
    let parts: Vec<&str> = addr.split(':').collect();
    if parts.len() < 4 {
        return Err(Error::InvalidMetaAddress);
    }
    
    // Validate version
    let version: u8 = parts[1].parse()
        .map_err(|_| Error::InvalidMetaAddress)?;
    
    if version != 1 && version != 2 {
        return Err(Error::UnsupportedVersion);
    }
    
    Ok(())
}
```

### 4. Logging

```rust
// Never log sensitive data
log::info!("Payment prepared for user {}", user_id);  // OK
log::debug!("Spending key: {:?}", spending_key);      // NEVER DO THIS
```

---

## Performance Optimization

### 1. Caching

```rust
use redis::AsyncCommands;

// Cache stealth addresses
pub async fn get_cached_stealth_address(
    redis: &mut redis::aio::Connection,
    cache_key: &str,
) -> Option<StealthAddressOutput> {
    let cached: Option<String> = redis.get(cache_key).await.ok()?;
    serde_json::from_str(&cached?).ok()
}

pub async fn cache_stealth_address(
    redis: &mut redis::aio::Connection,
    cache_key: &str,
    output: &StealthAddressOutput,
) -> Result<()> {
    let serialized = serde_json::to_string(output)?;
    redis.set_ex(cache_key, serialized, 3600).await?;
    Ok(())
}
```

### 2. Batch Scanning

```rust
// Scan in batches to avoid overwhelming RPC
pub async fn scan_in_batches(
    scanner: &mut StealthScanner,
    from_slot: u64,
    to_slot: u64,
    batch_size: u64,
) -> StealthResult<Vec<DetectedPayment>> {
    let mut all_payments = Vec::new();
    let mut current_slot = from_slot;
    
    while current_slot < to_slot {
        let batch_end = (current_slot + batch_size).min(to_slot);
        let payments = scanner.scan_for_payments(
            Some(current_slot),
            Some(batch_end)
        ).await?;
        
        all_payments.extend(payments);
        current_slot = batch_end;
        
        // Rate limiting
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    Ok(all_payments)
}
```

### 3. Connection Pooling

```rust
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect(&database_url)
    .await?;
```

---

## Troubleshooting

### Common Issues

#### 1. BLE Not Working

**Symptoms**: BLE initialization fails, peers not discovered

**Solutions**:
- Check Bluetooth is enabled on device
- Verify permissions granted (iOS: Info.plist, Android: runtime permissions)
- Check device supports BLE (Bluetooth 4.0+)
- Restart Bluetooth adapter

#### 2. Payments Not Settling

**Symptoms**: Payments stuck in "queued" status

**Solutions**:
- Check network connectivity
- Verify Solana RPC endpoint is accessible
- Check payment queue processor is running
- Review logs for errors

#### 3. Scanning Not Finding Payments

**Symptoms**: Known payments not detected during scanning

**Solutions**:
- Verify viewing key is correct
- Check scan slot range includes payment slot
- Ensure transaction includes stealth metadata
- Review blockchain client configuration

#### 4. Storage Errors

**Symptoms**: Cannot save/load keys

**Solutions**:
- iOS: Check Keychain access permissions
- Android: Verify Keystore is available
- Check device has sufficient storage
- Review error logs for specific issues

---

## Support and Resources

### Documentation
- API Documentation: `crates/stealth/API_DOCUMENTATION.md`
- BLE Mesh Documentation: `crates/ble-mesh/API_DOCUMENTATION.md`
- Requirements: `.kiro/specs/ble-mesh-stealth-transfers/requirements.md`
- Design: `.kiro/specs/ble-mesh-stealth-transfers/design.md`

### Examples
- Stealth Wallet: `crates/stealth/examples/stealth_wallet.rs`
- Mesh Payment: `crates/stealth/examples/mesh_payment.rs`
- Shield/Unshield: `crates/stealth/examples/shield_unshield.rs`

### External Resources
- EIP-5564: https://eips.ethereum.org/EIPS/eip-5564
- Solana Documentation: https://docs.solana.com/
- btleplug: https://github.com/deviceplug/btleplug

---

## Changelog

### Version 0.1.0 (Initial Release)
- Stealth address generation and scanning
- BLE mesh networking integration
- Payment queue with auto-settlement
- iOS and Android platform support
- REST API and WebSocket events
- QR code support
- Hybrid post-quantum mode

