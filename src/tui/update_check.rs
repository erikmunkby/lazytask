use serde::Deserialize;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;
use ureq::Agent;

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
}

/// Spawns a background thread that checks GitHub for a newer release.
/// Returns a receiver that yields the latest version string if an update is available.
pub fn spawn_update_check() -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = check_latest_version(&tx);
    });
    rx
}

fn check_latest_version(tx: &mpsc::Sender<String>) -> Result<(), Box<dyn std::error::Error>> {
    let agent: Agent = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build()
        .into();

    let release: GithubRelease = agent
        .get("https://api.github.com/repos/erikmunkby/lazytask/releases/latest")
        .header("User-Agent", "lazytask")
        .call()?
        .body_mut()
        .read_json()?;

    let remote = strip_tag_prefix(&release.tag_name);
    let current = env!("CARGO_PKG_VERSION");

    if is_newer(remote, current) {
        let _ = tx.send(remote.to_string());
    }

    Ok(())
}

fn strip_tag_prefix(tag: &str) -> &str {
    tag.find(|c: char| c.is_ascii_digit())
        .map(|i| &tag[i..])
        .unwrap_or(tag)
}

/// Returns true if `remote` is a strictly newer semver than `current`.
fn is_newer(remote: &str, current: &str) -> bool {
    let parse = |s: &str| -> Option<Vec<u64>> { s.split('.').map(|p| p.parse().ok()).collect() };
    let Some(r) = parse(remote) else { return false };
    let Some(c) = parse(current) else {
        return false;
    };
    r > c
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn newer_patch() {
        assert!(is_newer("0.2.1", "0.2.0"));
    }

    #[test]
    fn newer_minor() {
        assert!(is_newer("0.3.0", "0.2.0"));
    }

    #[test]
    fn newer_major() {
        assert!(is_newer("1.0.0", "0.2.0"));
    }

    #[test]
    fn same_version() {
        assert!(!is_newer("0.2.0", "0.2.0"));
    }

    #[test]
    fn older_version() {
        assert!(!is_newer("0.1.0", "0.2.0"));
    }

    #[test]
    fn malformed_remote() {
        assert!(!is_newer("abc", "0.2.0"));
    }

    #[test]
    fn malformed_current() {
        assert!(!is_newer("0.3.0", "abc"));
    }

    #[test]
    fn strip_lazytask_v_prefix() {
        assert_eq!(super::strip_tag_prefix("lazytask-v0.4.0"), "0.4.0");
    }

    #[test]
    fn strip_v_prefix() {
        assert_eq!(super::strip_tag_prefix("v1.2.3"), "1.2.3");
    }

    #[test]
    fn strip_no_prefix() {
        assert_eq!(super::strip_tag_prefix("0.4.0"), "0.4.0");
    }

    #[ignore] // repo is currently private
    #[test]
    fn integration_github_api() {
        let (tx, rx) = std::sync::mpsc::channel();
        let result = check_latest_version(&tx);
        assert!(result.is_ok());
        // Version comparison depends on actual latest release
        let _ = rx.try_recv();
    }

    fn check_latest_version(
        tx: &std::sync::mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        super::check_latest_version(tx)
    }
}
