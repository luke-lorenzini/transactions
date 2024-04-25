use std::collections::{HashMap, HashSet};

// use log::{debug, warn};
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
    pub disputes: HashSet<u32>,
}

pub struct Transactor {
    // TODO: remove public access and make getter to pass tests
    pub client_accounts: HashMap<u16, Account>,
}

impl Default for Transactor {
    fn default() -> Self {
        Self::new()
    }
}

impl Transactor {
    pub fn new() -> Transactor {
        let client_accounts = HashMap::new();
        Transactor { client_accounts }
    }

    pub fn process_a_record(&mut self, record: Transaction) {
        match record.transaction_type.expect("Record is valid") {
            TransactionTypes::Deposit => {
                // debug!("Deposit:{:?}", record);

                match self
                    .client_accounts
                    .get_mut(&record.client.expect("Client exists"))
                {
                    Some(v) => {
                        // Check here that a duplicate transaction record doesn't exist
                        if v.transactions.get(&record.tx.expect("Tx exists")).is_none() {
                            let amount = record.amount.expect("Amount exists");

                            // A deposit is a credit to the client's asset
                            // account, meaning it should increase the available
                            // and total funds of the client account
                            v.available += amount;
                            v.total += amount;
                            v.transactions.insert(record.tx.expect("Tx exists"), record);
                        }
                    }
                    None => {
                        let amount = record.amount.expect("Amount exists");
                        let mut transaction = HashMap::new();
                        transaction.insert(record.tx.expect("Tx exists"), record);

                        self.client_accounts.insert(
                            record.client.expect("Client exists"),
                            Account {
                                available: amount,
                                held: dec!(0.0),
                                total: amount,
                                locked: false,
                                transactions: transaction,
                                disputes: HashSet::new(),
                            },
                        );
                    }
                }
            }
            TransactionTypes::Withdrawal => {
                // debug!("Withdrawal:{:?}", record);

                if let Some(v) = self
                    .client_accounts
                    .get_mut(&record.client.expect("Client exists"))
                {
                    // Check here that a duplicate transaction record doesn't exist
                    if v.transactions.get(&record.tx.expect("Tx exists")).is_none() {
                        let amount = record.amount.expect("Amount exists");

                        // If a client does not have sufficient available
                        // funds the withdrawal should fail and the total amount
                        // of funds should not change
                        if v.available >= amount {
                            // A withdraw is a debit to the client's asset account,
                            // meaning it should decrease the available and total
                            // funds of the client account
                            v.available -= amount;
                            v.total -= amount;
                            v.transactions.insert(record.tx.expect("Tx exists"), record);
                        }
                    }
                }
            }
            TransactionTypes::Dispute => {
                // debug!("Disputing a transaction:{:?}", record);

                if let Some(v) = self
                    .client_accounts
                    .get_mut(&record.client.expect("Client exists"))
                {
                    // If condition is met, act. What if available is already less than amount, placing new balance < 0 ?
                    if let Some(local_trans) =
                        v.transactions.get(&record.tx.expect("Transaction exists"))
                    {
                        if !v
                            .disputes
                            .contains(&local_trans.tx.expect("Transaction exists"))
                        {
                            // debug!(
                            //     "Did not find duplicate transaction: {}. Adding",
                            //     local_trans.tx.expect("Transaction exists")
                            // );
                            v.disputes
                                .insert(local_trans.tx.expect("Transaction exists"));

                            let amount = local_trans.amount.expect("Amount is some");

                            // A dispute represents a client's claim that a transaction was erroneous and should be reversed.
                            // The transaction shouldn't be reversed yet but the associated funds should be held.
                            // This means that the clients' available funds should decrease by the amount
                            // disputed, their held funds should increase by the amount disputed, while their total funds should remain the same.
                            v.available -= amount;
                            v.held += amount;
                            // Do we need to add disputes to the ledger ?
                            // v.transactions.insert(record.tx.expect("Tx exists"), record);
                        } else {
                            // warn!(
                            //     "Disputed transaction already exists: {}. Skipping",
                            //     local_trans.tx.expect("Transaction exists")
                            // );
                        }
                    }
                }
            }
            TransactionTypes::Resolve => {
                // debug!("Resolving a dispute:{:?}", record);

                if let Some(v) = self
                    .client_accounts
                    .get_mut(&record.client.expect("Client exists"))
                {
                    // If condition is met, act. What if available is already less than amount, placing new balance < 0 ?
                    if let Some(local_trans) =
                        v.transactions.get(&record.tx.expect("Transaction exists"))
                    {
                        if v.disputes
                            .contains(&local_trans.tx.expect("Transaction exists"))
                        {
                            // debug!(
                            //     "Found matching transaction: {}. Removing",
                            //     local_trans.tx.expect("Transaction exists")
                            // );

                            v.disputes
                                .remove(&local_trans.tx.expect("Transaction exists"));

                            let amount = local_trans.amount.expect("Amount is some");

                            // A resolve represents a resolution to a dispute, releasing the associated held funds. Funds that
                            // were previously disputed are no longer disputed. This means that the clients held funds should
                            // decrease by the amount no longer disputed, their available funds should increase by the amount
                            // no longer disputed, and their total funds should remain the same.
                            v.available += amount;
                            v.held -= amount;
                            // Do we need to add disputes to the ledger ?
                            // v.transactions.insert(record.tx.expect("Tx exists"), record);
                        } else {
                            // warn!(
                            //     "Resolve transaction does not exist: {}. Skipping",
                            //     local_trans.tx.expect("Transaction exists")
                            // );
                        }
                    }
                }
            }
            TransactionTypes::Chargeback => {
                // debug!("Chargeback:{:?}", record);

                if let Some(v) = self
                    .client_accounts
                    .get_mut(&record.client.expect("Client exists"))
                {
                    // If condition is met, act. What if available is already less than amount, placing new balance < 0 ?
                    if let Some(local_trans) =
                        v.transactions.get(&record.tx.expect("Transaction exists"))
                    {
                        if v.disputes
                            .contains(&local_trans.tx.expect("Transaction exists"))
                        {
                            // debug!(
                            //     "Found matching transaction: {}. Removing",
                            //     local_trans.tx.expect("Transaction exists")
                            // );

                            v.disputes
                                .remove(&local_trans.tx.expect("Transaction exists"));

                            let amount = local_trans.amount.expect("Amount is some");

                            // A chargeback is the final state of a dispute and represents the client reversing a transaction.
                            // Funds that were held have now been withdrawn. This means that the clients held funds and total
                            // funds should decrease by the amount previously disputed. If a chargeback occurs the client's
                            // account should be immediately frozen.

                            v.total -= amount;
                            v.held -= amount;
                            v.locked = true;
                            // Do we need to add disputes to the ledger ?
                            // v.transactions.insert(record.tx.expect("Tx exists"), record);
                        } else {
                            // warn!(
                            //     "Chargeback transaction does not exist: {}. Skipping",
                            //     local_trans.tx.expect("Transaction exists")
                            // );
                        }
                    }
                }
            }
        }
    }

    pub fn display_output(&self) {
        println!("client, available, held, total, locked");

        for client in &self.client_accounts {
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

    pub fn is_record_valid(record: &Transaction) -> bool {
        record.transaction_type.is_some() && record.client.is_some() && record.tx.is_some()
        // && record.amount.is_some()
    }
}
