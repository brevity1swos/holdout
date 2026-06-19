use std::fmt;

use crate::candidate::Candidate;
use crate::oracle::{Case, OracleKind, OracleSpec};

pub fn parse_inputs(text: &str) -> Vec<String> {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect()
}

#[derive(Debug)]
pub enum RecordError {
    Run(std::io::Error),
}

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecordError::Run(e) => write!(f, "failed to run reference: {e}"),
        }
    }
}

impl std::error::Error for RecordError {}

impl From<std::io::Error> for RecordError {
    fn from(e: std::io::Error) -> Self {
        RecordError::Run(e)
    }
}

pub fn generate(generator: &str, cap: usize) -> Result<Vec<String>, RecordError> {
    let cand = Candidate::from_shell(generator);
    let out = cand.run("")?;
    let mut inputs = parse_inputs(&out);
    if cap > 0 && inputs.len() > cap {
        inputs.truncate(cap);
    }
    Ok(inputs)
}

pub fn record(
    reference: &str,
    inputs: &[String],
    visible_count: usize,
) -> Result<OracleSpec, RecordError> {
    let cand = Candidate::from_shell(reference);
    let mut visible = Vec::new();
    let mut heldout = Vec::new();
    for (i, input) in inputs.iter().enumerate() {
        let expected = cand.run(input)?;
        if i < visible_count {
            visible.push(Case {
                name: format!("v{i}"),
                input: input.clone(),
                expected,
            });
        } else {
            let h = heldout.len();
            heldout.push(Case {
                name: format!("h{h}"),
                input: input.clone(),
                expected,
            });
        }
    }
    Ok(OracleSpec {
        kind: OracleKind::GoldenTrace,
        reference: Some(reference.to_string()),
        visible,
        heldout,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_one_input_per_nonempty_line() {
        let got = parse_inputs("2\n3\n\n5\n");
        assert_eq!(got, vec!["2".to_string(), "3".to_string(), "5".to_string()]);
    }

    #[test]
    fn generate_runs_command_and_caps() {
        // `seq 1 5` emits 1..5 on its own lines.
        let all = generate("seq 1 5", 0).unwrap();
        assert_eq!(all, vec!["1", "2", "3", "4", "5"]);
        let capped = generate("seq 1 5", 3).unwrap();
        assert_eq!(capped, vec!["1", "2", "3"]);
    }

    use crate::oracle::OracleKind;

    #[test]
    fn record_captures_reference_outputs_and_splits() {
        // Reference squares its input; capture over 4 inputs, 1 visible.
        let inputs = vec![
            "2".to_string(),
            "3".to_string(),
            "5".to_string(),
            "7".to_string(),
        ];
        let spec = record("awk {print($1*$1)}", &inputs, 1).unwrap();
        assert_eq!(spec.kind, OracleKind::GoldenTrace);
        assert_eq!(spec.reference.as_deref(), Some("awk {print($1*$1)}"));
        assert_eq!(spec.visible.len(), 1);
        assert_eq!(spec.heldout.len(), 3);
        // Captured truth: 2->4 visible; 3->9, 5->25, 7->49 heldout.
        assert_eq!(spec.visible[0].input, "2");
        assert_eq!(spec.visible[0].expected, "4");
        assert_eq!(spec.heldout[0].expected, "9");
        assert_eq!(spec.heldout[2].expected, "49");
        assert_eq!(spec.heldout[0].name, "h0");
    }
}
