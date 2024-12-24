pub fn trim_quotes(s: &str) -> &str {
    s.trim_matches('"')
}