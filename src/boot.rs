use std::time::Instant;

pub const SCRIPT: [(&str, &str); 6] = [
    ("relay", "niflheim:443 sealed"),
    ("blades", "all modules armed"),
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
        ((self.elapsed() - 0.08) / 0.95).clamp(0.0, 1.0)
    }

    /// 0..=1 progress of the loading bar.
    pub fn progress(&self) -> f32 {
        (self.elapsed() / 2.9).clamp(0.0, 1.0)
    }

    pub fn log_lines(&self) -> usize {
        let time = self.elapsed() - 0.55;
        if time <= 0.0 {
            0
        } else {
            ((time * 2.4) as usize).min(SCRIPT.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Boot;

    #[test]
    fn boot_reveals_and_completes() {
        let mut boot = Boot::new();
        boot.seek(0.0);
        assert_eq!(boot.reveal(), 0.0);
        assert_eq!(boot.log_lines(), 0);
        assert!(!boot.done());

        boot.seek(1.5);
        assert!(boot.reveal() > 0.9);
        assert!(boot.progress() > 0.4);
        assert!(boot.log_lines() > 0);

        boot.seek(3.3);
        assert!(boot.done());
        assert_eq!(boot.log_lines(), 6);
    }

    #[test]
    fn skip_finishes_boot() {
        let mut boot = Boot::new();
        boot.skip();
        assert!(boot.done());
    }
}
