use std::fs::{self, File};
use std::io::Read;
use std::sync::mpsc::Sender;

use md5::Md5;
use sha2::Sha256;

use super::{digest, say, value};
use crate::app::Task;

const SAMPLE_LIMIT: u64 = 32 * 1024 * 1024;
const HEXDUMP_BYTES: usize = 256;

pub fn run_autopsy(inputs: &[String], tx: &Sender<Task>) -> Result<(), String> {
    let path = value(inputs, 0).trim();
    if path.is_empty() {
        return Err("point me at a file path".into());
    }

    let metadata = fs::metadata(path).map_err(|error| format!("cannot stat {path}: {error}"))?;
    if metadata.is_dir() {
        return Err("that path is a directory, not a file".into());
    }

    let size = metadata.len();
    let file = File::open(path).map_err(|error| format!("cannot open {path}: {error}"))?;
    let mut data = Vec::new();
    file.take(SAMPLE_LIMIT)
        .read_to_end(&mut data)
        .map_err(|error| format!("read failed: {error}"))?;

    say(tx, "⟡ artifact autopsy");
    say(tx, format!("path     {path}"));
    if size > SAMPLE_LIMIT {
        say(
            tx,
            format!("size     {} bytes · sampling first {}", size, SAMPLE_LIMIT),
        );
    } else {
        say(tx, format!("size     {} bytes", size));
    }
    let data_entropy = entropy(&data);
    say(tx, format!("magic    {}", detect_magic(&data)));
    say(
        tx,
        format!(
            "entropy  {:.3} bits/byte · {}",
            data_entropy,
            entropy_note(data_entropy)
        ),
    );
    say(tx, format!("sha256   {}", digest::<Sha256>(&data)));
    say(tx, format!("md5      {}", digest::<Md5>(&data)));
    say(tx, "");

    say(tx, "hexdump  (first 256 bytes)");
    for (index, chunk) in data.chunks(16).take(HEXDUMP_BYTES / 16).enumerate() {
        say(tx, format!("  {}", hexdump_row(index * 16, chunk)));
    }

    say(tx, "");
    say(tx, "text preview");
    let preview_sample = &data[..data.len().min(8192)];
    if preview_sample.is_empty() {
        say(tx, "  (empty file)");
    } else if looks_textual(preview_sample) {
        let text = String::from_utf8_lossy(preview_sample);
        let mut shown = 0;
        for line in text.lines() {
            if shown >= 10 {
                break;
            }
            let trimmed = line.trim_end_matches('\r');
            if trimmed.is_empty() {
                continue;
            }
            say(tx, format!("  {}", clip(trimmed, 96)));
            shown += 1;
        }
        if shown == 0 {
            say(tx, "  (empty)");
        }
    } else {
        say(tx, "  binary data - preview suppressed");
    }

    Ok(())
}

fn detect_magic(data: &[u8]) -> &'static str {
    const MAGIC: &[(&[u8], usize, &str)] = &[
        (b"PK\x03\x04", 0, "ZIP archive"),
        (b"PK\x05\x06", 0, "empty ZIP archive"),
        (b"\x1f\x8b", 0, "GZIP archive"),
        (b"Rar!\x1a\x07", 0, "RAR archive"),
        (b"\x7fELF", 0, "ELF binary"),
        (b"MZ", 0, "PE / DOS executable"),
        (b"\x89PNG\r\n\x1a\n", 0, "PNG image"),
        (b"\xff\xd8\xff", 0, "JPEG image"),
        (b"GIF87a", 0, "GIF image"),
        (b"GIF89a", 0, "GIF image"),
        (b"BM", 0, "BMP image"),
        (b"II*\x00", 0, "TIFF image"),
        (b"MM\x00*", 0, "TIFF image"),
        (b"%PDF-", 0, "PDF document"),
        (b"SQLite format 3\x00", 0, "SQLite database"),
        (b"7z\xbc\xaf\x27\x1c", 0, "7-Zip archive"),
        (b"BZh", 0, "BZIP2 archive"),
        (b"\xfd7zXZ\x00", 0, "XZ archive"),
        (b"OggS", 0, "Ogg media"),
        (b"ID3", 0, "MP3 audio"),
        (b"\x00asm", 0, "WebAssembly module"),
        (b"\x1a\x45\xdf\xa3", 0, "Matroska / WebM"),
        (b"RIFF", 0, "RIFF container"),
        (b"ustar", 257, "TAR archive"),
        (b"ftyp", 4, "MP4 / HEIF container"),
        (b"\x00\x00\x01\x00", 0, "ICO icon"),
        (b"\x00\x00\x02\x00", 0, "CUR cursor"),
        (b"{\\rtf", 0, "RTF document"),
        (b"<!DOCTYPE html", 0, "HTML document"),
        (b"<html", 0, "HTML document"),
        (b"<?xml", 0, "XML document"),
    ];

    for (magic, offset, name) in MAGIC {
        if data.len() >= offset + magic.len() && &data[*offset..*offset + magic.len()] == *magic {
            return name;
        }
    }

    if looks_textual(data) {
        return "text";
    }
    if data.is_empty() {
        return "empty file";
    }

    "unknown binary"
}

fn entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut counts = [0usize; 256];
    for byte in data {
        counts[*byte as usize] += 1;
    }

    let length = data.len() as f64;
    counts
        .iter()
        .filter(|count| **count > 0)
        .map(|count| {
            let probability = *count as f64 / length;
            -probability * probability.log2()
        })
        .sum()
}

fn entropy_note(entropy: f64) -> &'static str {
    if entropy >= 7.5 {
        "high · compressed or encrypted"
    } else if entropy >= 5.5 {
        "elevated"
    } else if entropy > 0.0 {
        "low · structured or textual"
    } else {
        "empty"
    }
}

fn hexdump_row(offset: usize, chunk: &[u8]) -> String {
    let mut hex_part = String::with_capacity(48);
    for index in 0..16 {
        match chunk.get(index) {
            Some(byte) => hex_part.push_str(&format!("{byte:02x}")),
            None => hex_part.push_str("  "),
        }
        if index != 15 {
            hex_part.push(' ');
        }
    }

    let ascii: String = (0..16)
        .map(|index| match chunk.get(index) {
            Some(byte) if (0x20..=0x7e).contains(byte) => *byte as char,
            Some(_) => '.',
            None => ' ',
        })
        .collect();

    format!("{offset:08x}  {hex_part}  |{ascii}|")
}

fn looks_textual(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }

    let sample = &data[..data.len().min(4096)];
    let printable = sample
        .iter()
        .copied()
        .filter(|byte| {
            matches!(byte, b'\n' | b'\r' | b'\t') || (0x20..=0x7e).contains(byte) || *byte >= 0x80
        })
        .count();

    if printable * 100 / sample.len() < 90 {
        return false;
    }

    std::str::from_utf8(sample).is_ok() || sample.iter().all(|byte| *byte < 0x80)
}

fn clip(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        text.to_string()
    } else {
        let clipped: String = text.chars().take(limit.saturating_sub(1)).collect();
        format!("{clipped}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_common_magic() {
        assert_eq!(detect_magic(b"PK\x03\x04rest"), "ZIP archive");
        assert_eq!(detect_magic(b"\x1f\x8b\x08\x00"), "GZIP archive");
        assert_eq!(detect_magic(b"\x7fELF\x02\x01"), "ELF binary");
        assert_eq!(detect_magic(b"\x89PNG\r\n\x1a\n"), "PNG image");
        assert_eq!(detect_magic(b"hello world\n"), "text");
        assert_eq!(detect_magic(b""), "empty file");
    }

    #[test]
    fn entropy_matches_extremes() {
        assert_eq!(entropy(b""), 0.0);
        assert_eq!(entropy(b"aaaaaaaa"), 0.0);

        let all_bytes: Vec<u8> = (0..=255u8).collect();
        assert!((entropy(&all_bytes) - 8.0).abs() < 1e-9);
    }

    #[test]
    fn text_detection_handles_utf8() {
        assert!(looks_textual("héla\nthe gate is open".as_bytes()));
        assert!(!looks_textual(&[0x00, 0x01, 0x02, 0x03]));
    }

    #[test]
    fn hexdump_has_offset_and_ascii() {
        let row = hexdump_row(16, b"hela");
        assert!(row.starts_with("00000010"));
        assert!(row.contains("68 65 6c 61"));
        assert!(row.ends_with("|hela            |"));
    }
}
