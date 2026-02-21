use serde::{Deserialize, Serialize};

/// Represents a token account with balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAccount {
    pub mint: String,
    pub owner: String,
    pub amount: u64,
    pub decimals: u8,
}

/// Represents wallet balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    pub address: String,
    pub sol_balance: u64, // in lamports
    pub token_accounts: Vec<TokenAccount>,
}
