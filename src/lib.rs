use std::time::{Duration, Instant};

pub const TOTAL: usize = 5 * 1024 * 1024 * 32;
pub static DATA: &[u8] = &[0x61; 512 << 10];

#[derive(Debug)]
pub struct Timer(Instant);

impl Timer {
    pub fn start() -> Self {
        Timer(Instant::now())
    }

    pub fn stop(self) -> Duration {
        let now = Instant::now();
        now - self.0
    }
}
