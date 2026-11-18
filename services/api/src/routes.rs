//! HTTP endpoints. The API never holds keys: issuance endpoints prepare the
//! data and the contract call; the issuer signs and submits it themselves.

use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::{
    chain::ChainReader,
    credential::{CredentialRecord, SchemaRef},
    merkle,
    payload::{VerifyPayload, VERSION},
    schema::{is_address, SchemaDocument},
    verify::{verify, VerifyResult},
    Error,
};

/// Most credentials accepted in one batch request.
pub const MAX_BATCH: usize = 1_000;

pub struct AppState<R> {
    pub reader: R,
    /// Registry contract id; credentials are issued under schemas in this registry.
    pub registry_id: String,
    /// Credentials contract id; issuance calls target it.
    pub credentials_id: String,
    /// Base URL of the verifier web page used in verification links.
    pub verify_base_url: String,
}

pub fn router<R: ChainReader>(state: Arc<AppState<R>>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/credentials", post(issue::<R>))
        .route("/v1/credentials/batch", post(issue_batch::<R>))
        .route("/v1/verify", post(verify_credential::<R>))
        .route("/v1/verify/link", post(make_link::<R>))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

/// A contract call for the issuer to sign and submit (for example with the SDK or a wallet).
#[derive(Debug, Serialize)]
pub struct ContractCall {
    pub contract: String,
    pub function: &'static str,
    pub args: Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IssueRequest {
    pub issuer: String,
    pub schema_id: u64,
    pub schema_document: SchemaDocument,
    pub subject: String,
    pub claims: Map<String, Value>,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct IssueResponse {
    pub credential: CredentialRecord,
    /// Hex SHA-256 of the canonical credential record (`data_hash`).
    pub commitment: String,
    pub call: ContractCall,
}

fn check_issuer(issuer: &str) -> Result<(), Error> {
    if is_address(issuer) {
        Ok(())
    } else {
        Err(Error::InvalidRequest(
            "issuer is not a valid Stellar address".to_string(),
        ))
    }
}

fn new_record<R>(
    state: &AppState<R>,
    document: &SchemaDocument,
    schema_id: u64,
    subject: String,
    claims: Map<String, Value>,
) -> Result<CredentialRecord, Error> {
    let record = CredentialRecord::new(
        SchemaRef {
            registry: state.registry_id.clone(),
            id: schema_id,
        },
        subject,
        claims,
    )?;
    record.validate(document)?;
    Ok(record)
}

async fn issue<R: ChainReader>(
    State(state): State<Arc<AppState<R>>>,
    Json(req): Json<IssueRequest>,
) -> Result<Json<IssueResponse>, Error> {
    check_issuer(&req.issuer)?;
    req.schema_document.validate()?;
    let credential = new_record(
        &state,
        &req.schema_document,
        req.schema_id,
        req.subject.clone(),
        req.claims,
    )?;
    let commitment = hex::encode(credential.commitment()?);
    Ok(Json(IssueResponse {
        call: ContractCall {
            contract: state.credentials_id.clone(),
            function: "issue",
            args: json!({
                "issuer": req.issuer,
                "schema_id": req.schema_id,
                "subject": req.subject,
                "data_hash": commitment,
                "expires_at": req.expires_at,
            }),
        },
        credential,
        commitment,
    }))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchItem {
    pub subject: String,
    pub claims: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchRequest {
    pub issuer: String,
    pub schema_id: u64,
    pub schema_document: SchemaDocument,
    pub credentials: Vec<BatchItem>,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct BatchEntry {
    pub credential: CredentialRecord,
    /// Merkle proof (hex sibling hashes, leaf to root) the holder keeps.
    pub proof: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchResponse {
    /// Hex Merkle root passed to `issue_batch`.
    pub root: String,
    pub credentials: Vec<BatchEntry>,
    pub call: ContractCall,
}

async fn issue_batch<R: ChainReader>(
    State(state): State<Arc<AppState<R>>>,
    Json(req): Json<BatchRequest>,
) -> Result<Json<BatchResponse>, Error> {
    check_issuer(&req.issuer)?;
    req.schema_document.validate()?;
    if req.credentials.is_empty() || req.credentials.len() > MAX_BATCH {
        return Err(Error::InvalidRequest(format!(
            "a batch must contain 1 to {MAX_BATCH} credentials"
        )));
    }
    let records = req
        .credentials
        .into_iter()
        .map(|item| {
            new_record(
                &state,
                &req.schema_document,
                req.schema_id,
                item.subject,
                item.claims,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let leaves = records
        .iter()
        .map(CredentialRecord::canonical_bytes)
        .collect::<Result<Vec<_>, _>>()?;
    let tree = merkle::build(&leaves)
        .ok_or_else(|| Error::InvalidRequest("a batch must not be empty".to_string()))?;
    let root = hex::encode(tree.root);

    let credentials = records
        .into_iter()
        .zip(tree.proofs)
        .map(|(credential, proof)| BatchEntry {
            credential,
            proof: proof.iter().map(hex::encode).collect(),
        })
        .collect();
    Ok(Json(BatchResponse {
        call: ContractCall {
            contract: state.credentials_id.clone(),
            function: "issue_batch",
            args: json!({
                "issuer": req.issuer,
                "schema_id": req.schema_id,
                "root": root,
                "expires_at": req.expires_at,
            }),
        },
        root,
        credentials,
    }))
}

/// Either a payload object or a link produced by `/v1/verify/link`.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum VerifyRequest {
    Payload { payload: VerifyPayload },
    Link { link: String },
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

async fn verify_credential<R: ChainReader>(
    State(state): State<Arc<AppState<R>>>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResult>, Error> {
    let payload = match req {
        VerifyRequest::Payload { payload } => payload,
        VerifyRequest::Link { link } => VerifyPayload::from_link(&link)?,
    };
    let result = verify(&state.reader, &state.registry_id, &payload, unix_now()).await?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkRequest {
    pub attestation_id: u64,
    pub schema_document: SchemaDocument,
    pub credential: CredentialRecord,
    pub proof: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct LinkResponse {
    /// The verification link; also the content to encode in a QR code.
    pub link: String,
}

async fn make_link<R: ChainReader>(
    State(state): State<Arc<AppState<R>>>,
    Json(req): Json<LinkRequest>,
) -> Result<Json<LinkResponse>, Error> {
    let payload = VerifyPayload {
        v: VERSION,
        attestation_id: req.attestation_id,
        schema_document: req.schema_document,
        credential: req.credential,
        proof: req.proof,
    };
    payload
        .credential
        .validate(&payload.schema_document)
        .and_then(|()| payload.schema_document.validate())?;
    Ok(Json(LinkResponse {
        link: payload.to_link(&state.verify_base_url)?,
    }))
}
