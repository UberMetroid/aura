use std::time::Duration;

const THUMBNAIL_TIMEOUT: Duration = Duration::from_secs(1);

pub fn html_to_text(html: String) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            text.push(c);
        }
    }

    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .trim()
        .to_string()
}

pub fn strip_emojis(s: &str) -> String {
    s.chars()
        .filter(|&c| {
            let u = c as u32;
            !((0x1F300..=0x1F9FF).contains(&u)
                || (0x2600..=0x26FF).contains(&u)
                || (0x2700..=0x27BF).contains(&u)
                || (0x1F000..=0x1F0FF).contains(&u)
                || (0x1F1E0..=0x1F1FF).contains(&u)
                || (0x1F900..=0x1F9FF).contains(&u)
                || (0x1FA00..=0x1FAFF).contains(&u))
        })
        .collect()
}

pub fn process_snippet(snippet: &str) -> String {
    let clean = html_to_text(snippet.to_string());
    let stripped = strip_emojis(&clean);

    if stripped.starts_with("[data:image") {
        return String::new();
    }

    stripped.trim().to_string()
}

pub async fn fetch_thumbnail_as_data_url(
    url: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let response = client.get(url).timeout(THUMBNAIL_TIMEOUT).send().await?;
    if !response.status().is_success() {
        return Err("Thumbnail fetch failed".into());
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let bytes = response.bytes().await?;
    use base64::{Engine as _, engine::general_purpose};
    let b64 = general_purpose::STANDARD.encode(&bytes);

    Ok(format!("data:{};base64,{}", content_type, b64))
}

/// Emit a structured error to the tracing stack at `ERROR` level.
///
/// Previously this function wrote to `aura-errors.log` at the process
/// working directory, which is a deployment-dependent location (and inside
/// the Nix container it's read-only). The tracing stack initialized in
/// `main.rs` already routes errors to `error.log` (when `LOG_DIR` is set)
/// and to stderr, so this function is now a thin wrapper around
/// `tracing::error!`. Existing call sites continue to compile because the
/// signature is unchanged.
pub fn log_error_to_file(context: &str, error_msg: &str) {
    tracing::error!(target: "upstream", context = %context, error = %error_msg);
}

pub fn map_reqwest_error(e: reqwest::Error, url: &str) -> String {
    let msg = e.to_string();
    if e.is_timeout() {
        format!(
            "Timeout Error: Request to search service at '{}' timed out after 10 seconds. Check if SearXNG is overloaded or slow.",
            url
        )
    } else if e.is_connect() {
        if msg.contains("dns") || msg.contains("resolve") {
            format!(
                "DNS Error: Could not resolve hostname for search service at '{}'. Verify network DNS settings and internet connection.",
                url
            )
        } else {
            format!(
                "Connection Refused: Could not connect to search service at '{}'. Check if SearXNG is running, listening on port 8888, or blocked by a firewall.",
                url
            )
        }
    } else if e.is_decode() {
        format!(
            "JSON Decode Error: Received invalid response body format from search service at '{}'. details: {}",
            url, msg
        )
    } else {
        format!(
            "HTTP Connection Error: Failed to request search service at '{}'. details: {}",
            url, msg
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Golden output for the HTML→text converter: tags are stripped and
    /// the common named entities are decoded.
    #[test]
    fn html_to_text_strips_tags_and_decodes_entities() {
        let html = "<p>Hello <b>world</b> &amp; goodbye &lt;tag&gt; &#39;quoted&#39;</p>";
        let text = html_to_text(html.to_string());
        assert_eq!(text, "Hello world & goodbye <tag> 'quoted'");
    }

    /// Tags that span newlines still strip correctly. The implementation
    /// walks character-by-character so any character outside `<…>` is kept.
    #[test]
    fn html_to_text_preserves_whitespace_outside_tags() {
        let html = "<div>line1\nline2</div>";
        let text = html_to_text(html.to_string());
        assert_eq!(text, "line1\nline2");
    }

    /// Entities are decoded but unknown entities pass through verbatim.
    /// This is intentional: a fully-conformant HTML entity decoder is out
    /// of scope for this utility.
    #[test]
    fn html_to_text_passes_through_unknown_entities() {
        let html = "a &unknown; b";
        let text = html_to_text(html.to_string());
        assert_eq!(text, "a &unknown; b");
    }

    /// `process_snippet` returns empty string when the input starts with
    /// `[data:image` (a base64 thumbnail marker). This is how Aura filters
    /// out embedded thumbnail data from search snippets.
    #[test]
    fn process_snippet_filters_data_image_prefix() {
        let snippet = "[data:image/png;base64,iVBORw0KGgo=]";
        assert_eq!(process_snippet(snippet), "");
    }

    /// `process_snippet` strips emojis from the snippet body.
    #[test]
    fn process_snippet_strips_emojis() {
        let snippet = "Hello \u{1F600} world \u{1F4A9}";
        let result = process_snippet(snippet);
        assert!(!result.contains('\u{1F600}'));
        assert!(!result.contains('\u{1F4A9}'));
        assert!(result.contains("Hello"));
        assert!(result.contains("world"));
    }
}
