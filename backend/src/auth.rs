use axum::{extract::ConnectInfo, http::HeaderMap};
use axum_extra::extract::cookie::CookieJar;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

pub struct AuthState {
    pub pin: Option<String>,
    pub max_attempts: usize,
    pub enable_translation: bool,
    pub login_attempts: DashMap<String, (usize, Instant)>,
}

impl AuthState {
    pub fn new(pin: Option<String>, max_attempts: usize, enable_translation: bool) -> Self {
        Self {
            pin,
            max_attempts,
            enable_translation,
            login_attempts: DashMap::new(),
        }
    }

    pub fn is_authenticated(&self, cookie_jar: &CookieJar, headers: &HeaderMap) -> bool {
        let pin_env = match &self.pin {
            Some(p) => p,
            None => return true,
        };

        let cookie_pin = cookie_jar.get("RUSTSEARCH_PIN").map(|c| c.value());
        let header_pin = headers.get("x-pin").and_then(|h| h.to_str().ok());

        match (cookie_pin, header_pin) {
            (Some(cookie), _) => secure_compare(cookie, &hash_pin(pin_env)),
            (None, Some(hdr)) => secure_compare(hdr, pin_env),
            (None, None) => false,
        }
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

pub fn hash_pin(pin: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    let result = hasher.finalize();
    result
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

pub fn get_client_ip(connect_info: &ConnectInfo<SocketAddr>, headers: &HeaderMap) -> String {
    if let Some(cf_connecting_ip) = headers.get("cf-connecting-ip") {
        if let Ok(ip) = cf_connecting_ip.to_str() {
            return ip.to_string();
        }
    }
    if let Some(x_forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(ip_list) = x_forwarded_for.to_str() {
            if let Some(ip) = ip_list.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }
    if let Some(x_real_ip) = headers.get("x-real-ip") {
        if let Ok(ip) = x_real_ip.to_str() {
            return ip.to_string();
        }
    }
    connect_info.ip().to_string()
}

pub type SharedAuthState = Arc<AuthState>;
