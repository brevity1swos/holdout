use holdout::{generate, verify, Candidate};

#[test]
fn generated_inputs_drive_live_differential_verification() {
    // Fresh inputs come from the generator — they did not exist when the
    // candidate was written, so they cannot be memorized.
    let inputs = generate("seq 1 25", 0).unwrap();
    assert_eq!(inputs.len(), 25);

    let reference = Candidate::from_shell("awk {print($1*$1)}");

    // A faithful candidate matches the reference on every generated input.
    let good = verify(
        &reference,
        &Candidate::from_shell("awk {print($1^2)}"),
        &inputs,
    )
    .unwrap();
    assert!(good.ok());
    assert_eq!(good.passed, 25);

    // A behavior-changing candidate is caught, with the first divergence reported.
    let bad = verify(
        &reference,
        &Candidate::from_shell("awk {print($1+$1)}"),
        &inputs,
    )
    .unwrap();
    assert!(!bad.ok());
    assert!(bad.first_divergence.is_some());
    assert!(bad.reward < 1.0);
}
