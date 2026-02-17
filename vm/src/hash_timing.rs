use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Instant;

#[derive(Debug, Clone, Copy, Default)]
pub struct HashTimingStat {
    pub count: u64,
    pub total_us: u64,
}

static HASH_LOGS_ENABLED: OnceLock<bool> = OnceLock::new();

thread_local! {
    static HASH_TIMING_STATS: RefCell<HashMap<&'static str, HashTimingStat>> =
        RefCell::new(HashMap::new());
}

pub struct HashTimingGuard {
    label: &'static str,
    start: Option<Instant>,
}

impl HashTimingGuard {
    pub fn new(label: &'static str) -> Self {
        if hash_logs_enabled() {
            Self { label, start: Some(Instant::now()) }
        } else {
            Self { label, start: None }
        }
    }
}

impl Drop for HashTimingGuard {
    fn drop(&mut self) {
        let Some(start) = self.start else { return; };
        let elapsed = start.elapsed();
        let elapsed_us = elapsed.as_micros() as u64;
        HASH_TIMING_STATS.with(|stats| {
            let mut stats = stats.borrow_mut();
            let entry = stats.entry(self.label).or_default();
            entry.count += 1;
            entry.total_us = entry.total_us.saturating_add(elapsed_us);
        });
    }
}

pub fn hash_logs_enabled() -> bool {
    *HASH_LOGS_ENABLED.get_or_init(|| {
        let value = std::env::var("BLOCKIFIER_HASH_LOGS").unwrap_or_default();
        if value.is_empty() {
            return false;
        }
        match value.to_ascii_lowercase().as_str() {
            "0" | "false" | "no" | "off" => false,
            _ => true,
        }
    })
}

pub fn reset_hash_timing() {
    HASH_TIMING_STATS.with(|stats| stats.borrow_mut().clear());
}

pub fn hash_timing_snapshot() -> Vec<(&'static str, HashTimingStat)> {
    HASH_TIMING_STATS.with(|stats| stats.borrow().iter().map(|(k, v)| (*k, *v)).collect())
}
