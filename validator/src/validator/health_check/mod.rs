use crate::validator::config;
use log::debug;
use std::time::Duration;

pub(crate) struct HealthChecker {
    directory_server: String,
    interval: Duration,
}

impl HealthChecker {
    pub fn new(config: config::HealthCheck) -> Self {
        HealthChecker {
            directory_server: config.directory_server,
            interval: Duration::from_secs_f64(config.interval),
        }
    }

    pub fn run(self) {
        debug!(
            "healthcheck run. will use directory at: {:?} and run every {:?}",
            self.directory_server, self.interval,
        )
    }
}
