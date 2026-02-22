# Stealth Address API Documentation

## Overview

This document provides comprehensive API documentation for the stealth address crate, including usage examples, error handling patterns, and integration guidelines.

## Table of Contents

1. [Core Cryptographic Primitives](#core-cryptographic-primitives)
2. [Key Pair Management](#key-pair-management)
3. [Stealth Address Generation](#stealth-address-generation)
4. [Blockchain Scanning](#blockchain-scanning)
5. [Wallet Manager](#wallet-manager)
6. [Payment Queue](#payment-queue)
7. [Network Monitoring](#network-monitoring)
8. [Secure Storage](#secure-storage)
9. [QR Code Support](#qr-code-support)
10. [Hybrid Post-Quantum Mode](#hybrid-post-quantum-mode)
11. [Error Handling](#error-handling)

---

## Core Cryptographic Primitives

### `StealthCrypto`

Low-level cryptographic operations for stealth addresses.

#### Methods

##### `ed25519_to_curve25519`

Convert Ed25519 public key to Curve25519 format for ECDH operations.

```rust
pub fn ed25519_to_curve25519(ed_pk: &[u8; 32]) -> StealthResult<[u8; 32]>
```

**Parameters:**
- `ed_pk`: Ed25519 public key (32 bytes)

**Returns:**
- `Ok([u8; 32])`: Curve25519 public key
- `Err(StealthError::InvalidCurvePoint)`: Invalid Ed25519 key

**Example:**
```rust
use stealth::StealthCrypto;

let ed_pk = solana_keypair.pubkey().to_bytes();
let curve_pk = StealthCrypto::ed25519_to_curve25519(&ed_pk)?;
```


##### `ecdh`

Perform Elliptic Curve Diffie-Hellman key exchange.

```rust
pub fn ecdh(secret_key: &[u8; 32], public_key: &[u8; 32]) -> StealthResult<[u8; 32]>
```

**Parameters:**
- `secret_key`: Your Curve25519 secret key (32 bytes)
- `public_key`: Their Curve25519 public key (32 bytes)

**Returns:**
- `Ok([u8; 32])`: Shared secret (32 bytes)
- `Err(StealthError::CryptoError)`: ECDH computation failed

**Example:**
```rust
let my_secret = [1u8; 32];
let their_public = [2u8; 32];
let shared_secret = StealthCrypto::ecdh(&my_secret, &their_public)?;
```

**Note:** ECDH is symmetric: `ecdh(a_secret, b_public) == ecdh(b_secret, a_public)`

##### `point_add`

Add two points on the edwards25519 curve.

```rust
pub fn point_add(point_a: &[u8; 32], point_b: &[u8; 32]) -> StealthResult<[u8; 32]>
```

**Parameters:**
- `point_a`: First edwards25519 point (32 bytes)
- `point_b`: Second edwards25519 point (32 bytes)

**Returns:**
- `Ok([u8; 32])`: Sum of the two points
- `Err(StealthError::InvalidCurvePoint)`: Invalid point

**Example:**
```rust
let spending_pk = keypair.spending_public_key().to_bytes();
let derived_point = [3u8; 32];
let stealth_address = StealthCrypto::point_add(&spending_pk, &derived_point)?;
```

##### `derive_viewing_tag`

Compute viewing tag from shared secret for efficient scanning.

```rust
pub fn derive_viewing_tag(shared_secret: &[u8; 32]) -> [u8; 4]
```

**Parameters:**
- `shared_secret`: ECDH shared secret (32 bytes)

**Returns:**
- `[u8; 4]`: First 4 bytes of SHA256(shared_secret)

**Example:**
```rust
let viewing_tag = StealthCrypto::derive_viewing_tag(&shared_secret);
// Use viewing_tag for efficient blockchain filtering
```


##### `encrypt_mesh_payload` / `decrypt_mesh_payload`

Encrypt/decrypt payloads for mesh relay using XChaCha20-Poly1305.

```rust
pub fn encrypt_mesh_payload(
    plaintext: &[u8],
    shared_key: &[u8; 32],
    nonce: &[u8; 24]
) -> StealthResult<Vec<u8>>

pub fn decrypt_mesh_payload(
    ciphertext: &[u8],
    shared_key: &[u8; 32],
    nonce: &[u8; 24]
) -> StealthResult<Vec<u8>>
```

**Parameters:**
- `plaintext`/`ciphertext`: Data to encrypt/decrypt
- `shared_key`: 32-byte encryption key
- `nonce`: 24-byte nonce (must be unique per message)

**Returns:**
- `Ok(Vec<u8>)`: Encrypted/decrypted data
- `Err(StealthError::EncryptionFailed)`: Encryption failed
- `Err(StealthError::DecryptionFailed)`: Decryption or authentication failed

**Example:**
```rust
use rand::RngCore;

let mut nonce = [0u8; 24];
rand::thread_rng().fill_bytes(&mut nonce);

let plaintext = b"payment request data";
let ciphertext = StealthCrypto::encrypt_mesh_payload(plaintext, &shared_key, &nonce)?;

// Later, decrypt
let decrypted = StealthCrypto::decrypt_mesh_payload(&ciphertext, &shared_key, &nonce)?;
assert_eq!(decrypted, plaintext);
```

---

## Key Pair Management

### `StealthKeyPair`

Manages stealth address key pairs with separate spending and viewing keys.

#### Methods

##### `generate_standard`

Generate a new standard stealth key pair (version 1).

```rust
pub fn generate_standard() -> StealthResult<Self>
```

**Returns:**
- `Ok(StealthKeyPair)`: New key pair with version 1
- `Err(StealthError)`: Key generation failed

**Example:**
```rust
use stealth::StealthKeyPair;

let keypair = StealthKeyPair::generate_standard()?;
let meta_address = keypair.to_meta_address();
println!("Share this meta-address: {}", meta_address);
```


##### `to_meta_address`

Format key pair as a meta-address string.

```rust
pub fn to_meta_address(&self) -> String
```

**Returns:**
- Standard format: `stealth:1:<spending_pk>:<viewing_pk>`
- Hybrid format: `stealth:2:<spending_pk>:<viewing_pk>:<kyber_pk>`

**Example:**
```rust
let meta_address = keypair.to_meta_address();
// stealth:1:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp...
```

##### `from_meta_address`

Parse a meta-address string into a key pair.

```rust
pub fn from_meta_address(meta_addr: &str) -> StealthResult<Self>
```

**Parameters:**
- `meta_addr`: Meta-address string

**Returns:**
- `Ok(StealthKeyPair)`: Parsed key pair
- `Err(StealthError::InvalidMetaAddress)`: Invalid format or version

**Example:**
```rust
let meta_addr = "stealth:1:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp...";
let keypair = StealthKeyPair::from_meta_address(meta_addr)?;
```

##### `spending_public_key` / `viewing_public_key`

Get public keys from the key pair.

```rust
pub fn spending_public_key(&self) -> Pubkey
pub fn viewing_public_key(&self) -> Pubkey
```

**Returns:**
- Solana `Pubkey` for spending or viewing

**Example:**
```rust
let spending_pk = keypair.spending_public_key();
let viewing_pk = keypair.viewing_public_key();
```

##### `export_encrypted` / `import_encrypted`

Backup and restore key pairs with password encryption.

```rust
pub fn export_encrypted(&self, password: &str) -> StealthResult<Vec<u8>>
pub fn import_encrypted(data: &[u8], password: &str) -> StealthResult<Self>
```

**Parameters:**
- `password`: User password for encryption/decryption
- `data`: Encrypted key pair data

**Returns:**
- `Ok(Vec<u8>)` or `Ok(StealthKeyPair)`: Encrypted data or restored key pair
- `Err(StealthError::EncryptionFailed)`: Export failed
- `Err(StealthError::DecryptionFailed)`: Wrong password or corrupted data

**Example:**
```rust
// Backup
let password = "secure_password_123";
let encrypted_backup = keypair.export_encrypted(password)?;
std::fs::write("keypair_backup.enc", &encrypted_backup)?;

// Restore
let encrypted_data = std::fs::read("keypair_backup.enc")?;
let restored_keypair = StealthKeyPair::import_encrypted(&encrypted_data, password)?;
```


---

## Stealth Address Generation

### `StealthAddressGenerator`

Sender-side stealth address derivation.

#### Methods

##### `new`

Create a new generator instance.

```rust
pub fn new() -> Self
```

##### `generate_stealth_address`

Generate a one-time stealth address for a receiver.

```rust
pub async fn generate_stealth_address(
    &self,
    receiver_meta_address: &str,
    ephemeral_keypair: Option<Keypair>
) -> StealthResult<StealthAddressOutput>
```

**Parameters:**
- `receiver_meta_address`: Receiver's meta-address string
- `ephemeral_keypair`: Optional ephemeral key (generated if None)

**Returns:**
- `Ok(StealthAddressOutput)`: Contains stealth address, ephemeral public key, viewing tag, and shared secret
- `Err(StealthError::InvalidMetaAddress)`: Invalid meta-address format

**Example:**
```rust
use stealth::StealthAddressGenerator;

let generator = StealthAddressGenerator::new();
let receiver_meta = "stealth:1:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp...";

let output = generator.generate_stealth_address(receiver_meta, None).await?;

println!("Send payment to: {}", output.stealth_address);
println!("Include ephemeral key: {}", output.ephemeral_public_key);
println!("Viewing tag: {:?}", output.viewing_tag);
```

### `StealthAddressOutput`

Output from stealth address generation.

```rust
pub struct StealthAddressOutput {
    pub stealth_address: Pubkey,
    pub ephemeral_public_key: Pubkey,
    pub viewing_tag: [u8; 4],
    pub shared_secret: [u8; 32],
}
```

**Fields:**
- `stealth_address`: One-time payment address
- `ephemeral_public_key`: Must be published on-chain for receiver scanning
- `viewing_tag`: First 4 bytes of shared secret hash (for efficient scanning)
- `shared_secret`: ECDH shared secret (keep private)

---

## Blockchain Scanning

### `StealthScanner`

Receiver-side blockchain scanning for incoming stealth payments.

#### Methods

##### `new`

Create a new scanner instance.

```rust
pub fn new(
    keypair: &StealthKeyPair,
    blockchain_client: Arc<SolanaClient>
) -> Self
```

**Parameters:**
- `keypair`: Your stealth key pair (uses viewing key for scanning)
- `blockchain_client`: Solana RPC client


##### `scan_for_payments`

Scan blockchain for incoming stealth payments.

```rust
pub async fn scan_for_payments(
    &mut self,
    from_slot: Option<u64>,
    to_slot: Option<u64>
) -> StealthResult<Vec<DetectedPayment>>
```

**Parameters:**
- `from_slot`: Starting slot (uses last scan index if None)
- `to_slot`: Ending slot (uses current slot if None)

**Returns:**
- `Ok(Vec<DetectedPayment>)`: List of detected payments
- `Err(StealthError::BlockchainError)`: RPC error

**Example:**
```rust
use stealth::StealthScanner;
use std::sync::Arc;

let scanner = StealthScanner::new(&keypair, Arc::clone(&blockchain_client));

// Scan recent blocks
let payments = scanner.scan_for_payments(None, None).await?;

for payment in payments {
    println!("Received {} lamports at {}", payment.amount, payment.stealth_address);
    println!("Ephemeral key: {}", payment.ephemeral_public_key);
}
```

##### `derive_spending_key`

Derive private key for spending a detected payment.

```rust
pub fn derive_spending_key(
    &self,
    ephemeral_public_key: &Pubkey,
    spending_secret_key: &[u8; 32]
) -> StealthResult<Keypair>
```

**Parameters:**
- `ephemeral_public_key`: From detected payment
- `spending_secret_key`: Your spending secret key

**Returns:**
- `Ok(Keypair)`: Private key for spending the stealth address
- `Err(StealthError)`: Derivation failed

**Example:**
```rust
let spending_keypair = scanner.derive_spending_key(
    &payment.ephemeral_public_key,
    &keypair.spending_secret_key()
)?;

// Use spending_keypair to transfer funds from stealth address
```

### `DetectedPayment`

Information about a detected stealth payment.

```rust
pub struct DetectedPayment {
    pub stealth_address: Pubkey,
    pub amount: u64,
    pub ephemeral_public_key: Pubkey,
    pub viewing_tag: [u8; 4],
    pub slot: u64,
    pub signature: Signature,
}
```

---

## Wallet Manager

### `StealthWalletManager`

High-level API for stealth wallet operations.

#### Methods

##### `new`

Create a new wallet manager.

```rust
pub fn new(
    keypair: StealthKeyPair,
    blockchain_client: Arc<SolanaClient>,
    storage: Arc<dyn SecureStorage>
) -> Self
```


##### `get_meta_address`

Get your meta-address for receiving payments.

```rust
pub fn get_meta_address(&self) -> String
```

**Example:**
```rust
let meta_address = wallet_manager.get_meta_address();
// Share this with senders
```

##### `prepare_payment`

Prepare a stealth payment to a receiver.

```rust
pub fn prepare_payment(
    &self,
    receiver_meta_address: &str,
    amount: u64
) -> StealthResult<PreparedPayment>
```

**Parameters:**
- `receiver_meta_address`: Receiver's meta-address
- `amount`: Payment amount in lamports

**Returns:**
- `Ok(PreparedPayment)`: Payment ready to send
- `Err(StealthError)`: Invalid meta-address or preparation failed

**Example:**
```rust
let prepared = wallet_manager.prepare_payment(
    "stealth:1:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp...",
    1_000_000_000 // 1 SOL
)?;
```

##### `send_payment`

Send or queue a prepared payment.

```rust
pub async fn send_payment(
    &mut self,
    prepared: PreparedPayment
) -> StealthResult<PaymentStatus>
```

**Parameters:**
- `prepared`: Payment from `prepare_payment()`

**Returns:**
- `Ok(PaymentStatus::Settled(signature))`: Payment sent on-chain
- `Ok(PaymentStatus::Queued)`: Payment queued (offline)
- `Err(StealthError)`: Payment failed

**Example:**
```rust
match wallet_manager.send_payment(prepared).await? {
    PaymentStatus::Settled(sig) => {
        println!("Payment sent: {}", sig);
    }
    PaymentStatus::Queued => {
        println!("Payment queued (offline)");
    }
    _ => {}
}
```

##### `scan_incoming`

Scan for incoming stealth payments.

```rust
pub async fn scan_incoming(&mut self) -> StealthResult<Vec<DetectedPayment>>
```

**Example:**
```rust
let payments = wallet_manager.scan_incoming().await?;
println!("Found {} payments", payments.len());
```

##### `shield`

Convert regular funds to stealth address (break transaction graph).

```rust
pub async fn shield(
    &mut self,
    amount: u64,
    source_keypair: &Keypair
) -> StealthResult<Signature>
```

**Parameters:**
- `amount`: Amount to shield (lamports)
- `source_keypair`: Source wallet keypair

**Returns:**
- `Ok(Signature)`: Transaction signature
- `Err(StealthError)`: Shield operation failed

**Example:**
```rust
let sig = wallet_manager.shield(
    5_000_000_000, // 5 SOL
    &source_keypair
).await?;
println!("Shielded funds: {}", sig);
```


##### `unshield`

Convert stealth funds back to regular address.

```rust
pub async fn unshield(
    &mut self,
    detected_payment: &DetectedPayment,
    destination: &Pubkey
) -> StealthResult<Signature>
```

**Parameters:**
- `detected_payment`: Payment from scanning
- `destination`: Regular address to receive funds

**Returns:**
- `Ok(Signature)`: Transaction signature
- `Err(StealthError)`: Unshield operation failed

**Example:**
```rust
let sig = wallet_manager.unshield(
    &detected_payment,
    &regular_wallet_pubkey
).await?;
println!("Unshielded funds: {}", sig);
```

---

## Payment Queue

### `PaymentQueue`

Manages offline payment queue with automatic settlement.

#### Methods

##### `new`

Create a new payment queue.

```rust
pub fn new(
    storage: Arc<dyn SecureStorage>,
    blockchain_client: Arc<SolanaClient>
) -> Self
```

##### `enqueue`

Add a payment to the queue.

```rust
pub async fn enqueue(&mut self, payment: PreparedPayment) -> StealthResult<PaymentId>
```

**Returns:**
- `Ok(PaymentId)`: Unique payment identifier
- `Err(StealthError::QueueFull)`: Queue at capacity (1000 max)

**Example:**
```rust
let payment_id = payment_queue.enqueue(prepared_payment).await?;
println!("Payment queued: {}", payment_id);
```

##### `get_status`

Check payment status.

```rust
pub fn get_status(&self, id: &PaymentId) -> Option<PaymentStatus>
```

**Returns:**
- `Some(PaymentStatus)`: Current status
- `None`: Payment not found

**Example:**
```rust
match payment_queue.get_status(&payment_id) {
    Some(PaymentStatus::Queued) => println!("Waiting for network"),
    Some(PaymentStatus::Settling) => println!("Processing..."),
    Some(PaymentStatus::Settled(sig)) => println!("Completed: {}", sig),
    Some(PaymentStatus::Failed(err)) => println!("Failed: {}", err),
    None => println!("Payment not found"),
}
```

##### `process_queue`

Manually process queued payments (called automatically when online).

```rust
pub async fn process_queue(&mut self) -> StealthResult<Vec<SettlementResult>>
```

**Example:**
```rust
let results = payment_queue.process_queue().await?;
for result in results {
    println!("Payment {}: {:?}", result.payment_id, result.status);
}
```


##### `start_auto_settlement`

Start background task for automatic settlement.

```rust
pub fn start_auto_settlement(&self) -> JoinHandle<()>
```

**Example:**
```rust
let handle = payment_queue.start_auto_settlement();
// Queue will automatically process when network is available
```

### `PaymentStatus`

Payment state in the queue.

```rust
pub enum PaymentStatus {
    Queued,
    Settling,
    Settled(Signature),
    Failed(String),
}
```

---

## Network Monitoring

### `NetworkMonitor`

Monitors network connectivity for auto-settlement.

#### Methods

##### `new`

Create a new network monitor.

```rust
pub fn new() -> Self
```

##### `start`

Start monitoring network connectivity.

```rust
pub fn start(&self)
```

##### `is_online`

Check current connectivity status.

```rust
pub fn is_online(&self) -> bool
```

**Example:**
```rust
use stealth::NetworkMonitor;

let monitor = NetworkMonitor::new();
monitor.start();

if monitor.is_online() {
    println!("Network available");
}
```

##### `on_connectivity_change`

Register callback for connectivity changes.

```rust
pub fn on_connectivity_change<F>(&mut self, callback: F)
where
    F: Fn(bool) + Send + Sync + 'static
```

**Example:**
```rust
monitor.on_connectivity_change(|is_online| {
    if is_online {
        println!("Network restored - processing queue");
    } else {
        println!("Network lost - queueing payments");
    }
});
```

---

## Secure Storage

### `SecureStorage` Trait

Platform-agnostic secure storage interface.

```rust
pub trait SecureStorage: Send + Sync {
    async fn store_keypair(&self, keypair: &StealthKeyPair) -> StealthResult<()>;
    async fn load_keypair(&self) -> StealthResult<Option<StealthKeyPair>>;
    async fn store_queue(&self, queue: &[QueuedPayment]) -> StealthResult<()>;
    async fn load_queue(&self) -> StealthResult<Vec<QueuedPayment>>;
    async fn store_scan_index(&self, index: u64) -> StealthResult<()>;
    async fn load_scan_index(&self) -> StealthResult<Option<u64>>;
}
```

**Platform Implementations:**
- iOS: `IosKeychainStorage` (uses iOS Keychain)
- Android: `AndroidKeystoreStorage` (uses Android Keystore)


**Example:**
```rust
use stealth::storage::IosKeychainStorage;

let storage = Arc::new(IosKeychainStorage::new());
storage.store_keypair(&keypair).await?;

// Later
let loaded_keypair = storage.load_keypair().await?;
```

---

## QR Code Support

### `QrCodeHandler`

QR code encoding/decoding for meta-addresses.

#### Methods

##### `encode_meta_address`

Encode meta-address as QR code.

```rust
pub fn encode_meta_address(meta_address: &str) -> StealthResult<Vec<u8>>
```

**Parameters:**
- `meta_address`: Meta-address string

**Returns:**
- `Ok(Vec<u8>)`: PNG image data
- `Err(StealthError)`: Encoding failed

**Example:**
```rust
use stealth::QrCodeHandler;

let meta_address = keypair.to_meta_address();
let qr_png = QrCodeHandler::encode_meta_address(&meta_address)?;

// Save or display QR code
std::fs::write("my_stealth_address.png", qr_png)?;
```

##### `decode_meta_address`

Decode meta-address from QR code image.

```rust
pub fn decode_meta_address(image_data: &[u8]) -> StealthResult<String>
```

**Parameters:**
- `image_data`: QR code image (PNG, JPEG, etc.)

**Returns:**
- `Ok(String)`: Decoded meta-address
- `Err(StealthError::QrDecodeFailed)`: Invalid QR code

**Example:**
```rust
let image_data = std::fs::read("scanned_qr.png")?;
let meta_address = QrCodeHandler::decode_meta_address(&image_data)?;

// Use meta-address to prepare payment
let prepared = wallet_manager.prepare_payment(&meta_address, amount)?;
```

---

## Hybrid Post-Quantum Mode

### `HybridStealthKeyPair`

Post-quantum resistant stealth key pair using ML-KEM-768 + X25519.

#### Methods

##### `generate_hybrid`

Generate a hybrid key pair (version 2).

```rust
pub fn generate_hybrid() -> StealthResult<Self>
```

**Example:**
```rust
use stealth::HybridStealthKeyPair;

let hybrid_keypair = HybridStealthKeyPair::generate_hybrid()?;
let meta_address = hybrid_keypair.to_meta_address();
// stealth:2:...:...:... (includes Kyber public key)
```

##### `to_meta_address`

Format as hybrid meta-address.

```rust
pub fn to_meta_address(&self) -> String
```

**Returns:**
- Format: `stealth:2:<spending_pk>:<viewing_pk>:<kyber_pk>`

### `StealthAddressGenerator::generate_hybrid_stealth_address`

Generate hybrid stealth address with post-quantum security.

```rust
pub fn generate_hybrid_stealth_address(
    &self,
    receiver_meta_address: &str,
    ephemeral_keypair: Option<Keypair>
) -> StealthResult<HybridStealthAddressOutput>
```

**Returns:**
- `Ok(HybridStealthAddressOutput)`: Contains base output + Kyber ciphertext

**Example:**
```rust
let generator = StealthAddressGenerator::new();
let hybrid_output = generator.generate_hybrid_stealth_address(
    "stealth:2:...:...:...",
    None
)?;

// hybrid_output.base contains standard fields
// hybrid_output.kyber_ciphertext contains post-quantum encapsulation
```


---

## Error Handling

### `StealthError`

All errors in the stealth crate.

```rust
pub enum StealthError {
    InvalidMetaAddress(String),
    InvalidCurvePoint(String),
    CryptoError(String),
    EncryptionFailed(String),
    DecryptionFailed(String),
    BlockchainError(String),
    StorageError(String),
    QrEncodeFailed(String),
    QrDecodeFailed(String),
    QueueFull,
    PaymentNotFound,
    NetworkError(String),
}
```

### Error Handling Patterns

#### Pattern 1: Propagate with `?`

```rust
pub async fn my_function() -> StealthResult<()> {
    let keypair = StealthKeyPair::generate_standard()?;
    let output = generator.generate_stealth_address(&meta_addr, None).await?;
    Ok(())
}
```

#### Pattern 2: Match on specific errors

```rust
match wallet_manager.send_payment(prepared).await {
    Ok(PaymentStatus::Settled(sig)) => {
        println!("Success: {}", sig);
    }
    Ok(PaymentStatus::Queued) => {
        println!("Queued for later");
    }
    Err(StealthError::BlockchainError(e)) => {
        eprintln!("RPC error: {}", e);
        // Retry logic
    }
    Err(StealthError::QueueFull) => {
        eprintln!("Queue full, wait for settlements");
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

#### Pattern 3: Convert to application errors

```rust
use stealth::StealthError;

#[derive(Debug)]
pub enum AppError {
    Stealth(StealthError),
    // ... other errors
}

impl From<StealthError> for AppError {
    fn from(e: StealthError) -> Self {
        AppError::Stealth(e)
    }
}

pub async fn app_function() -> Result<(), AppError> {
    let keypair = StealthKeyPair::generate_standard()?; // Auto-converts
    Ok(())
}
```

### Retry Strategies

#### Blockchain Operations

```rust
use tokio::time::{sleep, Duration};

async fn send_with_retry(
    wallet_manager: &mut StealthWalletManager,
    prepared: PreparedPayment,
    max_retries: u32
) -> StealthResult<Signature> {
    let mut retries = 0;
    
    loop {
        match wallet_manager.send_payment(prepared.clone()).await {
            Ok(PaymentStatus::Settled(sig)) => return Ok(sig),
            Err(StealthError::BlockchainError(_)) if retries < max_retries => {
                retries += 1;
                let backoff = Duration::from_millis(100 * 2u64.pow(retries));
                sleep(backoff).await;
            }
            Err(e) => return Err(e),
            _ => {}
        }
    }
}
```

#### Storage Operations

```rust
async fn store_with_retry(
    storage: &dyn SecureStorage,
    keypair: &StealthKeyPair
) -> StealthResult<()> {
    for attempt in 0..3 {
        match storage.store_keypair(keypair).await {
            Ok(()) => return Ok(()),
            Err(StealthError::StorageError(_)) if attempt < 2 => {
                sleep(Duration::from_secs(1)).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(StealthError::StorageError("Max retries exceeded".into()))
}
```


---

## Common Usage Patterns

### Complete Sender Flow

```rust
use stealth::{StealthAddressGenerator, StealthWalletManager};
use std::sync::Arc;

async fn send_stealth_payment(
    wallet_manager: &mut StealthWalletManager,
    receiver_meta_address: &str,
    amount_lamports: u64
) -> StealthResult<()> {
    // 1. Prepare payment
    let prepared = wallet_manager.prepare_payment(receiver_meta_address, amount_lamports)?;
    
    println!("Sending {} lamports to stealth address: {}", 
        amount_lamports, prepared.stealth_address);
    
    // 2. Send or queue payment
    match wallet_manager.send_payment(prepared).await? {
        PaymentStatus::Settled(sig) => {
            println!("Payment sent on-chain: {}", sig);
        }
        PaymentStatus::Queued => {
            println!("Payment queued (offline) - will settle automatically");
        }
        _ => {}
    }
    
    Ok(())
}
```

### Complete Receiver Flow

```rust
async fn receive_stealth_payments(
    wallet_manager: &mut StealthWalletManager
) -> StealthResult<()> {
    // 1. Share your meta-address
    let meta_address = wallet_manager.get_meta_address();
    println!("Share this address: {}", meta_address);
    
    // 2. Periodically scan for payments
    let payments = wallet_manager.scan_incoming().await?;
    
    for payment in payments {
        println!("Received {} lamports", payment.amount);
        println!("Stealth address: {}", payment.stealth_address);
        
        // 3. Optionally unshield to regular address
        let regular_address = /* your regular wallet */;
        let sig = wallet_manager.unshield(&payment, &regular_address).await?;
        println!("Unshielded: {}", sig);
    }
    
    Ok(())
}
```

### Shield/Unshield Flow

```rust
async fn privacy_flow(
    wallet_manager: &mut StealthWalletManager,
    source_keypair: &Keypair,
    amount: u64
) -> StealthResult<()> {
    // 1. Shield: Break transaction graph
    println!("Shielding {} lamports...", amount);
    let shield_sig = wallet_manager.shield(amount, source_keypair).await?;
    println!("Shielded: {}", shield_sig);
    
    // 2. Wait and scan
    tokio::time::sleep(Duration::from_secs(30)).await;
    let payments = wallet_manager.scan_incoming().await?;
    
    // 3. Unshield to new address
    if let Some(payment) = payments.first() {
        let new_address = Pubkey::new_unique();
        let unshield_sig = wallet_manager.unshield(payment, &new_address).await?;
        println!("Unshielded to new address: {}", unshield_sig);
        println!("Transaction graph broken - no on-chain linkage");
    }
    
    Ok(())
}
```

### Offline Payment Queue

```rust
async fn offline_payment_example(
    wallet_manager: &mut StealthWalletManager,
    payment_queue: &mut PaymentQueue,
    network_monitor: &NetworkMonitor
) -> StealthResult<()> {
    // 1. Prepare payment
    let prepared = wallet_manager.prepare_payment(receiver_meta, amount)?;
    
    // 2. Check network status
    if !network_monitor.is_online() {
        println!("Offline - queueing payment");
        let payment_id = payment_queue.enqueue(prepared).await?;
        println!("Payment queued: {}", payment_id);
        
        // 3. Monitor status
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                if let Some(status) = payment_queue.get_status(&payment_id) {
                    match status {
                        PaymentStatus::Settled(sig) => {
                            println!("Payment settled: {}", sig);
                            break;
                        }
                        PaymentStatus::Failed(err) => {
                            eprintln!("Payment failed: {}", err);
                            break;
                        }
                        _ => {}
                    }
                }
            }
        });
    } else {
        // Send immediately
        wallet_manager.send_payment(prepared).await?;
    }
    
    Ok(())
}
```


### QR Code Integration

```rust
use stealth::QrCodeHandler;

async fn qr_code_payment_flow() -> StealthResult<()> {
    // Receiver: Generate and display QR code
    let keypair = StealthKeyPair::generate_standard()?;
    let meta_address = keypair.to_meta_address();
    let qr_png = QrCodeHandler::encode_meta_address(&meta_address)?;
    
    // Display QR code in UI or save to file
    std::fs::write("receive_address.png", qr_png)?;
    
    // Sender: Scan QR code and send payment
    let scanned_image = std::fs::read("scanned_qr.png")?;
    let receiver_meta = QrCodeHandler::decode_meta_address(&scanned_image)?;
    
    let prepared = wallet_manager.prepare_payment(&receiver_meta, 1_000_000)?;
    wallet_manager.send_payment(prepared).await?;
    
    Ok(())
}
```

### Backup and Recovery

```rust
async fn backup_and_restore() -> StealthResult<()> {
    // Backup
    let keypair = StealthKeyPair::generate_standard()?;
    let password = "secure_password_123";
    
    let encrypted_backup = keypair.export_encrypted(password)?;
    std::fs::write("stealth_keypair_backup.enc", &encrypted_backup)?;
    
    println!("Backup saved. Store this file securely!");
    
    // Restore (on new device or after data loss)
    let backup_data = std::fs::read("stealth_keypair_backup.enc")?;
    let restored_keypair = StealthKeyPair::import_encrypted(&backup_data, password)?;
    
    // Verify restoration
    assert_eq!(keypair.to_meta_address(), restored_keypair.to_meta_address());
    println!("Keypair restored successfully");
    
    Ok(())
}
```

---

## Performance Considerations

### Caching

The `StealthAddressGenerator` automatically caches derived addresses to avoid redundant cryptographic computations:

```rust
// First call: performs full ECDH + point addition
let output1 = generator.generate_stealth_address(meta_addr, Some(ephemeral_kp.clone())).await?;

// Second call with same inputs: uses cached result
let output2 = generator.generate_stealth_address(meta_addr, Some(ephemeral_kp)).await?;

// Outputs are identical (from cache)
assert_eq!(output1.stealth_address, output2.stealth_address);
```

### Viewing Tag Optimization

Scanning uses viewing tags to filter transactions before expensive ECDH:

```rust
// Without viewing tags: O(n) ECDH operations for n transactions
// With viewing tags: O(n) hash comparisons + O(m) ECDH for m matches
// Typical speedup: 64x (4-byte tag = 1/2^32 false positive rate)

let payments = scanner.scan_for_payments(None, None).await?;
// Efficiently scans 1000+ transactions per second
```

### Batch Operations

When processing multiple payments, batch blockchain operations:

```rust
// Instead of individual settlements
for payment in queued_payments {
    wallet_manager.send_payment(payment).await?; // Slow
}

// Use batch processing
let results = payment_queue.process_queue().await?; // Fast
```

---

## Security Best Practices

### 1. Key Management

```rust
// ✅ DO: Use secure storage
let storage = Arc::new(IosKeychainStorage::new());
storage.store_keypair(&keypair).await?;

// ❌ DON'T: Store keys in plaintext
std::fs::write("keypair.json", serde_json::to_string(&keypair)?)?; // INSECURE
```

### 2. Password Handling

```rust
// ✅ DO: Use strong passwords for backups
let password = generate_strong_password(); // 16+ chars, mixed case, symbols
let backup = keypair.export_encrypted(&password)?;

// ❌ DON'T: Use weak passwords
let backup = keypair.export_encrypted("password123")?; // WEAK
```

### 3. Nonce Management

```rust
// ✅ DO: Generate unique nonces
use rand::RngCore;
let mut nonce = [0u8; 24];
rand::thread_rng().fill_bytes(&mut nonce);

// ❌ DON'T: Reuse nonces
let nonce = [0u8; 24]; // INSECURE - never reuse nonces
```

### 4. Error Logging

```rust
// ✅ DO: Log errors without sensitive data
match keypair.export_encrypted(password) {
    Err(e) => log::error!("Export failed: {}", e), // Safe
    Ok(data) => { /* ... */ }
}

// ❌ DON'T: Log sensitive data
log::debug!("Spending key: {:?}", keypair.spending_secret_key()); // INSECURE
```

### 5. Viewing Key Separation

```rust
// ✅ DO: Use viewing key for scanning (view-only wallet)
let scanner = StealthScanner::new(&keypair, blockchain_client);
// Scanner only has viewing key, cannot spend

// ✅ DO: Keep spending key offline for cold storage
let spending_key = keypair.spending_secret_key();
// Store spending_key on hardware wallet or air-gapped device
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_address_round_trip() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let meta_addr = keypair.to_meta_address();
        let parsed = StealthKeyPair::from_meta_address(&meta_addr).unwrap();
        assert_eq!(keypair.to_meta_address(), parsed.to_meta_address());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_payment() {
    let receiver_keypair = StealthKeyPair::generate_standard().unwrap();
    let receiver_meta = receiver_keypair.to_meta_address();
    
    let generator = StealthAddressGenerator::new();
    let output = generator.generate_stealth_address(&receiver_meta, None).await.unwrap();
    
    // Simulate on-chain payment
    // ...
    
    let scanner = StealthScanner::new(&receiver_keypair, blockchain_client);
    let payments = scanner.scan_for_payments(None, None).await.unwrap();
    
    assert_eq!(payments.len(), 1);
    assert_eq!(payments[0].stealth_address, output.stealth_address);
}
```

---

## Troubleshooting

### Issue: "Invalid curve point" error

**Cause:** Malformed Ed25519 or Curve25519 key

**Solution:**
```rust
// Validate keys before use
match StealthCrypto::ed25519_to_curve25519(&ed_pk) {
    Ok(curve_pk) => { /* use curve_pk */ }
    Err(StealthError::InvalidCurvePoint(msg)) => {
        eprintln!("Invalid key: {}", msg);
        // Generate new key or request valid key from user
    }
    Err(e) => { /* other error */ }
}
```

### Issue: Scanning doesn't find payments

**Possible causes:**
1. Scanning wrong slot range
2. Payment not yet confirmed
3. Incorrect viewing key

**Solution:**
```rust
// 1. Scan wider range
let payments = scanner.scan_for_payments(Some(slot - 1000), None).await?;

// 2. Wait for confirmation
tokio::time::sleep(Duration::from_secs(30)).await;

// 3. Verify viewing key matches
assert_eq!(keypair.viewing_public_key(), expected_viewing_pk);
```

### Issue: Queue full error

**Cause:** More than 1000 queued payments

**Solution:**
```rust
// Process queue to make space
let results = payment_queue.process_queue().await?;
println!("Settled {} payments", results.len());

// Or wait for auto-settlement
tokio::time::sleep(Duration::from_secs(60)).await;
```

### Issue: Decryption failed

**Possible causes:**
1. Wrong password
2. Corrupted backup data
3. Wrong encryption key

**Solution:**
```rust
match StealthKeyPair::import_encrypted(&data, password) {
    Err(StealthError::DecryptionFailed(_)) => {
        // Try alternative password or restore from different backup
        eprintln!("Wrong password or corrupted data");
    }
    Ok(keypair) => { /* success */ }
    Err(e) => { /* other error */ }
}
```

---

## Additional Resources

- **Requirements Document**: `.kiro/specs/ble-mesh-stealth-transfers/requirements.md`
- **Design Document**: `.kiro/specs/ble-mesh-stealth-transfers/design.md`
- **Implementation Tasks**: `.kiro/specs/ble-mesh-stealth-transfers/tasks.md`
- **EIP-5564 Specification**: https://eips.ethereum.org/EIPS/eip-5564
- **Solana Documentation**: https://docs.solana.com/
- **Curve25519 Reference**: https://cr.yp.to/ecdh.html

---

## Changelog

### Version 0.1.0 (Current)
- Initial implementation
- Standard stealth addresses (version 1)
- Blockchain scanning with viewing tags
- Payment queue with auto-settlement
- Platform-specific secure storage (iOS/Android)
- QR code support
- Hybrid post-quantum mode (version 2)
- BLE mesh integration

