//! JSON canonicalization (RFC 8785, JCS) for the subset Asterism hashes.
//!
//! Supported: objects, arrays, strings, booleans, null and integers. Floats
//! are rejected because their canonical form is the hard part of JCS and no
//! Asterism document needs them. Object keys are sorted by UTF-16 code units,
//! as the RFC requires. Strings use `serde_json` escaping, which matches JCS
//! (only `"`, `\`, and control characters are escaped; control characters use
//! the short forms `\b \t \n \f \r` or lower-case `\u00xx`).

use serde_json::Value;

use crate::Error;

/// Serializes `value` canonically.
pub fn canonical_json(value: &Value) -> Result<String, Error> {
    let mut out = String::new();
    write(value, &mut out)?;
    Ok(out)
}

fn write(value: &Value, out: &mut String) -> Result<(), Error> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            if n.is_f64() {
                return Err(Error::InvalidCredential(
                    "floating point numbers are not allowed".to_string(),
                ));
            }
            out.push_str(&n.to_string());
        }
        Value::String(s) => out.push_str(&Value::String(s.clone()).to_string()),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write(item, out)?;
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
            out.push('{');
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&Value::String((*key).clone()).to_string());
                out.push(':');
                if let Some(v) = map.get(*key) {
                    write(v, out)?;
                }
            }
            out.push('}');
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sorts_keys_and_drops_whitespace() {
        let v = json!({ "b": 1, "a": { "d": [true, null], "c": "x" } });
        assert_eq!(
            canonical_json(&v).unwrap(),
            r#"{"a":{"c":"x","d":[true,null]},"b":1}"#
        );
    }

    #[test]
    fn insertion_order_does_not_matter() {
        let one: Value = serde_json::from_str(r#"{"x":1,"y":2}"#).unwrap();
        let two: Value = serde_json::from_str(r#"{"y":2,"x":1}"#).unwrap();
        assert_eq!(canonical_json(&one).unwrap(), canonical_json(&two).unwrap());
    }

    #[test]
    fn escapes_like_jcs() {
        let v = json!({ "s": "a\"b\\c\n\u{1}é" });
        assert_eq!(
            canonical_json(&v).unwrap(),
            "{\"s\":\"a\\\"b\\\\c\\n\\u0001é\"}"
        );
    }

    #[test]
    fn rejects_floats() {
        assert!(canonical_json(&json!({ "n": 1.5 })).is_err());
    }

    #[test]
    fn accepts_integers() {
        assert_eq!(
            canonical_json(&json!({ "n": -7, "m": 18446744073709551615u64 })).unwrap(),
            r#"{"m":18446744073709551615,"n":-7}"#
        );
    }
}
