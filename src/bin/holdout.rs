use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use holdout::oracle::{self, OracleSpec};

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

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match &cli.command {
        Cmd::Seal { oracle } => run_seal(oracle),
    };
    match result {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(2)
        }
    }
}
