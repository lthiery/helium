use helium_api::{accounts, transactions::*, Client, IntoVec};
use prettytable::{cell, row, Table};
use std::fs::File;
use structopt::StructOpt;

mod accounting;
mod types;

use accounting::*;
use types::*;

use chrono::{DateTime, Utc};

#[derive(Debug, StructOpt)]
pub struct Cli {
    address: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::from_args();

    let client = Client::default();
    let transactions = accounts::rewards(&client, &cli.address).into_vec().await?;

    let mut table = Table::new();
    table.add_row(row![
        "Type",
        "Date",
        "Block",
        "Hash",
        "Counterparty",
        "HNT",
        "DC",
        "Fee",
    ]);
    for txn in transactions {
        if let Data::RewardsV1(_) = &txn.data {
            table.add_row(txn.to_row(&Address::from_str(&cli.address)?, &client).await);
        }
        if let Data::RewardsV2(_) = &txn.data {
            table.add_row(txn.to_row(&Address::from_str(&cli.address)?, &client).await);
        }
    }

    let time: DateTime<Utc> = Utc::now();
    let out = File::create(format!(
        "{}_{}.csv",
        cli.address,
        time.format("%Y-%m-%d_%H-%M-%S").to_string()
    ))?;
    table.to_csv(out)?;
    Ok(())
}
