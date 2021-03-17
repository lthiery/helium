use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::result::Result;

mod types;
use types::*;

pub struct Client {
    address: String,
    client: reqwest::Client,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(tag = "method")]
#[serde(rename_all = "snake_case")]
enum Method {
    WalletList,
    BlockHeight,
    WalletCreate { params: Password },
    WalletUnlock { params: UnlockParams },
    WalletLock { params: LockParams },
    PendingTransactionStatus { params: PendingTxnStatus },
    WalletPay { params: PaymentParams },
}

impl Client {
    pub fn new(address: String) -> Client {
        Client {
            address,
            client: reqwest::Client::new(),
        }
    }

    pub async fn post<T: DeserializeOwned, D: Serialize>(
        &self,
        data: D,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let request = self.client.post(&self.address).json(&data);
        let body = request.send().await?.text().await?;
        let result: T = serde_json::from_str(&body)?;
        Ok(result)
    }

    pub async fn get_height(&self) -> Result<usize, Box<dyn std::error::Error>> {
        #[derive(Clone, Deserialize, Debug)]
        struct Response {
            result: usize,
        }

        let request = JsonRpc::new(Method::BlockHeight);
        let result: Response = self.post(request).await?;
        Ok(result.result)
    }

    pub async fn pending_transaction_status(
        &self,
        hash: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        #[derive(Clone, Deserialize, Debug)]
        struct Response {
            result: String,
        }

        let request = JsonRpc::new(Method::PendingTransactionStatus {
            params: PendingTxnStatus { hash },
        });
        let result: Response = self.post(request).await?;
        Ok(result.result)
    }

    pub async fn create_wallet(
        &self,
        password: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let request = JsonRpc::new(Method::WalletCreate {
            params: Password { password },
        });
        let result: Response = self.post(request).await?;
        Ok(result.result)
    }

    pub async fn lock_wallet(&self, address: String) -> Result<bool, Box<dyn std::error::Error>> {
        #[derive(Clone, Deserialize, Debug)]
        struct Response {
            result: bool,
        }

        let request = JsonRpc::new(Method::WalletLock {
            params: LockParams { address },
        });
        let result: Response = self.post(request).await?;
        Ok(result.result)
    }
    pub async fn list_wallets(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        #[derive(Clone, Deserialize, Debug)]
        struct Response {
            result: Vec<String>,
        }

        let request = JsonRpc::new(Method::WalletList);
        let result: Response = self.post(request).await?;
        Ok(result.result)
    }

    pub async fn unlock_wallet(
        &self,
        address: &str,
        password: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let request = JsonRpc::new(Method::WalletUnlock {
            params: UnlockParams {
                password: password.into(),
                address: address.into(),
            },
        });

        #[derive(Deserialize)]
        struct Response {
            result: bool,
        }
        let result: Response = self.post(request).await?;
        Ok(result.result)
    }

    pub async fn pay(
        &self,
        address: &str,
        payee: &str,
        bones: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        #[derive(Clone, Deserialize, Debug)]
        struct Response {
            result: Result,
        }
        #[derive(Clone, Deserialize, Debug)]
        struct Result {
            hash: String,
        }

        let request = JsonRpc::new(Method::WalletPay {
            params: PaymentParams {
                address: address.to_string(),
                payee: payee.to_string(),
                bones,
            },
        });
        let result: Response = self.post(request).await?;
        Ok(result.result.hash)
    }
}
