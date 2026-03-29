use super::errors::ServiceError;
use chrono::Local;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug)]
pub enum PasteResult {
    Image { markdown: String },
    Text(String),
    Empty,
}

/// Reads the system clipboard and returns an image (saved as PNG) or text.
///
/// Images are saved under `.tasks/assets/` and the returned markdown uses
/// `../assets/<file>` so it resolves correctly from any bucket subdir.
pub fn paste_from_clipboard(tasks_root: &Path) -> Result<PasteResult, ServiceError> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| ServiceError::Io(format!("clipboard: {e}")))?;

    // Try image first
    if let Ok(img_data) = clipboard.get_image() {
        let filename = generate_filename();
        let assets_dir = tasks_root.join("assets");
        fs::create_dir_all(&assets_dir)
            .map_err(|e| ServiceError::Io(format!("create assets dir: {e}")))?;

        save_png(&assets_dir.join(&filename), &img_data)?;

        return Ok(PasteResult::Image {
            markdown: format!("![image](../assets/{filename})"),
        });
    }

    // Fall back to text
    if let Ok(text) = clipboard.get_text()
        && !text.is_empty()
    {
        return Ok(PasteResult::Text(text));
    }

    Ok(PasteResult::Empty)
}

fn generate_filename() -> String {
    let now = Local::now();
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let suffix = format!("{:04x}", nanos & 0xFFFF);
    now.format(&format!("img-%Y%m%d-%H%M%S-{suffix}.png"))
        .to_string()
}

/// Deletes asset files referenced in task details via markdown image links.
///
/// Only deletes files that actually reside inside `.tasks/assets/` after
/// canonicalization, preventing path-traversal attacks.
pub fn cleanup_task_assets(tasks_root: &Path, details: &str) {
    let assets_dir = tasks_root.join("assets");
    if !assets_dir.is_dir() {
        return;
    }

    for filename in extract_asset_filenames(details) {
        let path = assets_dir.join(&filename);
        // Canonicalize to prevent traversal (e.g. `../../README.md`)
        if let Ok(canonical) = path.canonicalize()
            && let Ok(canonical_assets) = assets_dir.canonicalize()
            && canonical.starts_with(&canonical_assets)
            && canonical.is_file()
        {
            let _ = fs::remove_file(canonical);
        }
    }
}

/// Extracts bare filenames from `![...](../assets/<filename>)` markdown image refs.
fn extract_asset_filenames(details: &str) -> Vec<String> {
    regex::Regex::new(r"!\[[^\]]*\]\(\.\./assets/([^)/]+)\)")
        .unwrap()
        .captures_iter(details)
        .map(|cap| cap[1].to_string())
        .collect()
}

fn save_png(path: &Path, img_data: &arboard::ImageData) -> Result<(), ServiceError> {
    let rgba = image::RgbaImage::from_raw(
        img_data.width as u32,
        img_data.height as u32,
        img_data.bytes.to_vec(),
    )
    .ok_or_else(|| ServiceError::Io("invalid image data from clipboard".to_string()))?;

    rgba.save(path)
        .map_err(|e| ServiceError::Io(format!("save png: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_asset_filenames_from_markdown() {
        let details = "some text ![img](../assets/img-20260328-120000-ab12.png) more text";
        assert_eq!(
            extract_asset_filenames(details),
            vec!["img-20260328-120000-ab12.png"]
        );
    }

    #[test]
    fn extracts_multiple_asset_references() {
        let details = "![a](../assets/one.png)\n![b](../assets/two.png)";
        assert_eq!(extract_asset_filenames(details), vec!["one.png", "two.png"]);
    }

    #[test]
    fn ignores_non_asset_image_links() {
        let details = "![x](https://example.com/img.png) ![y](../assets/ok.png)";
        assert_eq!(extract_asset_filenames(details), vec!["ok.png"]);
    }

    #[test]
    fn ignores_traversal_in_filename() {
        let details = "![x](../assets/../../etc/passwd)";
        // Regex requires no slashes in filename, so this won't match
        assert!(extract_asset_filenames(details).is_empty());
    }

    #[test]
    fn ignores_old_format_without_dotdot() {
        let details = "![x](assets/img.png)";
        assert!(extract_asset_filenames(details).is_empty());
    }
}
