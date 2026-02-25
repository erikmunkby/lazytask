use super::*;
use chrono::{TimeZone, Utc};

#[test]
fn validates_title_char_limit() {
    let err = validate_title("This title is definitely way too long to be valid here").unwrap_err();
    assert!(
        err.to_string()
            .contains("title must be at most 50 characters")
    );
}

#[test]
fn normalizes_file_name() {
    let file_name = normalize_file_name("Hello, Rust World!").unwrap();
    assert_eq!(file_name, "hello-rust-world");
}

#[test]
fn validates_learning_line_count() {
    let err = parse_learning_lines("").unwrap_err();
    assert!(err.to_string().contains("1-3"));

    let ok = parse_learning_lines("only one").unwrap();
    assert_eq!(ok.len(), 1);

    let too_many = parse_learning_lines("a\nb\nc\nd").unwrap_err();
    assert!(too_many.to_string().contains("1-3"));
}

#[test]
fn normalizes_escaped_newlines() {
    let normalized = normalize_escaped_newlines("one\\ntwo\\r\\nthree");
    assert_eq!(normalized, "one\ntwo\nthree");
}

#[test]
fn parse_learning_lines_normalizes_escaped_newlines() {
    let lines = parse_learning_lines("one\\ntwo").unwrap();
    assert_eq!(lines, vec!["one".to_string(), "two".to_string()]);
}

#[test]
fn validates_discard_note_length_and_normalization() {
    let normalized = validate_discard_note("  why\\nnot  ").unwrap();
    assert_eq!(normalized, "why\nnot");

    let empty = validate_discard_note("   ").unwrap_err();
    assert!(empty.to_string().contains("1-120"));

    let too_long = validate_discard_note(&"x".repeat(121)).unwrap_err();
    assert!(too_long.to_string().contains("1-120"));
}

#[test]
fn formats_local_human_without_rfc3339_markers() {
    let ts = Utc.with_ymd_and_hms(2026, 2, 22, 6, 30, 45).unwrap();
    let formatted = format_local_human(ts);

    assert!(!formatted.contains('T'));
    assert!(!formatted.ends_with('Z'));
    assert!(formatted.contains(' '));
    assert!(formatted.contains(':'));
}

#[test]
fn timezone_label_uses_city_segment_from_tz_name() {
    assert_eq!(
        time::timezone_label_from_tz("Europe/Stockholm").unwrap(),
        "Stockholm"
    );
    assert_eq!(
        time::timezone_label_from_tz("America/Los_Angeles").unwrap(),
        "Los Angeles"
    );
}
