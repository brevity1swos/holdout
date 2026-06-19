use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcedurePolicy {
    #[serde(default)]
    pub forbidden: Vec<String>,
    #[serde(default)]
    pub required: Vec<String>,
}

/// Violations: each forbidden substring present in `trace`, and each required
/// substring absent from it.
pub fn check_procedure(trace: &str, policy: &ProcedurePolicy) -> Vec<String> {
    let mut violations = Vec::new();
    for f in &policy.forbidden {
        if trace.contains(f.as_str()) {
            violations.push(format!("forbidden step present: {f:?}"));
        }
    }
    for r in &policy.required {
        if !trace.contains(r.as_str()) {
            violations.push(format!("required step missing: {r:?}"));
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_forbidden_present_and_required_absent() {
        let policy = ProcedurePolicy {
            forbidden: vec!["auth=skipped".into()],
            required: vec!["validated".into()],
        };
        // Clean trace: required present, forbidden absent.
        assert!(check_procedure("validated input ok", &policy).is_empty());
        // Forbidden present.
        let v = check_procedure("auth=skipped; validated", &policy);
        assert_eq!(v.len(), 1);
        assert!(v[0].contains("forbidden"));
        // Required absent (forbidden also absent → only the required miss counts).
        let v = check_procedure("did nothing", &policy);
        assert_eq!(v.len(), 1);
        assert!(v[0].contains("required"));
    }
}
