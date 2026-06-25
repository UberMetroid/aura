use axum::{
    http::HeaderValue,
    middleware,
    response::Response,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

mod auth;
mod circuit_breaker;
mod config;
mod duckduckgo;
mod grokipedia;
mod handlers;
mod inference;
mod merged;
mod invoke;
mod search;
mod searxng;
mod status;
mod utils;
mod wikipedia;

use auth::{AuthState, SharedAuthState};
use config::Config;
use search::{SearchService, SharedSearchService};
use status::{SharedStatusTracker, StatusTracker};

#[derive(Clone)]
pub struct AppState {
    config: Config,
    auth: SharedAuthState,
    search: SharedSearchService,
    status: SharedStatusTracker,
}

#[tokio::main]
async fn main() {
    let log_dir = std::env::var("LOG_DIR").ok().or_else(|| {
        let data_dir = std::path::Path::new("/app/data");
        if data_dir.is_dir() {
            Some("/app/data/log".to_string())
        } else {
            Some("/app/log".to_string())
        }
    });

    let (file_layer_error, file_layer_app) = if let Some(ref dir) = log_dir {
        if dir == "off" || dir == "none" || dir == "false" {
            (None, None)
        } else {
            let _ = std::fs::create_dir_all(dir);
            let error_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(std::path::Path::new(dir).join("error.log"))
                .ok();
            let app_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(std::path::Path::new(dir).join("app.log"))
                .ok();

            let error_layer = error_file.map(|file| {
                tracing_subscriber::fmt::layer()
                    .with_writer(std::sync::Mutex::new(file))
                    .with_ansi(false)
                    .with_filter(tracing_subscriber::filter::LevelFilter::WARN)
            });

            let app_layer = app_file.map(|file| {
                tracing_subscriber::fmt::layer()
                    .with_writer(std::sync::Mutex::new(file))
                    .with_ansi(false)
                    .with_filter(tracing_subscriber::filter::LevelFilter::INFO)
            });

            (error_layer, app_layer)
        }
    } else {
        (None, None)
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,rust_search=info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .with(file_layer_error)
        .with(file_layer_app)
        .init();

    info!("Starting Aura server...");

    let config = Config::load();

    let auth = Arc::new(AuthState::new(
        config.pin.clone(),
        config.max_attempts,
        config.enable_translation,
    ));

    // Spawn periodic login attempts cleaner to prevent memory leaks
    let clean_auth = auth.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            clean_auth.login_attempts.retain(|_, (_, last_time)| {
                last_time.elapsed() < std::time::Duration::from_secs(15 * 60)
            });
        }
    });

    let search = SearchService::new(
        config.search_provider.clone(),
        config.searxng_base_url.clone(),
    );
    let status = StatusTracker::new();

    let state = AppState {
        config: config.clone(),
        auth: auth.clone(),
        search: search.clone(),
        status: status.clone(),
    };

    let api_routes = Router::new()
        .route("/status", get(handlers::handle_status))
        .route("/api/pin-required", get(handlers::handle_pin_required))
        .route("/api/verify-pin", post(handlers::handle_verify_pin))
        .route("/api/logout", post(handlers::handle_logout))
        .route("/search/text", get(handlers::handle_search_text))
        .route("/search/images", get(handlers::handle_search_images))
        .route("/inference", post(handlers::handle_inference_endpoint))
        .route("/api/generate-invoke-image", post(invoke::handle_generate_invoke_image))
        .route("/api/invoke-image-status/:item_id", get(invoke::handle_invoke_image_status))
        .route("/api/invoke-image-file/:image_name", get(invoke::handle_invoke_image_file))
        .with_state(state.clone());

    let static_service =
        ServeDir::new(&config.static_dir).fallback(tower_http::services::ServeFile::new(
            std::path::Path::new(&config.static_dir).join("index.html"),
        ));

    let app = Router::new()
        .nest("/", api_routes)
        .fallback_service(static_service)
        .layer(middleware::from_fn(response_headers_middleware))
        .layer(CompressionLayer::new());

    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid host/port format");

    info!("Aura server listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn response_headers_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let path = req.uri().path().to_string();
    let is_api = path.starts_with("/search/")
        || path.starts_with("/status")
        || path.starts_with("/api/")
        || path.starts_with("/inference");

    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    headers.insert(
        "Cross-Origin-Embedder-Policy",
        HeaderValue::from_static("require-corp"),
    );
    headers.insert(
        "Cross-Origin-Opener-Policy",
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        "Cross-Origin-Resource-Policy",
        HeaderValue::from_static("cross-origin"),
    );

    if is_api {
        if let Ok(val) = HeaderValue::from_str("no-store, no-cache, must-revalidate, proxy-revalidate") {
            headers.insert("Cache-Control", val);
        }
    } else {
        let cache_control = if path.starts_with("/assets/") {
            "public, max-age=31536000, immutable"
        } else if path == "/" || path.ends_with(".html") {
            "no-cache"
        } else {
            "public, max-age=86400, must-revalidate"
        };
        if let Ok(val) = HeaderValue::from_str(cache_control) {
            headers.insert("Cache-Control", val);
        }
    }

    response
}
