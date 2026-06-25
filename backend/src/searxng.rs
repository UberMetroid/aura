use crate::circuit_breaker::CircuitBreaker;
use crate::utils::{
    fetch_thumbnail_as_data_url, html_to_text, log_error_to_file, map_reqwest_error,
    process_snippet,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::warn;

const MAX_RETRIES: usize = 3;
const BASE_RETRY_DELAY: Duration = Duration::from_secs(1);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearxngSearchResult {
    pub title: Option<String>,
    pub url: String,
    pub content: Option<String>,
    pub category: Option<String>,
    pub template: Option<String>,
    pub engine: Option<String>,
    pub img_src: Option<String>,
    pub iframe_src: Option<String>,
    pub thumbnail: Option<String>,
    pub thumbnail_src: Option<String>,
}

#[derive(Deserialize, Debug)]
struct SearxngSearchResponse {
    pub results: Option<Vec<SearxngSearchResult>>,
}

pub async fn search_text_searxng(
    searxng_base_url: &str,
    circuit_breaker: &CircuitBreaker,
    query: &str,
    limit: usize,
) -> Result<Vec<(String, String, String)>, String> {
    let raw_results = perform_search(searxng_base_url, circuit_breaker, query, "text").await?;
    let deduped = deduplicate_results(raw_results);

    let processed: Vec<(String, String, String)> = deduped
        .into_iter()
        .filter_map(|r| {
            let content = r.content?;
            let title = html_to_text(r.title.unwrap_or_default());
            let snippet = process_snippet(&content);
            if title.is_empty() || snippet.is_empty() {
                return None;
            }
            Some((title, snippet, r.url))
        })
        .take(limit)
        .collect();

    Ok(processed)
}

pub async fn search_images_searxng(
    searxng_base_url: &str,
    circuit_breaker: &CircuitBreaker,
    query: &str,
    limit: usize,
) -> Result<Vec<(String, String, String, String)>, String> {
    let raw_results = perform_search(searxng_base_url, circuit_breaker, query, "images").await?;
    let deduped = deduplicate_results(raw_results);

    let processed: Vec<(String, String, String, String)> = deduped
        .into_iter()
        .filter_map(|r| {
            let is_vid = r.category.as_deref() == Some("videos");
            let thumbnail_source = if is_vid {
                r.thumbnail.or(r.thumbnail_src)
            } else {
                r.thumbnail_src.or(r.thumbnail)
            }?;
            let source_url = if is_vid {
                r.iframe_src.or_else(|| Some(r.url.clone()))
            } else {
                r.img_src
            }?;
            let title = html_to_text(r.title.unwrap_or_default());
            Some((title, r.url, thumbnail_source, source_url))
        })
        .collect();

    if processed.is_empty() {
        return Ok(Vec::new());
    }

    let mut jobs = Vec::new();
    for (title, url, thumbnail_source, source_url) in processed {
        jobs.push(tokio::spawn(async move {
            match fetch_thumbnail_as_data_url(&thumbnail_source).await {
                Ok(data_url) => Some((title, url, data_url, source_url)),
                Err(_) => None,
            }
        }));
    }

    let mut results = Vec::new();
    for job in jobs {
        if let Ok(Some(item)) = job.await {
            results.push(item);
            if results.len() >= limit {
                break;
            }
        }
    }

    Ok(results)
}

pub async fn check_searxng_health(
    searxng_base_url: &str,
    circuit_breaker: &CircuitBreaker,
) -> bool {
    let client = reqwest::Client::new();
    let health_url = format!("{}/healthz", searxng_base_url.trim_end_matches('/'));
    match client
        .get(&health_url)
        .timeout(Duration::from_secs(2))
        .send()
        .await
    {
        Ok(resp) => {
            if let Ok(text) = resp.text().await {
                if text.trim() == "OK" {
                    return perform_search(searxng_base_url, circuit_breaker, "test", "text")
                        .await
                        .is_ok();
                }
            }
            false
        }
        Err(_) => false,
    }
}

async fn perform_search(
    searxng_base_url: &str,
    circuit_breaker: &CircuitBreaker,
    query: &str,
    search_type: &str,
) -> Result<Vec<SearxngSearchResult>, String> {
    if let Err(e) = circuit_breaker.check_allow() {
        return Err(e.to_string());
    }

    let client = reqwest::Client::new();
    let categories = if search_type == "text" {
        "general"
    } else {
        "images,videos"
    };

    let search_url = format!(
        "{}/search?lang=auto&safesearch=1&format=json&q={}&categories={}",
        searxng_base_url.trim_end_matches('/'),
        percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC),
        categories
    );

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            let delay = BASE_RETRY_DELAY * 2u32.pow((attempt - 1) as u32);
            warn!(
                "SearXNG retry in {}ms (attempt {}/{})",
                delay.as_millis(),
                attempt,
                MAX_RETRIES
            );
            tokio::time::sleep(delay).await;
        }

        let response = match client
            .get(&search_url)
            .header("Accept", "application/json")
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                circuit_breaker.record_failure();
                let err_msg = map_reqwest_error(e, &search_url);
                log_error_to_file("SearXNG Query Connection", &err_msg);
                return Err(err_msg);
            }
        };

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::INTERNAL_SERVER_ERROR
                && attempt < MAX_RETRIES
            {
                continue;
            }
            circuit_breaker.record_failure();
            let status = response.status();
            let err_msg = if status == reqwest::StatusCode::FORBIDDEN
                || status == reqwest::StatusCode::TOO_MANY_REQUESTS
            {
                format!("Search blocked ({} {}): SearXNG rate-limit or bot protection triggered. Try again later.", status.as_u16(), status.canonical_reason().unwrap_or("Forbidden"))
            } else {
                format!(
                    "Search service error ({} {}): The search backend returned an error.",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("Internal Server Error")
                )
            };
            log_error_to_file("SearXNG Query HTTP Status", &err_msg);
            return Err(err_msg);
        }

        circuit_breaker.record_success();
        let data = match response.json::<SearxngSearchResponse>().await {
            Ok(d) => d,
            Err(e) => {
                let err_msg = format!(
                    "Decode Error: Received invalid JSON format from search service. Details: {}",
                    e
                );
                log_error_to_file("SearXNG Query JSON Decode", &err_msg);
                return Err(err_msg);
            }
        };
        return Ok(data.results.unwrap_or_default());
    }

    circuit_breaker.record_failure();
    let err_msg = "Search service failed after maximum retries".to_string();
    log_error_to_file("SearXNG Query Retries Exceeded", &err_msg);
    Err(err_msg)
}

fn deduplicate_results(results: Vec<SearxngSearchResult>) -> Vec<SearxngSearchResult> {
    let mut urls = std::collections::HashSet::new();
    results
        .into_iter()
        .filter(|r| urls.insert(r.url.clone()))
        .collect()
}
