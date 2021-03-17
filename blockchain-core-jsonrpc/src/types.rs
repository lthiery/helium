use super::Method;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Debug)]
pub(crate) struct Response {
    pub result: String,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub(crate) struct JsonRpc {
    jsonrpc: String,
    id: String,
    #[serde(flatten)]
    method: Method,
}
impl JsonRpc {
    pub fn new(request: Method) -> JsonRpc {
        JsonRpc {
            jsonrpc: "2.0".into(),
            id: rand::random::<usize>().to_string(),
            method: request,
        }
    }
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub(crate) struct UnlockParams {
    pub address: String,
    pub password: String,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub(crate) struct LockParams {
    pub address: String,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub(crate) struct Password {
    pub password: String,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub(crate) struct PaymentParams {
    pub address: String,
    pub payee: String,
    pub bones: usize,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub(crate) struct PendingTxnStatus {
    pub hash: String,
}
