//! Credential schema documents (RFC 0003, decisions 2 and 3).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::{canonical::canonical_json, Error};

/// Document format version this API understands.
pub const FORMAT: &str = "credential-schema/1";

/// Registry limit on schema names, in bytes (`MAX_NAME_LEN`).
const MAX_NAME_LEN: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    String,
    Integer,
    Boolean,
    Date,
    Datetime,
    Address,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FieldSpec {
    #[serde(rename = "type")]
    pub kind: FieldType,
    pub required: bool,
}

/// The off-chain schema document whose hash is the registry's `fields_hash`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaDocument {
    pub asterism: String,
    pub name: String,
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub fields: BTreeMap<String, FieldSpec>,
}

impl SchemaDocument {
    /// Checks the document is well formed.
    pub fn validate(&self) -> Result<(), Error> {
        let bad = |m: &str| Err(Error::InvalidSchema(m.to_string()));
        if self.asterism != FORMAT {
            return bad("unsupported format version");
        }
        if self.name.is_empty() || self.name.len() > MAX_NAME_LEN {
            return bad("name must be 1 to 64 bytes");
        }
        if self.fields.is_empty() {
            return bad("at least one field is required");
        }
        for name in self.fields.keys() {
            let ok = !name.is_empty()
                && name.len() <= 64
                && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');
            if !ok {
                return bad("field names must be 1 to 64 ASCII letters, digits or underscores");
            }
        }
        Ok(())
    }

    /// `SHA-256(JCS(document))`, the value registered as `fields_hash`.
    pub fn fields_hash(&self) -> Result<[u8; 32], Error> {
        let value = serde_json::to_value(self).map_err(|e| Error::InvalidSchema(e.to_string()))?;
        Ok(Sha256::digest(canonical_json(&value)?.as_bytes()).into())
    }

    /// Checks `claims` against the schema: required fields present, no
    /// unknown fields, every value of the declared type.
    pub fn validate_claims(&self, claims: &Map<String, Value>) -> Result<(), Error> {
        let bad = |m: String| Err(Error::InvalidCredential(m));
        for key in claims.keys() {
            if !self.fields.contains_key(key) {
                return bad(format!("unknown field `{key}`"));
            }
        }
        for (name, spec) in &self.fields {
            match claims.get(name) {
                None if spec.required => return bad(format!("missing required field `{name}`")),
                None => {}
                Some(value) if !matches_type(spec.kind, value) => {
                    return bad(format!("field `{name}` is not a valid {:?}", spec.kind));
                }
                Some(_) => {}
            }
        }
        Ok(())
    }
}

fn matches_type(kind: FieldType, value: &Value) -> bool {
    match (kind, value) {
        (FieldType::String, Value::String(_)) => true,
        (FieldType::Integer, Value::Number(n)) => n.is_i64() || n.is_u64(),
        (FieldType::Boolean, Value::Bool(_)) => true,
        (FieldType::Date, Value::String(s)) => is_date(s),
        (FieldType::Datetime, Value::String(s)) => is_datetime(s),
        (FieldType::Address, Value::String(s)) => is_address(s),
        _ => false,
    }
}

/// A Stellar account (`G...`) or contract (`C...`) strkey with a valid checksum.
pub fn is_address(s: &str) -> bool {
    matches!(
        stellar_strkey::Strkey::from_string(s),
        Ok(stellar_strkey::Strkey::PublicKeyEd25519(_) | stellar_strkey::Strkey::Contract(_))
    )
}

fn digits(s: &str, len: usize) -> Option<u32> {
    if s.len() == len && s.bytes().all(|b| b.is_ascii_digit()) {
        s.parse().ok()
    } else {
        None
    }
}

/// `YYYY-MM-DD` with a real calendar date.
fn is_date(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    let [y, m, d] = parts[..] else {
        return false;
    };
    let (Some(year), Some(month), Some(day)) = (digits(y, 4), digits(m, 2), digits(d, 2)) else {
        return false;
    };
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let max = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=max).contains(&day)
}

/// RFC 3339 `YYYY-MM-DDTHH:MM:SS[.fraction](Z|+HH:MM|-HH:MM)`.
fn is_datetime(s: &str) -> bool {
    let Some((date, time)) = s.split_once('T') else {
        return false;
    };
    if !is_date(date) {
        return false;
    }
    let (clock, offset) = if let Some(clock) = time.strip_suffix('Z') {
        (clock, None)
    } else if let Some(i) = time.rfind(['+', '-']) {
        (&time[..i], Some(&time[i + 1..]))
    } else {
        return false;
    };
    if let Some(offset) = offset {
        let Some((h, m)) = offset.split_once(':') else {
            return false;
        };
        if !matches!((digits(h, 2), digits(m, 2)), (Some(0..=23), Some(0..=59))) {
            return false;
        }
    }
    let (whole, fraction) = match clock.split_once('.') {
        Some((w, f)) => (w, Some(f)),
        None => (clock, None),
    };
    if let Some(f) = fraction {
        if f.is_empty() || !f.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
    }
    let parts: Vec<&str> = whole.split(':').collect();
    let [h, m, sec] = parts[..] else {
        return false;
    };
    matches!(
        (digits(h, 2), digits(m, 2), digits(sec, 2)),
        (Some(0..=23), Some(0..=59), Some(0..=60))
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::json;

    pub fn degree_schema() -> SchemaDocument {
        serde_json::from_value(json!({
            "asterism": "credential-schema/1",
            "name": "University degree",
            "version": 1,
            "fields": {
                "degree": { "type": "string", "required": true },
                "graduated": { "type": "date", "required": true },
                "honours": { "type": "string", "required": false }
            }
        }))
        .unwrap()
    }

    fn claims(v: Value) -> Map<String, Value> {
        v.as_object().cloned().unwrap_or_default()
    }

    #[test]
    fn valid_schema_passes() {
        assert!(degree_schema().validate().is_ok());
    }

    #[test]
    fn rejects_malformed_schemas() {
        let mut s = degree_schema();
        s.asterism = "credential-schema/2".into();
        assert!(s.validate().is_err());

        let mut s = degree_schema();
        s.name = "x".repeat(65);
        assert!(s.validate().is_err());

        let mut s = degree_schema();
        s.fields.clear();
        assert!(s.validate().is_err());

        let mut s = degree_schema();
        s.fields.insert(
            "bad name".into(),
            FieldSpec {
                kind: FieldType::String,
                required: false,
            },
        );
        assert!(s.validate().is_err());
    }

    #[test]
    fn unknown_keys_in_the_document_are_rejected() {
        let result = serde_json::from_value::<SchemaDocument>(json!({
            "asterism": "credential-schema/1", "name": "n", "version": 1,
            "fields": { "a": { "type": "string", "required": true } },
            "extra": true
        }));
        assert!(result.is_err());
    }

    #[test]
    fn fields_hash_is_stable_and_order_independent() {
        let a = degree_schema();
        let b: SchemaDocument = serde_json::from_str(
            r#"{"fields":{"honours":{"required":false,"type":"string"},"graduated":{"required":true,"type":"date"},"degree":{"required":true,"type":"string"}},"version":1,"name":"University degree","asterism":"credential-schema/1"}"#,
        )
        .unwrap();
        assert_eq!(a.fields_hash().unwrap(), b.fields_hash().unwrap());

        let mut c = degree_schema();
        c.version = 2;
        assert_ne!(a.fields_hash().unwrap(), c.fields_hash().unwrap());
    }

    #[test]
    fn claims_are_checked_against_the_schema() {
        let s = degree_schema();
        assert!(s
            .validate_claims(&claims(
                json!({ "degree": "BSc", "graduated": "2026-07-15" })
            ))
            .is_ok());
        assert!(s
            .validate_claims(&claims(
                json!({ "degree": "BSc", "graduated": "2026-07-15", "honours": "First" })
            ))
            .is_ok());
        // Missing required field.
        assert!(s
            .validate_claims(&claims(json!({ "degree": "BSc" })))
            .is_err());
        // Unknown field.
        assert!(s
            .validate_claims(&claims(
                json!({ "degree": "BSc", "graduated": "2026-07-15", "gpa": 4 })
            ))
            .is_err());
        // Wrong type.
        assert!(s
            .validate_claims(&claims(json!({ "degree": 5, "graduated": "2026-07-15" })))
            .is_err());
        // Invalid date.
        assert!(s
            .validate_claims(&claims(
                json!({ "degree": "BSc", "graduated": "2026-02-30" })
            ))
            .is_err());
    }

    #[test]
    fn date_and_datetime_formats() {
        assert!(is_date("2024-02-29"));
        assert!(!is_date("2023-02-29"));
        assert!(!is_date("2023-13-01"));
        assert!(!is_date("23-01-01"));
        assert!(is_datetime("2026-11-18T17:21:49Z"));
        assert!(is_datetime("2026-11-18T17:21:49.123+01:00"));
        assert!(!is_datetime("2026-11-18 17:21:49Z"));
        assert!(!is_datetime("2026-11-18T25:00:00Z"));
        assert!(!is_datetime("2026-11-18T17:21:49"));
    }

    #[test]
    fn addresses_need_a_valid_checksum() {
        let g = stellar_strkey::ed25519::PublicKey([7; 32]).to_string();
        let c = stellar_strkey::Contract([9; 32]).to_string();
        assert!(is_address(&g));
        assert!(is_address(&c));
        assert!(!is_address("GABC"));
        let mut broken = g.clone();
        broken.replace_range(10..11, if &g[10..11] == "A" { "B" } else { "A" });
        assert!(!is_address(&broken));
    }
}
