# transactions

[![Rust](https://github.com/luke-lorenzini/transactions/actions/workflows/rust.yml/badge.svg)](https://github.com/luke-lorenzini/transactions/actions/workflows/rust.yml)

## Structure

This implementation reads a csv file using an MPSC (multi-producer single consumer) fashion. It is designed to be multi threaded where there are N producer threads and 1 consumer thread. This allows for parallel throughput of large, separate datasets.

When a record is read in a producer thread it is immediately (after verification) sent down a channel to a consumer thread, effectively ingesting a csv and consuming at the same time. The implementation can easily be swapped (or created in another thread) to receive stream data from the web, or other data sources. This would require the use of an Async runtime such as Tokio, but has been omitted for simplicity.

Dispute tracking is managed via two mechanisms, the first is an in-memory hashmap of transactions, to determine their IDs and values, the second is a vector to track existing disputes.

This program implements the 5 simple transaction types:

- Deposit
- Withdrawal
- Dispute
- Resolve
- Chargeback

Each of the transaction types closely follows the guidelines laid out in the instructions.

Some additions have been added to the logic to enable transaction logging. Each account uses a HashMap in order to store record of the the transactions which have been performed on it. This enables disputes and dispute tracking.

## Assumptions / Points of note

1) Disputes, Resolutions, and Chargebacks are not logged in the ledger.
2) Duplicate transaction IDs are silently dropped.

## TODO

This program is designed to scale via the use of MPSC threading model. A new thread could be spawned which adds more records to the channel for the receiver to process. However, an in-memory map has been created for transaction (ledger) logging. Given that transaction IDs are u32, it is infeasable to maintain the entire ledge in volatile memory. A more scaleable solution is to occassionaly write the ledge to persistent storage. However, not having additional requirements means there are potential many solutions.
