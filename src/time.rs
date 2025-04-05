use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TimeInterval {
    start_time: Instant,
    end_time: Option<Instant>,
}

impl TimeInterval {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            end_time: None,
        }
    }

    pub fn stop(&mut self) -> Duration {
        self.end_time = Some(Instant::now());
        // UNWRAP: Both start_time and end_time are guaranteed to be set at this point
        self.elapsed().unwrap()
    }

    pub fn elapsed(&self) -> Option<Duration> {
        self.end_time.map(|end| end.duration_since(self.start_time))
    }
}

pub fn format(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let hours = minutes / 60;

    let seconds = seconds % 60;
    let minutes = minutes % 60;

    if hours > 0 {
        return format!("{}h {}m {}s", hours, minutes, seconds);
    } else if minutes > 0 {
        return format!("{}m {}s", minutes, seconds);
    } else {
        return format!("{}s", seconds);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hms() {
        let hms = Duration::from_secs(3661);
        assert_eq!(format(hms), "1h 1m 1s");
    }

    #[test]
    fn test_ms() {
        let ms = Duration::from_secs(61);
        assert_eq!(format(ms), "1m 1s");
    }

    #[test]
    fn test_s() {
        let s = Duration::from_secs(1);
        assert_eq!(format(s), "1s");
    }
}
