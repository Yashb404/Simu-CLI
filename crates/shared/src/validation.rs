pub const MAX_STEPS: usize = 50;
pub const MAX_OUTPUT_LINES_PER_STEP: usize = 100;

pub fn is_valid_slug(slug: &str) -> bool {
    let len = slug.len();
    if len < 3 || len > 60 {
        return false;
    }
    // Only lowercase alphanumeric and hyphens
    slug.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub fn is_valid_hex_color(color: &str) -> bool {
    let len = color.len();
    if len != 4 && len != 7 {
        return false;
    }
    if !color.starts_with('#') {
        return false;
    }
    color[1..].chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_slugs() {
        assert!(is_valid_slug("my-cli-tool"));
        assert!(is_valid_slug("v1-0-0"));
        assert!(is_valid_slug("a-b-c"));
    }

    #[test]
    fn test_invalid_slugs() {
        assert!(!is_valid_slug("My-Cli")); // Uppercase not allowed
        assert!(!is_valid_slug("my cli")); // Spaces not allowed
        assert!(!is_valid_slug("ab")); // Too short (min 3)
        assert!(!is_valid_slug("my_cli_tool")); // Underscores not allowed
    }

    #[test]
    fn test_valid_hex_colors() {
        assert!(is_valid_hex_color("#fff"));
        assert!(is_valid_hex_color("#FF0055"));
        assert!(is_valid_hex_color("#123456"));
    }

    #[test]
    fn test_invalid_hex_colors() {
        assert!(!is_valid_hex_color("FF0055")); // Missing #
        assert!(!is_valid_hex_color("#ff0055g")); // Invalid hex character
        assert!(!is_valid_hex_color("#12345")); // Wrong length
    }
}
