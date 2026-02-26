use super::DomainError;

/// Converts escaped newline sequences into actual newlines.
///
/// This keeps CLI/JSON inputs consistent with interactively entered multiline text.
pub fn normalize_escaped_newlines(input: &str) -> String {
    input.replace("\\r\\n", "\n").replace("\\n", "\n")
}

/// Parses a learning payload into 1-3 non-empty bullet lines.
///
/// Input supports escaped newlines (`\n`, `\r\n`) so AI callers can pass
/// learnings in JSON-safe strings.
pub fn parse_learning_lines(input: &str) -> Result<Vec<String>, DomainError> {
    let normalized = normalize_escaped_newlines(input);
    let lines: Vec<String> = normalized
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    if !(1..=3).contains(&lines.len()) {
        return Err(DomainError::ValidationError(
            "learning must contain 1-3 non-empty lines".to_string(),
        ));
    }

    Ok(lines)
}
