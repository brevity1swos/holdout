use serde::Serialize;

use crate::candidate::Candidate;
use crate::grade::Divergence;

#[derive(Debug, Clone, Serialize)]
pub struct VerifyReport {
    pub total: usize,
    pub passed: usize,
    pub pass_rate: f64,
    pub first_divergence: Option<Divergence>,
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
pub fn verify(
    reference: &Candidate,
    candidate: &Candidate,
    inputs: &[String],
) -> std::io::Result<VerifyReport> {
    let mut passed = 0usize;
    let mut first_divergence: Option<Divergence> = None;
    for (i, input) in inputs.iter().enumerate() {
        let expected = reference.run(input)?;
        let actual = candidate.run(input)?;
        if actual == expected {
            passed += 1;
        } else if first_divergence.is_none() {
            first_divergence = Some(Divergence {
                case: format!("g{i}"),
                input: input.clone(),
                expected,
                actual,
            });
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
}
