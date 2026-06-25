use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tracing::{info, warn};

const CIRCUIT_BREAKER_THRESHOLD: u64 = 5;
const CIRCUIT_BREAKER_TIMEOUT: Duration = Duration::from_secs(60);

pub struct CircuitBreaker {
    failures: AtomicU64,
    last_failure_time: AtomicU64, // Epoch millis
    is_open: AtomicBool,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            failures: AtomicU64::new(0),
            last_failure_time: AtomicU64::new(0),
            is_open: AtomicBool::new(false),
        }
    }

    pub fn check_allow(&self) -> Result<(), &'static str> {
        if self.is_open.load(Ordering::Relaxed) {
            let last_fail = self.last_failure_time.load(Ordering::Relaxed);
            let now = current_time_millis();
            if now.saturating_sub(last_fail) < CIRCUIT_BREAKER_TIMEOUT.as_millis() as u64 {
                return Err("Circuit breaker is open - SearXNG service temporarily unavailable");
            }
            self.is_open.store(false, Ordering::Relaxed);
            self.failures.store(0, Ordering::Relaxed);
            info!("Circuit breaker reset");
        }
        Ok(())
    }

    pub fn record_failure(&self) {
        let fails = self.failures.fetch_add(1, Ordering::Relaxed) + 1;
        self.last_failure_time
            .store(current_time_millis(), Ordering::Relaxed);
        if fails >= CIRCUIT_BREAKER_THRESHOLD {
            self.is_open.store(true, Ordering::Relaxed);
            warn!("Circuit breaker opened after {} failures", fails);
        }
    }

    pub fn record_success(&self) {
        self.failures.store(0, Ordering::Relaxed);
        self.is_open.store(false, Ordering::Relaxed);
    }
}

fn current_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}
