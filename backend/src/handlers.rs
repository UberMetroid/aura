use axum::{
    extract::{ConnectInfo, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use rand::Rng;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tracing::error;

use crate::auth;
use crate::inference;
use crate::AppState;

#[derive(serde::Deserialize)]
pub struct SearchQueryParams {
    pub q: String,
    pub limit: Option<usize>,
}

#[derive(serde::Deserialize)]
pub struct VerifyPinRequest {
    pub pin: String,
}

pub async fn handle_status(State(state): State<AppState>) -> Response {
    state
        .status
        .get_status_response(&state.auth, &state.search)
        .await
        .into_response()
}

pub async fn handle_pin_required(
    State(state): State<AppState>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let client_ip = auth::get_client_ip(&connect_info, &headers);
    let attempts = state.auth.login_attempts.get(&client_ip);

    let (failed_count, last_attempt) = match attempts {
        Some(ref_val) => *ref_val.value(),
        None => (0, Instant::now()),
    };

    let lockout_duration = Duration::from_secs(15 * 60); // 15 minutes
    let locked =
        failed_count >= state.auth.max_attempts && last_attempt.elapsed() < lockout_duration;
    let attempts_left = if locked {
        0
    } else if failed_count >= state.auth.max_attempts {
        state.auth.max_attempts
    } else {
        state.auth.max_attempts - failed_count
    };

    let lockout_minutes = if locked {
        let elapsed = last_attempt.elapsed();
        let remaining = lockout_duration.saturating_sub(elapsed);
        remaining.as_secs().div_ceil(60)
    } else {
        0
    };

    Json(serde_json::json!({
        "required": state.auth.pin.is_some(),
        "length": state.auth.pin.as_ref().map(|p| p.len()).unwrap_or(0),
        "locked": locked,
        "attempts_left": attempts_left,
        "lockout_minutes": lockout_minutes,
        "enable_translation": state.auth.enable_translation,
        "enable_themes": state.config.enable_themes,
        "enable_print": state.config.enable_print,
    }))
}

pub async fn handle_verify_pin(
    State(state): State<AppState>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    cookie_jar: axum_extra::extract::cookie::CookieJar,
    Json(payload): Json<VerifyPinRequest>,
) -> impl IntoResponse {
    let client_ip = auth::get_client_ip(&connect_info, &headers);
    let lockout_duration = Duration::from_secs(15 * 60);

    // Check lockout status
    {
        if let Some(entry) = state.auth.login_attempts.get(&client_ip) {
            let (failed_count, last_attempt) = *entry.value();
            if failed_count >= state.auth.max_attempts && last_attempt.elapsed() < lockout_duration
            {
                let remaining = lockout_duration.saturating_sub(last_attempt.elapsed());
                let minutes = remaining.as_secs().div_ceil(60);
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    cookie_jar,
                    Json(serde_json::json!({
                        "valid": false,
                        "error": format!("Too many attempts. Please try again in {} minutes.", minutes),
                        "attempts_left": 0,
                        "locked": true,
                        "lockout_minutes": minutes,
                    }))
                ).into_response();
            }
        }
    }

    let pin_env = match &state.auth.pin {
        Some(p) => p,
        None => {
            return (
                StatusCode::OK,
                cookie_jar,
                Json(serde_json::json!({
                    "valid": true,
                    "error": null,
                    "attempts_left": null,
                    "locked": null,
                    "lockout_minutes": null,
                })),
            )
                .into_response();
        }
    };

    // Slow down timing attacks slightly
    let delay_ms = {
        let mut rng = rand::thread_rng();
        rng.gen_range(50..150)
    };
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;

    let valid = auth::secure_compare(&payload.pin, pin_env);

    if valid {
        state.auth.login_attempts.remove(&client_ip);

        let is_secure = headers
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("https"))
            .unwrap_or(false);

        let cookie = axum_extra::extract::cookie::Cookie::build((
            "AURA_PIN",
            auth::hash_pin(&payload.pin),
        ))
        .http_only(true)
        .secure(is_secure)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .path("/")
        .build();

        let updated_jar = cookie_jar.add(cookie);

        (
            StatusCode::OK,
            updated_jar,
            Json(serde_json::json!({
                "valid": true,
                "error": null,
                "attempts_left": null,
                "locked": null,
                "lockout_minutes": null,
            })),
        )
            .into_response()
    } else {
        let mut entry = state
            .auth
            .login_attempts
            .entry(client_ip)
            .or_insert((0, Instant::now()));
        entry.0 = entry.0.saturating_add(1);
        entry.1 = Instant::now();
        let left = state.auth.max_attempts.saturating_sub(entry.0);

        (
            StatusCode::UNAUTHORIZED,
            cookie_jar,
            Json(serde_json::json!({
                "valid": false,
                "error": format!("Invalid PIN. {} attempts remaining before lockout.", left),
                "attempts_left": left,
                "locked": left == 0,
                "lockout_minutes": if left == 0 { 15 } else { 0 },
            })),
        )
            .into_response()
    }
}

pub async fn handle_logout(
    cookie_jar: axum_extra::extract::cookie::CookieJar,
) -> impl IntoResponse {
    let cookie = axum_extra::extract::cookie::Cookie::build(("AURA_PIN", ""))
        .path("/")
        .build();
    (StatusCode::OK, cookie_jar.remove(cookie))
}

pub async fn handle_search_text(
    State(state): State<AppState>,
    cookie_jar: axum_extra::extract::cookie::CookieJar,
    headers: HeaderMap,
    Query(params): Query<SearchQueryParams>,
) -> impl IntoResponse {
    if !state.auth.is_authenticated(&cookie_jar, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
            .into_response();
    }

    let limit = params.limit.unwrap_or(30);
    match state.search.search_text(&params.q, limit).await {
        Ok(results) => {
            state.status.increment_textual();
            (StatusCode::OK, Json(results)).into_response()
        }
        Err(e) => {
            error!("Search text error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

pub async fn handle_search_images(
    State(state): State<AppState>,
    cookie_jar: axum_extra::extract::cookie::CookieJar,
    headers: HeaderMap,
    Query(params): Query<SearchQueryParams>,
) -> impl IntoResponse {
    if !state.auth.is_authenticated(&cookie_jar, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
            .into_response();
    }

    let limit = params.limit.unwrap_or(30);
    match state.search.search_images(&params.q, limit).await {
        Ok(results) => {
            state.status.increment_graphical();
            (StatusCode::OK, Json(results)).into_response()
        }
        Err(e) => {
            error!("Search images error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

pub async fn handle_inference_endpoint(
    State(state): State<AppState>,
    cookie_jar: axum_extra::extract::cookie::CookieJar,
    headers: HeaderMap,
    Json(body): Json<inference::ChatCompletionRequest>,
) -> Response {
    if !state.auth.is_authenticated(&cookie_jar, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
            .into_response();
    }

    inference::handle_inference(&state.config, body).await
}
