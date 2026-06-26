use gloo_net::http::Request;
use leptos::{SignalSet, WriteSignal};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GenerateInvokeImageResponse {
    pub item_id: Option<i64>,
    pub error: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct InvokeImageStatusResponse {
    pub status: String,
    pub image_name: Option<String>,
    pub error: Option<String>,
}

pub async fn generate_invoke_image(prompt: &str) -> Result<i64, String> {
    let req_body = serde_json::json!({ "prompt": prompt });
    let resp = Request::post("/api/generate-invoke-image")
        .json(&req_body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status() == 401 {
        return Err("Unauthorized".to_string());
    }
    if !resp.ok() {
        return Err("Failed to start InvokeAI image generation".to_string());
    }

    let val = resp
        .json::<GenerateInvokeImageResponse>()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(err) = val.error {
        Err(err)
    } else if let Some(item_id) = val.item_id {
        Ok(item_id)
    } else {
        Err("Invalid server response".to_string())
    }
}

pub async fn get_invoke_image_status(item_id: i64) -> Result<InvokeImageStatusResponse, String> {
    let resp = Request::get(&format!("/api/invoke-image-status/{}", item_id))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status() == 401 {
        return Err("Unauthorized".to_string());
    }
    if !resp.ok() {
        return Err("Failed to check status".to_string());
    }

    let val = resp
        .json::<InvokeImageStatusResponse>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(val)
}

pub fn spawn_invoke_generation(
    query: String,
    set_image: WriteSignal<Option<String>>,
    set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    web_sys::console::log_1(
        &format!("InvokeAI: Starting image generation for query: {}", query).into(),
    );
    set_loading.set(true);
    set_image.set(None);
    set_error.set(None);
    leptos::spawn_local(async move {
        match generate_invoke_image(&query).await {
            Ok(item_id) => {
                web_sys::console::log_1(
                    &format!(
                        "InvokeAI: Enqueued successfully. Received item_id = {}",
                        item_id
                    )
                    .into(),
                );
                poll_status(item_id, set_image, set_loading, set_error);
            }
            Err(e) => {
                web_sys::console::log_1(
                    &format!("InvokeAI: Failed to start generation: {}", e).into(),
                );
                set_error.set(Some(e));
                set_loading.set(false);
            }
        }
    });
}

fn poll_status(
    item_id: i64,
    set_image: WriteSignal<Option<String>>,
    set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    web_sys::console::log_1(
        &format!("InvokeAI: Scheduling next poll for item_id = {}", item_id).into(),
    );
    gloo_timers::callback::Timeout::new(1000, move || {
        leptos::spawn_local(async move {
            web_sys::console::log_1(
                &format!("InvokeAI: Querying status for item_id = {}", item_id).into(),
            );
            match get_invoke_image_status(item_id).await {
                Ok(status_resp) => {
                    web_sys::console::log_1(
                        &format!("InvokeAI: Status response = {}", status_resp.status).into(),
                    );
                    if status_resp.status == "completed" {
                        if let Some(image_name) = status_resp.image_name {
                            let url = format!("/api/invoke-image-file/{}", image_name);
                            web_sys::console::log_1(
                                &format!(
                                    "InvokeAI: Generation completed successfully. Image URL = {}",
                                    url
                                )
                                .into(),
                            );
                            set_image.set(Some(url));
                        } else {
                            web_sys::console::log_1(
                                &"InvokeAI: Generation completed but image_name is missing!".into(),
                            );
                            set_error.set(Some("Completed but no image returned".to_string()));
                        }
                        set_loading.set(false);
                    } else if status_resp.status == "failed" {
                        let err_msg = status_resp
                            .error
                            .unwrap_or_else(|| "Generation failed".to_string());
                        web_sys::console::log_1(
                            &format!("InvokeAI: Generation failed. Error = {}", err_msg).into(),
                        );
                        set_error.set(Some(err_msg));
                        set_loading.set(false);
                    } else {
                        web_sys::console::log_1(
                            &"InvokeAI: Generation still running. Rescheduling poll...".into(),
                        );
                        poll_status(item_id, set_image, set_loading, set_error);
                    }
                }
                Err(e) => {
                    web_sys::console::log_1(
                        &format!("InvokeAI: Poll query failed with network/API error = {}", e)
                            .into(),
                    );
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    })
    .forget();
}

pub fn strip_speaker_labels(mut s: &str) -> String {
    loop {
        let mut changed = false;
        let trimmed = s.trim_start_matches(|c| {
            c == ' ' || c == '*' || c == '_' || c == ':' || c == '-' || c == '\n' || c == '\r'
        });
        for prefix in &["ai assistant", "ai", "overview", "answer", "assistant"] {
            if trimmed.to_lowercase().starts_with(prefix) {
                let next_char = trimmed.chars().nth(prefix.len());
                if next_char.is_none() || next_char.is_none_or(|c| !c.is_alphanumeric()) {
                    let mut rest = &trimmed[prefix.len()..];
                    rest = rest.trim_start_matches(|c| {
                        c == ' '
                            || c == '*'
                            || c == '_'
                            || c == ':'
                            || c == '-'
                            || c == '\n'
                            || c == '\r'
                    });
                    s = rest;
                    changed = true;
                    break;
                }
            }
        }
        if !changed {
            break;
        }
    }
    s.to_string()
}
