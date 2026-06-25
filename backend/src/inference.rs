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

pub async fn handle_inference(config: &Config, body: ChatCompletionRequest) -> Response {
    let client = reqwest::Client::new();
    let completions_url = format!(
        "{}/v1/chat/completions",
        config.ollama_base_url.trim_end_matches('/')
    );

    info!(
        "Forwarding chat completion to Ollama model '{}' at '{}'",
        config.ollama_model, completions_url
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
