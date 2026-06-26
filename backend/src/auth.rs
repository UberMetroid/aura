use axum::{
    extract::ConnectInfo,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;
use dashmap::{DashMap, DashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

pub struct AuthState {
    pub pin: Option<String>,
    pub max_attempts: usize,
    pub enable_translation: bool,
    pub login_attempts: DashMap<String, (usize, Instant)>,
    pub active_sessions: DashSet<String>,
    pub rate_limiter: DashMap<String, Vec<Instant>>,
}

impl AuthState {
    pub fn new(pin: Option<String>, max_attempts: usize, enable_translation: bool) -> Self {
        Self {
            pin,
            max_attempts,
            enable_translation,
            login_attempts: DashMap::new(),
            active_sessions: DashSet::new(),
            rate_limiter: DashMap::new(),
        }
    }

    pub fn is_authenticated(&self, cookie_jar: &CookieJar, headers: &HeaderMap) -> bool {
        let pin_env = match &self.pin {
            Some(p) => p,
            None => return true,
        };

        let cookie_pin = cookie_jar.get("AURA_PIN").map(|c| c.value());
        let header_pin = headers.get("x-pin").and_then(|h| h.to_str().ok());

        match (cookie_pin, header_pin) {
            (Some(cookie), _) => self.active_sessions.contains(cookie),
            (None, Some(hdr)) => secure_compare(hdr, pin_env),
            (None, None) => false,
        }
    }

    pub fn check_rate_limit(&self, ip: String) -> bool {
        let max_requests = 100;
        let window = std::time::Duration::from_secs(60);
        let now = Instant::now();

        let mut entry = self.rate_limiter.entry(ip).or_default();
        entry.retain(|&t| now.duration_since(t) < window);

        if entry.len() >= max_requests {
            false
        } else {
            entry.push(now);
            true
        }
    }

    pub fn clean_old_rate_limits(&self) {
        let window = std::time::Duration::from_secs(60);
        let now = Instant::now();
        self.rate_limiter.retain(|_, timestamps| {
            timestamps.retain(|&t| now.duration_since(t) < window);
            !timestamps.is_empty()
        });
    }
}

pub fn secure_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn get_client_ip(connect_info: &ConnectInfo<SocketAddr>, headers: &HeaderMap) -> String {
    if let Some(cf_connecting_ip) = headers.get("cf-connecting-ip")
        && let Ok(ip) = cf_connecting_ip.to_str()
    {
        return ip.to_string();
    }
    if let Some(x_forwarded_for) = headers.get("x-forwarded-for")
        && let Ok(ip_list) = x_forwarded_for.to_str()
        && let Some(ip) = ip_list.split(',').next()
    {
        return ip.trim().to_string();
    }
    if let Some(x_real_ip) = headers.get("x-real-ip")
        && let Ok(ip) = x_real_ip.to_str()
    {
        return ip.to_string();
    }
    connect_info.ip().to_string()
}

pub type SharedAuthState = Arc<AuthState>;

pub fn generate_session_id() -> String {
    use std::fs::File;
    use std::io::Read;
    let file = File::open("/dev/urandom").ok();
    let mut bytes = [0u8; 16];
    if let Some(mut f) = file
        && f.read_exact(&mut bytes).is_ok()
    {
        return bytes.iter().map(|b| format!("{:02x}", b)).collect();
    }
    let random_val = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(random_val.to_string().as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

pub async fn rate_limit_middleware(
    axum::extract::State(auth): axum::extract::State<SharedAuthState>,
    req: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let addr = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .cloned()
        .unwrap_or_else(|| ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 0))));

    let ip = get_client_ip(&addr, req.headers());

    if !auth.check_rate_limit(ip) {
        let body = serde_json::json!({
            "error": "Too many requests. Please slow down."
        });
        let mut response = axum::response::Json(body).into_response();
        *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
        return Ok(response);
    }

    Ok(next.run(req).await)
}
