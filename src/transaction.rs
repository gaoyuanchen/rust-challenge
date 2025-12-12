use rust_decimal::Decimal;
use crate::types::{AccountProfile, CsvInputRow, Transaction, TransactionId, TransactionParsingError, TransactionProcessingError, TransactionState};

impl AccountProfile {
    /// The main handler for transaction
    /// When we accept a transaction, we will return Ok(()) and mutate the state of `AccountProfile`
    /// When we reject a transaction, we will return Err with TransactionProcessingError
    pub fn process_transaction(&mut self, id: TransactionId, transaction: Transaction) -> Result<(), TransactionProcessingError> {
        if self.frozen {
            return Err(TransactionProcessingError::AccountIsFrozen);
        }
        match transaction {
            Transaction::Deposit(amount) => {
                self.validate_unique_id(id)?;
                self.deposit_transactions.insert(id, (TransactionState::Normal, amount));
                self.available += amount;
            }
            Transaction::Withdrawal(amount) => {
                // My assumption here is that the tx ID should be unique for deposit and withdrawal
                // Note that even if the withdrawal was rejected due to other reason, we still consume this ID
                self.validate_unique_id(id)?;
                if self.available < amount {
                    return Err(TransactionProcessingError::AvailableAmountTooLow(self.available, amount));
                }
                self.available -= amount;
            }
            Transaction::Dispute => {
                let available = self.available;
                let (state, amount) = self.get_deposit_transaction(id)?;
                if state != &TransactionState::Normal {
                    return Err(TransactionProcessingError::InvalidTransactionState);
                }
                // This is a special case where the user already withdrawal the fund
                // The instruction didn't mention how to handle this case, here I assume we need to reject this dispute
                if available < amount {
                    return Err(TransactionProcessingError::AvailableAmountTooLow(available, amount));
                }
                *state = TransactionState::UnderDispute;
                self.available -= amount;
                self.held += amount;
            }
            Transaction::Resolve => {
                let (state, amount) = self.get_deposit_transaction(id)?;
                if state != &TransactionState::UnderDispute {
                    return Err(TransactionProcessingError::InvalidTransactionState);
                }
                *state = TransactionState::Normal;
                self.available += amount;
                self.held -= amount;
            }
            Transaction::Chargeback => {
                let (state, amount) = self.get_deposit_transaction(id)?;
                if state != &TransactionState::UnderDispute {
                    return Err(TransactionProcessingError::InvalidTransactionState);
                }
                *state = TransactionState::Chargeback;
                self.held -= amount;
                self.frozen = true;
            }
        }
        Ok(())
    }

    fn get_deposit_transaction(&mut self, id: TransactionId) -> Result<(&mut TransactionState, Decimal), TransactionProcessingError> {
        match self.deposit_transactions.get_mut(&id) {
            None => Err(TransactionProcessingError::InvalidTransactionId(id)),
            Some((state, amount)) => Ok((state, *amount)),
        }
    }

    fn validate_unique_id(&mut self, id: TransactionId) -> Result<(), TransactionProcessingError> {
        if self.transaction_ids.contains(&id) {
            return Err(TransactionProcessingError::InvalidTransactionId(id));
        }
        self.transaction_ids.insert(id);
        Ok(())
    }
}

pub fn parse_transaction(row: &CsvInputRow) -> Result<Transaction, TransactionParsingError> {
    match row.transaction_type.as_str() {
        "deposit" => Ok(Transaction::Deposit(row.amount.ok_or(TransactionParsingError::MissingAmount)?)),
        "withdrawal" => Ok(Transaction::Withdrawal(row.amount.ok_or(TransactionParsingError::MissingAmount)?)),
        "dispute" => Ok(Transaction::Dispute),
        "resolve" => Ok(Transaction::Resolve),
        "chargeback" => Ok(Transaction::Chargeback),
        _ => Err(TransactionParsingError::InvalidType),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_transaction_processing() {
        let mut profile = AccountProfile::default();

        // Deposit
        let res = profile.process_transaction(1, Transaction::Deposit(Decimal::from(10)));
        assert!(res.is_ok());
        assert_eq!(profile.available, Decimal::from(10));
        assert_eq!(profile.held, Decimal::from(0));
        assert!(profile.deposit_transactions.contains_key(&1));
        assert_eq!(profile.frozen, false);

        let res = profile.process_transaction(1, Transaction::Deposit(Decimal::from(10)));
        assert!(res.is_err());

        let res = profile.process_transaction(2, Transaction::Deposit(Decimal::from(5)));
        assert!(res.is_ok());
        assert_eq!(profile.available, Decimal::from(15));
        assert_eq!(profile.held, Decimal::from(0));
        assert_eq!(profile.deposit_transactions.len(), 2);
        assert_eq!(profile.frozen, false);

        // Withdrawal
        let res = profile.process_transaction(3, Transaction::Withdrawal(Decimal::from(2)));
        assert!(res.is_ok());
        assert_eq!(profile.available, Decimal::from(13));
        assert_eq!(profile.held, Decimal::from(0));
        assert_eq!(profile.deposit_transactions.len(), 2);
        assert_eq!(profile.frozen, false);

        // Dispute -> Resolve
        let res = profile.process_transaction(1, Transaction::Dispute);
        assert!(res.is_ok());
        assert_eq!(profile.available, Decimal::from(3));
        assert_eq!(profile.held, Decimal::from(10));
        assert_eq!(profile.deposit_transactions.get(&1).unwrap().0, TransactionState::UnderDispute);
        assert_eq!(profile.frozen, false);

        let res = profile.process_transaction(1, Transaction::Dispute);
        assert!(res.is_err());

        let res = profile.process_transaction(3, Transaction::Dispute);
        assert!(res.is_err());

        let res = profile.process_transaction(1, Transaction::Resolve);
        assert!(res.is_ok());
        assert_eq!(profile.available, Decimal::from(13));
        assert_eq!(profile.held, Decimal::from(0));
        assert_eq!(profile.deposit_transactions.get(&1).unwrap().0, TransactionState::Normal);
        assert_eq!(profile.frozen, false);

        let res = profile.process_transaction(1, Transaction::Resolve);
        assert!(res.is_err());

        // Dispute -> ChargeBack
        let res = profile.process_transaction(2, Transaction::Dispute);
        assert!(res.is_ok());
        assert_eq!(profile.available, Decimal::from(8));
        assert_eq!(profile.held, Decimal::from(5));
        assert_eq!(profile.deposit_transactions.get(&2).unwrap().0, TransactionState::UnderDispute);
        assert_eq!(profile.frozen, false);

        let res = profile.process_transaction(2, Transaction::Chargeback);
        assert!(res.is_ok());
        assert_eq!(profile.available, Decimal::from(8));
        assert_eq!(profile.held, Decimal::from(0));
        assert_eq!(profile.deposit_transactions.get(&2).unwrap().0, TransactionState::Chargeback);
        assert_eq!(profile.frozen, true);

        let res = profile.process_transaction(4, Transaction::Deposit(Decimal::from(20)));
        assert!(res.is_err());
    }

    #[test]
    fn test_dispute_after_withdrawal() {
        let mut profile = AccountProfile::default();

        let res = profile.process_transaction(1, Transaction::Deposit(Decimal::from(10)));
        assert!(res.is_ok());

        let res = profile.process_transaction(2, Transaction::Withdrawal(Decimal::from(10)));
        assert!(res.is_ok());

        let res = profile.process_transaction(1, Transaction::Dispute);
        assert!(res.is_err());
    }

    #[test]
    fn test_duplicated_id() {
        let mut profile = AccountProfile::default();

        let res = profile.process_transaction(1, Transaction::Deposit(Decimal::from(10)));
        assert!(res.is_ok());

        let res = profile.process_transaction(1, Transaction::Withdrawal(Decimal::from(10)));
        assert!(res.is_err());
    }
}
