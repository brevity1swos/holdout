use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct Candidate {
    program: String,
    args: Vec<String>,
}

impl Candidate {
    pub fn from_shell(cmd: &str) -> Candidate {
        let mut parts = cmd.split_whitespace().map(|s| s.to_string());
        let program = parts.next().unwrap_or_default();
        let args = parts.collect();
        Candidate { program, args }
    }

    pub fn run_capturing(&self, input: &str) -> std::io::Result<(String, String)> {
        let mut child = Command::new(&self.program)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        child
            .stdin
            .take()
            .expect("stdin piped")
            .write_all(input.as_bytes())?;
        let output = child.wait_with_output()?;
        Ok((
            String::from_utf8_lossy(&output.stdout)
                .trim_end()
                .to_string(),
            String::from_utf8_lossy(&output.stderr)
                .trim_end()
                .to_string(),
        ))
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
}
