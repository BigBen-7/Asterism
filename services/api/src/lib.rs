//! Asterism API: prepares credential issuance and verifies credentials.
//!
//! - Issuance (`POST /v1/credentials`, `POST /v1/credentials/batch`) validates
//!   claims against a schema document, builds the credential record and its
//!   commitment (or Merkle tree) and returns the contract call for the issuer
//!   to sign. The API holds no keys.
//! - Verification (`POST /v1/verify`) checks a presented credential against
//!   registry state read from the indexer's Postgres tables.
//! - `POST /v1/verify/link` builds the link / QR payload (see [`payload`]).
//!
//! The formats are specified in RFC 0003 and documented in `docs/api.md`.

pub mod canonical;
pub mod chain;
pub mod credential;
mod error;
pub mod merkle;
pub mod payload;
pub mod routes;
pub mod schema;
pub mod verify;

pub use error::Error;
