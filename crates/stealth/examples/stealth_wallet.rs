//! Example: Stealth Wallet Operations
//!
//! This example demonstrates basic stealth wallet operations including:
//! - Generating a stealth key pair
//! - Creating and sharing a meta-address
//! - Preparing and sending stealth payments
//! - Scanning for incoming payments
//! - Backing up and restoring keys
//!
//! Run with: cargo run --example stealth_wallet

use stealth::{
    StealthKeyPair, StealthWalletManager, StealthAddressGenerator, StealthScanner,
    QrCodeHandler, StealthResult,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> StealthResult<()> {
    println!("=== Stealth Wallet Example ===\n");

    // 1. Generate a new stealth key pair
    println!("1. Generating stealth key pair...");
    let keypair = StealthKeyPair::generate_standard()?;
    println!("   ✓ Key pair generated (version 1)");

    // 2. Get meta-address for receiving payments
    let meta_address = keypair.to_meta_address();
    println!("\n2. Your meta-address (share this to receive payments):");
    println!("   {}", meta_address);

    // 3. Generate QR code for easy sharing
    println!("\n3. Generating QR code...");
    let qr_png = QrCodeHandler::encode_meta_address(&meta_address)?;
    std::fs::write("stealth_address_qr.png", qr_png)?;
    println!("   ✓ QR code saved to: stealth_address_qr.png");

    // 4. Prepare a payment to another user
    println!("\n4. Preparing a stealth payment...");
    let receiver_meta = "stealth:1:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpQS47pMV5E7bSkYQBx1:7cvkjYAkUYs4W8XcXsHNrdKieRvoBd7Y9Qpspbr6jT1N";
    let amount = 1_000_000_000; // 1 SOL in lamports

    let generator = StealthAddressGenerator::new();
    let output = generator.generate_stealth_address(receiver_meta, None).await?;

    println!("   ✓ Stealth address generated:");
    println!("     Address: {}", output.stealth_address);
    println!("     Ephemeral key: {}", output.ephemeral_public_key);
    println!("     Viewing tag: {:?}", output.viewing_tag);

    // 5. Backup key pair
    println!("\n5. Backing up key pair...");
    let password = "secure_password_123";
    let encrypted_backup = keypair.export_encrypted(password)?;
    std::fs::write("keypair_backup.enc", &encrypted_backup)?;
    println!("   ✓ Encrypted backup saved to: keypair_backup.enc");
    println!("   ⚠ Store this file securely!");

    // 6. Restore from backup
    println!("\n6. Testing backup restoration...");
    let backup_data = std::fs::read("keypair_backup.enc")?;
    let restored_keypair = StealthKeyPair::import_encrypted(&backup_data, password)?;
    
    if keypair.to_meta_address() == restored_keypair.to_meta_address() {
        println!("   ✓ Backup restoration successful");
    } else {
        println!("   ✗ Backup restoration failed");
    }

    // 7. Key information
    println!("\n7. Key pair information:");
    println!("   Spending public key: {}", keypair.spending_public_key());
    println!("   Viewing public key: {}", keypair.viewing_public_key());

    println!("\n=== Example Complete ===");
    println!("\nNext steps:");
    println!("  - Share your meta-address or QR code to receive payments");
    println!("  - Use the wallet manager to send payments");
    println!("  - Scan the blockchain for incoming payments");
    println!("  - Keep your backup file secure!");

    Ok(())
}
