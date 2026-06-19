use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::HoldoutError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    pub attempt: usize,
    pub mode: String,
    pub total: usize,
    pub passed: usize,
    pub reward: f64,
    pub ok: bool,
    #[serde(default)]
    pub note: Option<String>,
}

pub fn read_log(path: &Path) -> Result<Vec<LogRecord>, HoldoutError> {
    match std::fs::read_to_string(path) {
        Ok(text) => text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).map_err(HoldoutError::from))
            .collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(HoldoutError::from(e)),
    }
}

pub fn append_log(path: &Path, mut record: LogRecord) -> Result<LogRecord, HoldoutError> {
    use std::io::Write;
    let existing = read_log(path)?;
    record.attempt = existing.len();
    let mut line = serde_json::to_string(&record)?;
    line.push('\n');
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    f.write_all(line.as_bytes())?;
    Ok(record)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Trend {
    Converged,
    Improving,
    Stuck,
    Regressing,
    Empty,
}

#[derive(Debug, Clone, Serialize)]
pub struct Digest {
    pub attempts: usize,
    pub latest_reward: f64,
    pub best_reward: f64,
    pub trend: Trend,
    pub drift_flag: Option<String>,
}

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

pub fn digest(records: &[LogRecord]) -> Digest {
    if records.is_empty() {
        return Digest {
            attempts: 0,
            latest_reward: 0.0,
            best_reward: 0.0,
            trend: Trend::Empty,
            drift_flag: None,
        };
    }
    let rewards: Vec<f64> = records.iter().map(|r| r.reward).collect();
    let n = rewards.len();
    let latest = rewards[n - 1];
    let best = rewards.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let trend = if records[n - 1].ok {
        Trend::Converged
    } else if n >= 2 && rewards[n - 1] < rewards[n - 2] - 1e-9 {
        Trend::Regressing
    } else if n >= 3
        && approx(rewards[n - 1], rewards[n - 2])
        && approx(rewards[n - 2], rewards[n - 3])
    {
        Trend::Stuck
    } else {
        Trend::Improving
    };
    let drift_flag = match trend {
        Trend::Stuck => Some(format!(
            "no improvement over last 3 attempts (reward {latest:.2}) — consider interrupting"
        )),
        Trend::Regressing => Some(format!(
            "reward dropped to {latest:.2} — candidate is getting worse"
        )),
        _ => None,
    };
    Digest {
        attempts: n,
        latest_reward: latest,
        best_reward: best,
        trend,
        drift_flag,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(reward: f64, ok: bool) -> LogRecord {
        LogRecord {
            attempt: 0,
            mode: "verify".into(),
            total: 10,
            passed: 0,
            reward,
            ok,
            note: None,
        }
    }

    #[test]
    fn append_then_read_roundtrips_and_numbers_attempts() {
        let dir = std::env::temp_dir();
        let path = dir.join("holdout_runlog_test.jsonl");
        let _ = std::fs::remove_file(&path);
        let r0 = append_log(&path, rec(0.2, false)).unwrap();
        let r1 = append_log(&path, rec(0.5, false)).unwrap();
        assert_eq!(r0.attempt, 0);
        assert_eq!(r1.attempt, 1);
        let back = read_log(&path).unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back[1].attempt, 1);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn read_missing_file_is_empty() {
        let path = std::env::temp_dir().join("holdout_runlog_absent.jsonl");
        let _ = std::fs::remove_file(&path);
        assert!(read_log(&path).unwrap().is_empty());
    }

    #[test]
    fn digest_flags_stuck_and_converged() {
        let stuck = vec![rec(0.4, false), rec(0.4, false), rec(0.4, false)];
        let d = digest(&stuck);
        assert_eq!(d.trend, Trend::Stuck);
        assert!(d.drift_flag.is_some());

        let converged = vec![rec(0.4, false), rec(1.0, true)];
        let d = digest(&converged);
        assert_eq!(d.trend, Trend::Converged);
        assert!(d.drift_flag.is_none());

        let regress = vec![rec(0.8, false), rec(0.3, false)];
        assert_eq!(digest(&regress).trend, Trend::Regressing);

        assert_eq!(digest(&[]).trend, Trend::Empty);
    }
}
