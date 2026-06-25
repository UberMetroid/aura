use crate::types::{ChatCompletionRequest, ImageSearchResult, TextSearchResult};
use gloo_net::http::Request;

pub struct AuthStatus {
    pub pin_required: bool,
    pub is_authorized: bool,
    pub enable_translation: bool,
}

pub async fn check_auth_status() -> Result<AuthStatus, String> {
    let resp = Request::get("/api/pin-required")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let data = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;
    let required = data
        .get("required")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let enable_translation = data
        .get("enable_translation")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    if !required {
        Ok(AuthStatus {
            pin_required: false,
            is_authorized: true,
            enable_translation,
        })
    } else {
        // Check if we are already authenticated via cookie
        let test_resp = Request::get("/search/text?q=auth_test&limit=1")
            .send()
            .await;
        let is_auth = match test_resp {
            Ok(r) => r.status() == 200,
            Err(_) => false,
        };
        Ok(AuthStatus {
            pin_required: true,
            is_authorized: is_auth,
            enable_translation,
        })
    }
}

pub async fn verify_pin(pin: &str) -> Result<bool, String> {
    let req_body = serde_json::json!({ "pin": pin });
    let resp = Request::post("/api/verify-pin")
        .json(&req_body)
        .unwrap()
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let val = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;
    let valid = val.get("valid").and_then(|v| v.as_bool()).unwrap_or(false);
    if valid {
        Ok(true)
    } else {
        let err_msg = val
            .get("error")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Invalid password.".to_string());
        Err(err_msg)
    }
}

pub async fn logout() -> Result<(), String> {
    let _ = Request::post("/api/logout")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn search_text(query: &str) -> Result<Vec<TextSearchResult>, String> {
    let search_url = format!(
        "/search/text?q={}&limit=10",
        percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC)
    );
    let resp = Request::get(&search_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.status() == 401 {
        return Err("Unauthorized".to_string());
    }
    if !resp.ok() {
        if let Ok(err_val) = resp.json::<serde_json::Value>().await {
            if let Some(err_str) = err_val.get("error").and_then(|v| v.as_str()) {
                return Err(err_str.to_string());
            }
        }
        return Err(format!(
            "Search request failed with status {}",
            resp.status()
        ));
    }
    let results = resp
        .json::<Vec<(String, String, String)>>()
        .await
        .map_err(|e| e.to_string())?;
    let mapped = results
        .into_iter()
        .map(|(title, snippet, url)| TextSearchResult {
            title,
            snippet,
            url,
        })
        .collect();
    Ok(mapped)
}

pub async fn search_images(query: &str) -> Result<Vec<ImageSearchResult>, String> {
    let search_url = format!(
        "/search/images?q={}&limit=12",
        percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC)
    );
    let resp = Request::get(&search_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.status() == 401 {
        return Err("Unauthorized".to_string());
    }
    if !resp.ok() {
        if let Ok(err_val) = resp.json::<serde_json::Value>().await {
            if let Some(err_str) = err_val.get("error").and_then(|v| v.as_str()) {
                return Err(err_str.to_string());
            }
        }
        return Err(format!(
            "Search request failed with status {}",
            resp.status()
        ));
    }
    let results = resp
        .json::<Vec<(String, String, String, String)>>()
        .await
        .map_err(|e| e.to_string())?;
    let mapped = results
        .into_iter()
        .map(|(title, url, thumbnail, source_url)| ImageSearchResult {
            title,
            url,
            thumbnail,
            source_url,
        })
        .collect();
    Ok(mapped)
}

pub async fn stream_inference<F>(
    chat_req: &ChatCompletionRequest,
    mut on_chunk: F,
) -> Result<(), String>
where
    F: FnMut(&str),
{
    use wasm_bindgen::JsCast;
    let inf_resp = Request::post("/inference")
        .json(chat_req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if inf_resp.status() == 401 {
        return Err("Unauthorized".to_string());
    }
    if inf_resp.status() != 200 {
        return Err("Failed to connect to the Ollama server".to_string());
    }

    if let Some(body) = inf_resp.body() {
        let reader = body
            .get_reader()
            .dyn_into::<web_sys::ReadableStreamDefaultReader>()
            .unwrap();
        let mut buffer = String::new();

        loop {
            let read_result = wasm_bindgen_futures::JsFuture::from(reader.read())
                .await
                .map_err(|e| format!("{:?}", e))?;
            let done = js_sys::Reflect::get(&read_result, &"done".into())
                .unwrap()
                .as_bool()
                .unwrap_or(false);
            if done {
                break;
            }

            let value = js_sys::Reflect::get(&read_result, &"value".into()).unwrap();
            let chunk = js_sys::Uint8Array::new(&value);
            let bytes = chunk.to_vec();
            let text = String::from_utf8_lossy(&bytes);
            buffer.push_str(&text);

            let mut lines = Vec::new();
            let mut remaining = String::new();

            let mut parts = buffer.split('\n');
            if let Some(last) = parts.next_back() {
                remaining = last.to_string();
            }
            for part in parts {
                lines.push(part.to_string());
            }
            buffer = remaining;

            for line in lines {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line == "data: [DONE]" {
                    break;
                }
                if line.starts_with("data: ") {
                    let json_str = &line[6..];
                    if let Ok(parsed) = serde_json::from_str::<crate::types::SseChunk>(json_str) {
                        if let Some(choice) = parsed.choices.first() {
                            if let Some(ref content) = choice.delta.content {
                                on_chunk(content);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
