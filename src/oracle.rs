use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::HoldoutError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    pub name: String,
    pub input: String,
    pub expected: String,
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
