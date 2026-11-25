use std::fs;
use std::net::UdpSocket;

pub struct SystemInfo {
    pub user: String,
    pub host: String,
    pub os: String,
    pub kernel: String,
    pub arch: String,
    pub uptime: String,
    pub shell: String,
    pub term: String,
    pub cpu: String,
    pub memory: String,
    pub ip: String,
    pub cwd: String,
}

impl SystemInfo {
    pub fn collect() -> Self {
        Self {
            user: env_or("USER", "hela"),
            host: read("/proc/sys/kernel/hostname").unwrap_or_else(|| "niflheim".into()),
            os: pretty_os(),
            kernel: read("/proc/sys/kernel/osrelease")
                .unwrap_or_else(|| std::env::consts::OS.into()),
            arch: std::env::consts::ARCH.into(),
            uptime: uptime(),
            shell: base(env_or("SHELL", "sh")),
            term: env_or("TERM", "tty"),
            cpu: cpu_model(),
            memory: memory(),
            ip: local_ip(),
            cwd: std::env::current_dir()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|_| "?".into()),
        }
    }
}

fn env_or(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn read(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn base(path: String) -> String {
    path.rsplit('/').next().unwrap_or("sh").to_string()
}

fn pretty_os() -> String {
    if let Ok(text) = fs::read_to_string("/etc/os-release") {
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("PRETTY_NAME=") {
                return rest.trim_matches('"').to_string();
            }
        }
    }
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}

fn uptime() -> String {
    let raw = read("/proc/uptime").unwrap_or_default();
    let seconds: f64 = raw
        .split_whitespace()
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0.0);
    let total = seconds as u64;
    let days = total / 86_400;
    let hours = (total % 86_400) / 3_600;
    let minutes = (total % 3_600) / 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

fn cpu_model() -> String {
    if let Ok(text) = fs::read_to_string("/proc/cpuinfo") {
        for line in text.lines() {
            if let Some((key, value)) = line.split_once(':') {
                if key.trim() == "model name" {
                    let clean = value.split_whitespace().collect::<Vec<_>>().join(" ");
                    return truncate(&clean, 34);
                }
            }
        }
    }

    format!(
        "{} cores",
        std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1)
    )
}

fn memory() -> String {
    let text = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut total = 0f64;
    let mut available = 0f64;

    for line in text.lines() {
        if let Some(value) = line.strip_prefix("MemTotal:") {
            total = kb(value);
        } else if let Some(value) = line.strip_prefix("MemAvailable:") {
            available = kb(value);
        }
    }

    if total <= 0.0 {
        return "unknown".into();
    }

    format!(
        "{:.1}G / {:.1}G",
        (total - available) / 1_048_576.0,
        total / 1_048_576.0
    )
}

fn kb(raw: &str) -> f64 {
    raw.split_whitespace()
        .next()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn local_ip() -> String {
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket.connect("8.8.8.8:80")?;
            socket.local_addr()
        })
        .map(|address| address.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".into())
}

pub fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let mut output: String = text.chars().take(max.saturating_sub(1)).collect();
        output.push('…');
        output
    }
}

#[cfg(test)]
mod tests {
    use super::truncate;

    #[test]
    fn truncate_keeps_short_text() {
        assert_eq!(truncate("hela", 8), "hela");
    }

    #[test]
    fn truncate_respects_limit() {
        assert_eq!(truncate("abcdef", 4), "abc…");
        assert_eq!(truncate("abcdef", 0), "…");
    }
}
