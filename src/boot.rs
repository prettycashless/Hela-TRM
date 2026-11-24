use std::time::Instant;

pub const SCRIPT: [(&str, &str); 6] = [
    ("relay", "niflheim:443 sealed"),
    ("blades", "11 modules armed"),
    ("cipher", "sha-2 tables verified"),
    ("ghost", "interface id randomised"),
    ("gates", "nine realms mapped"),
    ("hela", "the gate is open"),
];

pub struct Boot {
    start: Instant,
    offset: f32,
    skipped: bool,
}

impl Boot {
    pub const DURATION: f32 = 3.2;

    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            offset: 0.0,
            skipped: false,
        }
    }

    pub fn elapsed(&self) -> f32 {
        self.start.elapsed().as_secs_f32() + self.offset
    }

    #[cfg(test)]
    pub fn seek(&mut self, seconds: f32) {
        self.offset = seconds;
    }

    pub fn skip(&mut self) {
        self.skipped = true;
    }

    pub fn done(&self) -> bool {
        self.skipped || self.elapsed() >= Self::DURATION
    }

    /// 0..=1 reveal of the wordmark.
    pub fn reveal(&self) -> f32 {
        (self.elapsed() / 1.1).min(1.0)
    }

    /// 0..=1 progress of the loading bar.
    pub fn progress(&self) -> f32 {
        (self.elapsed() / 2.7).min(1.0)
    
    pub fn log_lines(&self) -> usize {
        let t = self.elapsed() - 0.55;
        if t <= 0.0 {
            0
        } else {
            ((t * 2.6) as usize).min(SCRIPT.len())
        }
    }
}
