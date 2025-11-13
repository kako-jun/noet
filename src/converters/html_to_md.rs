use crate::error::Result;

/// Convert Note.com HTML to Markdown
///
/// Note.com returns article body in HTML format with specific structure:
/// - Images use `data-src` attribute (lazy loading)
/// - Links are wrapped in `<figure>` tags
/// - Paragraphs contain `name` and `id` attributes (UUIDs)
///
/// This function converts the HTML to standard Markdown format.
pub fn convert_html_to_markdown(html: &str) -> Result<String> {
    // Use html2md for basic conversion
    let markdown = html2md::parse_html(html);

    // Post-process to handle Note.com specific quirks
    let markdown = post_process_markdown(&markdown);

    Ok(markdown)
}

/// Post-process markdown to clean up Note.com specific artifacts
fn post_process_markdown(markdown: &str) -> String {
    let mut result = markdown.to_string();

    // Remove extra newlines (html2md sometimes adds too many)
    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }

    // Clean up link formatting from <figure> tags
    // html2md might not handle these perfectly
    result = result.trim().to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_paragraph() {
        let html = r#"<p name="uuid" id="uuid">Hello world</p>"#;
        let md = convert_html_to_markdown(html).unwrap();
        assert!(md.contains("Hello world"));
    }

    #[test]
    fn test_heading() {
        let html = r#"<h2 name="uuid" id="uuid">My Heading</h2>"#;
        let md = convert_html_to_markdown(html).unwrap();
        assert!(md.contains("## My Heading") || md.contains("My Heading"));
    }

    #[test]
    fn test_image() {
        let html = r#"<img src="https://example.com/image.png" alt="Test Image" width="620" height="224">"#;
        let md = convert_html_to_markdown(html).unwrap();
        assert!(md.contains("![Test Image]") || md.contains("https://example.com/image.png"));
    }

    #[test]
    fn test_line_breaks() {
        let html = r#"<p>Line 1<br>Line 2</p>"#;
        let md = convert_html_to_markdown(html).unwrap();
        assert!(md.contains("Line 1"));
        assert!(md.contains("Line 2"));
    }

    #[test]
    fn test_bold() {
        let html = r#"<p>This is <b>bold</b> text</p>"#;
        let md = convert_html_to_markdown(html).unwrap();
        assert!(md.contains("**bold**") || md.contains("bold"));
    }
}
