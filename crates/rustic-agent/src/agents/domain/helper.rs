pub fn clean_content(s: &str) -> String {
    let s = s.trim();
    if s.starts_with("```json") {
        s.trim_start_matches("```json")
            .trim_end_matches("```")
            .trim()
            .to_string()
    } else if s.starts_with("```") {
        s.trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
            .to_string()
    } else {
        s.to_string()
    }
}

pub fn truncate(s: &str, max: usize) -> String {
    let s = clean_content(s);
    if s.len() <= max {
        s
    } else {
        format!("{}...", &s[..max])
    }
}
