use tokio::time::{sleep, Duration, Instant};

#[derive(Clone, Debug)]
pub struct RateLimit {
    units: u32,
    min_elasped_per_units: Duration,
}

impl RateLimit {
    pub fn new(units: u32, min_elasped_per_units: Duration) -> Self {
        Self {
            units,
            min_elasped_per_units,
        }
    }

    pub fn get_min_elapsed(&self, units: u32) -> Option<Duration> {
        if self.units < units {
            // Can't have min elapsed for greater units
            None
        } else {
            let factor = units as f32 / self.units as f32;

            Some(self.min_elasped_per_units.mul_f32(factor))
        }
    }
}

pub struct RateLimiter {
    min_elapsed: Duration,
    last_used_at: Option<Instant>,
}

impl RateLimiter {
    pub fn new(min_elapsed: Duration) -> Self {
        Self {
            min_elapsed,
            last_used_at: None,
        }
    }

    pub async fn throttle(&mut self) {
        if let Some(last_used_at) = self.last_used_at {
            let elapsed = Instant::now().duration_since(last_used_at);
            if let Some(time_left) = self.min_elapsed.checked_sub(elapsed) {
                sleep(time_left).await;
            }
        }

        self.last_used_at = Some(Instant::now());
    }
}
