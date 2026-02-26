use super::DomainError;
use chrono::{DateTime, Local, Utc};
use std::fs;

/// Parses an RFC3339 timestamp into UTC.
pub fn parse_utc(ts: &str) -> Result<DateTime<Utc>, DomainError> {
    DateTime::parse_from_rfc3339(ts)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|err| DomainError::ParseError(format!("invalid timestamp '{ts}': {err}")))
}

/// Formats UTC timestamps in stable RFC3339 form with second precision.
pub fn format_utc(ts: DateTime<Utc>) -> String {
    ts.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Formats a UTC timestamp as local wall time plus a human timezone label.
pub fn format_local_human(ts: DateTime<Utc>) -> String {
    let local = ts.with_timezone(&Local);
    let tz_label = local_timezone_label();
    format!("{} {}", local.format("%Y-%m-%d %H:%M:%S"), tz_label)
}

/// Returns a compact relative label (`just now`, `5m ago`, `2h ago`, `3d ago`).
pub fn format_relative(then: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let delta = now.signed_duration_since(then);
    let secs = delta.num_seconds();

    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86_400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86_400)
    }
}

/// Chooses the best available local timezone label for display.
///
/// Preference order is `TZ`, `/etc/localtime` zone path, local abbreviation, then offset.
fn local_timezone_label() -> String {
    if let Ok(tz) = std::env::var("TZ")
        && let Some(label) = timezone_label_from_tz(&tz)
    {
        return label;
    }

    if let Ok(path) = fs::read_link("/etc/localtime")
        && let Some(path_str) = path.to_str()
        && let Some((_, tz)) = path_str.rsplit_once("zoneinfo/")
        && let Some(label) = timezone_label_from_tz(tz)
    {
        return label;
    }

    let abbr = Local::now().format("%Z").to_string();
    if !abbr.trim().is_empty() {
        return abbr;
    }

    Local::now().format("%:z").to_string()
}

/// Extracts a friendly timezone label from an IANA-style timezone string.
pub(crate) fn timezone_label_from_tz(tz: &str) -> Option<String> {
    let cleaned = tz.trim().trim_start_matches(':');
    if cleaned.is_empty() {
        return None;
    }

    let city = cleaned.rsplit('/').next().unwrap_or(cleaned);
    let label = city.replace('_', " ").trim().to_string();
    if label.is_empty() { None } else { Some(label) }
}
