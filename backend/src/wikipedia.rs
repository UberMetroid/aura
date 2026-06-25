use crate::utils::{html_to_text, log_error_to_file, process_snippet};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tracing::warn;

const MAX_RETRIES: usize = 3;
const BASE_RETRY_DELAY: Duration = Duration::from_secs(1);

#[derive(Deserialize, Debug)]
struct WikiSearchItem {
    pub title: String,
    pub snippet: String,
}

#[derive(Deserialize, Debug)]
struct WikiQuery {
    pub search: Vec<WikiSearchItem>,
}

#[derive(Deserialize, Debug)]
struct WikiResponse {
    pub query: WikiQuery,
}

/// Performs a query to Wikipedia's public search API
pub async fn search_wikipedia(
    query: &str,
    limit: usize,
) -> Result<Vec<(String, String, String)>, String> {
    let client = Client::new();
    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&format=json",
        percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC)
    );

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            let delay = BASE_RETRY_DELAY * 2u32.pow((attempt - 1) as u32);
            warn!(
                "Wikipedia retry in {}ms (attempt {}/{})",
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
                    let err_msg = format!("Wikipedia connection failed: {}", e);
                    log_error_to_file("Wikipedia Query Connection", &err_msg);
                    return Err(err_msg);
                }
                continue;
            }
        };

        if !response.status().is_success() {
            if attempt == MAX_RETRIES {
                let err_msg = format!("Wikipedia returned error status: {}", response.status());
                log_error_to_file("Wikipedia Query Status", &err_msg);
                return Err(err_msg);
            }
            continue;
        }

        let data = match response.json::<WikiResponse>().await {
            Ok(d) => d,
            Err(e) => {
                let err_msg = format!("Wikipedia JSON parse failed: {}", e);
                log_error_to_file("Wikipedia Query JSON Decode", &err_msg);
                return Err(err_msg);
            }
        };

        let mut results = Vec::new();
        for item in data.query.search.into_iter().take(limit) {
            let title = html_to_text(item.title.clone());
            let snippet = process_snippet(&item.snippet);
            let encoded_title = percent_encoding::utf8_percent_encode(
                &item.title.replace(' ', "_"),
                percent_encoding::NON_ALPHANUMERIC,
            )
            .to_string();
            let page_url = format!("https://en.wikipedia.org/wiki/{}", encoded_title);

            if !title.is_empty() && !snippet.is_empty() {
                results.push((title, snippet, page_url));
            }
        }

        return Ok(results);
    }

    Err("Wikipedia search failed after all retries".to_string())
}
