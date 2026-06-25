use crate::utils::{html_to_text, log_error_to_file, process_snippet};
use regex::Regex;
use reqwest::Client;
use std::time::Duration;
use tracing::warn;

const MAX_RETRIES: usize = 3;
const BASE_RETRY_DELAY: Duration = Duration::from_secs(1);

/// Performs a direct query to DuckDuckGo's HTML endpoint using a POST request
pub async fn search_duckduckgo(
    query: &str,
    limit: usize,
) -> Result<Vec<(String, String, String)>, String> {
    let client = Client::new();
    let url = "https://html.duckduckgo.com/html";

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            let delay = BASE_RETRY_DELAY * 2u32.pow((attempt - 1) as u32);
            warn!(
                "DuckDuckGo retry in {}ms (attempt {}/{})",
                delay.as_millis(),
                attempt,
                MAX_RETRIES
            );
            tokio::time::sleep(delay).await;
        }

        let mut form_data = std::collections::HashMap::new();
        form_data.insert("q", query);
        form_data.insert("b", "");
        form_data.insert("kl", "wt-wt");

        let response = match client.post(url)
            .form(&form_data)
            .header("Referer", "https://html.duckduckgo.com/")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                if attempt == MAX_RETRIES {
                    let err_msg = format!("DuckDuckGo connection failed: {}", e);
                    log_error_to_file("DuckDuckGo Query Connection", &err_msg);
                    return Err(err_msg);
                }
                continue;
            }
        };

        if !response.status().is_success() {
            if attempt == MAX_RETRIES {
                let err_msg = format!("DuckDuckGo returned error status: {}", response.status());
                log_error_to_file("DuckDuckGo Query Status", &err_msg);
                return Err(err_msg);
            }
            continue;
        }

        let html = match response.text().await {
            Ok(t) => t,
            Err(e) => {
                let err_msg = format!("DuckDuckGo response read failed: {}", e);
                log_error_to_file("DuckDuckGo Query Read", &err_msg);
                return Err(err_msg);
            }
        };

        // Parse DDG HTML page
        let parsed = parse_ddg_html(&html, limit);
        return Ok(parsed);
    }

    Err("DuckDuckGo search failed after all retries".to_string())
}

fn parse_ddg_html(html: &str, limit: usize) -> Vec<(String, String, String)> {
    let mut results = Vec::new();

    // Split HTML into result body blocks
    let blocks: Vec<&str> = html.split("class=\"result results_links").collect();
    if blocks.len() <= 1 {
        return results;
    }

    let re_title = Regex::new(r#"class="result__a"[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#).ok();
    let re_snippet = Regex::new(r#"class="result__snippet"[^>]*>(.*?)</a>"#).ok();

    if let (Some(re_title), Some(re_snippet)) = (re_title, re_snippet) {
        // Skip index 0 which contains headers
        for block in blocks.iter().skip(1) {
            let url_and_title = if let Some(caps) = re_title.captures(block) {
                let url = caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let title = caps
                    .get(2)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                Some((url, title))
            } else {
                None
            };

            let snippet = if let Some(caps) = re_snippet.captures(block) {
                caps.get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default()
            } else {
                String::new()
            };

            if let Some((url, title_raw)) = url_and_title {
                let title = html_to_text(title_raw);
                let snippet_clean = process_snippet(&snippet);

                if !title.is_empty() && !snippet_clean.is_empty() && !url.is_empty() {
                    results.push((title, snippet_clean, url));
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }
    }

    results
}
