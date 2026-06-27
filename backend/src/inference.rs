use crate::config::Config;
use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChatCompletionRequest {
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f64>,
    #[serde(rename = "top_p")]
    pub top_p: Option<f64>,
    #[serde(rename = "frequency_penalty")]
    pub frequency_penalty: Option<f64>,
    #[serde(rename = "presence_penalty")]
    pub presence_penalty: Option<f64>,
    #[serde(rename = "max_tokens")]
    pub max_tokens: Option<u32>,
}

#[derive(Serialize)]
struct ForwardRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    frequency_penalty: Option<f64>,
    presence_penalty: Option<f64>,
    max_tokens: Option<u32>,
    stream: bool,
}

pub async fn handle_inference(config: &Config, mut body: ChatCompletionRequest) -> Response {
    // ── Defense-in-depth: cap the total prompt size ───────────────────
    //
    // The user controls the message array. Without a cap, a single request
    // can force the server to assemble arbitrarily large prompts for
    // Ollama — a trivial DoS vector. We sum the content lengths and reject
    // requests exceeding `config.max_prompt_chars` (default 8000) with a
    // 400 Bad Request before any network call.
    let total_chars: usize = body.messages.iter().map(|m| m.content.len()).sum();
    if total_chars > config.max_prompt_chars {
        error!(
            "Rejecting inference request: {} chars exceeds limit of {}",
            total_chars, config.max_prompt_chars
        );
        return (
            StatusCode::BAD_REQUEST,
            format!(
                "Prompt too large: {} characters (max {})",
                total_chars, config.max_prompt_chars
            ),
        )
            .into_response();
    }

    // ── Optional system prompt prefix ─────────────────────────────────
    //
    // If `AURA_SYSTEM_PROMPT` is set, prepend it as a system message so
    // the model is constrained to the search-assistant role. This reduces
    // the attack surface for prompt injection from user-supplied messages:
    // the system prompt cannot be displaced by user content, only appended.
    if let Some(system) = config.system_prompt.as_deref() {
        // Only prepend if the caller did not already supply a system message.
        let has_system = body.messages.iter().any(|m| m.role == "system");
        if !has_system && !system.is_empty() {
            body.messages.insert(
                0,
                ChatMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                },
            );
        }
    }

    let client = reqwest::Client::new();
    let completions_url = format!(
        "{}/v1/chat/completions",
        config.ollama_base_url.trim_end_matches('/')
    );

    info!(
        "Forwarding chat completion to Ollama model '{}' at '{}' ({} chars)",
        config.ollama_model, completions_url, total_chars
    );

    let forward_req = ForwardRequest {
        model: config.ollama_model.clone(),
        messages: body.messages,
        temperature: body.temperature,
        top_p: body.top_p,
        frequency_penalty: body.frequency_penalty,
        presence_penalty: body.presence_penalty,
        max_tokens: body.max_tokens,
        stream: true,
    };

    match client
        .post(&completions_url)
        .json(&forward_req)
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                let stream = resp.bytes_stream().map(|chunk| {
                    chunk.map_err(|e| {
                        error!("Stream error during read: {}", e);
                        axum::Error::new(e)
                    })
                });

                let mut headers = HeaderMap::new();
                headers.insert(
                    "Content-Type",
                    HeaderValue::from_static("text/event-stream"),
                );
                headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
                headers.insert("Connection", HeaderValue::from_static("keep-alive"));
                headers.insert("X-Accel-Buffering", HeaderValue::from_static("no"));

                (headers, Body::from_stream(stream)).into_response()
            } else {
                let status = resp.status();
                let err_body = resp.text().await.unwrap_or_default();
                error!("Ollama returned error status ({}): {}", status, err_body);
                (status, format!("Ollama returned error: {}", err_body)).into_response()
            }
        }
        Err(e) => {
            error!(
                "Failed to connect to Ollama at '{}': {}",
                completions_url, e
            );
            (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Ollama connection error: {}", e),
            )
                .into_response()
        }
    }
}
