use std::io::Write;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use crate::candidate::{Candidate, Run};
use crate::error::HoldoutError;

/// A human-authored invariant. `command` is run once per (input, candidate
/// output); it receives `{"input":..,"output":..}` JSON on stdin and exits 0 if
/// the invariant holds, non-zero if it is violated. This is greenfield grading:
/// no reference output is needed — the human states properties the output must
/// satisfy ("is sorted", "len(out)==len(in)", "round-trips"), holdout checks them
/// over fresh/held-out inputs with the same ungameable machinery as the rest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub command: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PropertySet {
    #[serde(default)]
    pub properties: Vec<Property>,
}

#[derive(Serialize)]
struct Payload<'a> {
    input: &'a str,
    output: &'a str,
}

/// Run every property on one (input, output); return the names of violated ones.
/// Properties are trusted (human-authored), so they are not wall-clock bounded.
pub fn check_properties(
    input: &str,
    output: &str,
    set: &PropertySet,
) -> std::io::Result<Vec<String>> {
    let payload = serde_json::to_string(&Payload { input, output })?;
    let mut violated = Vec::new();
    for p in &set.properties {
        let mut parts = p.command.split_whitespace();
        let prog = parts.next().unwrap_or_default();
        let args: Vec<&str> = parts.collect();
        let mut child = Command::new(prog)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        // The predicate may exit before reading stdin (e.g. `true`/`false`) —
        // a broken-pipe write is not an error here.
        let _ = child
            .stdin
            .take()
            .expect("stdin piped")
            .write_all(payload.as_bytes());
        if !child.wait()?.success() {
            violated.push(p.name.clone());
        }
    }
    Ok(violated)
}

#[derive(Debug, Clone, Serialize)]
pub struct PropReport {
    pub total: usize,
    pub passed: usize,
    pub pass_rate: f64,
    pub first_violation: Option<String>,
    pub reward: f64,
}

impl PropReport {
    pub fn ok(&self) -> bool {
        self.total > 0 && self.passed == self.total
    }
}

/// Run `candidate` (wall-clock bounded) over `inputs` and check every property on
/// each (input, output). An input passes only if ALL properties hold.
pub fn assess(
    candidate: &Candidate,
    inputs: &[String],
    set: &PropertySet,
) -> Result<PropReport, HoldoutError> {
    let mut passed = 0usize;
    let mut first_violation: Option<String> = None;
    for input in inputs {
        let output = match candidate.exec(input)? {
            Run::Done { stdout, .. } => stdout,
            Run::TimedOut => {
                if first_violation.is_none() {
                    first_violation = Some(format!("input {input:?}: candidate timed out"));
                }
                continue;
            }
        };
        let violated = check_properties(input, &output, set)?;
        if violated.is_empty() {
            passed += 1;
        } else if first_violation.is_none() {
            first_violation = Some(format!("input {input:?}: violates {}", violated.join(", ")));
        }
    }
    let total = inputs.len();
    let pass_rate = if total == 0 {
        1.0
    } else {
        passed as f64 / total as f64
    };
    Ok(PropReport {
        total,
        passed,
        pass_rate,
        first_violation,
        reward: pass_rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_properties_reports_violated_names() {
        // `true` always holds (exit 0); `false` always violates (exit 1).
        let set = PropertySet {
            properties: vec![
                Property {
                    name: "always".into(),
                    command: "true".into(),
                },
                Property {
                    name: "never".into(),
                    command: "false".into(),
                },
            ],
        };
        let violated = check_properties("in", "out", &set).unwrap();
        assert_eq!(violated, vec!["never".to_string()]);
    }

    #[test]
    fn assess_counts_inputs_satisfying_all_properties() {
        // A candidate that echoes stdin; property "always" holds, so all pass.
        // (A realistic file-based predicate is exercised in tests/properties.rs;
        // property commands are whitespace-split, so real predicates are script
        // files, not inline `python3 -c "..."`.)
        let set = PropertySet {
            properties: vec![Property {
                name: "always".into(),
                command: "true".into(),
            }],
        };
        let r = assess(
            &Candidate::from_shell("cat"),
            &["a".into(), "b".into()],
            &set,
        )
        .unwrap();
        assert_eq!(r.total, 2);
        assert_eq!(r.passed, 2);
        assert!(r.ok());

        let bad = PropertySet {
            properties: vec![Property {
                name: "never".into(),
                command: "false".into(),
            }],
        };
        let r = assess(&Candidate::from_shell("cat"), &["a".into()], &bad).unwrap();
        assert!(!r.ok());
        assert!(r.first_violation.unwrap().contains("never"));
    }
}
