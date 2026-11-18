//! Soroban RPC client for `getEvents` and `getHealth`.
//!
//! Events are requested with `xdrFormat: "json"`, so topics and values arrive
//! already decoded (`topicJson`, `valueJson`) and no XDR parsing is needed.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::Error;

/// A contract event as returned by `getEvents` with `xdrFormat: "json"`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcEvent {
    /// Unique, ordered RPC event id (for example `0024909839654191104-0000000000`).
    pub id: String,
    pub ledger: i64,
    /// RFC 3339 close time of the ledger.
    pub ledger_closed_at: String,
    pub contract_id: String,
    pub tx_hash: String,
    pub topic_json: Vec<Value>,
    pub value_json: Value,
}

/// One page of `getEvents` results.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventsPage {
    pub events: Vec<RpcEvent>,
    /// Cursor of the last event in the page; pass it back to continue.
    pub cursor: Option<String>,
    pub latest_ledger: i64,
    pub oldest_ledger: i64,
}

/// `getHealth` result. `ledger_retention_window` is how many ledgers of
/// events the RPC keeps.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    pub status: String,
    pub latest_ledger: i64,
    pub oldest_ledger: i64,
    pub ledger_retention_window: i64,
}

/// Where to continue reading events from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Position {
    /// Start at a ledger (inclusive). Must be inside the RPC's retention window.
    Ledger(i64),
    /// Continue after an RPC event cursor.
    Cursor(String),
}

/// Anything that can serve pages of events. Implemented by [`RpcClient`];
/// tests use an in-memory source.
#[allow(async_fn_in_trait)]
pub trait EventSource {
    async fn get_events(
        &self,
        position: &Position,
        contract_id: &str,
        limit: u32,
    ) -> Result<EventsPage, Error>;
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

#[derive(Debug, Deserialize)]
struct Envelope<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

/// JSON-RPC client for a Soroban RPC endpoint.
#[derive(Debug, Clone)]
pub struct RpcClient {
    http: reqwest::Client,
    url: String,
}

impl RpcClient {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            url: url.into(),
        }
    }

    async fn call<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, Error> {
        let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
        let envelope: Envelope<T> = self
            .http
            .post(&self.url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        match (envelope.result, envelope.error) {
            (_, Some(e)) => Err(Error::Rpc {
                code: e.code,
                message: e.message,
            }),
            (Some(result), None) => Ok(result),
            (None, None) => Err(Error::EmptyRpcResponse),
        }
    }

    /// Fetches the RPC's health, including its event retention window.
    pub async fn get_health(&self) -> Result<Health, Error> {
        self.call("getHealth", json!({})).await
    }
}

impl EventSource for RpcClient {
    async fn get_events(
        &self,
        position: &Position,
        contract_id: &str,
        limit: u32,
    ) -> Result<EventsPage, Error> {
        let mut params = json!({
            "filters": [{ "type": "contract", "contractIds": [contract_id] }],
            "pagination": { "limit": limit },
            "xdrFormat": "json",
        });
        match position {
            Position::Ledger(ledger) => params["startLedger"] = json!(ledger),
            Position::Cursor(cursor) => params["pagination"]["cursor"] = json!(cursor),
        }
        self.call("getEvents", params).await
    }
}
