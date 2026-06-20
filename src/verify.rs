use serde::Serialize;

use crate::candidate::Candidate;
use crate::error::HoldoutError;
use crate::grade::Divergence;
use crate::procedure::{check_procedure, ProcedurePolicy};

#[derive(Debug, Clone, Serialize)]
pub struct VerifyReport {
    pub total: usize,
    pub passed: usize,
    pub pass_rate: f64,
    pub first_divergence: Option<Divergence>,
    pub procedure_violations: usize,
    pub first_violation: Option<String>,
    pub reward: f64,
}

impl VerifyReport {
    pub fn ok(&self) -> bool {
        self.total > 0 && self.passed == self.total
    }
}

/// Run `reference` and `candidate` on each input and compare their outputs.
/// The reference is the live oracle; inputs the candidate never saw cannot be
/// memorized. `reward` equals the pass rate.
/// An optional `policy` checks the candidate's stderr trace; a run that matches
/// the reference but trips the policy is disqualified (corrupt success).
pub fn verify(
    reference: &Candidate,
    candidate: &Candidate,
    inputs: &[String],
    policy: Option<&ProcedurePolicy>,
) -> Result<VerifyReport, HoldoutError> {
    let mut passed = 0usize;
    let mut first_divergence: Option<Divergence> = None;
    let mut procedure_violations = 0usize;
    let mut first_violation: Option<String> = None;

    for (i, input) in inputs.iter().enumerate() {
        let expected = reference.run(input)?;
        let (actual, trace) = candidate.run_capturing(input)?;
        let output_match = actual == expected;
        let violations = match policy {
            Some(p) => check_procedure(&trace, p),
            None => Vec::new(),
        };
        if output_match && violations.is_empty() {
            passed += 1;
        } else {
            if !output_match && first_divergence.is_none() {
                first_divergence = Some(Divergence {
                    case: format!("g{i}"),
                    input: input.clone(),
                    expected,
                    actual,
                });
            }
            if !violations.is_empty() {
                procedure_violations += 1;
                if first_violation.is_none() {
                    first_violation = Some(format!("input {input:?}: {}", violations.join("; ")));
                }
            }
        }
    }
    let total = inputs.len();
    let pass_rate = if total == 0 {
        1.0
    } else {
        passed as f64 / total as f64
    };
    Ok(VerifyReport {
        total,
        passed,
        pass_rate,
        first_divergence,
        procedure_violations,
        first_violation,
        reward: pass_rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> Vec<String> {
        ["2", "3", "5", "7"].iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn matching_candidate_passes_all() {
        let r = verify(
            &Candidate::from_shell("awk {print($1*$1)}"),
            &Candidate::from_shell("awk {print($1^2)}"),
            &inputs(),
            None,
        )
        .unwrap();
        assert_eq!(r.total, 4);
        assert_eq!(r.passed, 4);
        assert_eq!(r.pass_rate, 1.0);
        assert!(r.first_divergence.is_none());
        assert!(r.ok());
        assert_eq!(r.reward, 1.0);
    }

    #[test]
    fn diverging_candidate_is_caught_with_first_divergence() {
        let r = verify(
            &Candidate::from_shell("awk {print($1*$1)}"),
            &Candidate::from_shell("awk {print($1+$1)}"),
            &inputs(),
            None,
        )
        .unwrap();
        assert!(!r.ok());
        assert!(r.passed < 4);
        let d = r.first_divergence.expect("a divergence");
        // First input 2: reference 4, candidate 2+2=4 → matches; input 3: ref 9, cand 6 → diverges.
        assert_eq!(d.input, "3");
        assert_eq!(d.expected, "9");
        assert_eq!(d.actual, "6");
    }

    #[test]
    fn corrupt_success_is_disqualified() {
        let policy = ProcedurePolicy {
            forbidden: vec!["FORBIDDEN".into()],
            required: vec![],
        };
        let reference = Candidate::from_shell("cat");
        // `tee /dev/stderr` copies stdin to BOTH stdout (matches cat) and stderr (the trace).
        // The trace therefore contains the input; a forbidden input is a corrupt success.
        let candidate = Candidate::from_shell("tee /dev/stderr");

        // Clean input: output matches, trace clean → passes.
        let clean = verify(
            &reference,
            &candidate,
            &["hello".to_string()],
            Some(&policy),
        )
        .unwrap();
        assert!(clean.ok());
        assert_eq!(clean.procedure_violations, 0);

        // Forbidden input: output STILL matches cat, but the trace contains "FORBIDDEN"
        // → disqualified (corrupt success).
        let corrupt = verify(
            &reference,
            &candidate,
            &["FORBIDDEN".to_string()],
            Some(&policy),
        )
        .unwrap();
        assert!(!corrupt.ok());
        assert_eq!(corrupt.procedure_violations, 1);
        assert!(corrupt.first_violation.is_some());
    }
}
