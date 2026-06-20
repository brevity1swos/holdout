//! holdout — a tamper-resistant, agent-facing verification oracle.
//!
//! Grade a candidate command against a *sealed, held-out* suite of
//! input→expected cases it cannot read or edit, exposing the
//! validation−heldout gap that reveals memorization / reward hacking.

pub mod candidate;
pub mod error;
pub mod grade;
pub mod oracle;
pub mod perturb;
pub mod procedure;
pub mod record;
pub mod runlog;
pub mod verify;

pub use candidate::{Candidate, Run};
pub use error::HoldoutError;
pub use grade::{grade, Divergence, GradeOpts, GradeReport};
pub use oracle::{Case, OracleKind, OracleSpec};
pub use procedure::{check_procedure, ProcedurePolicy};
pub use record::{generate, parse_inputs, record};
pub use runlog::{append_log, digest, read_log, Digest, LogRecord, Trend};
pub use verify::{verify, VerifyReport};
