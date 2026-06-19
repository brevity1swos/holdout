use holdout::oracle::{self, Case, OracleKind, OracleSpec};
use holdout::{grade, perturb::perturb, Candidate, GradeOpts};

fn square_oracle() -> OracleSpec {
    OracleSpec {
        kind: OracleKind::HeldoutCases,
        reference: None,
        visible: vec![Case {
            name: "v1".into(),
            input: "2".into(),
            expected: "4".into(),
        }],
        heldout: vec![
            Case {
                name: "h1".into(),
                input: "3".into(),
                expected: "9".into(),
            },
            Case {
                name: "h2".into(),
                input: "5".into(),
                expected: "25".into(),
            },
            Case {
                name: "h3".into(),
                input: "7".into(),
                expected: "49".into(),
            },
        ],
    }
}

#[test]
fn seal_verifies_then_grading_separates_memorizer_from_real() {
    let spec = square_oracle();
    let sealed = oracle::seal_hex(&spec);
    assert!(oracle::verify_seal(&spec, &sealed).is_ok());

    let hardened = perturb(&spec, 0x5eed);
    let opts = GradeOpts::default();

    let real = grade(
        &Candidate::from_shell("awk {print($1*$1)}"),
        &hardened,
        &opts,
    )
    .unwrap();
    assert!(real.passed(&opts));
    assert_eq!(real.delta_gap, 0.0);

    let memorizer = grade(&Candidate::from_shell("awk {print(4)}"), &hardened, &opts).unwrap();
    assert!(!memorizer.passed(&opts));
    assert_eq!(memorizer.heldout_score, 0.0);
    assert!(memorizer.delta_gap > 0.5);
    assert!(memorizer.reward < real.reward);
}
