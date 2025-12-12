use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

pub type ClientId = u16;
pub type TransactionId = u32;

/// Different transactions and transaction specific data.
/// Note that we don't store the common fields like client and tx here
#[derive(Debug)]
pub enum Transaction {
    Deposit(Decimal),
    Withdrawal(Decimal),
    Dispute,
    Resolve,
    Chargeback,
}

/// The dispute states for a (deposit) transaction
/// They are used and changed in `Dispute`, `Resolve`, `Chargeback` transactions
#[derive(Debug, Eq, PartialEq, Default)]
pub enum TransactionState {
    #[default]
    Normal,
    UnderDispute,
    Chargeback,
}

/// The data we store for a single client
/// For each deposit transaction we store the dispute state and their amount
#[derive(Debug, Eq, PartialEq, Default)]
pub struct AccountProfile {
    pub available: Decimal,
    pub held: Decimal,
    pub deposit_transactions: HashMap<TransactionId, (TransactionState, Decimal)>, // tx -> (state, amount)
    pub transaction_ids: HashSet<TransactionId>,
    pub frozen: bool,
}

/// This is used to parse input csv
#[derive(Deserialize, Debug)]
pub struct CsvInputRow {
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub client: ClientId,
    pub tx: TransactionId,
    pub amount: Option<Decimal>,
}

/// Error type for transaction processing
#[derive(Debug, Error)]
pub enum TransactionProcessingError {
    #[error("account is frozen")]
    AccountIsFrozen,
    #[error("invalid transaction id: {0}")]
    InvalidTransactionId(TransactionId),
    #[error("available amount {0} is less than withdrawal request amount {1}")]
    AvailableAmountTooLow(Decimal, Decimal),
    #[error("transaction is not in the expected state")]
    InvalidTransactionState,
}

/// Error type for transaction parsing
#[derive(Debug, Error)]
pub enum TransactionParsingError {
    #[error("missing amount")]
    MissingAmount,
    #[error("invalid type")]
    InvalidType,
}
