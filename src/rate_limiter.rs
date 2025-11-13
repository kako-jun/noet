use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Rate limiter to prevent API abuse and IP bans
/// Implements a simple fixed-delay rate limiting strategy
pub struct RateLimiter {
    last_request: Mutex<Option<Instant>>,
    delay: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter with specified delay between requests
    pub fn new(delay_ms: u64) -> Self {
        Self {
            last_request: Mutex::new(None),
            delay: Duration::from_millis(delay_ms),
        }
    }

    /// Wait if needed to respect rate limit
    /// This ensures minimum delay between consecutive API requests
    pub async fn wait(&self) {
        let wait_time = {
            let mut last = self.last_request.lock().unwrap();

            if let Some(last_time) = *last {
                let elapsed = last_time.elapsed();
                if elapsed < self.delay {
                    Some(self.delay - elapsed)
                } else {
                    *last = Some(Instant::now());
                    None
                }
            } else {
                // First request, no need to wait
                *last = Some(Instant::now());
                None
            }
        }; // Lock is dropped here

        if let Some(duration) = wait_time {
            sleep(duration).await;
            // Update timestamp after sleeping
            let mut last = self.last_request.lock().unwrap();
            *last = Some(Instant::now());
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        // Default to 500ms delay (2 requests per second)
        Self::new(500)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_first_request_no_wait() {
        let limiter = RateLimiter::new(100);
        let start = Instant::now();
        limiter.wait().await;
        let elapsed = start.elapsed();

        // First request should not wait (allow some margin for test execution)
        assert!(elapsed < Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_rate_limiter_enforces_delay() {
        let limiter = RateLimiter::new(100);

        // First request
        limiter.wait().await;

        // Second request immediately after
        let start = Instant::now();
        limiter.wait().await;
        let elapsed = start.elapsed();

        // Should wait approximately 100ms
        assert!(elapsed >= Duration::from_millis(90)); // Allow 10ms margin
        assert!(elapsed < Duration::from_millis(150)); // Upper bound
    }

    #[tokio::test]
    async fn test_rate_limiter_no_wait_if_delay_passed() {
        let limiter = RateLimiter::new(50);

        // First request
        limiter.wait().await;

        // Wait longer than delay
        sleep(Duration::from_millis(100)).await;

        // Second request should not wait
        let start = Instant::now();
        limiter.wait().await;
        let elapsed = start.elapsed();

        assert!(elapsed < Duration::from_millis(20));
    }

    #[test]
    fn test_rate_limiter_default() {
        let limiter = RateLimiter::default();
        assert_eq!(limiter.delay, Duration::from_millis(500));
    }
}
