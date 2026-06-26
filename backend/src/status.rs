use crate::auth::SharedAuthState;
use crate::search::SharedSearchService;
use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub struct StatusTracker {
    start_time: Instant,
    pub textual_searches: AtomicU64,
    pub graphical_searches: AtomicU64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BuildStatus {
    timestamp: String,
    git_commit: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusResponse {
    uptime: String,
    sessions: usize,
    textual_searches: u64,
    graphical_searches: u64,
    average_textual_searches_per_session: f64,
    average_graphical_searches_per_session: f64,
    web_search_service_status: &'static str,
    build: BuildStatus,
}

impl StatusTracker {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            start_time: Instant::now(),
            textual_searches: AtomicU64::new(0),
            graphical_searches: AtomicU64::new(0),
        })
    }

    pub fn increment_textual(&self) {
        self.textual_searches.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_graphical(&self) {
        self.graphical_searches.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn get_status_response(
        &self,
        auth: &SharedAuthState,
        search: &SharedSearchService,
    ) -> impl axum::response::IntoResponse {
        let uptime_duration = self.start_time.elapsed();
        let uptime = format_duration(uptime_duration);

        let sessions = auth.login_attempts.len();
        let textual = self.textual_searches.load(Ordering::Relaxed);
        let graphical = self.graphical_searches.load(Ordering::Relaxed);

        let average_textual = if sessions > 0 {
            (textual as f64 / sessions as f64 * 10.0).round() / 10.0
        } else {
            0.0
        };

        let average_graphical = if sessions > 0 {
            (graphical as f64 / sessions as f64 * 10.0).round() / 10.0
        } else {
            0.0
        };

        let search_status = if search.get_status().await {
            "healthy"
        } else {
            "unhealthy"
        };

        let build_time_millis: u64 = env!("BUILD_TIMESTAMP").parse().unwrap_or(0);
        let build_timestamp = std::time::UNIX_EPOCH + Duration::from_millis(build_time_millis);
        let build_time_iso = match build_timestamp.duration_since(std::time::UNIX_EPOCH) {
            Ok(_) => {
                let datetime: chrono::DateTime<chrono::Utc> = build_timestamp.into();
                datetime.to_rfc3339()
            }
            Err(_) => "".to_string(),
        };

        let git_commit = env!("GIT_COMMIT_HASH").to_string();

        let response = StatusResponse {
            uptime,
            sessions,
            textual_searches: textual,
            graphical_searches: graphical,
            average_textual_searches_per_session: average_textual,
            average_graphical_searches_per_session: average_graphical,
            web_search_service_status: search_status,
            build: BuildStatus {
                timestamp: build_time_iso,
                git_commit,
            },
        };

        axum::Json(response)
    }
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs == 0 {
        return "0 seconds".to_string();
    }
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{} day{}", days, if days == 1 { "" } else { "s" }));
    }
    if hours > 0 {
        parts.push(format!(
            "{} hour{}",
            hours,
            if hours == 1 { "" } else { "s" }
        ));
    }
    if mins > 0 {
        parts.push(format!(
            "{} minute{}",
            mins,
            if mins == 1 { "" } else { "s" }
        ));
    }
    if s > 0 || parts.is_empty() {
        parts.push(format!("{} second{}", s, if s == 1 { "" } else { "s" }));
    }

    parts.join(", ")
}

pub type SharedStatusTracker = Arc<StatusTracker>;
