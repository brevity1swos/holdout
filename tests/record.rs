use holdout::oracle::{self, OracleKind};
use holdout::{grade, perturb::perturb, record, Candidate, GradeOpts};

#[test]
fn record_then_grade_catches_behavior_change_on_unseen_input() {
    let inputs: Vec<String> = ["2", "3", "5", "7", "11"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    // Capture a trusted reference (square). 1 visible, 4 held-out the candidate never sees.
    let spec = record("awk {print($1*$1)}", &inputs, 1).unwrap();
    assert_eq!(spec.kind, OracleKind::GoldenTrace);

    // Seal + verify the captured oracle.
    let sealed = oracle::seal_hex(&spec);
    assert!(oracle::verify_seal(&spec, &sealed).is_ok());

    let hardened = perturb(&spec, 0x5eed);
    let opts = GradeOpts::default();

    // A faithful refactor (different syntax, same behavior) passes.
    let faithful = grade(
        &Candidate::from_shell("awk {print($1^2)}"),
        &hardened,
        &opts,
    )
    .unwrap();
    assert!(
        faithful.passed(&opts),
        "faithful refactor should pass: {faithful:?}"
    );

    // A behavior-changing "refactor" (double instead of square) is caught on held-out inputs.
    let broken = grade(
        &Candidate::from_shell("awk {print($1+$1)}"),
        &hardened,
        &opts,
    )
    .unwrap();
    assert!(!broken.passed(&opts));
    assert!(broken.heldout_score < 1.0);
    assert!(broken.first_divergence.is_some());
}
