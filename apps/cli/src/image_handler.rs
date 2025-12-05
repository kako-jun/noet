//! Image handling for articles

use crate::error::{NoetError, Result};
use base64::{engine::general_purpose, Engine as _};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Image data to be uploaded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub local_path: String,
    pub filename: String,
    pub caption: String,
    pub mime_type: String,
    pub data: String, // base64 encoded
}

/// Parsed image reference from Markdown
#[derive(Debug, Clone)]
pub struct ImageReference {
    pub caption: String,
    pub path: String,
}

/// Extract image references from Markdown content
/// Matches: ![caption](path)
pub fn extract_image_references(markdown: &str) -> Vec<ImageReference> {
    let re = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();

    re.captures_iter(markdown)
        .map(|cap| ImageReference {
            caption: cap[1].to_string(),
            path: cap[2].to_string(),
        })
        .collect()
}

/// Read image file and convert to base64
pub fn read_image_as_base64(path: &Path) -> Result<(String, String)> {
    // Read file as bytes
    let bytes = fs::read(path)?;

    // Determine MIME type from extension
    let mime_type = match path.extension().and_then(|s| s.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => {
            return Err(NoetError::InvalidInput(format!(
                "Unsupported image format: {:?}",
                path.extension()
            )))
        }
    };

    // Encode to base64
    let base64_data = general_purpose::STANDARD.encode(&bytes);

    Ok((mime_type.to_string(), base64_data))
}

/// Process images from Markdown file
/// Returns list of ImageData for upload
pub fn process_images(markdown_path: &Path, markdown: &str) -> Result<Vec<ImageData>> {
    let references = extract_image_references(markdown);
    let mut images = Vec::new();

    let base_dir = markdown_path
        .parent()
        .ok_or_else(|| NoetError::InvalidInput("Cannot determine base directory".to_string()))?;

    for ref_data in references {
        // Skip URLs (http://, https://)
        if ref_data.path.starts_with("http://") || ref_data.path.starts_with("https://") {
            continue;
        }

        // Skip already uploaded images (assets.st-note.com)
        if ref_data.path.contains("st-note.com") {
            continue;
        }

        // Resolve relative path
        let image_path = if Path::new(&ref_data.path).is_absolute() {
            PathBuf::from(&ref_data.path)
        } else {
            base_dir.join(&ref_data.path)
        };

        // Check if file exists
        if !image_path.exists() {
            eprintln!(
                "Warning: Image file not found: {} (referenced as {})",
                image_path.display(),
                ref_data.path
            );
            continue;
        }

        // Read and encode image
        let (mime_type, base64_data) = read_image_as_base64(&image_path)?;

        let filename = image_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| NoetError::InvalidInput("Invalid filename".to_string()))?
            .to_string();

        images.push(ImageData {
            local_path: ref_data.path.clone(),
            filename,
            caption: ref_data.caption.clone(),
            mime_type,
            data: base64_data,
        });
    }

    Ok(images)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_image_references() {
        let markdown = r#"
# Test Article

This is a test.

![A nice cat](./images/cat.jpg)

Some text.

![Another image](../photos/dog.png)

![No caption](image.gif)
        "#;

        let refs = extract_image_references(markdown);

        assert_eq!(refs.len(), 3);
        assert_eq!(refs[0].caption, "A nice cat");
        assert_eq!(refs[0].path, "./images/cat.jpg");
        assert_eq!(refs[1].caption, "Another image");
        assert_eq!(refs[1].path, "../photos/dog.png");
        assert_eq!(refs[2].caption, "No caption");
        assert_eq!(refs[2].path, "image.gif");
    }

    #[test]
    fn test_skip_urls() {
        let markdown = r#"
![Remote](https://example.com/image.jpg)
![Already uploaded](https://assets.st-note.com/img/123.webp)
![Local](./local.jpg)
        "#;

        let refs = extract_image_references(markdown);
        assert_eq!(refs.len(), 3);

        // process_images should skip URLs
        // (This would need a temporary file to test fully)
    }
}
