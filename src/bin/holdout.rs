use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand};
use holdout::oracle::{self, OracleSpec};
use holdout::{
    append_log, assess, digest, generate, grade, parse_inputs, perturb::perturb, read_log, record,
    verify, Candidate, GradeOpts, LogRecord, ProcedurePolicy, PropertySet, Trend,
};

#[derive(Parser)]
#[command(name = "holdout", version, about = "A verifier you cannot game.")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Compute and write the tamper seal for an oracle file.
    Seal {
        #[arg(long)]
        oracle: PathBuf,
    },
    /// Capture a trusted reference's behavior into a sealed GoldenTrace oracle.
    Record {
        #[arg(long)]
        reference: String,
        #[arg(long)]
        inputs: PathBuf,
        #[arg(long, default_value_t = 0)]
        visible: usize,
        #[arg(long)]
        out: PathBuf,
        /// Store BLAKE3 hashes of the held-out answers instead of plaintext, so
        /// reading the oracle file reveals nothing.
        #[arg(long)]
        hash_expected: bool,
    },
    /// Grade a candidate command against the sealed oracle.
    Grade {
        #[arg(long)]
        oracle: PathBuf,
        #[arg(long)]
        candidate: String,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value_t = 0.0)]
        gap_tolerance: f64,
        #[arg(long)]
        no_perturb: bool,
        /// Expected oracle seal, supplied out-of-band by the trusted caller.
        /// Overrides the on-disk sidecar (also reads the HOLDOUT_SEAL env var).
        #[arg(long)]
        seal: Option<String>,
        /// Per-candidate-run wall-clock budget in ms (0 disables).
        #[arg(long, default_value_t = 5000)]
        timeout_ms: u64,
    },
    /// Verify a candidate against a live reference over freshly generated inputs.
    Verify {
        #[arg(long)]
        reference: String,
        #[arg(long)]
        candidate: String,
        #[arg(long)]
        generator: String,
        #[arg(long, default_value_t = 0)]
        n: usize,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        policy: Option<PathBuf>,
        #[arg(long)]
        log: Option<PathBuf>,
        /// Per-candidate-run wall-clock budget in ms (0 disables).
        #[arg(long, default_value_t = 5000)]
        timeout_ms: u64,
    },
    /// Render a human-readable digest of a run log (for mid-loop interruption).
    Watch {
        #[arg(long)]
        log: PathBuf,
    },
    /// Grade a candidate against human-authored property predicates (greenfield —
    /// no reference needed). Each property command receives {"input":..,"output":..}
    /// on stdin and exits 0 (holds) / non-zero (violated).
    Properties {
        #[arg(long)]
        candidate: String,
        #[arg(long)]
        generator: String,
        #[arg(long)]
        properties: PathBuf,
        #[arg(long, default_value_t = 0)]
        n: usize,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value_t = 5000)]
        timeout_ms: u64,
    },
}

/// Build a candidate command, bounding its wall-clock execution when
/// `timeout_ms > 0` (0 disables the budget). Untrusted candidates should
/// always carry a budget so an infinite-loop bug can't hang the grader.
fn build_candidate(cmd: &str, timeout_ms: u64) -> Candidate {
    let cand = Candidate::from_shell(cmd);
    if timeout_ms > 0 {
        cand.with_timeout(Duration::from_millis(timeout_ms))
    } else {
        cand
    }
}

fn run_seal(oracle: &Path) -> anyhow::Result<()> {
    let spec = OracleSpec::load(oracle)?;
    let hex = oracle::seal_hex(&spec);
    let seal_path = oracle::seal_path(oracle);
    std::fs::write(&seal_path, &hex)?;
    println!("{hex}");
    Ok(())
}

fn run_record(
    reference: &str,
    inputs: &PathBuf,
    visible: usize,
    out: &PathBuf,
    hash_expected: bool,
) -> anyhow::Result<()> {
    let text = std::fs::read_to_string(inputs)?;
    let parsed = parse_inputs(&text);
    let spec = record(reference, &parsed, visible, hash_expected)?;
    let json = serde_json::to_string_pretty(&spec)?;
    std::fs::write(out, &json)?;
    let hex = oracle::seal_hex(&spec);
    let seal_path = oracle::seal_path(out);
    std::fs::write(&seal_path, &hex)?;
    println!(
        "recorded {} visible + {} heldout cases from {:?} -> {:?} (sealed {})",
        spec.visible.len(),
        spec.heldout.len(),
        reference,
        out,
        &hex[..16]
    );
    Ok(())
}

enum Outcome {
    Passed,
    Failed,
}

fn run_grade(
    oracle: &Path,
    candidate: &str,
    json: bool,
    gap_tolerance: f64,
    no_perturb: bool,
    seal: &Option<String>,
    timeout_ms: u64,
) -> anyhow::Result<Outcome> {
    let spec = OracleSpec::load(oracle)?;
    // Resolve the expected seal out-of-band first — the trusted caller holds it
    // (--seal or HOLDOUT_SEAL), falling back to the on-disk sidecar for local use.
    // Out-of-band is what makes tampering detectable even when the agent can write
    // the workspace: it cannot forge a seal it never sees.
    let expected = if let Some(s) = seal {
        s.clone()
    } else if let Ok(s) = std::env::var("HOLDOUT_SEAL") {
        s
    } else {
        let seal_path = oracle::seal_path(oracle);
        std::fs::read_to_string(&seal_path).map_err(|_| {
            anyhow::anyhow!(
                "no seal: pass --seal / HOLDOUT_SEAL, or run `holdout seal` for a sidecar"
            )
        })?
    };
    oracle::verify_seal(&spec, expected.trim())?; // SealMismatch → Err → exit 2

    let graded_spec = if no_perturb {
        spec
    } else {
        perturb(&spec, 0x5eed)
    };
    let cand = build_candidate(candidate, timeout_ms);
    let opts = GradeOpts { gap_tolerance };
    let report = grade(&cand, &graded_spec, &opts)?;

    if json {
        println!("{}", serde_json::to_string(&report)?);
    } else {
        println!(
            "heldout {:.0}%  visible {:.0}%  gap {:.2}  reward {:.2}",
            report.heldout_score * 100.0,
            report.visible_score * 100.0,
            report.delta_gap,
            report.reward
        );
        if let Some(d) = &report.first_divergence {
            println!(
                "first divergence @ {}: input {:?} expected {:?} got {:?}",
                d.case, d.input, d.expected, d.actual
            );
        }
    }

    Ok(if report.passed(&opts) {
        Outcome::Passed
    } else {
        Outcome::Failed
    })
}

#[derive(Clone, Copy)]
struct VerifyArgs<'a> {
    reference: &'a str,
    candidate: &'a str,
    generator: &'a str,
    n: usize,
    json: bool,
    policy_path: &'a Option<PathBuf>,
    log_path: &'a Option<PathBuf>,
    timeout_ms: u64,
}

fn run_verify(args: VerifyArgs) -> anyhow::Result<Outcome> {
    let VerifyArgs {
        reference,
        candidate,
        generator,
        n,
        json,
        policy_path,
        log_path,
        timeout_ms,
    } = args;
    let inputs = generate(generator, n)?;
    if inputs.is_empty() {
        anyhow::bail!("generator produced no inputs");
    }
    let policy: Option<ProcedurePolicy> = match policy_path {
        Some(p) => Some(serde_json::from_slice(&std::fs::read(p)?)?),
        None => None,
    };
    let report = verify(
        &Candidate::from_shell(reference),
        &build_candidate(candidate, timeout_ms),
        &inputs,
        policy.as_ref(),
    )?;

    if let Some(lp) = log_path {
        let note = report.first_violation.clone().or_else(|| {
            report
                .first_divergence
                .as_ref()
                .map(|d| format!("diverged @ {}", d.case))
        });
        append_log(
            lp,
            LogRecord {
                attempt: 0,
                mode: "verify".into(),
                total: report.total,
                passed: report.passed,
                reward: report.reward,
                ok: report.ok(),
                note,
            },
        )?;
    }

    if json {
        println!("{}", serde_json::to_string(&report)?);
    } else {
        println!(
            "verified {}/{} fresh inputs  reward {:.2}  procedure-violations {}",
            report.passed, report.total, report.reward, report.procedure_violations
        );
        if let Some(d) = &report.first_divergence {
            println!(
                "first divergence @ {}: input {:?} reference {:?} candidate {:?}",
                d.case, d.input, d.expected, d.actual
            );
        }
        if let Some(v) = &report.first_violation {
            println!("first procedure violation: {v}");
        }
    }

    Ok(if report.ok() {
        Outcome::Passed
    } else {
        Outcome::Failed
    })
}

fn run_watch(log: &Path) -> anyhow::Result<()> {
    let records = read_log(log)?;
    let d = digest(&records);
    let trend = match d.trend {
        Trend::Converged => "CONVERGED",
        Trend::Improving => "IMPROVING",
        Trend::Stuck => "STUCK",
        Trend::Regressing => "REGRESSING",
        Trend::Empty => "EMPTY",
    };
    println!(
        "holdout watch — {} attempts | latest reward {:.2} | best {:.2} | {}",
        d.attempts, d.latest_reward, d.best_reward, trend
    );
    for r in &records {
        let mark = if r.ok { "ok" } else { "  " };
        let note = r.note.as_deref().unwrap_or("");
        println!(
            "  #{:<3} reward {:.2} {} {}",
            r.attempt, r.reward, mark, note
        );
    }
    if let Some(flag) = &d.drift_flag {
        println!("  ⚠ {flag}");
    }
    Ok(())
}

fn run_properties(
    candidate: &str,
    generator: &str,
    properties: &Path,
    n: usize,
    json: bool,
    timeout_ms: u64,
) -> anyhow::Result<Outcome> {
    let inputs = generate(generator, n)?;
    if inputs.is_empty() {
        anyhow::bail!("generator produced no inputs");
    }
    let set: PropertySet = serde_json::from_slice(&std::fs::read(properties)?)?;
    let report = assess(&build_candidate(candidate, timeout_ms), &inputs, &set)?;
    if json {
        println!("{}", serde_json::to_string(&report)?);
    } else {
        println!(
            "properties: {}/{} inputs satisfied all  reward {:.2}",
            report.passed, report.total, report.reward
        );
        if let Some(v) = &report.first_violation {
            println!("first violation: {v}");
        }
    }
    Ok(if report.ok() {
        Outcome::Passed
    } else {
        Outcome::Failed
    })
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match &cli.command {
        Cmd::Record {
            reference,
            inputs,
            visible,
            out,
            hash_expected,
        } => match run_record(reference, inputs, *visible, out, *hash_expected) {
            Ok(()) => ExitCode::from(0),
            Err(e) => {
                eprintln!("error: {e:#}");
                ExitCode::from(2)
            }
        },
        Cmd::Seal { oracle } => match run_seal(oracle) {
            Ok(()) => ExitCode::from(0),
            Err(e) => {
                eprintln!("error: {e:#}");
                ExitCode::from(2)
            }
        },
        Cmd::Grade {
            oracle,
            candidate,
            json,
            gap_tolerance,
            no_perturb,
            seal,
            timeout_ms,
        } => match run_grade(
            oracle,
            candidate,
            *json,
            *gap_tolerance,
            *no_perturb,
            seal,
            *timeout_ms,
        ) {
            Ok(Outcome::Passed) => ExitCode::from(0),
            Ok(Outcome::Failed) => ExitCode::from(1),
            Err(e) => {
                eprintln!("error: {e:#}");
                ExitCode::from(2)
            }
        },
        Cmd::Verify {
            reference,
            candidate,
            generator,
            n,
            json,
            policy,
            log,
            timeout_ms,
        } => match run_verify(VerifyArgs {
            reference,
            candidate,
            generator,
            n: *n,
            json: *json,
            policy_path: policy,
            log_path: log,
            timeout_ms: *timeout_ms,
        }) {
            Ok(Outcome::Passed) => ExitCode::from(0),
            Ok(Outcome::Failed) => ExitCode::from(1),
            Err(e) => {
                eprintln!("error: {e:#}");
                ExitCode::from(2)
            }
        },
        Cmd::Watch { log } => match run_watch(log) {
            Ok(()) => ExitCode::from(0),
            Err(e) => {
                eprintln!("error: {e:#}");
                ExitCode::from(2)
            }
        },
        Cmd::Properties {
            candidate,
            generator,
            properties,
            n,
            json,
            timeout_ms,
        } => match run_properties(candidate, generator, properties, *n, *json, *timeout_ms) {
            Ok(Outcome::Passed) => ExitCode::from(0),
            Ok(Outcome::Failed) => ExitCode::from(1),
            Err(e) => {
                eprintln!("error: {e:#}");
                ExitCode::from(2)
            }
        },
    }
}
