// Receipt Helper - provides receipt data for completed proximity transfers

use crate::TransferRequest;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Receipt data for a completed proximity transfer
/// 
/// This struct contains all the information needed to create a blockchain receipt
/// for a proximity transfer. It should be passed to the ReceiptService in the API layer.
#[derive(Debug, Clone)]
pub struct ProximityReceiptData {
    pub proximity_transfer_id: Uuid,
    pub amount: Decimal,
    pub currency: String,
    pub sender: String,
    pub recipient: String,
    pub transaction_hash: String,
}

impl ProximityReceiptData {
    /// Create receipt data from a completed transfer request
    /// 
    /// **Validates: Requirements 10.3, 10.4**
    pub fn from_transfer(request: &TransferRequest, transaction_hash: String) -> Self {
        Self {
            proximity_transfer_id: request.id,
            amount: request.amount,
            currency: request.asset.clone(),
            sender: request.sender_wallet.clone(),
            recipient: request.recipient_wallet.clone(),
            transaction_hash,
        }
    }
}
