use holdout::{assess, Candidate, Property, PropertySet};

/// Greenfield grading: there is NO reference output — the human states a property
/// the output must satisfy, and holdout checks it. Here the property is "output is
/// the input reversed", verified by a real predicate script reading {input,output}.
#[test]
fn greenfield_property_grading_without_a_reference() {
    let pred = std::env::temp_dir().join("holdout_rev_prop.py");
    std::fs::write(
        &pred,
        r#"import sys, json
d = json.load(sys.stdin)
sys.exit(0 if d["output"] == d["input"][::-1] else 1)
"#,
    )
    .unwrap();

    let set = PropertySet {
        properties: vec![Property {
            name: "reverses".into(),
            command: format!("python3 {}", pred.display()),
        }],
    };

    // Correct candidate: `rev` reverses each line. Satisfies the property.
    let good = assess(
        &Candidate::from_shell("rev"),
        &["abc".to_string(), "xy".to_string()],
        &set,
    )
    .unwrap();
    assert!(good.ok(), "{good:?}");
    assert_eq!(good.passed, 2);

    // Wrong candidate: `cat` (identity) violates the reverse property on "abc".
    let bad = assess(&Candidate::from_shell("cat"), &["abc".to_string()], &set).unwrap();
    assert!(!bad.ok());
    assert!(bad.first_violation.unwrap().contains("reverses"));

    let _ = std::fs::remove_file(&pred);
}
