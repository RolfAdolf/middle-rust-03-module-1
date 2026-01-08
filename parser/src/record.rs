use crate::common::{TransactionStatus, TransactionType};

/// Represents a bank transaction record.
///
/// This struct contains all the information about a single bank transaction,
/// including transaction ID, type, user IDs, amount, timestamp, status, and description.
#[derive(Debug, PartialEq, Eq)]
pub struct YPBankRecord {
    pub id: u64,
    pub transaction_type: TransactionType,
    pub from_user_id: u64,
    pub to_user_id: u64,
    pub amount: i64,
    pub ts: u64,
    pub status: TransactionStatus,
    pub description: String,
}

impl YPBankRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u64,
        transaction_type: TransactionType,
        from_user_id: u64,
        to_user_id: u64,
        amount: i64,
        ts: u64,
        status: TransactionStatus,
        description: String,
    ) -> Self {
        Self {
            id,
            transaction_type,
            from_user_id,
            to_user_id,
            amount,
            ts,
            status,
            description,
        }
    }
}
