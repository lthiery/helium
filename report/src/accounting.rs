use super::*;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use helium_api::{oracle, Dc, Hnt};
use prettytable::{Cell, Row};

#[async_trait]
pub trait ToRow {
    async fn to_row(&self, account: &Address, client: &Client) -> Row;
}

#[async_trait]
trait GetDifference {
    async fn get_difference(&self, account: &Address, client: &Client, height: u64) -> Difference;
}

#[async_trait]
impl GetDifference for PaymentV1 {
    async fn get_difference(
        &self,
        account: &Address,
        _client: &Client,
        _height: u64,
    ) -> Difference {
        let fee = self.proto.fee;

        let counterparty = Some(
            Address::from_vec(self.proto.payee.clone())
                .as_string()
                .clone(),
        );
        // This account is paying HNT
        if self.proto.payer == *account.as_vec() {
            Difference {
                counterparty,
                hnt: Hnt::from(-(self.proto.amount as isize)),
                dc: Dc::from(0),
                fee,
            }
        }
        // this account is receiving HNT
        else {
            Difference {
                counterparty,
                hnt: Hnt::from(self.proto.amount),
                dc: Dc::from(0),
                fee,
            }
        }
    }
}

#[async_trait]
impl GetDifference for PaymentV2 {
    async fn get_difference(
        &self,
        account: &Address,
        _client: &Client,
        _height: u64,
    ) -> Difference {
        let fee = self.proto.fee;

        // This account is paying HNT
        if self.proto.payer == *account.as_vec() {
            let counterparty = Some(if self.proto.payments.len() == 1 {
                Address::from_vec(self.proto.payments[0].payee.clone())
                    .as_string()
                    .clone()
            } else {
                "many_payees".to_string()
            });
            let mut hnt = Hnt::from(0).get_decimal();
            for payment in &self.proto.payments {
                hnt -= Hnt::from(payment.amount).get_decimal();
            }
            Difference {
                counterparty,
                hnt: Hnt::new(hnt),
                dc: Dc::from(0),
                fee,
            }
        }
        // this account is receiving HNT
        else {
            let counterparty = Some(Address::from_vec(self.proto.payer.clone()).to_string());
            let mut hnt = Hnt::from(0).get_decimal();
            for payment in &self.proto.payments {
                if payment.payee == *account.as_vec() {
                    hnt += Hnt::from(payment.amount).get_decimal();
                }
            }
            Difference {
                counterparty,
                hnt: Hnt::new(hnt),
                dc: Dc::from(0),
                fee,
            }
        }
    }
}

#[async_trait]
impl GetDifference for RewardsV1 {
    async fn get_difference(
        &self,
        _account: &Address,
        _client: &Client,
        _height: u64,
    ) -> Difference {
        let mut hnt = Hnt::from(0).get_decimal();
        // summate rewards for all reward types
        for reward in &self.proto.rewards {
            hnt += Hnt::from(reward.amount).get_decimal();
        }

        Difference {
            counterparty: Some("Rewards".to_string()),
            hnt: Hnt::new(hnt),
            dc: Dc::from(0),
            fee: 0,
        }
    }
}

#[async_trait]
impl GetDifference for TokenBurnV1 {
    async fn get_difference(&self, account: &Address, client: &Client, height: u64) -> Difference {
        // This account is burning HNT
        let (hnt, counterparty) = if self.proto.payer == *account.as_vec() {
            (
                Hnt::from(-(self.proto.amount as isize)),
                Some(Address::from_vec(self.proto.payee.clone()).to_string()),
            )
        }
        // This account is not burning any HNT,
        // so it must just be receiving the DC
        else {
            (
                Hnt::from(self.proto.amount),
                Some(Address::from_vec(self.proto.payer.clone()).to_string()),
            )
        };

        // This account is receiving DC
        let dc: Dc = if self.proto.payee == *account.as_vec() {
            let oracle_price = oracle::prices::at_block(client, height).await.unwrap();
            Dc::new(Hnt::from(self.proto.amount).get_decimal() * oracle_price.price.get_decimal())
        }
        // This account is not receiving HNT
        else {
            Dc::from(0)
        };

        let fee = self.proto.fee;

        Difference {
            counterparty,
            hnt,
            dc,
            fee,
        }
    }
}

struct Metadata {
    height: usize,
    hash: String,
    time: usize,
}

#[async_trait]
trait ToRowWithMetadata {
    async fn to_row_with_metadata(
        &self,
        account: &Address,
        client: &Client,
        metadata: Metadata,
    ) -> Row;
}

macro_rules! to_row {
    ($self:ident, $txn:ident, $account:ident, $client:ident) => {{
        let metadata = $self.get_metadata();
        $txn.to_row_with_metadata($account, $client, metadata).await
    }};
}

#[async_trait]
impl ToRow for Transaction {
    async fn to_row(&self, account: &Address, client: &Client) -> Row {
        match &self.data {
            Data::PaymentV1(payment) => to_row!(self, payment, account, client),
            Data::PaymentV2(payment_v2) => to_row!(self, payment_v2, account, client),
            Data::RewardsV1(reward) => to_row!(self, reward, account, client),
            Data::TokenBurnV1(burn) => to_row!(self, burn, account, client),
            Data::AddGatewayV1(add_gateway) => to_row!(self, add_gateway, account, client),
            Data::AssertLocationV1(assert_location) => {
                to_row!(self, assert_location, account, client)
            }
            Data::CoinbaseV1(coinbase) => to_row!(self, coinbase, account, client),
            Data::CreateHtlcV1(create_htlc) => to_row!(self, create_htlc, account, client),
            Data::GenGatewayV1(gen_gateway) => to_row!(self, gen_gateway, account, client),
            Data::ConsensusGroupV1(consensus_group) => {
                to_row!(self, consensus_group, account, client)
            }
            Data::OuiV1(oui) => to_row!(self, oui, account, client),
            Data::PocReceiptsV1(poc_receipts) => {
                to_row!(self, poc_receipts, account, client)
            }
            Data::PocRequestV1(poc_request) => to_row!(self, poc_request, account, client),
            Data::RedeemHtlcV1(redeem_htlc) => to_row!(self, redeem_htlc, account, client),
            Data::SecurityCoinbaseV1(security_coinbase) => {
                to_row!(self, security_coinbase, account, client)
            }
            Data::RoutingV1(routing) => to_row!(self, routing, account, client),
            Data::SecurityExchangeV1(security_exchange) => {
                to_row!(self, security_exchange, account, client)
            }
            Data::VarsV1(vars) => to_row!(self, vars, account, client),
            Data::DcCoinbaseV1(dc_coinbase) => to_row!(self, dc_coinbase, account, client),
            Data::TokenBurnExchangeRateV1(token_burn_exchange_rate) => {
                to_row!(self, token_burn_exchange_rate, account, client)
            }
            Data::BundleV1(bundle) => to_row!(self, bundle, account, client),

            Data::StateChannelOpenV1(state_channel_open) => {
                to_row!(self, state_channel_open, account, client)
            }

            Data::UpdateGatewayOuiV1(update_gateway_oui) => {
                to_row!(self, update_gateway_oui, account, client)
            }

            Data::StateChannelCloseV1(state_channel_close) => {
                to_row!(self, state_channel_close, account, client)
            }
            Data::PriceOracleV1(price_oracle) => {
                to_row!(self, price_oracle, account, client)
            }

            Data::GenPriceOracleV1(gen_price_oracle) => {
                to_row!(self, gen_price_oracle, account, client)
            }

            Data::TransferHotspotV1(transfer_hotspot) => {
                to_row!(self, transfer_hotspot, account, client)
            }
        }
    }
}

macro_rules! dummy_difference {
    ($txn:ident) => {
        #[async_trait]
        impl GetDifference for $txn {
            async fn get_difference(
                &self,
                _account: &Address,
                _client: &Client,
                _height: u64,
            ) -> Difference {
                Difference {
                    counterparty: None,
                    hnt: Hnt::from(0),
                    dc: Dc::from(0),
                    fee: 0,
                }
            }
        }
    };
}

trait GetMetadata {
    fn get_metadata(&self) -> Metadata;
}

impl GetMetadata for Transaction {
    fn get_metadata(&self) -> Metadata {
        Metadata {
            height: self.height,
            time: self.time,
            hash: self.hash.clone(),
        }
    }
}

fn utc_timestamp_from_epoch(time: usize) -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(time as i64, 0), Utc)
}

fn get_common_rows(metadata: &Metadata) -> (Cell, Cell, Cell) {
    (
        Cell::new(&utc_timestamp_from_epoch(metadata.time).to_rfc3339()),
        Cell::new(format!("{}", metadata.height).as_str()),
        Cell::new(&metadata.hash.to_string()),
    )
}

macro_rules! into_row {
    ($Txn:ident, $Label:expr) => {
        #[async_trait]
        impl ToRowWithMetadata for $Txn {
            async fn to_row_with_metadata(
                &self,
                account: &Address,
                client: &Client,
                metadata: Metadata,
            ) -> Row {
                // use metadata to generate the first few rows that are common
                let common = get_common_rows(&metadata);
                // calculate the effect on the account
                let difference = self
                    .get_difference(account, client, metadata.height as u64)
                    .await;

                // extract counterparty for row if there is one
                let counterparty = if let Some(counterparty) = &difference.counterparty {
                    counterparty.as_str()
                } else {
                    "NA"
                };

                Row::new(vec![
                    Cell::new(format!("{: <25}", $Label).as_str()),
                    common.0,
                    common.1,
                    common.2,
                    Cell::new(&counterparty),
                    Cell::new(format!("{}", &difference.hnt.to_string()).as_str()),
                    Cell::new(format!("{}", (&difference.dc.to_string())).as_str()),
                    Cell::new(format!("{}", (&difference.fee.to_string())).as_str()),
                ])
            }
        }
    };
}

into_row!(AddGatewayV1, "AddGatewayV1");
into_row!(AssertLocationV1, "AssertLocationV1");
into_row!(CoinbaseV1, "CoinbaseV1");
into_row!(CreateHtlcV1, "CreateHtlcV1");
into_row!(GenGatewayV1, "GenGatewayV1");
into_row!(ConsensusGroupV1, "ConsensusGroupV1");
into_row!(OuiV1, "OuiV1");
into_row!(PaymentV1, "PaymentV1");
into_row!(PocReceiptsV1, "PocReceiptsV1");
into_row!(PocRequestV1, "PocRequestV1");
into_row!(RedeemHtlcV1, "RedeemHtlcV1");
into_row!(SecurityCoinbaseV1, "SecurityCoinbaseV1");
into_row!(RoutingV1, "RoutingV1");
into_row!(SecurityExchangeV1, "SecurityExchangeV1");
into_row!(VarsV1, "VarsV1");
into_row!(RewardsV1, "RewardsV1");
into_row!(TokenBurnV1, "TokenBurnV1");
into_row!(DcCoinbaseV1, "DcCoinbaseV1");
into_row!(TokenBurnExchangeRateV1, "TokenBurnExchangeRateV1");
into_row!(StateChannelOpenV1, "StateChannelOpenV1");
into_row!(UpdateGatewayOuiV1, "UpdateGatewayOuiV1");
into_row!(StateChannelCloseV1, "StateChannelCloseV1");
into_row!(PaymentV2, "PaymentV2");
into_row!(PriceOracleV1, "PriceOracleV1");
into_row!(GenPriceOracleV1, "GenPriceOracleV1");
into_row!(BundleV1, "BundleV1");
into_row!(TransferHotspotV1, "TransferHotspotV1");

dummy_difference!(AddGatewayV1);
dummy_difference!(AssertLocationV1);
dummy_difference!(CoinbaseV1);
dummy_difference!(CreateHtlcV1);
dummy_difference!(GenGatewayV1);
dummy_difference!(ConsensusGroupV1);
dummy_difference!(OuiV1);
dummy_difference!(PocReceiptsV1);
dummy_difference!(PocRequestV1);
dummy_difference!(RedeemHtlcV1);
dummy_difference!(SecurityCoinbaseV1);
dummy_difference!(RoutingV1);
dummy_difference!(SecurityExchangeV1);
dummy_difference!(VarsV1);
dummy_difference!(DcCoinbaseV1);
dummy_difference!(TokenBurnExchangeRateV1);
dummy_difference!(StateChannelOpenV1);
dummy_difference!(UpdateGatewayOuiV1);
dummy_difference!(StateChannelCloseV1);
dummy_difference!(PriceOracleV1);
dummy_difference!(GenPriceOracleV1);
dummy_difference!(BundleV1);
dummy_difference!(TransferHotspotV1);
