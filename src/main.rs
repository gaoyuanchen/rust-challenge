use csv::ReaderBuilder;
use rust_challenge::transaction::parse_transaction;
use rust_challenge::types::{AccountProfile, ClientId, CsvInputRow};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;

/// Process the transactions inside csv file from `path` and mutate states in `accounts`
fn process_csv(
    accounts: &mut HashMap<ClientId, AccountProfile>,
    path: &str,
) -> Result<(), Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(file);

    // We will ignore all errors:
    // 1. csv parsing for a row
    // 2. transaction processing rejection (as instructed)
    // Note that we will not print error message and ignore them silently
    // We do this because we use stdout for the output, and we want to keep it clean
    for row in rdr.deserialize::<CsvInputRow>().flatten() {
        if let Ok(transaction) = parse_transaction(&row) {
            _ = accounts
                .entry(row.client)
                .or_default()
                .process_transaction(row.tx, transaction);
        }
    }
    Ok(())
}

fn output_accounts(accounts: &HashMap<ClientId, AccountProfile>) {
    println!("client,available,held,total,locked");
    // This will output clients in arbitrary order, but it is fine as mentioned in the instructions
    for (id, p) in accounts {
        // Output with 4 digits after decimal point
        println!(
            "{},{:.4},{:.4},{:.4},{}",
            id,
            p.available,
            p.held,
            p.available + p.held,
            p.frozen
        );
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = &env::args()
        .nth(1)
        .ok_or("missing argument: path to input csv file")?;
    let mut accounts: HashMap<ClientId, AccountProfile> = HashMap::new();
    process_csv(&mut accounts, path)?;
    output_accounts(&accounts);
    Ok(())
}
