pub fn parse_inputs(text: &str) -> Vec<String> {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_one_input_per_nonempty_line() {
        let got = parse_inputs("2\n3\n\n5\n");
        assert_eq!(got, vec!["2".to_string(), "3".to_string(), "5".to_string()]);
    }
}
