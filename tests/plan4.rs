use holdout::{append_log, digest, read_log, verify, Candidate, LogRecord, ProcedurePolicy, Trend};

#[test]
fn corrupt_success_disqualified_and_log_digest_flags_stuck() {
    // --- procedure gating: output correct, trace forbidden → disqualified ---
    let policy = ProcedurePolicy {
        forbidden: vec!["FORBIDDEN".into()],
        ..Default::default()
    };
    let report = verify(
        &Candidate::from_shell("cat"),
        &Candidate::from_shell("tee /dev/stderr"),
        &["FORBIDDEN".to_string()],
        Some(&policy),
    )
    .unwrap();
    assert!(!report.ok(), "corrupt success must be disqualified");
    assert_eq!(report.procedure_violations, 1);

    // --- watch digest: three flat attempts → STUCK with a drift flag ---
    let path = std::env::temp_dir().join("holdout_plan4_log.jsonl");
    let _ = std::fs::remove_file(&path);
    for _ in 0..3 {
        append_log(
            &path,
            LogRecord {
                attempt: 0,
                mode: "verify".into(),
                total: 5,
                passed: 1,
                reward: 0.2,
                ok: false,
                note: None,
            },
        )
        .unwrap();
    }
    let d = digest(&read_log(&path).unwrap());
    assert_eq!(d.attempts, 3);
    assert_eq!(d.trend, Trend::Stuck);
    assert!(d.drift_flag.is_some());
    let _ = std::fs::remove_file(&path);
}
