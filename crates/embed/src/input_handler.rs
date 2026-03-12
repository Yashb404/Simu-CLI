pub fn normalize_input(raw: &str) -> String {
    raw.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_user_input() {
        assert_eq!(normalize_input("  ls -la  "), "ls -la");
    }
}
