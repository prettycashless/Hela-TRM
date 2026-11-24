use std::fs;
use std::path::Path;

use super::{split_values, Task};

pub fn autopsy(path: &str, tx: &Sender<Task>) {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            let _ = tx.send(Task::Line(format!("! error reading: {e}")));
            return;
        }
    };

    // Magic bytes
    let magic = detect_magic(&data);
    say(tx, format!("magic: {magic}"));

    // Entropy
    let entropy = calculate_entropy(&data);
    say(tx, format!("entropy: {entropy:.3}"));

    // SHA256
    let hash = digest::<sha2::Sha256>(&data);
    say(tx, format!("sha256: {hash}"));

    // Hex dump (first 64 bytes)
    let hex: String = data.iter().take(64).map(|b| format!("{:02x}", b)).collect();
    say(tx, format!("hex: {hex}"));

    // Try text report
    if let Ok(text) = String::from_utf8(data.clone()) {
        let lines: Vec<&str> = text.lines().take(10).collect();
        say(tx, format!("text: {".to_string() + &lines.join(", ")));
    }
}

fn detect_magic(data: &[u8]) -> String {
    if data.len() >= 4 {
        match &data[0..4] {
            b"PK\x03\x04" => "ZIP archive".into(),
            b"\x1f\x8b" => "GZIP archive".into(),
            b"Rar" => "RAR archive".into(),
            b"\x7fELF" => "ELF binary".into(),
            b"MZ" => "PE/EXE binary".into(),
            _ => "unknown".into(),
        }
    } else {
        "unknown".into()
    }
}

fn calculate_entropy(data: &[u8]) -> f64 {
    use std::collections::HashMap;
    let mut freq = HashMap::new();
    for &byte in data {
        *freq.entry(byte).or_insert(0) += 1;
    }
    let data_len = data.len() as f64;
    let mut entropy = 0.0;
    for &count in freq.values() {
        let p = count as f64 / data_len;
        if p > 0.0 {
            entropy -= p * p.log2();
        }
    }
    entropy
}
