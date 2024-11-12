use std::{collections::HashMap, time::Duration};

use systemstat::{Platform, System};

pub struct EnginePerformance {
    pub system: System,
    times: HashMap<&'static str, Duration>,
}

impl Default for EnginePerformance {
    fn default() -> Self {
        Self {
            system: System::new(),
            times: HashMap::new(),
        }
    }
}

impl EnginePerformance {
    pub fn record_time(&mut self, name: &'static str, dur: Duration) {
        self.times.insert(name, dur);
    }

    pub fn get_time(&self, name: &'static str) -> Option<Duration> {
        self.times.get(name).cloned()
    }
}
