use crate::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct GenerateInvokeImageRequest {
    pub prompt: String,
}

#[derive(Serialize)]
pub struct GenerateInvokeImageResponse {
    pub item_id: Option<i64>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct InvokeImageStatusResponse {
    pub status: String,
    pub image_name: Option<String>,
    pub error: Option<String>,
}

fn get_invoke_base_url() -> String {
    std::env::var("INVOKE_BASE_URL").unwrap_or_else(|_| "http://100.68.35.70:9090".to_string())
}

pub async fn handle_generate_invoke_image(
    State(state): State<AppState>,
    cookie_jar: CookieJar,
    headers: HeaderMap,
    Json(payload): Json<GenerateInvokeImageRequest>,
) -> impl IntoResponse {
    if !state.auth.is_authenticated(&cookie_jar, &headers) {
        tracing::warn!("handle_generate_invoke_image: Unauthorized attempt");
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Unauthorized" })),
        )
            .into_response();
    }

    tracing::info!(
        "handle_generate_invoke_image: Received request for prompt: {}",
        payload.prompt
    );

    // Try loading graph_template.json from disk, fallback to sdxl_graph.json if not found
    let graph_template_str = match std::fs::read_to_string("graph_template.json") {
        Ok(s) => s,
        Err(_) => include_str!("sdxl_graph.json").to_string(),
    };

    let mut graph: Value = match serde_json::from_str(&graph_template_str) {
        Ok(g) => g,
        Err(e) => {
            tracing::error!(
                "handle_generate_invoke_image: Failed to parse graph template: {}",
                e
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to parse graph: {}", e) })),
            )
                .into_response();
        }
    };

    // Identify and update graph nodes dynamically using prefixes to remain model-agnostic
    let seed = rand::random::<u32>();
    let pos_style = ", highly detailed digital illustration, vibrant colors, clean lineart, cel-shaded, sharp focus, masterpiece";
    let neg_style = "ugly, deformed, noisy, blurry, low contrast, low quality, photorealistic, photo, long body, bad anatomy, bad hands, missing fingers, extra limbs, worst quality, lowres";

    if let Some(nodes) = graph.get_mut("nodes").and_then(|n| n.as_object_mut()) {
        for (key, node) in nodes {
            if key.starts_with("positive_prompt:") {
                if node.get("value").is_some() {
                    node["value"] = json!(format!("{}{}", payload.prompt, pos_style));
                }
            } else if key.starts_with("seed:") {
                if node.get("value").is_some() {
                    node["value"] = json!(seed);
                }
            } else if key.starts_with("negative_prompt:") || key.starts_with("neg_prompt:") {
                if node.get("value").is_some() {
                    node["value"] = json!(neg_style);
                } else if node.get("prompt").is_some() {
                    node["prompt"] = json!(neg_style);
                }
            }
        }
    }

    let enqueue_payload =
        json!({ "prepend": false, "batch": { "graph": graph, "workflow": null } });
    let client = reqwest::Client::new();
    let base_url = get_invoke_base_url();
    let enqueue_url = format!(
        "{}/api/v1/queue/default/enqueue_batch",
        base_url.trim_end_matches('/')
    );

    tracing::info!(
        "handle_generate_invoke_image: Enqueuing batch to InvokeAI at {}",
        enqueue_url
    );

    match client
        .post(&enqueue_url)
        .json(&enqueue_payload)
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                if let Ok(res_json) = resp.json::<Value>().await
                    && let Some(item_ids) = res_json.get("item_ids").and_then(|ids| ids.as_array())
                    && let Some(item_id) = item_ids.first().and_then(|id| id.as_i64())
                {
                    tracing::info!(
                        "handle_generate_invoke_image: Enqueued successfully. item_id = {}",
                        item_id
                    );
                    return (
                        StatusCode::OK,
                        Json(GenerateInvokeImageResponse {
                            item_id: Some(item_id),
                            error: None,
                        }),
                    )
                        .into_response();
                }
                tracing::error!(
                    "handle_generate_invoke_image: Invalid response format from InvokeAI"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Invalid response format from InvokeAI" })),
                )
                    .into_response()
            } else {
                let status_code = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                tracing::error!(
                    "handle_generate_invoke_image: InvokeAI returned error code {}: {}",
                    status_code,
                    err_text
                );
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({ "error": format!("InvokeAI returned error: {}", err_text) })),
                )
                    .into_response()
            }
        }
        Err(e) => {
            tracing::error!(
                "handle_generate_invoke_image: Failed to connect to InvokeAI: {}",
                e
            );
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("Failed to connect to InvokeAI: {}", e) })),
            )
                .into_response()
        }
    }
}

pub async fn handle_invoke_image_status(
    State(state): State<AppState>,
    cookie_jar: CookieJar,
    headers: HeaderMap,
    Path(item_id): Path<i64>,
) -> impl IntoResponse {
    if !state.auth.is_authenticated(&cookie_jar, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Unauthorized" })),
        )
            .into_response();
    }

    let client = reqwest::Client::new();
    let base_url = get_invoke_base_url();
    let status_url = format!(
        "{}/api/v1/queue/default/i/{}",
        base_url.trim_end_matches('/'),
        item_id
    );

    match client.get(&status_url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                if let Ok(status_json) = resp.json::<Value>().await {
                    let status = status_json
                        .get("status")
                        .and_then(|s| s.as_str())
                        .unwrap_or("unknown");
                    tracing::info!(
                        "handle_invoke_image_status: item_id = {}, status = {}",
                        item_id,
                        status
                    );
                    if status == "completed" {
                        let results = status_json.get("session").and_then(|s| s.get("results"));
                        let mut image_name = None;
                        if let Some(results_obj) = results.and_then(|r| r.as_object()) {
                            for (_, val) in results_obj {
                                if val.get("type").and_then(|t| t.as_str()) == Some("image_output")
                                    && let Some(name) = val
                                        .get("image")
                                        .and_then(|i| i.get("image_name"))
                                        .and_then(|n| n.as_str())
                                {
                                    image_name = Some(name.to_string());
                                    break;
                                }
                            }
                        }
                        tracing::info!(
                            "handle_invoke_image_status: item_id = {} completed, image_name = {:?}",
                            item_id,
                            image_name
                        );
                        return (
                            StatusCode::OK,
                            Json(InvokeImageStatusResponse {
                                status: "completed".to_string(),
                                image_name,
                                error: None,
                            }),
                        )
                            .into_response();
                    } else if status == "failed" {
                        let err_msg = status_json
                            .get("error_message")
                            .and_then(|e| e.as_str())
                            .unwrap_or("Unknown error")
                            .to_string();
                        tracing::error!(
                            "handle_invoke_image_status: item_id = {} failed: {}",
                            item_id,
                            err_msg
                        );
                        return (
                            StatusCode::OK,
                            Json(InvokeImageStatusResponse {
                                status: "failed".to_string(),
                                image_name: None,
                                error: Some(err_msg),
                            }),
                        )
                            .into_response();
                    } else {
                        return (
                            StatusCode::OK,
                            Json(InvokeImageStatusResponse {
                                status: status.to_string(),
                                image_name: None,
                                error: None,
                            }),
                        )
                            .into_response();
                    }
                }
                tracing::error!("handle_invoke_image_status: Invalid response JSON from InvokeAI");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Invalid response format from InvokeAI" })),
                )
                    .into_response()
            } else {
                tracing::error!(
                    "handle_invoke_image_status: InvokeAI returned error code checking status: {}",
                    resp.status()
                );
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({ "error": "Failed to fetch status from InvokeAI" })),
                )
                    .into_response()
            }
        }
        Err(e) => {
            tracing::error!(
                "handle_invoke_image_status: Failed to connect to InvokeAI status endpoint: {}",
                e
            );
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("Failed to connect to InvokeAI: {}", e) })),
            )
                .into_response()
        }
    }
}

pub async fn handle_invoke_image_file(
    State(state): State<AppState>,
    cookie_jar: CookieJar,
    headers: HeaderMap,
    Path(image_name): Path<String>,
) -> impl IntoResponse {
    if !state.auth.is_authenticated(&cookie_jar, &headers) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    tracing::info!(
        "handle_invoke_image_file: Requesting image file: {}",
        image_name
    );

    let client = reqwest::Client::new();
    let base_url = get_invoke_base_url();
    let file_url = format!(
        "{}/api/v1/images/i/{}/full",
        base_url.trim_end_matches('/'),
        image_name
    );

    match client.get(&file_url).send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                let content_type = resp
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("image/png")
                    .to_string();
                if let Ok(bytes) = resp.bytes().await {
                    tracing::info!(
                        "handle_invoke_image_file: Successfully served image {} ({} bytes)",
                        image_name,
                        bytes.len()
                    );
                    return ([(axum::http::header::CONTENT_TYPE, content_type)], bytes)
                        .into_response();
                }
            }
            tracing::error!(
                "handle_invoke_image_file: Failed to download file from InvokeAI, status: {}",
                status
            );
            (
                StatusCode::BAD_GATEWAY,
                "Failed to download image file from InvokeAI",
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(
                "handle_invoke_image_file: Failed to connect to InvokeAI file endpoint: {}",
                e
            );
            (
                StatusCode::BAD_GATEWAY,
                "Failed to connect to InvokeAI to fetch file",
            )
                .into_response()
        }
    }
}
