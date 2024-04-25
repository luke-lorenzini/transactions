use std::{env, error::Error, fs::File/*, time::Instant*/};

use csv::{ReaderBuilder, Trim};
// use log::{debug, trace, warn};
use tokio::sync::mpsc::channel;

use transactions::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let input_file = &args[1];
    // debug!("{:?}", input_file);

    let file = File::open(input_file)?;

    let mut transaction_processor = Transactor::new();

    // let start_time = Instant::now();

    let (tx, mut rx) = channel(32);

    // Note to tester:
    // To use multiple input-processing threads simply build with the feature 'additional_task'
    // cargo build --features=additional_task
    // This will create an additional async thread to process an additional input file.
    // NOTE: FILE NAME BELOW IS HARDCODED AND MUST BE PRESENT.
    #[cfg(feature = "additional_task")]
    tokio::spawn({
        let tx = tx.clone();
        async move {
            // !!!ENSURE THE FILE EXISTS!!!
            let file = File::open("input_data/test_data_2.csv");
            let file = file.unwrap();
            let mut csv_reader = ReaderBuilder::new().trim(Trim::All).from_reader(file);

            for result in csv_reader.deserialize() {
                let record: Transaction = result.unwrap();
                // debug!("{:?}", record);

                if Transactor::is_record_valid(&record) {
                    tx.send(record).await.unwrap();
                } else {
                    // warn!("Skipping a bad record: {:?}", record);
                }
            }

            // debug!("Closing thread 2");
        }
    });

    tokio::spawn({
        async move {
            let mut csv_reader = ReaderBuilder::new().trim(Trim::All).from_reader(file);

            for result in csv_reader.deserialize() {
                let record: Transaction = result.unwrap();
                // debug!("{:?}", record);

                if Transactor::is_record_valid(&record) {
                    tx.send(record).await.unwrap();
                } else {
                    // warn!("Skipping a bad record: {:?}", record);
                }
            }

            // debug!("Closing thread 1");
        }
    });

    // let mut record_count = 0;

    while let Some(received) = rx.recv().await {
        // record_count += 1;
        transaction_processor.process_a_record(received);
    }

    transaction_processor.display_output();

    // let duration = start_time.elapsed();
    // trace!("Processed {} records in {:?}", record_count, duration);

    Ok(())
}
