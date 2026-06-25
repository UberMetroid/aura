use crate::utils::{html_to_text, log_error_to_file, process_snippet};
use regex::Regex;
use reqwest::Client;
use std::time::Duration;
use tracing::warn;

const MAX_RETRIES: usize = 3;
const BASE_RETRY_DELAY: Duration = Duration::from_secs(1);

/// Performs a query to Grokipedia's search endpoint and parses results
pub async fn search_grokipedia(
    query: &str,
    limit: usize,
) -> Result<Vec<(String, String, String)>, String> {
    let client = Client::new();
    let url = format!(
        "https://grokipedia.com/search?q={}",
        percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC)
    );

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            let delay = BASE_RETRY_DELAY * 2u32.pow((attempt - 1) as u32);
            warn!(
                "Grokipedia retry in {}ms (attempt {}/{})",
                delay.as_millis(),
                attempt,
                MAX_RETRIES
            );
            tokio::time::sleep(delay).await;
        }

        let response = match client.get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                if attempt == MAX_RETRIES {
                    let err_msg = format!("Grokipedia connection failed: {}", e);
                    log_error_to_file("Grokipedia Query Connection", &err_msg);
                    return Err(err_msg);
                }
                continue;
            }
        };

        if !response.status().is_success() {
            if attempt == MAX_RETRIES {
                let err_msg = format!("Grokipedia returned error status: {}", response.status());
                log_error_to_file("Grokipedia Query Status", &err_msg);
                return Err(err_msg);
            }
            continue;
        }

        let html = match response.text().await {
            Ok(t) => t,
            Err(e) => {
                let err_msg = format!("Grokipedia response read failed: {}", e);
                log_error_to_file("Grokipedia Query Read", &err_msg);
                return Err(err_msg);
            }
        };

        let parsed = parse_grokipedia_html(&html, limit);
        return Ok(parsed);
    }

    Err("Grokipedia search failed after all retries".to_string())
}

fn parse_grokipedia_html(html: &str, limit: usize) -> Vec<(String, String, String)> {
    let mut results = Vec::new();

    // Regex to match search result link tags
    let re_item = Regex::new(
        r#"(?s)<a[^>]*href="(/page/[^"]+)"[^>]*data-search-snippet="([^"]+)"[^>]*>.*?<span[^>]*class="[^"]*text-fg-primary[^"]*"[^>]*>\s*(.*?)\s*</span>"#
    ).ok();

    if let Some(re_item) = re_item {
        for caps in re_item.captures_iter(html) {
            let path = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let snippet_raw = caps
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let title_raw = caps
                .get(3)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let title = html_to_text(title_raw);
            let snippet = process_snippet(&snippet_raw);
            let page_url = format!("https://grokipedia.com{}", path);

            if !title.is_empty() && !snippet.is_empty() && !path.is_empty() {
                results.push((title, snippet, page_url));
                if results.len() >= limit {
                    break;
                }
            }
        }
    }

    results
}
