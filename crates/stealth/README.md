# Stealth Address Crate

Privacy-preserving stealth address implementation for Solana, adapted from EIP-5564.

## Features

- **Stealth Address Generation**: One-time payment addresses with no on-chain linkage
- **Blockchain Scanning**: Efficient detection of incoming stealth payments using viewing tags
- **Key Management**: Separate spending and viewing keys for enhanced security
- **Payment Queue**: Offline payment queueing with automatic settlement
- **Post-Quantum Support**: Optional hybrid mode with ML-KEM-768 (Kyber)
- **Shield/Unshield**: Convert between regular and stealth addresses

## Module Structure

- `crypto`: Core cryptographic primitives (ECDH, point addition, encryption)
- `keypair`: Stealth key pair management and meta-address generation
- `generator`: Sender-side stealth address derivation
- `scanner`: Receiver-side blockchain scanning for incoming payments
- `storage`: Secure key storage abstraction
- `payment_queue`: Offline payment management with auto-settlement
- `wallet_manager`: High-level wallet operations
- `network_monitor`: Network connectivity monitoring
- `hybrid`: Post-quantum hybrid mode (optional)
- `qr`: QR code support for meta-addresses

## Meta-Address Format

**Standard Mode (Version 1)**:
```
stealth:1:<spending_pk_base58>:<viewing_pk_base58>
```

**Hybrid Mode (Version 2)**:
```
stealth:2:<spending_pk_base58>:<viewing_pk_base58>:<kyber_pk_base58>
```

## Implementation Status

This crate is currently in development. Module stubs have been created with `todo!()` placeholders.
Implementation will proceed according to the task list in `.kiro/specs/ble-mesh-stealth-transfers/tasks.md`.

## Dependencies

- `ed25519-dalek`: Ed25519 signatures (compatible with Solana 1.18)
- `curve25519-dalek`: Curve25519 ECDH operations
- `sha2`: SHA-256 hashing for viewing tags
- `chacha20poly1305`: XChaCha20-Poly1305 authenticated encryption
- `pqc_kyber`: Post-quantum Kyber/ML-KEM-768 (optional)
- `proptest`: Property-based testing framework

## Testing

The crate uses a dual testing approach:
- **Unit tests**: Specific examples and edge cases
- **Property-based tests**: Universal correctness properties (39 total)

Run tests with:
```bash
cargo test --package stealth
```
