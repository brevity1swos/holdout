use crate::oracle::OracleSpec;

/// Reorder held-out cases by a salted key and rename them to opaque ids.
/// Inputs and expected outputs are preserved exactly.
pub fn perturb(spec: &OracleSpec, salt: u64) -> OracleSpec {
    let mut cases = spec.heldout.clone();
    // Deterministic, salt-dependent order via a cheap hash of (salt, input).
    cases.sort_by_key(|c| {
        let mut h = salt;
        for b in c.input.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as u64);
        }
        h
    });
    for (i, c) in cases.iter_mut().enumerate() {
        c.name = format!("c{i}");
    }
    OracleSpec {
        kind: spec.kind.clone(),
        reference: spec.reference.clone(),
        visible: spec.visible.clone(),
        heldout: cases,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::{Case, OracleKind};

    fn spec() -> OracleSpec {
        OracleSpec {
            kind: OracleKind::HeldoutCases,
            reference: None,
            visible: vec![],
            heldout: vec![
                Case {
                    name: "alpha".into(),
                    input: "3".into(),
                    expected: "9".into(),
                },
                Case {
                    name: "beta".into(),
                    input: "5".into(),
                    expected: "25".into(),
                },
            ],
        }
    }

    #[test]
    fn perturb_renames_and_preserves_io() {
        let p = perturb(&spec(), 42);
        // Names are opaque ids now.
        assert!(p.heldout.iter().all(|c| c.name.starts_with('c')));
        assert!(p
            .heldout
            .iter()
            .all(|c| c.name != "alpha" && c.name != "beta"));
        // The (input, expected) multiset is preserved exactly.
        let mut io: Vec<_> = p
            .heldout
            .iter()
            .map(|c| (c.input.clone(), c.expected.clone()))
            .collect();
        io.sort();
        assert_eq!(
            io,
            vec![("3".into(), "9".into()), ("5".into(), "25".into())]
        );
    }

    #[test]
    fn perturb_is_deterministic_for_a_salt() {
        assert_eq!(
            perturb(&spec(), 7)
                .heldout
                .iter()
                .map(|c| c.input.clone())
                .collect::<Vec<_>>(),
            perturb(&spec(), 7)
                .heldout
                .iter()
                .map(|c| c.input.clone())
                .collect::<Vec<_>>()
        );
    }
}
