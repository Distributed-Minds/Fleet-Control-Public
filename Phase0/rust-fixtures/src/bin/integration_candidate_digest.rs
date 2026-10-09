//! Deterministic SHA-256 identity for the historical integration-candidate envelope.
//!
//! This calculates *prediction identities*, not Git commit object IDs or permission
//! to mutate a ref. The existing integration_candidate semantic oracle separately
//! checks supported operations, parent cardinality, and live-head staleness.
//! No external digest executable, network access, or additional Cargo crate is used.

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::process;

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256_hex(input: &[u8]) -> String {
    let mut state = [
        0x6a09e667_u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut padded = input.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());
    for block in padded.chunks_exact(64) {
        let mut words = [0_u32; 64];
        for (i, word) in words[..16].iter_mut().enumerate() {
            let p = i * 4;
            *word = u32::from_be_bytes([block[p], block[p + 1], block[p + 2], block[p + 3]]);
        }
        for i in 16..64 {
            let s0 = words[i - 15].rotate_right(7)
                ^ words[i - 15].rotate_right(18)
                ^ (words[i - 15] >> 3);
            let s1 = words[i - 2].rotate_right(17)
                ^ words[i - 2].rotate_right(19)
                ^ (words[i - 2] >> 10);
            words[i] = words[i - 16]
                .wrapping_add(s0)
                .wrapping_add(words[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for (i, word) in words.iter().enumerate() {
            let big1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(big1)
                .wrapping_add(choice)
                .wrapping_add(K[i])
                .wrapping_add(*word);
            let big0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let t2 = big0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (s, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *s = s.wrapping_add(value);
        }
    }
    let mut result = String::with_capacity(64);
    for part in state {
        write!(&mut result, "{part:08x}").expect("writing into a String cannot fail");
    }
    result
}

// Strict JSON parsing is necessary for identity-bearing envelopes: decoding
// directly into serde_json::Value silently keeps the last duplicate object key.
// Two different byte representations must not become an unreviewed identity
// through parser-dependent duplicate-key selection. Keys are compared *after*
// JSON escape decoding, at every nesting level.
struct StrictJson(Value);

impl<'de> Deserialize<'de> for StrictJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrictVisitor;

        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictJson;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("JSON without duplicate object keys")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson(Value::Null))
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson(Value::Bool(value)))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson(Value::from(value)))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson(Value::from(value)))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                serde_json::Number::from_f64(value)
                    .map(|number| StrictJson(Value::Number(number)))
                    .ok_or_else(|| de::Error::custom("non-finite JSON number"))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson(Value::String(value.to_owned())))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson(Value::String(value)))
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(StrictJson(value)) = sequence.next_element::<StrictJson>()? {
                    values.push(value);
                }
                Ok(StrictJson(Value::Array(values)))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = serde_json::Map::new();
                while let Some((key, StrictJson(value))) = map.next_entry::<String, StrictJson>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom(format!(
                            "duplicate JSON object key: {key}"
                        )));
                    }
                    values.insert(key, value);
                }
                Ok(StrictJson(Value::Object(values)))
            }
        }

        deserializer.deserialize_any(StrictVisitor)
    }
}

/// Reject malformed or undeclared envelope fields *before* hashing. Hashing a
/// serde_json::Value without typed shape admission would assign plausible IDs
/// to unknown/missing fields that the structural oracle would never accept.
fn admit_candidate_envelope(
    name: &str,
    candidate: &serde_json::Map<String, Value>,
) -> Result<(), String> {
    const FIELDS: &[&str] = &[
        "schema_version",
        "operation_kind",
        "target_commit",
        "source_commit",
        "parents",
        "parent_count",
        "tree",
        "metadata",
        "constructor_version",
        "compatibility_basis",
    ];
    const TEXT: &[&str] = &[
        "schema_version",
        "operation_kind",
        "target_commit",
        "source_commit",
        "tree",
        "constructor_version",
        "compatibility_basis",
    ];
    const META: &[&str] = &[
        "author",
        "author_time",
        "committer",
        "committer_time",
        "message",
        "encoding",
        "signature_policy",
    ];
    if candidate.len() != FIELDS.len()
        || candidate.keys().any(|key| !FIELDS.contains(&key.as_str()))
    {
        return Err(format!("{name}: missing or undeclared candidate field"));
    }
    for field in TEXT {
        if candidate.get(*field).and_then(Value::as_str).is_none() {
            return Err(format!("{name}: candidate field {field} is not a string"));
        }
    }
    if candidate["schema_version"] != "integration-candidate-v1"
        || candidate["operation_kind"] != "explicit-merge"
    {
        return Err(format!("{name}: unsupported candidate schema/operation"));
    }
    let parents = candidate["parents"]
        .as_array()
        .ok_or_else(|| format!("{name}: parents must be an array"))?;
    if parents.is_empty() || parents.iter().any(|parent| parent.as_str().is_none()) {
        return Err(format!(
            "{name}: parent identities must be nonempty strings"
        ));
    }
    if candidate["parent_count"].as_u64() != Some(parents.len() as u64) {
        return Err(format!("{name}: parent_count does not match parents"));
    }
    let metadata = candidate["metadata"]
        .as_object()
        .ok_or_else(|| format!("{name}: metadata must be an object"))?;
    if metadata.len() != META.len()
        || metadata.keys().any(|key| !META.contains(&key.as_str()))
        || META
            .iter()
            .any(|key| metadata.get(*key).and_then(Value::as_str).is_none())
    {
        return Err(format!(
            "{name}: missing, invalid, or undeclared metadata field"
        ));
    }
    Ok(())
}

fn candidate_ids(fixture_text: &str) -> Result<Vec<(String, String)>, String> {
    let StrictJson(root) = serde_json::from_str(fixture_text).map_err(|e| e.to_string())?;
    if root.get("schema_version").and_then(Value::as_str)
        != Some("integration-candidate-fixture-v1")
        || root.get("digest").and_then(Value::as_str) != Some("sha256")
    {
        return Err("unsupported fixture identity/digest".to_owned());
    }
    let cases = root
        .get("cases")
        .and_then(Value::as_array)
        .ok_or("missing candidate array")?;
    const REQUIRED: [&str; 4] = [
        "normal-two-parent",
        "reversed-parents-same-tree",
        "unsupported-three-parent",
        "compatible-constructor-migration",
    ];
    if cases.len() != REQUIRED.len() {
        return Err("missing or extra candidate identities".to_owned());
    }
    let mut seen = HashSet::new();
    let mut ids = Vec::with_capacity(cases.len());
    for case in cases {
        let name = case
            .get("name")
            .and_then(Value::as_str)
            .ok_or("candidate case missing string name")?;
        if !REQUIRED.contains(&name) || !seen.insert(name) {
            return Err(format!("unknown or duplicate case: {name}"));
        }
        let candidate = case
            .get("candidate")
            .and_then(Value::as_object)
            .ok_or("candidate envelope must be an object")?;
        if candidate.is_empty() {
            return Err(format!("{name}: empty candidate envelope"));
        }
        admit_candidate_envelope(name, candidate)?;
        // serde_json's default Map is a BTreeMap: object keys serialize in
        // lexical order, arrays retain their order, non-ASCII stays UTF-8.
        // This matches historical Python json.dumps(sort_keys=True,
        // separators=(',', ':'), ensure_ascii=False) for the typed fixture.
        let bytes =
            serde_json::to_vec(&Value::Object(candidate.clone())).map_err(|e| e.to_string())?;
        ids.push((name.to_owned(), sha256_hex(&bytes)));
    }
    Ok(ids)
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() > 1 {
        eprintln!("usage: integration_candidate_digest [fixture-path]");
        process::exit(2);
    }
    let path = args
        .first()
        .map(String::as_str)
        .unwrap_or("Phase0/fixtures/integration-candidate-v1.json");
    let result = fs::read_to_string(path)
        .map_err(|e| e.to_string())
        .and_then(|text| candidate_ids(&text));
    match result {
        Ok(ids) => {
            for (name, digest) in ids {
                println!("{name}: {digest}");
            }
            eprintln!("Digest identity only; does not authorize Git mutations");
        }
        Err(reason) => {
            eprintln!("FAIL: {reason}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HISTORICAL: &str = include_str!("../../../fixtures/integration-candidate-v1.json");

    #[test]
    fn sha256_standard_vectors_and_long_message() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(&vec![b'a'; 1_000_000]),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn historical_python_sha256_golden_identities() {
        let actual = candidate_ids(HISTORICAL).unwrap();
        assert_eq!(
            actual,
            [
                (
                    "normal-two-parent".to_owned(),
                    "7e8986591c4293e4eeebf751a2c12e19512d4792e5cd8267e6308fe0a479d8da".to_owned()
                ),
                (
                    "reversed-parents-same-tree".to_owned(),
                    "b9bea4f05c5cf67a3ed24dafae1600bb5f58e30563b96981afac9a8322efdaf8".to_owned()
                ),
                (
                    "unsupported-three-parent".to_owned(),
                    "75de8f8eefce4daf6e64ef71267cac4039fd4a1610e77700259ba1298c40c51a".to_owned()
                ),
                (
                    "compatible-constructor-migration".to_owned(),
                    "b0ce5db57c6ccc2ccff334ef9905f2488ca826e8d7cefe6f752cd7dd4f59483f".to_owned()
                ),
            ]
        );
        assert_ne!(actual[0].1, actual[1].1);
        assert_ne!(actual[0].1, actual[2].1);
        assert_ne!(actual[0].1, actual[3].1);
    }

    #[test]
    fn changed_semantic_bytes_change_digest_without_changing_case_name() {
        let original = candidate_ids(HISTORICAL).unwrap();
        let mut fixture: Value = serde_json::from_str(HISTORICAL).unwrap();
        let cases = fixture["cases"].as_array_mut().unwrap();
        cases[0]["candidate"]["metadata"]["message"] = Value::String("Changed\n".to_owned());
        let mutated = candidate_ids(&fixture.to_string()).unwrap();
        assert_ne!(original[0].1, mutated[0].1);
        assert_eq!(original[1..], mutated[1..]);
    }

    #[test]
    fn unicode_is_utf8_not_ascii_escaped_and_order_is_semantic() {
        let mut fixture: Value = serde_json::from_str(HISTORICAL).unwrap();
        fixture["cases"][0]["candidate"]["metadata"]["author"] =
            Value::String("Jörg ∑ 東京".to_owned());
        let unicode = candidate_ids(&fixture.to_string()).unwrap();
        fixture["cases"][0]["candidate"]["metadata"]["author"] =
            Value::String("J\\u00f6rg".to_owned());
        assert_ne!(
            unicode[0].1,
            candidate_ids(&fixture.to_string()).unwrap()[0].1
        );
        let baseline: Value = serde_json::from_str(HISTORICAL).unwrap();
        let mut reordered = baseline.clone();
        let a = reordered["cases"][0]["candidate"]["parents"][0].clone();
        let b = reordered["cases"][0]["candidate"]["parents"][1].clone();
        reordered["cases"][0]["candidate"]["parents"][0] = b;
        reordered["cases"][0]["candidate"]["parents"][1] = a;
        assert_ne!(
            candidate_ids(&baseline.to_string()).unwrap()[0].1,
            candidate_ids(&reordered.to_string()).unwrap()[0].1
        );
    }

    #[test]
    fn identity_rejects_duplicate_keys_at_all_nesting_levels() {
        // serde_json::Value by itself accepts these and silently overwrites
        // earlier values. In an identity envelope that is not safe.
        let cases = [
            HISTORICAL.replacen("\"digest\":", "\"digest\": \"sha1\", \"digest\":", 1),
            HISTORICAL.replacen("\"name\":", "\"name\": \"forged\", \"name\":", 1),
            HISTORICAL.replacen(
                "\"operation_kind\":",
                "\"operation_kind\": \"fast-forward\", \"operation_kind\":",
                1,
            ),
            HISTORICAL.replacen("\"author\":", "\"author\": \"forged\", \"author\":", 1),
        ];
        for input in cases {
            assert_ne!(input, HISTORICAL, "test must inject a duplicate key");
            assert!(
                serde_json::from_str::<Value>(&input).is_ok(),
                "regression control: generic Value parsing accepts duplicate keys"
            );
            assert!(
                candidate_ids(&input).is_err(),
                "identity calculator accepted ambiguous JSON"
            );
        }
        assert!(candidate_ids(HISTORICAL).is_ok());
    }

    #[test]
    fn escaped_key_aliases_are_rejected_after_json_unescaping() {
        let input = HISTORICAL.replacen("\"digest\":", r#""di\u0067est": "sha1", "digest":"#, 1);
        assert_ne!(input, HISTORICAL);
        assert!(serde_json::from_str::<Value>(&input).is_ok());
        assert!(candidate_ids(&input).is_err());
    }

    #[test]
    fn digest_rejects_undeclared_missing_and_wrong_typed_envelope_fields() {
        let original: Value = serde_json::from_str(HISTORICAL).unwrap();
        let mutate = |edit: fn(&mut Value)| {
            let mut fixture = original.clone();
            edit(&mut fixture["cases"][0]["candidate"]);
            candidate_ids(&fixture.to_string())
        };
        let invalid: [fn(&mut Value); 9] = [
            |c| c["unexpected"] = Value::String("extra".into()),
            |c| {
                c.as_object_mut().unwrap().remove("tree");
            },
            |c| c["source_commit"] = Value::from(42),
            |c| c["schema_version"] = Value::String("not-supported".into()),
            |c| c["operation_kind"] = Value::String("fast-forward".into()),
            |c| c["parents"] = Value::String("not-an-array".into()),
            |c| c["parent_count"] = Value::from(1),
            |c| c["metadata"]["unexpected"] = Value::String("extra".into()),
            |c| {
                c["metadata"].as_object_mut().unwrap().remove("author");
            },
        ];
        for (index, edit) in invalid.into_iter().enumerate() {
            assert!(
                mutate(edit).is_err(),
                "invalid envelope {index} received a digest"
            );
        }
        assert!(candidate_ids(HISTORICAL).is_ok());
    }

    #[test]
    fn rejects_malformed_fixture_and_identity_forgery() {
        assert!(candidate_ids("{").is_err());
        let mut fixture: Value = serde_json::from_str(HISTORICAL).unwrap();
        fixture["digest"] = Value::String("sha1".to_owned());
        assert!(candidate_ids(&fixture.to_string()).is_err());
        fixture["digest"] = Value::String("sha256".to_owned());
        fixture["cases"][1]["name"] = fixture["cases"][0]["name"].clone();
        assert!(candidate_ids(&fixture.to_string()).is_err());
        fixture["cases"][1]["name"] = Value::String("reversed-parents-same-tree".to_owned());
        fixture["cases"][0]["candidate"] = Value::Null;
        assert!(candidate_ids(&fixture.to_string()).is_err());
    }
}
