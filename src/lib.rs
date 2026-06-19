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
