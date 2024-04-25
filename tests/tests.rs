#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use transactions::*;

    #[test]
    fn simple_withdrawal() {
        let mut transaction_processor = Transactor::new();

        let trans_1 = Transaction {
            transaction_type: Some(TransactionTypes::Deposit),
            client: Some(1),
            tx: Some(1),
            amount: Some(dec!(10.0)),
        };
        let trans_2 = Transaction {
            transaction_type: Some(TransactionTypes::Withdrawal),
            client: Some(1),
            tx: Some(2),
            amount: Some(dec!(5.0)),
        };

        transaction_processor.process_a_record(trans_1);
        transaction_processor.process_a_record(trans_2);
        assert_eq!(Some(dec!(5.0)),Some(transaction_processor.client_accounts.get(&1).unwrap().available));
    }

    #[test]
    fn deposit_of_zero_creates_new_account() {
        let mut transaction_processor = Transactor::new();

        let trans_1 = Transaction {
            transaction_type: Some(TransactionTypes::Deposit),
            client: Some(1),
            tx: Some(1),
            amount: Some(dec!(0.0)),
        };

        transaction_processor.process_a_record(trans_1);
        assert_eq!(Some(dec!(0.0)),Some(transaction_processor.client_accounts.get(&1).unwrap().available));
        assert!(!transaction_processor.client_accounts.is_empty());
    }

    #[test]
    fn simple_increment_account() {
        let mut transaction_processor = Transactor::new();

        let trans_1 = Transaction {
            transaction_type: Some(TransactionTypes::Deposit),
            client: Some(1),
            tx: Some(1),
            amount: Some(dec!(10.0)),
        };
        let trans_2 = Transaction {
            transaction_type: Some(TransactionTypes::Deposit),
            client: Some(1),
            tx: Some(2),
            amount: Some(dec!(20.0)),
        };

        transaction_processor.process_a_record(trans_1);
        transaction_processor.process_a_record(trans_2);

        assert_eq!(Some(dec!(30.0)),Some(transaction_processor.client_accounts.get(&1).unwrap().available));
    }

    #[test]
    fn dispute_then_resolve_outcome() {
        let mut transaction_processor = Transactor::new();
        let mut tx_number = 1;
        let account_number = 1;

        let mut trans = Transaction {
            transaction_type: Some(TransactionTypes::Deposit),
            client: Some(account_number),
            tx: Some(tx_number),
            amount: Some(dec!(10.0)),
        };

        transaction_processor.process_a_record(trans);
        assert_eq!(Some(dec!(10.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().available));
        assert_eq!(Some(dec!(0.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().held));
        assert_eq!(Some(dec!(10.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().total));
        assert_eq!(Some(false),Some(transaction_processor.client_accounts.get(&account_number).unwrap().locked));

        tx_number += 1;
        trans = Transaction {
            transaction_type: Some(TransactionTypes::Deposit),
            client: Some(account_number),
            tx: Some(tx_number),
            amount: Some(dec!(5.0)),
        };

        transaction_processor.process_a_record(trans);
        assert_eq!(Some(dec!(15.0)), Some(transaction_processor.client_accounts.get(&account_number).unwrap().available));
        assert_eq!(Some(dec!(0.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().held));
        assert_eq!(Some(dec!(15.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().total));
        assert_eq!(Some(false),Some(transaction_processor.client_accounts.get(&account_number).unwrap().locked));

        tx_number += 1;
        trans = Transaction {
            transaction_type: Some(TransactionTypes::Withdrawal),
            client: Some(account_number),
            tx: Some(tx_number),
            amount: Some(dec!(6.0)),
        };

        transaction_processor.process_a_record(trans);
        assert_eq!(Some(dec!(9.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().available));
        assert_eq!(Some(dec!(0.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().held));
        assert_eq!(Some(dec!(9.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().total));
        assert_eq!(Some(false),Some(transaction_processor.client_accounts.get(&account_number).unwrap().locked));

        tx_number += 1;
        trans = Transaction {
            transaction_type: Some(TransactionTypes::Withdrawal),
            client: Some(account_number),
            tx: Some(tx_number),
            amount: Some(dec!(2.0)),
        };

        transaction_processor.process_a_record(trans);
        assert_eq!(Some(dec!(7.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().available));
        assert_eq!(Some(dec!(0.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().held));
        assert_eq!(Some(dec!(7.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().total));
        assert_eq!(Some(false),Some(transaction_processor.client_accounts.get(&account_number).unwrap().locked));

        trans = Transaction {
            transaction_type: Some(TransactionTypes::Dispute),
            client: Some(account_number),
            tx: Some(2),
            amount: None,
        };

        transaction_processor.process_a_record(trans);
        assert_eq!(Some(dec!(2.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().available));
        assert_eq!(Some(dec!(5.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().held));
        assert_eq!(Some(dec!(7.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().total));
        assert_eq!(Some(false),Some(transaction_processor.client_accounts.get(&account_number).unwrap().locked));

        trans = Transaction {
            transaction_type: Some(TransactionTypes::Resolve),
            client: Some(account_number),
            tx: Some(2),
            amount: None,
        };

        transaction_processor.process_a_record(trans);
        assert_eq!(Some(dec!(7.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().available));
        assert_eq!(Some(dec!(0.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().held));
        assert_eq!(Some(dec!(7.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().total));
        assert_eq!(Some(false),Some(transaction_processor.client_accounts.get(&account_number).unwrap().locked));
    }

    #[test]
    fn resolve_with_no_dispute_is_ignored() {
        let mut transaction_processor = Transactor::new();
        let tx_number = 1;
        let account_number = 1;

        let mut trans = Transaction {
            transaction_type: Some(TransactionTypes::Deposit),
            client: Some(account_number),
            tx: Some(tx_number),
            amount: Some(dec!(10.0)),
        };

        transaction_processor.process_a_record(trans);
        assert_eq!(Some(dec!(10.0)),Some(transaction_processor.client_accounts.get(&1).unwrap().available));
        assert_eq!(Some(dec!(0.0)),Some(transaction_processor.client_accounts.get(&1).unwrap().held));
        assert_eq!(Some(dec!(10.0)),Some(transaction_processor.client_accounts.get(&1).unwrap().total));
        assert_eq!(Some(false),Some(transaction_processor.client_accounts.get(&1).unwrap().locked));

        trans = Transaction {
            transaction_type: Some(TransactionTypes::Resolve),
            client: Some(account_number),
            tx: Some(1),
            amount: None,
        };

        transaction_processor.process_a_record(trans);
        assert!(transaction_processor.client_accounts.get(&1).unwrap().disputes.is_empty());
        assert_eq!(Some(dec!(10.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().available));
        assert_eq!(Some(dec!(0.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().held));
        assert_eq!(Some(dec!(10.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().total));
        assert_eq!(Some(false),Some(transaction_processor.client_accounts.get(&account_number).unwrap().locked));
    }

    #[test]
    fn dispute_then_chargeback_outcome() {
        let mut transaction_processor = Transactor::new();
        let mut tx_number = 1;
        let account_number = 1;

        let mut trans = Transaction {
            transaction_type: Some(TransactionTypes::Deposit),
            client: Some(account_number),
            tx: Some(tx_number),
            amount: Some(dec!(10.0)),
        };

        transaction_processor.process_a_record(trans);
        assert_eq!(Some(dec!(10.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().available));
        assert_eq!(Some(dec!(0.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().held));
        assert_eq!(Some(dec!(10.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().total));
        assert_eq!(Some(false),Some(transaction_processor.client_accounts.get(&account_number).unwrap().locked));

        tx_number += 1;
        trans = Transaction {
            transaction_type: Some(TransactionTypes::Deposit),
            client: Some(account_number),
            tx: Some(tx_number),
            amount: Some(dec!(5.0)),
        };

        transaction_processor.process_a_record(trans);
        assert_eq!(Some(dec!(15.0)), Some(transaction_processor.client_accounts.get(&account_number).unwrap().available));
        assert_eq!(Some(dec!(0.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().held));
        assert_eq!(Some(dec!(15.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().total));
        assert_eq!(Some(false),Some(transaction_processor.client_accounts.get(&account_number).unwrap().locked));

        tx_number += 1;
        trans = Transaction {
            transaction_type: Some(TransactionTypes::Withdrawal),
            client: Some(account_number),
            tx: Some(tx_number),
            amount: Some(dec!(6.0)),
        };

        transaction_processor.process_a_record(trans);
        assert_eq!(Some(dec!(9.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().available));
        assert_eq!(Some(dec!(0.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().held));
        assert_eq!(Some(dec!(9.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().total));
        assert_eq!(Some(false),Some(transaction_processor.client_accounts.get(&account_number).unwrap().locked));

        tx_number += 1;
        trans = Transaction {
            transaction_type: Some(TransactionTypes::Withdrawal),
            client: Some(account_number),
            tx: Some(tx_number),
            amount: Some(dec!(2.0)),
        };

        transaction_processor.process_a_record(trans);
        assert_eq!(Some(dec!(7.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().available));
        assert_eq!(Some(dec!(0.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().held));
        assert_eq!(Some(dec!(7.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().total));
        assert_eq!(Some(false),Some(transaction_processor.client_accounts.get(&account_number).unwrap().locked));

        trans = Transaction {
            transaction_type: Some(TransactionTypes::Dispute),
            client: Some(account_number),
            tx: Some(2),
            amount: None,
        };

        transaction_processor.process_a_record(trans);
        assert_eq!(Some(dec!(2.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().available));
        assert_eq!(Some(dec!(5.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().held));
        assert_eq!(Some(dec!(7.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().total));
        assert_eq!(Some(false),Some(transaction_processor.client_accounts.get(&account_number).unwrap().locked));

        trans = Transaction {
            transaction_type: Some(TransactionTypes::Chargeback),
            client: Some(account_number),
            tx: Some(2),
            amount: None,
        };

        transaction_processor.process_a_record(trans);
        assert_eq!(Some(dec!(2.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().available));
        assert_eq!(Some(dec!(0.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().held));
        assert_eq!(Some(dec!(2.0)),Some(transaction_processor.client_accounts.get(&account_number).unwrap().total));
        assert_eq!(Some(true),Some(transaction_processor.client_accounts.get(&account_number).unwrap().locked));
    }

    #[test]
    fn withdrawal_should_not_create_new_account() {
        let mut transaction_processor = Transactor::new();

        let trans_1 = Transaction {
            transaction_type: Some(TransactionTypes::Withdrawal),
            client: Some(1),
            tx: Some(1),
            amount: Some(dec!(10.0)),
        };

        transaction_processor.process_a_record(trans_1);
        assert!(transaction_processor.client_accounts.is_empty());
    }

    #[test]
    fn dispute_should_not_create_new_account() {
        let mut transaction_processor = Transactor::new();

        let trans_1 = Transaction {
            transaction_type: Some(TransactionTypes::Dispute),
            client: Some(1),
            tx: Some(1),
            amount: Some(dec!(10.0)),
        };

        transaction_processor.process_a_record(trans_1);
        assert!(transaction_processor.client_accounts.is_empty());
    }

    #[test]
    fn chargeback_should_not_create_new_account() {
        let mut transaction_processor = Transactor::new();

        let trans_1 = Transaction {
            transaction_type: Some(TransactionTypes::Chargeback),
            client: Some(1),
            tx: Some(1),
            amount: Some(dec!(10.0)),
        };

        transaction_processor.process_a_record(trans_1);
        assert!(transaction_processor.client_accounts.is_empty());
    }

    #[test]
    fn invalid_record_variants() {
        let mut a_record = Transaction {
            transaction_type: Some(TransactionTypes::Deposit),
            client: Some(0),
            tx: Some(0),
            amount: Some(dec!(0.0)),
        };

        assert!(Transactor::is_record_valid(&a_record));

        a_record = Transaction {
            transaction_type: Some(TransactionTypes::Deposit),
            client: Some(0),
            tx: None,
            amount: Some(dec!(0.0)),
        };

        assert_eq!(false, Transactor::is_record_valid(&a_record));

        a_record = Transaction {
            transaction_type: None,
            client: Some(0),
            tx: Some(0),
            amount: Some(dec!(0.0)),
        };

        assert_eq!(false, Transactor::is_record_valid(&a_record));
    }

    #[test]
    fn duplicate_tx_is_ignored() {
        let mut transaction_processor = Transactor::new();

        let trans_1 = Transaction {
            transaction_type: Some(TransactionTypes::Deposit),
            client: Some(1),
            tx: Some(1),
            amount: Some(dec!(10.0)),
        };
        let trans_2 = Transaction {
            transaction_type: Some(TransactionTypes::Deposit),
            client: Some(1),
            tx: Some(1),
            amount: Some(dec!(5.0)),
        };

        transaction_processor.process_a_record(trans_1);
        transaction_processor.process_a_record(trans_2);

        assert_eq!(Some(dec!(10.0)),Some(transaction_processor.client_accounts.get(&1).unwrap().available));
    }
}
