use super::DomainError;
use regex::Regex;
use std::sync::LazyLock;

/// Normalizes a title into a stable markdown file stem.
///
/// Non-alphanumeric runs collapse to `-`, output is lowercase, and leading/trailing
/// separators are removed.
pub fn normalize_file_name(title: &str) -> Result<String, DomainError> {
    static FILE_NAME_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^a-z0-9]+").unwrap());

    let lower = title.trim().to_lowercase();
    let collapsed = FILE_NAME_RE.replace_all(&lower, "-");
    let normalized = collapsed.trim_matches('-').to_string();
    if normalized.is_empty() {
        return Err(DomainError::ValidationError(
            "title must contain letters or numbers".to_string(),
        ));
    }

    Ok(normalized)
}
