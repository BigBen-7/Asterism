//! End-to-end tests of the HTTP API with an in-memory chain.

use std::{collections::HashMap, sync::Arc};

use asterism_api::{
    chain::{ChainReader, Commitment, OnChainAttestation},
    merkle::{self, Hash},
    routes::{router, AppState},
    schema::SchemaDocument,
    Error,
};
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use tower::ServiceExt;

#[derive(Default)]
struct FakeChain {
    attestations: HashMap<u64, OnChainAttestation>,
    schemas: HashMap<u64, Hash>,
}

impl ChainReader for FakeChain {
    async fn attestation(&self, id: u64) -> Result<Option<OnChainAttestation>, Error> {
        Ok(self.attestations.get(&id).cloned())
    }
    async fn schema_fields_hash(&self, schema_id: u64) -> Result<Option<Hash>, Error> {
        Ok(self.schemas.get(&schema_id).copied())
    }
}

fn registry_id() -> String {
    stellar_strkey::Contract([9; 32]).to_string()
}
fn credentials_id() -> String {
    stellar_strkey::Contract([10; 32]).to_string()
}
fn issuer() -> String {
    stellar_strkey::ed25519::PublicKey([6; 32]).to_string()
}
fn holder(n: u8) -> String {
    stellar_strkey::ed25519::PublicKey([n; 32]).to_string()
}

fn schema() -> Value {
    json!({
        "asterism": "credential-schema/1",
        "name": "University degree",
        "version": 1,
        "fields": {
            "degree": { "type": "string", "required": true },
            "graduated": { "type": "date", "required": true }
        }
    })
}

fn app(chain: FakeChain) -> Router {
    router(Arc::new(AppState {
        reader: chain,
        registry_id: registry_id(),
        credentials_id: credentials_id(),
        verify_base_url: "https://verify.example".to_string(),
    }))
}

async fn post(app: &Router, path: &str, body: Value) -> (StatusCode, Value) {
    let request = Request::post(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn hash_of(hex_str: &str) -> Hash {
    hex::decode(hex_str).unwrap().try_into().unwrap()
}

fn fields_hash() -> Hash {
    serde_json::from_value::<SchemaDocument>(schema())
        .unwrap()
        .fields_hash()
        .unwrap()
}

fn issue_body(subject: &str) -> Value {
    json!({
        "issuer": issuer(),
        "schema_id": 3,
        "schema_document": schema(),
        "subject": subject,
        "claims": { "degree": "BSc", "graduated": "2026-07-15" },
        "expires_at": 5000
    })
}

#[tokio::test]
async fn health_is_ok() {
    let response = app(FakeChain::default())
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn single_issuance_returns_the_record_and_the_call() {
    let (status, body) = post(
        &app(FakeChain::default()),
        "/v1/credentials",
        issue_body(&holder(1)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(
        body["credential"]["schema"]["registry"],
        json!(registry_id())
    );
    assert_eq!(body["credential"]["schema"]["id"], json!(3));
    assert_eq!(body["credential"]["salt"].as_str().map(str::len), Some(64));
    assert_eq!(body["call"]["contract"], json!(credentials_id()));
    assert_eq!(body["call"]["function"], json!("issue"));
    assert_eq!(body["call"]["args"]["data_hash"], body["commitment"]);
    assert_eq!(body["call"]["args"]["expires_at"], json!(5000));
    assert_eq!(body["call"]["args"]["issuer"], json!(issuer()));
}

#[tokio::test]
async fn issuance_rejects_bad_input() {
    let app = app(FakeChain::default());

    let mut bad_claims = issue_body(&holder(1));
    bad_claims["claims"] = json!({ "degree": "BSc" });
    assert_eq!(
        post(&app, "/v1/credentials", bad_claims).await.0,
        StatusCode::BAD_REQUEST
    );

    let mut bad_subject = issue_body("nope");
    bad_subject["claims"] = json!({ "degree": "BSc", "graduated": "2026-07-15" });
    assert_eq!(
        post(&app, "/v1/credentials", bad_subject).await.0,
        StatusCode::BAD_REQUEST
    );

    let mut bad_issuer = issue_body(&holder(1));
    bad_issuer["issuer"] = json!("nope");
    assert_eq!(
        post(&app, "/v1/credentials", bad_issuer).await.0,
        StatusCode::BAD_REQUEST
    );

    let mut bad_schema = issue_body(&holder(1));
    bad_schema["schema_document"]["asterism"] = json!("credential-schema/9");
    assert_eq!(
        post(&app, "/v1/credentials", bad_schema).await.0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn batch_issuance_returns_a_root_and_a_proof_per_credential() {
    let body = json!({
        "issuer": issuer(),
        "schema_id": 3,
        "schema_document": schema(),
        "credentials": [
            { "subject": holder(1), "claims": { "degree": "BSc", "graduated": "2026-07-15" } },
            { "subject": holder(2), "claims": { "degree": "MSc", "graduated": "2026-07-16" } },
            { "subject": holder(3), "claims": { "degree": "PhD", "graduated": "2026-07-17" } }
        ]
    });
    let (status, response) = post(&app(FakeChain::default()), "/v1/credentials/batch", body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["call"]["function"], json!("issue_batch"));
    assert_eq!(response["call"]["args"]["root"], response["root"]);

    let root = hash_of(response["root"].as_str().unwrap());
    let entries = response["credentials"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
    for entry in entries {
        let record: asterism_api::credential::CredentialRecord =
            serde_json::from_value(entry["credential"].clone()).unwrap();
        let proof: Vec<Hash> = entry["proof"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| hash_of(h.as_str().unwrap()))
            .collect();
        assert_eq!(
            merkle::compute_root(&record.canonical_bytes().unwrap(), &proof),
            root
        );
    }
}

#[tokio::test]
async fn batch_size_is_limited() {
    let app = app(FakeChain::default());
    let empty = json!({ "issuer": issuer(), "schema_id": 3, "schema_document": schema(), "credentials": [] });
    assert_eq!(
        post(&app, "/v1/credentials/batch", empty).await.0,
        StatusCode::BAD_REQUEST
    );

    let one =
        json!({ "subject": holder(1), "claims": { "degree": "BSc", "graduated": "2026-07-15" } });
    let too_many = json!({
        "issuer": issuer(), "schema_id": 3, "schema_document": schema(),
        "credentials": vec![one; 1001]
    });
    assert_eq!(
        post(&app, "/v1/credentials/batch", too_many).await.0,
        StatusCode::BAD_REQUEST
    );
}

/// Issues a credential through the API, "puts it on chain", then verifies it
/// both as a payload and through a link.
#[tokio::test]
async fn issued_credentials_verify_by_payload_and_by_link() {
    let (_, issued) = post(
        &app(FakeChain::default()),
        "/v1/credentials",
        issue_body(&holder(1)),
    )
    .await;

    let mut chain = FakeChain::default();
    chain.schemas.insert(3, fields_hash());
    chain.attestations.insert(
        7,
        OnChainAttestation {
            id: 7,
            schema_id: 3,
            issuer: issuer(),
            subject: Some(holder(1)),
            commitment: Commitment::Single(hash_of(issued["commitment"].as_str().unwrap())),
            expires_at: Some(u64::MAX),
            revoked: false,
        },
    );
    let app = app(chain);

    let (status, link) = post(
        &app,
        "/v1/verify/link",
        json!({ "attestation_id": 7, "schema_document": schema(), "credential": issued["credential"] }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let link = link["link"].as_str().unwrap().to_string();
    assert!(link.starts_with("https://verify.example/verify#"));

    let (status, by_link) = post(&app, "/v1/verify", json!({ "link": link })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(by_link["valid"], json!(true), "{by_link}");
    assert_eq!(by_link["status"], json!("valid"));

    let payload = json!({
        "v": 1, "attestation_id": 7, "schema_document": schema(), "credential": issued["credential"]
    });
    let (_, by_payload) = post(&app, "/v1/verify", json!({ "payload": payload })).await;
    assert_eq!(by_payload["valid"], json!(true));

    // A tampered credential is reported, not accepted.
    let mut tampered = payload;
    tampered["credential"]["claims"]["degree"] = json!("PhD");
    let (status, result) = post(&app, "/v1/verify", json!({ "payload": tampered })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(result["valid"], json!(false));
}

#[tokio::test]
async fn malformed_verification_requests_are_rejected() {
    let app = app(FakeChain::default());
    let (status, _) = post(
        &app,
        "/v1/verify",
        json!({ "link": "https://verify.example/verify" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = post(&app, "/v1/verify", json!({ "nothing": true })).await;
    assert!(status.is_client_error());
}
