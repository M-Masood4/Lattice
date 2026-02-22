//! Example: Shield and Unshield Operations
//!
//! This example demonstrates privacy-enhancing shield and unshield operations:
//! - Shield: Convert regular funds to stealth address (break transaction graph)
//! - Unshield: Convert stealth funds back to regular address
//!
//! These operations break on-chain linkage between addresses, enhancing privacy.
//!
//! Run with: cargo run --example shield_unshield

use stealth::{
    StealthKeyPair, StealthWalletManager, StealthScanner, DetectedPayment,
    StealthResult,
};
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> StealthResult<()> {
    println!("=== Shield/Unshield Privacy Operations ===\n");

    // 1. Setup wallets
    println!("1. Setting up wallets...");
    let stealth_keypair = StealthKeyPair::generate_standard()?;
    let regular_keypair = Keypair::new();
    
    println!("   ✓ Stealth wallet created");
    println!("     Meta-address: {}", stealth_keypair.to_meta_address());
    println!("   ✓ Regular wallet created");
    println!("     Address: {}", regular_keypair.pubkey());

    // 2. Shield operation - Break transaction graph
    println!("\n2. SHIELD OPERATION");
    println!("   Purpose: Break transaction graph linkage");
    println!("   ─────────────────────────────────────────");
    
    let shield_amount = 5_000_000_000; // 5 SOL
    println!("   Source: Regular wallet ({})", regular_keypair.pubkey());
    println!("   Amount: {} lamports (5 SOL)", shield_amount);
    println!("   Destination: Stealth address (one-time)");
    
    println!("\n   Process:");
    println!("   1. Generate unique stealth address");
    println!("   2. Transfer funds from regular → stealth");
    println!("   3. On-chain: No linkage visible");
    
    // Simulate shield operation
    println!("\n   ✓ Stealth address generated");
    println!("   ✓ Transaction submitted to Solana");
    println!("   ✓ Funds shielded successfully");
    
    println!("\n   Privacy achieved:");
    println!("   • Regular wallet → Stealth address: No visible link");
    println!("   • Transaction graph broken");
    println!("   • Observer cannot link addresses");

    // 3. Wait for confirmation
    println!("\n3. Waiting for blockchain confirmation...");
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("   ✓ Transaction confirmed");

    // 4. Scan for shielded payment
    println!("\n4. SCANNING FOR SHIELDED PAYMENT");
    println!("   ─────────────────────────────────────────");
    println!("   Using viewing key to scan blockchain...");
    
    // Simulate scanning
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("   ✓ Payment detected!");
    println!("     Amount: {} lamports", shield_amount);
    println!("     Stealth address: <unique-address>");
    println!("     Ephemeral key: <ephemeral-public-key>");

    // 5. Unshield operation - Convert back to regular
    println!("\n5. UNSHIELD OPERATION");
    println!("   Purpose: Convert stealth funds to regular address");
    println!("   ─────────────────────────────────────────");
    
    let new_regular_address = Pubkey::new_unique();
    println!("   Source: Stealth address");
    println!("   Amount: {} lamports (5 SOL)", shield_amount);
    println!("   Destination: New regular wallet");
    println!("     Address: {}", new_regular_address);
    
    println!("\n   Process:");
    println!("   1. Derive spending key for stealth address");
    println!("   2. Transfer funds from stealth → regular");
    println!("   3. Stealth address now empty");
    
    // Simulate unshield operation
    println!("\n   ✓ Spending key derived");
    println!("   ✓ Transaction submitted to Solana");
    println!("   ✓ Funds unshielded successfully");

    // 6. Privacy analysis
    println!("\n6. PRIVACY ANALYSIS");
    println!("   ─────────────────────────────────────────");
    println!("   Transaction flow:");
    println!("     Original wallet → Stealth address → New wallet");
    println!("\n   On-chain observer sees:");
    println!("     • Transaction 1: Wallet A → Address X");
    println!("     • Transaction 2: Address Y → Wallet B");
    println!("\n   Observer CANNOT determine:");
    println!("     ✗ That Address X and Address Y are related");
    println!("     ✗ That Wallet A and Wallet B are same owner");
    println!("     ✗ The connection between transactions");
    println!("\n   Privacy achieved:");
    println!("     ✓ Transaction graph broken");
    println!("     ✓ No on-chain linkage");
    println!("     ✓ Enhanced financial privacy");

    // 7. Use cases
    println!("\n7. COMMON USE CASES");
    println!("   ─────────────────────────────────────────");
    println!("   • Privacy-conscious payments");
    println!("   • Breaking transaction history");
    println!("   • Receiving funds anonymously");
    println!("   • Mixing services alternative");
    println!("   • Salary/payment privacy");
    println!("   • Donation anonymity");

    // 8. Best practices
    println!("\n8. BEST PRACTICES");
    println!("   ─────────────────────────────────────────");
    println!("   ✓ Shield before sensitive transactions");
    println!("   ✓ Use different regular addresses for unshield");
    println!("   ✓ Wait between shield and unshield");
    println!("   ✓ Combine with other privacy techniques");
    println!("   ✓ Keep viewing key separate from spending key");
    println!("   ⚠ Never reuse stealth addresses");

    println!("\n=== Example Complete ===");
    println!("\nKey concepts:");
    println!("  • Shield: Regular → Stealth (break graph)");
    println!("  • Unshield: Stealth → Regular (spend funds)");
    println!("  • Privacy: No on-chain linkage");
    println!("  • Security: Cryptographic guarantees");

    Ok(())
}
