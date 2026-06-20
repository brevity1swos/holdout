use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Outcome of running a candidate once: either it finished and produced output,
/// or it exceeded its wall-clock budget and was killed.
#[derive(Debug, Clone)]
pub enum Run {
    Done { stdout: String, stderr: String },
    TimedOut,
}

#[derive(Debug, Clone)]
pub struct Candidate {
    program: String,
    args: Vec<String>,
    timeout: Option<Duration>,
}

impl Candidate {
    pub fn from_shell(cmd: &str) -> Candidate {
        let mut parts = cmd.split_whitespace().map(|s| s.to_string());
        let program = parts.next().unwrap_or_default();
        let args = parts.collect();
        Candidate {
            program,
            args,
            timeout: None,
        }
    }

    /// Bound this candidate's wall-clock execution. Untrusted candidates (the
    /// thing being graded) should always carry a timeout — a non-terminating
    /// candidate would otherwise hang the grader forever.
    pub fn with_timeout(mut self, timeout: Duration) -> Candidate {
        self.timeout = Some(timeout);
        self
    }

    /// Run once, enforcing the wall-clock budget if set. stdin/stdout/stderr are
    /// each serviced on their own thread so a chatty child can't deadlock on a
    /// full pipe while we wait.
    pub fn exec(&self, input: &str) -> std::io::Result<Run> {
        let mut child = Command::new(&self.program)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().expect("stdin piped");
        let input_owned = input.to_string();
        let in_handle = thread::spawn(move || {
            let _ = stdin.write_all(input_owned.as_bytes());
            // dropping `stdin` here closes it, signalling EOF to the child
        });
        let mut stdout = child.stdout.take().expect("stdout piped");
        let mut stderr = child.stderr.take().expect("stderr piped");
        let out_handle = thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stdout.read_to_end(&mut buf);
            buf
        });
        let err_handle = thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stderr.read_to_end(&mut buf);
            buf
        });

        let timed_out = match self.timeout {
            None => {
                child.wait()?;
                false
            }
            Some(limit) => {
                let start = Instant::now();
                loop {
                    if child.try_wait()?.is_some() {
                        break false;
                    }
                    if start.elapsed() >= limit {
                        child.kill()?;
                        child.wait()?;
                        break true;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            }
        };

        let _ = in_handle.join();
        let out_bytes = out_handle.join().unwrap_or_default();
        let err_bytes = err_handle.join().unwrap_or_default();

        if timed_out {
            Ok(Run::TimedOut)
        } else {
            Ok(Run::Done {
                stdout: String::from_utf8_lossy(&out_bytes).trim_end().to_string(),
                stderr: String::from_utf8_lossy(&err_bytes).trim_end().to_string(),
            })
        }
    }

    pub fn run_capturing(&self, input: &str) -> std::io::Result<(String, String)> {
        match self.exec(input)? {
            Run::Done { stdout, stderr } => Ok((stdout, stderr)),
            Run::TimedOut => Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "candidate exceeded its wall-clock budget",
            )),
        }
    }

    pub fn run(&self, input: &str) -> std::io::Result<String> {
        Ok(self.run_capturing(input)?.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_cat_echoing_stdin() {
        // `cat` echoes stdin to stdout — a trivial identity candidate.
        let c = Candidate::from_shell("cat");
        let out = c.run("hello\n").unwrap();
        assert_eq!(out, "hello");
    }

    #[test]
    fn captures_program_with_args() {
        // `tr a-z A-Z` uppercases stdin.
        let c = Candidate::from_shell("tr a-z A-Z");
        let out = c.run("abc").unwrap();
        assert_eq!(out, "ABC");
    }

    #[test]
    fn run_capturing_returns_stdout_and_stderr() {
        // sh writes "out" to stdout and "trace" to stderr.
        let c = Candidate::from_shell("sh");
        let (out, err) = c.run_capturing("echo out; echo trace 1>&2\n").unwrap();
        assert_eq!(out, "out");
        assert_eq!(err, "trace");
    }

    #[test]
    fn timed_out_candidate_is_killed_and_reported() {
        // `sleep 10` would block forever relative to a 200ms budget.
        let c = Candidate::from_shell("sleep 10").with_timeout(Duration::from_millis(200));
        match c.exec("") {
            Ok(Run::TimedOut) => {}
            other => panic!("expected TimedOut, got {other:?}"),
        }
    }

    #[test]
    fn fast_candidate_completes_within_budget() {
        let c = Candidate::from_shell("cat").with_timeout(Duration::from_secs(5));
        match c.exec("hi") {
            Ok(Run::Done { stdout, .. }) => assert_eq!(stdout, "hi"),
            other => panic!("expected Done, got {other:?}"),
        }
    }
}
