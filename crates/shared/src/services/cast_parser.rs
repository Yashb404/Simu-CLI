//! Asciinema v2 `.cast` file parser.
//!
//! Extracts `CommandInteraction` pairs by replaying the event stream as a
//! pure state machine.  Timestamps are **completely ignored** — this is
//! intentional: the CLI demo engine is a command→output state machine, not a player.
//!
//! # Algorithm
//! 1. Skip the JSON header on line 1.
//! 2. Iterate events as `[time, type, data]` arrays.
//! 3. On `"i"` events: accumulate keystrokes.  When `\r` or `\n` is seen,
//!    mark the start of an output-collection window and flush any previous
//!    completed pair.
//! 4. On `"o"` events inside a collection window: accumulate output text.
//! 5. After all lines, flush any trailing in-progress pair.
//! 6. Optionally strip dangling prompts from the tail of each output string.

use serde_json::Value;

// ── Public types ─────────────────────────────────────────────────────────────

/// A single command → output pair extracted from a `.cast` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandInteraction {
    pub command: String,
    pub output: String,
}

/// Knobs passed to the parser by the caller.
#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    /// When `Some(patterns)`, any output whose final line matches one of the
    /// supplied prompt patterns will have that line stripped.  Pass `None` to
    /// disable the feature entirely.
    ///
    /// Patterns are matched with [`prompt_matches`]; see that function for the
    /// exact matching semantics.
    pub strip_trailing_prompt: Option<Vec<String>>,
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Parse `cast_content` (the full text of an asciinema v2 `.cast` file) and
/// return a list of `CommandInteraction` pairs.
///
/// # Errors
/// Returns an `Err(String)` if:
/// * The file is completely empty (no header line).
/// * Any event line contains malformed JSON.
/// * Any event line is not a JSON array of at least three elements.
pub fn extract_commands_from_cast(
    cast_content: &str,
    options: &ParseOptions,
) -> Result<Vec<CommandInteraction>, String> {
    let mut lines = cast_content.lines();

    // Line 1 is the header JSON object — we don't need its contents.
    let _header = lines.next().ok_or("Cast file is empty (no header line)")?;

    let mut interactions: Vec<CommandInteraction> = Vec::new();

    // ── State machine ────────────────────────────────────────────────────────
    //
    //  current_input   – raw keystrokes accumulated since the last flush
    //  current_output  – raw terminal output accumulated since Enter was hit
    //  is_reading_output – true once Enter has been detected in the input
    //
    let mut current_input = String::new();
    let mut current_output = String::new();
    let mut is_reading_output = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "{}" {
            continue; // Skip spurious blank lines or empty objects
        }

        let event: Value = serde_json::from_str(trimmed)
            .map_err(|e| format!("Failed to parse event JSON: {}", e))?;

        let event_arr = event
            .as_array()
            .ok_or("Event must be a JSON array [time, type, data]")?;

        if event_arr.len() < 3 {
            return Err(format!(
                "Event array too short: expected [time, type, data], got {} elements",
                event_arr.len()
            ));
        }

        let event_type = event_arr[1]
            .as_str()
            .ok_or("Event type (index 1) must be a string")?;

        let event_data = event_arr[2]
            .as_str()
            .ok_or("Event data (index 2) must be a string")?;

        match event_type {
            "i" => {
                // A fresh input event after Enter means the previous interaction ended.
                if is_reading_output {
                    flush_interaction(
                        &mut interactions,
                        &mut current_input,
                        &mut current_output,
                        &options.strip_trailing_prompt,
                    );
                    is_reading_output = false;
                }

                // Input event: user typed a character
                for ch in event_data.chars() {
                    match ch {
                        '\r' | '\n' => {
                            // User pressed Enter — switch to output-collection mode
                            is_reading_output = true;
                        }
                        _ => {
                            current_input.push(ch);
                        }
                    }
                }
            }
            "o" => {
                // Output event: shell returned text
                if is_reading_output {
                    current_output.push_str(event_data);
                }
            }
            _ => {
                // Ignore unknown event types (e.g., "m" for timing metadata)
            }
        }
    }

    // ── Flush the final in-progress pair ────────────────────────────────────
    // The file may end without a trailing "i" event, so we flush whatever we
    // were collecting.
    if is_reading_output {
        flush_interaction(
            &mut interactions,
            &mut current_input,
            &mut current_output,
            &options.strip_trailing_prompt,
        );
    }

    Ok(interactions)
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Finalize one pair and push it onto `interactions`.
///
/// Clears the mutable buffers in place regardless of whether a pair was
/// actually pushed (empty-command guard).
fn flush_interaction(
    interactions: &mut Vec<CommandInteraction>,
    raw_input: &mut String,
    raw_output: &mut String,
    prompt_patterns: &Option<Vec<String>>,
) {
    let command = clean_input(raw_input);
    if !command.is_empty() {
        let mut output = clean_output(raw_output);
        if let Some(patterns) = prompt_patterns {
            output = strip_trailing_prompt(output, patterns);
        }
        interactions.push(CommandInteraction { command, output });
    }
    raw_input.clear();
    raw_output.clear();
}

/// Reconstruct the intended command string from raw terminal input.
///
/// Handles:
/// * **DEL (`\x7f`) and BS (`\x08`)** — erase the preceding character, as a
///   real terminal would.
/// * **`\r` / `\n`** — stripped (they are the Enter key that ends the command).
fn clean_input(raw: &str) -> String {
    let stripped = strip_backspaces(raw);
    stripped
        .chars()
        .filter(|ch| *ch != '\r' && *ch != '\n')
        .collect::<String>()
        .trim()
        .to_string()
}

fn strip_backspaces(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if ch == '\x08' || ch == '\x7f' {
            out.pop();
        } else {
            out.push(ch);
        }
    }
    out
}

/// Strip the leading newline that most shells echo back immediately after the
/// user presses Enter (before the command output proper begins).
fn clean_output(raw: &str) -> String {
    // Prefer stripping \r\n first so we don't leave a dangling \r.
    if let Some(rest) = raw.strip_prefix("\r\n") {
        rest.to_string()
    } else if let Some(rest) = raw.strip_prefix('\n') {
        rest.to_string()
    } else {
        raw.to_string()
    }
}

// ── Trailing-prompt stripper ──────────────────────────────────────────────────

/// Remove a trailing prompt line from `output` if the last non-empty line
/// matches any of the supplied `patterns`.
///
/// # Matching semantics
/// A line "matches" a pattern when:
/// * The line **contains** the pattern as a substring, OR
/// * The line **ends with** one of the common prompt suffix characters
///   (`$`, `#`, `%`, `>`) after trimming — used as a heuristic when the
///   caller passes an empty-string pattern `""`.
///
/// The function preserves the original trailing whitespace / CRLF style of
/// the non-prompt portion of the output.
pub fn strip_trailing_prompt(output: String, patterns: &[String]) -> String {
    if patterns.is_empty() {
        return output;
    }

    // Split into lines, keeping the line endings so we can reassemble exactly.
    let lines: Vec<&str> = output.lines().collect();
    if lines.is_empty() {
        return output;
    }

    // Find the last non-empty line.
    let last_idx = lines.iter().rposition(|line| !line.trim().is_empty());
    let Some(idx) = last_idx else {
        return output; // All lines are empty.
    };

    let last_line = lines[idx];

    // Check if this line matches any pattern.
    if prompt_matches(last_line, patterns) {
        // Strip this line by rejoining everything *before* it.
        let mut result = lines[..idx].join("\n");
        if !result.is_empty() {
            result.push('\n');
        }
        result
    } else {
        output
    }
}

/// Returns `true` if `line` looks like a shell prompt given `patterns`.
///
/// * If any pattern is non-empty, a substring match is used (fast path for
///   known prompts such as `"user@host:~$"`).
/// * An empty pattern `""` activates the heuristic suffix check.
fn prompt_matches(line: &str, patterns: &[String]) -> bool {
    for pattern in patterns {
        if pattern.is_empty() {
            // Heuristic: check if line ends with a common shell prompt suffix.
            let trimmed = line.trim_end();
            if let Some(last_ch) = trimmed.chars().last()
                && matches!(last_ch, '$' | '#' | '%' | '>')
            {
                return true;
            }
        } else if line.contains(pattern) {
            return true;
        }
    }
    false
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cast_content() -> &'static str {
        r#"{"version": 2, "width": 80, "height": 24}
[0.1, "i", "echo hello"]
[0.2, "i", "\n"]
[0.3, "o", "\necho: hello\n"]
[1.0, "i", "ls"]
[1.1, "i", "\n"]
[1.2, "o", "\nfile1.txt\nfile2.txt\n"]
"#
    }

    #[test]
    fn test_empty_cast_file() {
        let result = extract_commands_from_cast("", &ParseOptions::default());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_basic_parsing() {
        let cast = sample_cast_content();
        let result = extract_commands_from_cast(cast, &ParseOptions::default()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].command, "echo hello");
        assert_eq!(result[1].command, "ls");
    }

    fn sample_cast_v2() -> &'static str {
        r#"{"version": 2, "width": 80, "height": 24}
[0.1, "i", "echo hello"]
[0.2, "i", "\n"]
[0.3, "o", "\necho: hello\n"]
[1.0, "i", "ls"]
[1.1, "i", "\n"]
[1.2, "o", "\nfile1.txt\nfile2.txt\n"]
"#
    }

    /// Asciinema v2 format test case provided by user
    fn sample_cast_v2_unicode() -> &'static str {
        r#"{"version": 2, "width": 100, "height": 50, "timestamp": 1509091818, "command": "/bin/bash", "env": {"TERM": "xterm-256color", "SHELL": "/bin/bash"}}
[0.000001, "o", "ż"]
[1.0, "o", "ółć"]
[2.3, "i", "\n"]
[5.600001, "r", "80x40"]
[10.5, "o", "\r\n"]
"#
    }

    /// Asciinema v3 format test case provided by user
    fn sample_cast_v3() -> &'static str {
        "{\"version\": 3, \"term\": {\"cols\": 100, \"rows\": 50, \"theme\": {\"fg\": \"#000000\", \"bg\": \"#ffffff\"}}, \"timestamp\": 1509091818, \"command\": \"/bin/bash\", \"env\": {\"TERM\": \"xterm-256color\", \"SHELL\": \"/bin/bash\"}}\n[0.000001, \"o\", \"ż\"]\n[1.0, \"o\", \"ółć\"]\n[0.3, \"i\", \"\\n\"]\n[1.600001, \"r\", \"80x40\"]\n[10.5, \"o\", \"\\r\\n\"]\n"
    }

    /// Complex v2 cast with multiple commands and output
    fn complex_cast_v2() -> &'static str {
        r#"{"version": 2, "width": 120, "height": 40, "timestamp": 1509091818}
[0.1, "i", "mkdir -p ~/test"]
[0.15, "i", "\n"]
[0.2, "o", "\n"]
[0.3, "i", "cd ~/test"]
[0.35, "i", "\n"]
[0.4, "o", "\n"]
[0.5, "i", "echo "]
[0.55, "i", "'hello world'"]
[0.6, "i", "\n"]
[0.7, "o", "\nhello world\n"]
[0.8, "i", "cat > file.txt"]
[0.85, "i", "\n"]
[0.9, "o", "\n"]
[1.0, "i", "test content"]
[1.05, "i", "\n"]
[1.1, "o", "\n"]
"#
    }

    #[test]
    fn test_basic_parsing_v2() {
        let cast = sample_cast_v2();
        let result = extract_commands_from_cast(cast, &ParseOptions::default()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].command, "echo hello");
        assert_eq!(result[0].output, "echo: hello\n");
        assert_eq!(result[1].command, "ls");
        assert_eq!(result[1].output, "file1.txt\nfile2.txt\n");
    }

    #[test]
    fn test_v2_unicode_cast() {
        let cast = sample_cast_v2_unicode();
        let result = extract_commands_from_cast(cast, &ParseOptions::default()).unwrap();
        // Output appears before any command input; this should parse without panic
        // and produce no interaction pair.
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_v3_cast_parsing() {
        let cast = sample_cast_v3();
        let result = extract_commands_from_cast(cast, &ParseOptions::default()).unwrap();
        // v3 sample has output before command input and should yield no pairs.
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_complex_v2_cast() {
        let cast = complex_cast_v2();
        let result = extract_commands_from_cast(cast, &ParseOptions::default()).unwrap();

        // Should extract 5 command/output pairs:
        // 1. "mkdir -p ~/test"
        // 2. "cd ~/test"
        // 3. "echo 'hello world'"
        // 4. "cat > file.txt"
        // 5. "test content"
        assert_eq!(result.len(), 5);
        assert_eq!(result[0].command, "mkdir -p ~/test");
        assert_eq!(result[1].command, "cd ~/test");
        assert_eq!(result[2].command, "echo 'hello world'");
        assert!(result[2].output.contains("hello world"));
        assert_eq!(result[3].command, "cat > file.txt");
        assert_eq!(result[4].command, "test content");
    }

    #[test]
    fn test_clean_input_with_backspace() {
        let raw = "helloo\x08\x08world";
        let cleaned = clean_input(raw);
        assert_eq!(cleaned, "hellworld");
    }

    #[test]
    fn test_clean_input_with_del() {
        let raw = "test\x7ftext";
        let cleaned = clean_input(raw);
        assert_eq!(cleaned, "testext");
    }

    #[test]
    fn test_clean_input_strips_enter() {
        let raw = "echo test\r\n";
        let cleaned = clean_input(raw);
        assert_eq!(cleaned, "echo test");
    }

    #[test]
    fn test_clean_input_strips_newline() {
        let raw = "ls -la\n";
        let cleaned = clean_input(raw);
        assert_eq!(cleaned, "ls -la");
    }

    #[test]
    fn test_clean_output_strips_leading_crlf() {
        let raw = "\r\noutput line 1\noutput line 2";
        let cleaned = clean_output(raw);
        assert_eq!(cleaned, "output line 1\noutput line 2");
    }

    #[test]
    fn test_clean_output_strips_leading_lf() {
        let raw = "\noutput line 1\noutput line 2";
        let cleaned = clean_output(raw);
        assert_eq!(cleaned, "output line 1\noutput line 2");
    }

    #[test]
    fn test_clean_output_preserves_content() {
        let raw = "no leading newline";
        let cleaned = clean_output(raw);
        assert_eq!(cleaned, "no leading newline");
    }

    #[test]
    fn test_strip_trailing_prompt_with_dollar() {
        let output = "result line\nuser@host:~$ ".to_string();
        let patterns = vec!["$".to_string()];
        let result = strip_trailing_prompt(output, &patterns);
        assert_eq!(result, "result line\n");
    }

    #[test]
    fn test_strip_trailing_prompt_with_hash() {
        let output = "result line\nroot# ".to_string();
        let patterns = vec!["#".to_string()];
        let result = strip_trailing_prompt(output, &patterns);
        assert_eq!(result, "result line\n");
    }

    #[test]
    fn test_strip_trailing_prompt_heuristic() {
        let output = "result\nuser@host:~$ ".to_string();
        let patterns = vec!["".to_string()]; // empty pattern triggers heuristic
        let result = strip_trailing_prompt(output, &patterns);
        assert_eq!(result, "result\n");
    }

    #[test]
    fn test_strip_trailing_prompt_heuristic_hash() {
        let output = "some command output\nroot# ".to_string();
        let patterns = vec!["".to_string()];
        let result = strip_trailing_prompt(output, &patterns);
        assert_eq!(result, "some command output\n");
    }

    #[test]
    fn test_strip_trailing_prompt_no_match() {
        let output = "result\njust text without prompt".to_string();
        let patterns = vec!["$".to_string()];
        let result = strip_trailing_prompt(output.clone(), &patterns);
        assert_eq!(result, output); // Should not strip
    }

    #[test]
    fn test_strip_trailing_prompt_empty_patterns() {
        let output = "result\nuser@host:~$ ".to_string();
        let patterns: Vec<String> = vec![];
        let result = strip_trailing_prompt(output.clone(), &patterns);
        assert_eq!(result, output); // Should not strip with empty patterns
    }

    #[test]
    fn test_prompt_matches_substring() {
        let patterns = vec!["user@host".to_string()];
        assert!(prompt_matches("user@host:~$ ", &patterns));
        assert!(prompt_matches("prefix user@host suffix", &patterns));
        assert!(!prompt_matches("other text", &patterns));
    }

    #[test]
    fn test_prompt_matches_empty_heuristic_dollar() {
        let patterns = vec!["".to_string()];
        assert!(prompt_matches("user@host:~$ ", &patterns));
        assert!(prompt_matches("$ ", &patterns));
    }

    #[test]
    fn test_prompt_matches_empty_heuristic_hash() {
        let patterns = vec!["".to_string()];
        assert!(prompt_matches("root# ", &patterns));
        assert!(prompt_matches("# ", &patterns));
    }

    #[test]
    fn test_prompt_matches_empty_heuristic_percent() {
        let patterns = vec!["".to_string()];
        assert!(prompt_matches("tcsh% ", &patterns));
        assert!(prompt_matches("% ", &patterns));
    }

    #[test]
    fn test_prompt_matches_empty_heuristic_angle() {
        let patterns = vec!["".to_string()];
        assert!(prompt_matches("python> ", &patterns));
        assert!(prompt_matches("> ", &patterns));
    }

    #[test]
    fn test_prompt_matches_empty_heuristic_no_match() {
        let patterns = vec!["".to_string()];
        assert!(!prompt_matches("just plain text", &patterns));
        assert!(!prompt_matches("no prompt here", &patterns));
    }

    #[test]
    fn test_malformed_json_event() {
        let cast = r#"{"version": 2}
[0.1, "i", "echo"]
invalid json here
"#;
        let result = extract_commands_from_cast(cast, &ParseOptions::default());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("JSON"));
    }

    #[test]
    fn test_empty_event_array() {
        let cast = r#"{"version": 2}
[]
"#;
        let result = extract_commands_from_cast(cast, &ParseOptions::default());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too short"));
    }

    #[test]
    fn test_event_with_unknown_type() {
        let cast = r#"{"version": 2}
[0.1, "x", "unknown event type"]
[0.2, "i", "echo hello"]
[0.3, "i", "\n"]
[0.4, "o", "\nhello\n"]
"#;
        let result = extract_commands_from_cast(cast, &ParseOptions::default()).unwrap();
        // Unknown event type "x" should be ignored, then we get echo hello
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "echo hello");
    }

    #[test]
    fn test_parse_options_default() {
        let opts = ParseOptions::default();
        assert!(opts.strip_trailing_prompt.is_none());
    }

    #[test]
    fn test_unicode_handling() {
        let cast = r#"{"version": 2}
[0.1, "i", "echo é à ü"]
[0.2, "i", "\n"]
[0.3, "o", "\né à ü\n"]
"#;
        let result = extract_commands_from_cast(cast, &ParseOptions::default()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "echo é à ü");
        assert!(result[0].output.contains("é"));
        assert!(result[0].output.contains("à"));
        assert!(result[0].output.contains("ü"));
    }

    #[test]
    fn test_empty_command_filtered() {
        let cast = r#"{"version": 2}
[0.1, "i", "\n"]
[0.2, "o", "\n"]
[0.3, "i", "echo test"]
[0.4, "i", "\n"]
[0.5, "o", "\ntest\n"]
"#;
        let result = extract_commands_from_cast(cast, &ParseOptions::default()).unwrap();
        // Empty command (just "\n") should be filtered out
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "echo test");
    }

    #[test]
    fn test_multiline_command() {
        let cast = r#"{"version": 2}
[0.1, "i", "for i in 1 2 3; do"]
[0.2, "i", "\n"]
[0.3, "o", "\n"]
[0.4, "i", "  echo $i"]
[0.5, "i", "\n"]
[0.6, "o", "\n1\n"]
"#;
        let result = extract_commands_from_cast(cast, &ParseOptions::default()).unwrap();
        // Each execute command (ending with \n) is separate
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].command, "for i in 1 2 3; do");
        assert_eq!(result[1].command, "echo $i");
    }

    #[test]
    fn test_backspace_stripped_from_command() {
        let cast = "{\"version\": 2}\n\
            [0.1, \"i\", \"ech\"]\n\
            [0.2, \"i\", \"\\u0008\\u0008\\u0008ls\"]\n\
            [0.3, \"i\", \"\\n\"]\n\
            [0.4, \"o\", \"\\nfile1\\n\"]\n";
        let result = extract_commands_from_cast(cast, &ParseOptions::default()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].command, "ls");
    }
}
