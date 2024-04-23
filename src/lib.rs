use std::{collections::HashMap, fs::File};

use csv::{Error, ReaderBuilder, Trim};
use log::{debug, warn};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Transaction {
    #[serde(alias = "type")]
    pub transaction_type: Option<TransactionTypes>,
    pub client: Option<u16>,
    pub tx: Option<u32>,
    pub amount: Option<Decimal>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum TransactionTypes {
    #[serde(alias = "deposit")]
    Deposit,
    #[serde(alias = "withdrawal")]
    Withdrawal,
    #[serde(alias = "dispute")]
    Dispute,
    #[serde(alias = "resolve")]
    Resolve,
    #[serde(alias = "chargeback")]
    Chargeback,
}

#[derive(Debug)]
pub struct Account {
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
    pub transactions: HashMap<u32, Transaction>,
    pub disputes: Vec<u32>,
}

pub fn read_input_file(file: File) -> Result<HashMap<u16, Account>, Error> {
    let mut client_accounts: HashMap<u16, Account> = HashMap::new();

    let mut csv_reader = ReaderBuilder::new().trim(Trim::All).from_reader(file);

    for result in csv_reader.deserialize() {
        let record: Transaction = result?;
        debug!("{:?}", record);

        if is_record_valid(&record) {
            process_a_record(record, &mut client_accounts);
        } else {
            warn!("Skipping a bad record: {:?}", record);
        }
    }

    Ok(client_accounts)
}

pub fn process_a_record(record: Transaction, client_accounts: &mut HashMap<u16, Account>) {
    match record.transaction_type.expect("Record is valid") {
        TransactionTypes::Deposit => {
            debug!("Deposit:{:?}", record);

            match client_accounts.get_mut(&record.client.expect("Client exists")) {
                Some(v) => {
                    // Check here that a duplicate transaction record doesn't exist
                    if v.transactions.get(&record.tx.expect("Tx exists")).is_none() {
                        let amount = record.amount.expect("Amount exists");

                        v.available += amount;
                        v.total += amount;
                        v.transactions.insert(record.tx.expect("Tx exists"), record);
                    }
                }
                None => {
                    let amount = record.amount.expect("Amount exists");
                    let mut transaction = HashMap::new();
                    transaction.insert(record.tx.expect("Tx exists"), record);

                    client_accounts.insert(
                        record.client.expect("Client exists"),
                        Account {
                            available: amount,
                            held: dec!(0.0),
                            total: amount,
                            locked: false,
                            transactions: transaction,
                            disputes: Vec::new(),
                        },
                    );
                }
            }
        }
        TransactionTypes::Withdrawal => {
            debug!("Withdrawal:{:?}", record);

            if let Some(v) = client_accounts.get_mut(&record.client.expect("Client exists")) {
                // Check here that a duplicate transaction record doesn't exist
                if v.transactions.get(&record.tx.expect("Tx exists")).is_none() {
                    let amount = record.amount.expect("Amount exists");

                    if v.available >= amount {
                        v.available -= amount;
                        v.total -= amount;
                        v.transactions.insert(record.tx.expect("Tx exists"), record);
                    }
                }
            }
        }
        TransactionTypes::Dispute => {
            debug!("Disputing a transaction:{:?}", record);

            if let Some(v) = client_accounts.get_mut(&record.client.expect("Client exists")) {
                // If condition is met, act. What if available is already less than amount, placing new balance < 0 ?
                if let Some(tx) = v.transactions.get(&record.tx.expect("Transaction exists")) {
                    if !v.disputes.contains(&tx.tx.expect("Transaction exists")) {
                        debug!(
                            "Did not find duplicate transaction: {}. Adding",
                            tx.tx.expect("Transaction exists")
                        );
                        v.disputes.push(tx.tx.expect("Transaction exists"));

                        let amount = tx.amount.expect("Amount is some");

                        v.available -= amount;
                        v.held += amount;
                        // Do we need to add disputes to the ledger ?
                        // v.transactions.insert(record.tx.expect("Tx exists"), record);
                    } else {
                        warn!(
                            "Disputed transaction already exists: {}. Skipping",
                            tx.tx.expect("Transaction exists")
                        );
                    }
                }
            }
        }
        TransactionTypes::Resolve => {
            debug!("Resolving a dispute:{:?}", record);

            if let Some(v) = client_accounts.get_mut(&record.client.expect("Client exists")) {
                // If condition is met, act. What if available is already less than amount, placing new balance < 0 ?
                if let Some(tx) = v.transactions.get(&record.tx.expect("Transaction exists")) {
                    if v.disputes.contains(&tx.tx.expect("Transaction exists")) {
                        debug!(
                            "Found matching transaction: {}. Removing",
                            tx.tx.expect("Transaction exists")
                        );
                        v.disputes.swap_remove(0);

                        let amount = tx.amount.expect("Amount is some");

                        v.available += amount;
                        v.held -= amount;
                        // Do we need to add disputes to the ledger ?
                        // v.transactions.insert(record.tx.expect("Tx exists"), record);
                    } else {
                        warn!(
                            "Resolve transaction does not exist: {}. Skipping",
                            tx.tx.expect("Transaction exists")
                        );
                    }
                }
            }
        }
        TransactionTypes::Chargeback => {
            debug!("Chargeback:{:?}", record);

            if let Some(v) = client_accounts.get_mut(&record.client.expect("Client exists")) {
                // If condition is met, act. What if available is already less than amount, placing new balance < 0 ?
                if let Some(tx) = v.transactions.get(&record.tx.expect("Transaction exists")) {
                    if v.disputes.contains(&tx.tx.expect("Transaction exists")) {
                        debug!(
                            "Found matching transaction: {}. Removing",
                            tx.tx.expect("Transaction exists")
                        );
                        v.disputes.swap_remove(0);

                        let amount = tx.amount.expect("Amount is some");

                        // v.available -= amount;
                        v.held -= amount;
                        v.locked = true;
                        // Do we need to add disputes to the ledger ?
                        // v.transactions.insert(record.tx.expect("Tx exists"), record);
                    } else {
                        warn!(
                            "Chargeback transaction does not exist: {}. Skipping",
                            tx.tx.expect("Transaction exists")
                        );
                    }
                }
            }
        }
    }

    for client in client_accounts {
        debug!("{:?}", client);
    }
    debug!("");
}

pub fn is_record_valid(record: &Transaction) -> bool {
    record.transaction_type.is_some() && record.client.is_some() && record.tx.is_some()
    // && record.amount.is_some()
}

pub fn display_output(client_accounts: HashMap<u16, Account>) {
    println!("client, available, held, total, locked");

    for client in client_accounts {
        println!(
            "{}, {}, {}, {}, {}",
            client.0,
            client.1.available.round_dp(4).normalize(),
            client.1.held.round_dp(4).normalize(),
            client.1.total.round_dp(4).normalize(),
            client.1.locked
        );
    }
}
