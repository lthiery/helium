use chrono::prelude::*;
use chrono::{DateTime, Utc};
use helium_api::{accounts, oracle, Client, IntoVec, models::{Hnt, Usd}};
use prettytable::{cell, row, Row, Table};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use toml;

use std::error::Error;
use csv::{Writer, Reader};

#[derive(Deserialize, Debug)]
struct Config {
    accounts: HashMap<String, Account>,
}
#[derive(Deserialize, Debug)]
struct Account {
    pubkey: String,
    ownership: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Entry {
    pub timestamp: String,
    pub hash: String,
    pub block: u64,
    pub amount: Decimal,
    pub oracle_price: Usd,
    pub ownership: i64,
    pub usd_value: Decimal,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewEntry {
    pub pubkey: String,
    pub timestamp: String,
    pub hash: String,
    pub block: u64,
    pub amount: Decimal,
    pub oracle_price: Usd,
    pub ownership: i64,
    pub usd_value: Decimal,
}

impl NewEntry {
    fn from_entry(entry:Entry, pubkey: String) -> NewEntry {
        NewEntry {
            pubkey,
            timestamp: entry.timestamp,
            hash: entry.hash,
            block: entry.block,
            amount: entry.amount,
            oracle_price: entry.oracle_price,
            ownership: entry.ownership,
            usd_value: entry.usd_value,
        }

    }
}

impl Entry {
    fn into_row(self) -> Row {
        row![
            self.timestamp,
            self.hash,
            self.block,
            self.amount,
            self.oracle_price,
            self.ownership,
            self.usd_value,
        ]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = toml::from_str(&fs::read_to_string("accounts.toml")?)?;

    let min: DateTime<Utc> = Utc.ymd(2021, 1, 1).and_hms(0, 0, 0);
    let max: DateTime<Utc> = Utc.ymd(2021, 5, 31
    ).and_hms(23, 59, 59);

    let mut grand_out = Writer::from_path(format!("output/details_{}_{}.csv", min, max))?;

    for (label, account) in config.accounts {
        println!("Importing {}", label);
        let mut rdr = Reader::from_path(format!("output/{}_{}_{}.csv", account.pubkey, min, max))?;
        for result in rdr.deserialize() {
            let result: Result<Entry, _>=  result;
            match result {
                Ok(entry) => {
                    let new_entry = NewEntry::from_entry(entry, account.pubkey.clone());
                    grand_out.serialize(new_entry);
                },
                Err(e) => println!("EOF"),
            }
        }
    }
    grand_out.flush();
    Ok(())
}
