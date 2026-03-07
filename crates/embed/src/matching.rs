use shared::models::demo::MatchMode;

pub fn command_matches(mode: &MatchMode, expected: &str, actual: &str) -> bool {
    match mode {
        MatchMode::Exact => expected == actual,
        MatchMode::Fuzzy => actual.contains(expected),
        MatchMode::Wildcard => wildcard_match(expected, actual),
        MatchMode::Any => true,
    }
}

fn wildcard_match(pattern: &str, input: &str) -> bool {
    // Minimal wildcard support for MVP: '*' can match any sequence.
    if pattern == "*" {
        return true;
    }

    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == input;
    }

    let mut search_start = 0usize;
    for (idx, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if idx == 0 && !input[search_start..].starts_with(part) {
            return false;
        }

        if let Some(found) = input[search_start..].find(part) {
            search_start += found + part.len();
        } else {
            return false;
        }
    }

    if let Some(last) = parts.last() {
        if !last.is_empty() && !input.ends_with(last) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_matching_works() {
        assert!(wildcard_match("npm *", "npm install"));
        assert!(wildcard_match("git * origin", "git push origin"));
        assert!(!wildcard_match("cargo * test", "cargo test"));
    }
}
