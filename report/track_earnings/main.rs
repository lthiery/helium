use chrono::prelude::*;
use chrono::{DateTime, Utc};
use helium_api::{accounts, oracle, Client, IntoVec, models::{Hnt, Usd}};
use prettytable::{cell, row, Row, Table};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use toml;

#[derive(Deserialize, Debug)]
struct Config {
    accounts: HashMap<String, Account>,
}
#[derive(Deserialize, Debug)]
struct Account {
    pubkey: String,
    ownership: f64,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub pubkey: String,
    pub timestamp: String,
    pub hash: String,
    pub block: u64,
    pub amount: Decimal,
    pub oracle_price: Usd,
    pub ownership: i64,
    pub usd_value: Decimal,
}

impl Entry {
    fn into_row(self) -> Row {
        row![
            self.pubkey,
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
use helium_api::models::QueryTimeRange;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = toml::from_str(&fs::read_to_string("accounts.toml")?)?;
    // let min: DateTime<Utc> = Utc.ymd(2021, 1, 1).and_hms(0, 0, 0);
    // let max: DateTime<Utc> = Utc.ymd(2021, 3, 31).and_hms(23, 59, 59);
    let min: DateTime<Utc> = Utc.ymd(2021, 9, 24).and_hms(0, 0, 0);
    let max: DateTime<Utc> = Utc.ymd(2021, 12, 21
    ).and_hms(23, 59, 59);

    let params = QueryTimeRange {
        min_time: min.to_rfc3339(),
        max_time: max.to_rfc3339(),
    };
    let mut grand_table = Table::new();
    let mut details_table = Table::new();

    let grand_out = File::create(format!("output/summary_{}_{}.csv", min, max))?;
    let details_out = File::create(format!("output/details_{}_{}.csv", min, max))?;
    details_table.add_row(row![
            "pubkey",
            "timestamp",
            "hash",
            "block",
            "amount",
            "oracle_price",
            "ownership",
            "usd_value",
        ]);

    let mut grand_total_hnt = Hnt::from(0).get_decimal();
    let mut grand_total_usd = Usd::from(0).get_decimal();

    for (label, account) in config.accounts {
        let out = File::create(format!("output/{}_{}_{}.csv", account.pubkey, min, max))?;

        let mut table = Table::new();
        table.add_row(row![
            "pubkey",
            "timestamp",
            "hash",
            "block",
            "amount",
            "oracle_price",
            "ownership",
            "usd_value",
        ]);

        let client = Client::new_with_base_url("https://helium-api.stakejoy.com/v1/".to_string(), "helium-ledger-cli/2.1.2");

        let mut rewards_result = accounts::rewards(&client, &account.pubkey, &params)
            .into_vec()
            .await;

        while rewards_result.is_err() {
            println!("trying again");
            rewards_result = accounts::rewards(&client, &account.pubkey, &params)
                .into_vec()
                .await;
        }
        let rewards = rewards_result?;

        let mut total_hnt = Hnt::from(0).get_decimal();
        let mut total_usd = Usd::from(0).get_decimal();
        for reward in rewards {
            let mut oracle_price_result = oracle::prices::at_block(&client, reward.block).await;
            while oracle_price_result.is_err() {
                oracle_price_result = oracle::prices::at_block(&client, reward.block).await;
            }
            let oracle_price = oracle_price_result?.price;

            let ownership = (account.ownership * 100.00) as i64;
            let usd_value = reward.amount.get_decimal()
                * oracle_price.get_decimal()
                * Decimal::new(ownership, 2);
            let amount = reward.amount.get_decimal() * Decimal::new(ownership, 2);

            total_usd += usd_value;
            total_hnt += amount;
            let row = Entry {
                pubkey: account.pubkey.clone(),
                timestamp: reward.timestamp.to_rfc3339(),
                hash: reward.hash,
                block: reward.block,
                amount,
                oracle_price,
                ownership,
                usd_value,
            };
            let row = row.into_row();
            table.add_row(row.clone());
            details_table.add_row(row);
        }
        grand_total_hnt += total_hnt;
        grand_total_usd += total_usd;
        table.add_row(row!["", "", "", "", total_hnt, "", "", total_usd]);
        grand_table.add_row(row![label, account.pubkey, total_hnt, total_usd]);

        println!("{:28} {:38} {:25} {:25}", label, account.pubkey, total_hnt, total_usd);
        table.to_csv(out)?;
    }
    grand_table.add_row(row!["", "", grand_total_hnt, grand_total_usd]);

    println!("{:106} {:25}", grand_total_hnt, grand_total_usd);
    grand_table.to_csv(grand_out)?;
    details_table.to_csv(details_out)?;
    Ok(())
}
