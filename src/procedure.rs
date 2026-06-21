use serde::{Deserialize, Serialize};

/// A procedure policy checked against a candidate run's trace (its stderr).
///
/// This is trace-level *string* checking, not instrumented syscall/tool tracing:
/// it catches a corrupt success only insofar as the candidate's trace records the
/// step. `forbidden`/`required` are presence/absence; `require_order` additionally
/// checks that steps occur in a given relative order.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcedurePolicy {
    #[serde(default)]
    pub forbidden: Vec<String>,
    #[serde(default)]
    pub required: Vec<String>,
    /// Substrings that must appear in the trace in this relative order.
    #[serde(default)]
    pub require_order: Vec<String>,
}

/// Violations: each forbidden substring present, each required substring absent,
/// and each `require_order` step that is missing or out of order.
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
    // Ordered requirements: walk forward, each step must appear after the previous.
    let mut search_from = 0usize;
    for step in &policy.require_order {
        match trace[search_from..].find(step.as_str()) {
            Some(pos) => search_from += pos + step.len(),
            None if trace.contains(step.as_str()) => {
                violations.push(format!("step out of required order: {step:?}"));
            }
            None => violations.push(format!("required-order step missing: {step:?}")),
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
            ..Default::default()
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

    #[test]
    fn flags_out_of_order_and_missing_ordered_steps() {
        let policy = ProcedurePolicy {
            require_order: vec!["auth_check".into(), "db_write".into()],
            ..Default::default()
        };
        // Correct order → clean.
        assert!(check_procedure("auth_check ok; then db_write done", &policy).is_empty());
        // Reversed → out-of-order violation.
        let v = check_procedure("db_write done; auth_check skipped-after", &policy);
        assert_eq!(v.len(), 1);
        assert!(v[0].contains("out of required order"), "{v:?}");
        // Second step absent → missing violation.
        let v = check_procedure("auth_check only", &policy);
        assert_eq!(v.len(), 1);
        assert!(v[0].contains("missing"), "{v:?}");
    }
}
