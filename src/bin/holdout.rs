use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use holdout::oracle::{self, OracleSpec};
use holdout::{
    generate, grade, parse_inputs, perturb::perturb, record, verify, Candidate, GradeOpts,
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
    },
}

fn run_seal(oracle: &Path) -> anyhow::Result<()> {
    let spec = OracleSpec::load(oracle)?;
    let hex = oracle::seal_hex(&spec);
    let seal_path = {
        let mut p = oracle.to_path_buf().into_os_string();
        p.push(".seal");
        PathBuf::from(p)
    };
    std::fs::write(&seal_path, &hex)?;
    println!("{hex}");
    Ok(())
}

fn run_record(
    reference: &str,
    inputs: &PathBuf,
    visible: usize,
    out: &PathBuf,
) -> anyhow::Result<()> {
    let text = std::fs::read_to_string(inputs)?;
    let parsed = parse_inputs(&text);
    let spec = record(reference, &parsed, visible)?;
    let json = serde_json::to_string_pretty(&spec)?;
    std::fs::write(out, &json)?;
    let hex = oracle::seal_hex(&spec);
    let seal_path = {
        let mut p = out.clone().into_os_string();
        p.push(".seal");
        PathBuf::from(p)
    };
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
) -> anyhow::Result<Outcome> {
    let spec = OracleSpec::load(oracle)?;
    let seal_path = {
        let mut p = oracle.to_path_buf().into_os_string();
        p.push(".seal");
        PathBuf::from(p)
    };
    let expected = std::fs::read_to_string(&seal_path).map_err(|_| {
        anyhow::anyhow!("missing seal file {seal_path:?}; run `holdout seal` first")
    })?;
    oracle::verify_seal(&spec, expected.trim())?; // SealMismatch → Err → exit 2

    let graded_spec = if no_perturb {
        spec
    } else {
        perturb(&spec, 0x5eed)
    };
    let cand = Candidate::from_shell(candidate);
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

fn run_verify(
    reference: &str,
    candidate: &str,
    generator: &str,
    n: usize,
    json: bool,
) -> anyhow::Result<Outcome> {
    let inputs = generate(generator, n)?;
    if inputs.is_empty() {
        anyhow::bail!("generator produced no inputs");
    }
    let report = verify(
        &Candidate::from_shell(reference),
        &Candidate::from_shell(candidate),
        &inputs,
    )?;

    if json {
        println!("{}", serde_json::to_string(&report)?);
    } else {
        println!(
            "verified {}/{} fresh inputs  reward {:.2}",
            report.passed, report.total, report.reward
        );
        if let Some(d) = &report.first_divergence {
            println!(
                "first divergence @ {}: input {:?} reference {:?} candidate {:?}",
                d.case, d.input, d.expected, d.actual
            );
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
        } => match run_record(reference, inputs, *visible, out) {
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
        } => match run_grade(oracle, candidate, *json, *gap_tolerance, *no_perturb) {
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
        } => match run_verify(reference, candidate, generator, *n, *json) {
            Ok(Outcome::Passed) => ExitCode::from(0),
            Ok(Outcome::Failed) => ExitCode::from(1),
            Err(e) => {
                eprintln!("error: {e:#}");
                ExitCode::from(2)
            }
        },
    }
}
