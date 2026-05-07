use backon::ExponentialBuilder;
use std::time::Duration;

pub fn default_backoff() -> ExponentialBuilder {
    ExponentialBuilder::default()
        .with_min_delay(Duration::from_millis(200))
        .with_max_delay(Duration::from_secs(30))
        .with_max_times(5)
        .with_jitter()
}
