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

    pub fn run(&self, input: &str) -> std::io::Result<String> {
        let mut child = Command::new(&self.program)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        child
            .stdin
            .take()
            .expect("stdin piped")
            .write_all(input.as_bytes())?;
        let output = child.wait_with_output()?;
        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string())
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
}
