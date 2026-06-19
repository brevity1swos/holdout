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
pub mod record;

pub use candidate::Candidate;
pub use error::HoldoutError;
pub use grade::{grade, Divergence, GradeOpts, GradeReport};
pub use oracle::{Case, OracleKind, OracleSpec};
pub use record::{parse_inputs, record, RecordError};
