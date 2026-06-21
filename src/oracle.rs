use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::HoldoutError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    pub name: String,
    pub input: String,
    /// The expected output, OR — when prefixed `blake3:` — its BLAKE3 hash.
    /// Hashing (via `record --hash-expected`) keeps the held-out answers out of
    /// the oracle file, so an agent that can read the file still cannot read or
    /// reproduce the answers without actually computing the right output.
    pub expected: String,
}

/// Prefix marking an `expected` value as a BLAKE3 hex hash rather than literal.
pub const HASH_PREFIX: &str = "blake3:";

pub fn hash_expected(output: &str) -> String {
    format!("{HASH_PREFIX}{}", blake3::hash(output.as_bytes()).to_hex())
}

impl Case {
    /// True if `actual` is the expected output — comparing against the stored
    /// hash when this case is hashed, else against the literal.
    pub fn matches(&self, actual: &str) -> bool {
        match self.expected.strip_prefix(HASH_PREFIX) {
            Some(hex) => blake3::hash(actual.as_bytes()).to_hex().as_str() == hex,
            None => actual == self.expected,
        }
    }

    /// What to show as "expected" in a divergence — never leak the hash.
    pub fn expected_display(&self) -> &str {
        if self.expected.starts_with(HASH_PREFIX) {
            "<hashed>"
        } else {
            &self.expected
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OracleKind {
    HeldoutCases,
    GoldenTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleSpec {
    pub kind: OracleKind,
    #[serde(default)]
    pub reference: Option<String>,
    pub visible: Vec<Case>,
    pub heldout: Vec<Case>,
}

impl OracleSpec {
    pub fn load(path: &Path) -> Result<OracleSpec, HoldoutError> {
        let bytes = std::fs::read(path)?;
        let spec = serde_json::from_slice(&bytes)?;
        Ok(spec)
    }

    /// Deterministic serialization used as the input to the seal hash.
    /// `serde_json::to_vec` is stable for a given struct shape (fields in
    /// declaration order, arrays in element order), which is all the seal needs.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("OracleSpec serializes")
    }
}

/// Returns the conventional `.seal` sidecar path for a given oracle file.
/// Appends `.seal` as a suffix (e.g. `oracle.json` → `oracle.json.seal`).
pub fn seal_path(oracle: &Path) -> std::path::PathBuf {
    let mut p = oracle.to_path_buf().into_os_string();
    p.push(".seal");
    std::path::PathBuf::from(p)
}

pub fn seal(spec: &OracleSpec) -> [u8; 32] {
    *blake3::hash(&spec.canonical_bytes()).as_bytes()
}

pub fn seal_hex(spec: &OracleSpec) -> String {
    blake3::hash(&spec.canonical_bytes()).to_hex().to_string()
}

pub fn verify_seal(spec: &OracleSpec, expected_hex: &str) -> Result<(), HoldoutError> {
    if seal_hex(spec) == expected_hex {
        Ok(())
    } else {
        Err(HoldoutError::SealMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_trace_kind_roundtrips_with_provenance() {
        let spec = OracleSpec {
            kind: OracleKind::GoldenTrace,
            reference: Some("./old_impl".into()),
            visible: vec![],
            heldout: vec![Case {
                name: "h1".into(),
                input: "3".into(),
                expected: "9".into(),
            }],
        };
        let json = serde_json::to_string(&spec).unwrap();
        let back: OracleSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(back.kind, OracleKind::GoldenTrace);
        assert_eq!(back.reference.as_deref(), Some("./old_impl"));
    }

    #[test]
    fn mvp_oracle_without_reference_field_still_loads() {
        // Backward-compat: MVP oracle JSON has no `reference` key.
        let json = r#"{"kind":"HeldoutCases","visible":[],"heldout":[]}"#;
        let spec: OracleSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.kind, OracleKind::HeldoutCases);
        assert!(spec.reference.is_none());
    }

    #[test]
    fn loads_oracle_from_json() {
        let json = r#"{
            "kind": "HeldoutCases",
            "visible": [{"name": "v1", "input": "2", "expected": "4"}],
            "heldout": [{"name": "h1", "input": "3", "expected": "9"}]
        }"#;
        let spec: OracleSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.kind, OracleKind::HeldoutCases);
        assert_eq!(spec.visible.len(), 1);
        assert_eq!(spec.heldout[0].expected, "9");
    }

    #[test]
    fn canonical_bytes_are_stable() {
        let spec = OracleSpec {
            kind: OracleKind::HeldoutCases,
            reference: None,
            visible: vec![],
            heldout: vec![Case {
                name: "h1".into(),
                input: "3".into(),
                expected: "9".into(),
            }],
        };
        assert_eq!(spec.canonical_bytes(), spec.canonical_bytes());
    }

    #[test]
    fn seal_detects_tampering() {
        let mut spec = OracleSpec {
            kind: OracleKind::HeldoutCases,
            reference: None,
            visible: vec![],
            heldout: vec![Case {
                name: "h1".into(),
                input: "3".into(),
                expected: "9".into(),
            }],
        };
        let sealed = seal_hex(&spec);
        // Unchanged oracle verifies.
        assert!(verify_seal(&spec, &sealed).is_ok());
        // Tampering with an expected output (the classic reward hack) is caught.
        spec.heldout[0].expected = "anything".into();
        assert!(matches!(
            verify_seal(&spec, &sealed),
            Err(HoldoutError::SealMismatch)
        ));
    }
}
