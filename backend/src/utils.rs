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
    use base64::{engine::general_purpose, Engine as _};
    let b64 = general_purpose::STANDARD.encode(&bytes);

    Ok(format!("data:{};base64,{}", content_type, b64))
}

pub fn log_error_to_file(context: &str, error_msg: &str) {
    use std::fs::OpenOptions;
    use std::io::Write;
    let time_str = chrono::Utc::now().to_rfc3339();
    let log_line = format!("[{}] {}: {}\n", time_str, context, error_msg);
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("rustsearch-errors.log")
    {
        let _ = file.write_all(log_line.as_bytes());
    }
}

pub fn map_reqwest_error(e: reqwest::Error, url: &str) -> String {
    let msg = e.to_string();
    if e.is_timeout() {
        format!("Timeout Error: Request to search service at '{}' timed out after 10 seconds. Check if SearXNG is overloaded or slow.", url)
    } else if e.is_connect() {
        if msg.contains("dns") || msg.contains("resolve") {
            format!("DNS Error: Could not resolve hostname for search service at '{}'. Verify network DNS settings and internet connection.", url)
        } else {
            format!("Connection Refused: Could not connect to search service at '{}'. Check if SearXNG is running, listening on port 8888, or blocked by a firewall.", url)
        }
    } else if e.is_decode() {
        format!("JSON Decode Error: Received invalid response body format from search service at '{}'. details: {}", url, msg)
    } else {
        format!(
            "HTTP Connection Error: Failed to request search service at '{}'. details: {}",
            url, msg
        )
    }
}
