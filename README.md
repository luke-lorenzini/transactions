# transactions

[![Rust](https://github.com/luke-lorenzini/transactions/actions/workflows/rust.yml/badge.svg)](https://github.com/luke-lorenzini/transactions/actions/workflows/rust.yml)

## Structure

This implementation reads a csv file using an mpsc (multi-producer single consumer) model. It is designed to be multi threaded where there are N producer threads and 1 consumer thread. This allows for parallel throughput of large, separate datasets.

After a record is read by a producer thread (and validated) it is immediately sent down a channel to a consumer thread, effectively ingesting a csv and consuming at the same time. The implementation in one of the producer threads can easily be swapped (or created in another thread) to receive a stream of data from the web, or other data sources.

The main.rs file contains the logic for running the three steps of the program, input -> process -> output. All of the actual logic is contained in lib.rs. There is a struct, Transactor, which can be used for all the heavy lifting.

This program implements the 5 simple transaction types:

- Deposit
- Withdrawal
- Dispute
- Resolve
- Chargeback

Each of the transaction types closely the guidelines laid out in the instructions (with a degree of interpretation, described in 'Assumptions' below).

Some additions have been added to the logic to enable transaction logging. Each account uses a HashMap in order to store record of the the transactions which have been performed on it. This enables disputes and dispute tracking.

Dispute tracking is managed via two mechanisms, the first is an in-memory hashmap of transactions, to determine their IDs and values, the second is a hashset to track existing disputes.

## Running

```bash
cargo run -- filename.csv > output_file.csv
```

## Tests

A comprehensive set of tests to exercise the logic exists in the tests folder, simply run:

```bash
cargo test
```

## Optional feature

A feature has been created to demonstrate multi-threaded input. It lives in main.rs.

The feature is called 'additional_task'. It contains a hard-coded file name which needs to be changed by the user at compile time (demo purposes only). The body of the task could easily be swapped to consume data from the web to be processed.

```bash
cargo run --features=additional_task -- filename.csv > output_file.csv
```

## Assumptions / Points of note

1) Disputes are handled the same for both withdrawals and deposits. I feel as though the logic should be modified (i.e. negate the 'amount' when disputing a withdrawal). However, without sufficient test data, I do not want to assume this.
2) Disputes, Resolutions, and Chargebacks are not logged in the transaction ledger.
3) Duplicate transaction IDs are silently dropped.
4) If an account has been frozen due to a chargeback, it can later be transacted on. I suspect if it has been frozen, further transactions should be blocked, but do not see mention of this in the instructions.
5) The program handles only good input data. Additional columns or separators, and incorrect types are not handled as the instructions state input is valid.

## TODO

This program is designed to scale via the use of mpsc threading model. A new thread could be spawned which adds more records to the channel for the receiver to process. However, an in-memory hashmap has been created for the transaction (ledger). Given that transaction IDs are u32, it is infeasible to maintain the entire ledger in volatile memory. A more scalable solution would be to occasionally write the ledger to persistent storage. However, not having additional requirements means there are potential many solutions.

## Example transactions

### Resolve Example

Here is an example of a dispute / resolution combination with expected value. This code is implemented in the test case 'dispute_then_resolve_outcome'.

transaction, amount | avail, held, total

deposit: 10:    | 10 , 0 , 10

deposit: 5:     | 15 , 0 , 15

withdrawal: 6:  |  9 , 0 ,  9

withdrawal: 2:  |  7 , 0 ,  7

dispute: 5:     |  2 , 5 ,  7

resolve: 5:     |  7 , 0 ,  7

### Chargeback Example

This code is implemented in the test case 'dispute_then_chargeback_outcome'.

transaction, amount | avail, held, total

deposit: 10:    | 10 , 0 , 10

deposit: 5:     | 15 , 0 , 15

withdrawal: 6:  |  9 , 0 ,  9

withdrawal: 2:  |  7 , 0 ,  7

dispute: 5:     |  2 , 5 ,  7

chargeback: 5:  |  2 , 0 ,  2

## Tools

A helper program, 'generate_data.py' can be found in the 'tools' folder. This program can be used to generate large, arbitrary datasets. A key parameter is 'number_of_rows'. Use this to adjust the file size. The filename can also be modified.

```bash
python tools/generate_data.py
```

## Test Data

Test files have been removed due to submission requirements.
