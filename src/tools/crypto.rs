use std::sync::mpsc::Sender;

use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use md5::Md5;
use rand::seq::SliceRandom;
use rand::Rng;
use sha1::Sha1;
use sha2::{Sha256, Sha512};

use super::{digest, say, value};
use crate::app::Task;

pub fn run_hash(inputs: &[String], tx: &Sender<Task>) -> Result<(), String> {
    let text = value(inputs, 0);
    if text.is_empty() {
        return Err("nothing to digest - type some text first".into());
    }

    let bytes = text.as_bytes();
    say(tx, "⟡ hash forge");
    say(tx, format!("input    {text}"));
    say(tx, format!("bytes    {}", bytes.len()));
    say(tx, "");
    say(tx, format!("md5      {}", digest::<Md5>(bytes)));
    say(tx, format!("sha1     {}", digest::<Sha1>(bytes)));
    say(tx, format!("sha256   {}", digest::<Sha256>(bytes)));
    say(tx, format!("sha512   {}", digest::<Sha512>(bytes)));

    Ok(())
}

pub fn run_hash_id(inputs: &[String], tx: &Sender<Task>) -> Result<(), String> {
    let raw = value(inputs, 0);
    let normalized: String = raw
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    if normalized.is_empty() {
        return Err("paste a hexadecimal digest first".into());
    }
    if !normalized
        .chars()
        .all(|character| character.is_ascii_hexdigit())
    {
        return Err("that is not a hexadecimal digest".into());
    }

    let lower = normalized.to_ascii_lowercase();
    let kind = match lower.len() {
        32 => "MD5 / NTLM / MD4",
        40 => "SHA-1",
        56 => "SHA-224",
        64 => "SHA-256",
        96 => "SHA-384",
        128 => "SHA-512",
        _ => "unknown width",
    };

    say(tx, "⟡ hash fingerprint");
    say(tx, format!("digest   {lower}"));
    say(tx, format!("width    {} hex chars", lower.len()));
    say(tx, format!("likely   {kind}"));

    if let Some(known) = known_digest(&lower) {
        say(tx, format!("match    {known}"));
    } else {
        say(tx, "match    no known fingerprint");
    }
    say(tx, "");
    say(
        tx,
        "note     length can identify a family, not the original input",
    );

    Ok(())
}

fn known_digest(digest: &str) -> Option<&'static str> {
    match digest {
        "d41d8cd98f00b204e9800998ecf8427e" => Some("MD5 of an empty string"),
        "900150983cd24fb0d6963f7d28e17f72" => Some("MD5 of \"abc\""),
        "a9993e364706816aba3e25717850c26c9cd0d89d" => Some("SHA-1 of \"abc\""),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad" => {
            Some("SHA-256 of \"abc\"")
        }
        "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f" => {
            Some("SHA-512 of \"abc\"")
        }
        _ => None,
    }
}

pub fn run_base64(inputs: &[String], tx: &Sender<Task>) -> Result<(), String> {
    let text = value(inputs, 0);
    if text.is_empty() {
        return Err("nothing to encode - type some text first".into());
    }

    let bytes = text.as_bytes();
    say(tx, "⟡ base64 loom");
    say(tx, format!("input    {text}"));
    say(tx, format!("bytes    {}", bytes.len()));
    say(tx, format!("standard {}", STANDARD.encode(bytes)));
    say(tx, format!("urlsafe  {}", URL_SAFE_NO_PAD.encode(bytes)));
    say(tx, "");

    let cleaned: String = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();

    if let Some(decoded) = decode_any(&cleaned) {
        say(tx, format!("decoded  {} bytes", decoded.len()));
        match std::str::from_utf8(&decoded) {
            Ok(decoded_text) => {
                say(tx, "utf8     yes");
                for (index, line) in decoded_text.lines().take(12).enumerate() {
                    say(tx, format!("{:04}  {}", index + 1, clip(line, 100)));
                }
                if decoded_text.trim().is_empty() {
                    say(tx, "         (empty or whitespace only)");
                }
            }
            Err(_) => {
                say(tx, "utf8     no - first bytes as hex");
                let preview = &decoded[..decoded.len().min(128)];
                say(tx, format!("hex      {}", hex::encode(preview)));
            }
        }
    } else {
        say(tx, "decode   input is not valid base64");
    }

    Ok(())
}

fn decode_any(input: &str) -> Option<Vec<u8>> {
    for engine in [&STANDARD, &STANDARD_NO_PAD, &URL_SAFE, &URL_SAFE_NO_PAD] {
        if let Ok(bytes) = engine.decode(input) {
            return Some(bytes);
        }
    }
    None
}

fn clip(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        text.to_string()
    } else {
        let clipped: String = text.chars().take(limit.saturating_sub(1)).collect();
        format!("{clipped}…")
    }
}

pub fn run_caesar(inputs: &[String], tx: &Sender<Task>) -> Result<(), String> {
    let text = value(inputs, 0);
    if text.is_empty() {
        return Err("the wheel needs a message".into());
    }
    let shift = value(inputs, 1)
        .trim()
        .parse::<i64>()
        .map_err(|_| "shift must be a whole number".to_string())?;
    let shift = shift.rem_euclid(26) as u8;

    say(tx, "⟡ caesar wheel");
    say(tx, format!("plain    {text}"));
    say(tx, format!("shift    {shift}"));
    say(tx, format!("cipher   {}", rotate(text, shift)));
    say(tx, "");
    say(tx, "brute force");
    for candidate in 1..26u8 {
        let marker = if candidate == shift { "*" } else { " " };
        say(
            tx,
            format!("{candidate:02} {marker} {}", rotate(text, candidate)),
        );
    }

    Ok(())
}

fn rotate(text: &str, shift: u8) -> String {
    text.chars()
        .map(|character| {
            if character.is_ascii_lowercase() {
                let offset = character as u8 - b'a';
                (b'a' + (offset + shift) % 26) as char
            } else if character.is_ascii_uppercase() {
                let offset = character as u8 - b'A';
                (b'A' + (offset + shift) % 26) as char
            } else {
                character
            }
        })
        .collect()
}

pub fn run_jwt(inputs: &[String], tx: &Sender<Task>) -> Result<(), String> {
    let raw = value(inputs, 0).trim();
    let token = raw
        .strip_prefix("Bearer ")
        .or_else(|| raw.strip_prefix("bearer "))
        .unwrap_or(raw)
        .trim();

    if token.is_empty() {
        return Err("paste a JWT first".into());
    }

    let segments: Vec<&str> = token.split('.').collect();
    if segments.len() < 2 {
        return Err("a JWT needs at least a header and a payload".into());
    }

    say(tx, "⟡ token peel");
    say(tx, format!("segments {}", segments.len()));

    for (title, part) in [("header", segments[0]), ("payload", segments[1])] {
        say(tx, "");
        say(tx, title);
        match decode_base64url(part) {
            Ok(bytes) => {
                let pretty = pretty_json(&bytes);
                for line in pretty.lines().take(40) {
                    say(tx, format!("  {line}"));
                }
            }
            Err(error) => say(tx, format!("  decode failed: {error}")),
        }
    }

    if let Some(signature) = segments.get(2) {
        say(tx, "");
        if signature.is_empty() {
            say(tx, "signature absent");
        } else {
            say(
                tx,
                format!("signature {} chars (not verified)", signature.len()),
            );
        }
    }

    say(tx, "");
    say(tx, "note     decoding is not verification");

    Ok(())
}

fn decode_base64url(part: &str) -> Result<Vec<u8>, String> {
    for engine in [&URL_SAFE_NO_PAD, &URL_SAFE, &STANDARD_NO_PAD, &STANDARD] {
        if let Ok(bytes) = engine.decode(part) {
            return Ok(bytes);
        }
    }
    Err("not valid base64url".into())
}

fn pretty_json(bytes: &[u8]) -> String {
    match serde_json::from_slice::<serde_json::Value>(bytes) {
        Ok(value) => serde_json::to_string_pretty(&value)
            .unwrap_or_else(|_| String::from_utf8_lossy(bytes).into_owned()),
        Err(_) => String::from_utf8_lossy(bytes).into_owned(),
    }
}

pub fn run_wordlist(inputs: &[String], tx: &Sender<Task>) -> Result<(), String> {
    let pattern = value(inputs, 0);
    if pattern.is_empty() {
        return Err("the mutator needs a seed pattern".into());
    }

    let characters: Vec<char> = pattern.chars().collect();
    if characters.len() > 64 {
        return Err("pattern too long - keep it under 64 characters".into());
    }

    let limit = value(inputs, 1)
        .trim()
        .parse::<usize>()
        .unwrap_or(256)
        .clamp(1, 4096);

    let theoretical = characters
        .iter()
        .filter(|character| character.is_ascii_alphabetic())
        .fold(1usize, |count, _| count.saturating_mul(2));

    let mut variants: Vec<String> = vec![String::new()];

    for character in &characters {
        let choices = case_variants(*character);
        let mut next = Vec::with_capacity((variants.len() * choices.len()).min(limit));

        'build: for prefix in &variants {
            for choice in &choices {
                let mut candidate = String::with_capacity(prefix.len() + 1);
                candidate.push_str(prefix);
                candidate.push(*choice);
                next.push(candidate);
                if next.len() >= limit {
                    break 'build;
                }
            }
        }

        variants = next;
        if variants.len() >= limit {
            break;
        }
    }

    say(tx, "⟡ case mutator");
    say(tx, format!("pattern  {pattern}"));
    say(tx, format!("theory   {theoretical} possible case variants"));
    say(
        tx,
        format!("limit    {limit} · emitting {}", variants.len()),
    );
    say(tx, "");

    for (index, variant) in variants.iter().enumerate() {
        say(tx, format!("{:04}  {variant}", index + 1));
    }

    if (variants.len() as u64) < theoretical as u64 {
        say(tx, "");
        say(tx, "… truncated at the configured limit");
    }

    Ok(())
}

fn case_variants(character: char) -> Vec<char> {
    if character.is_ascii_alphabetic() {
        let lower = character.to_ascii_lowercase();
        let upper = character.to_ascii_uppercase();
        if lower == upper {
            vec![character]
        } else {
            vec![lower, upper]
        }
    } else {
        vec![character]
    }
}

pub fn run_password(inputs: &[String], tx: &Sender<Task>) -> Result<(), String> {
    const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const DIGITS: &[u8] = b"0123456789";
    const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{}:,.?/";

    let length = value(inputs, 0)
        .trim()
        .parse::<usize>()
        .unwrap_or(20)
        .clamp(8, 128);
    let count = value(inputs, 1)
        .trim()
        .parse::<usize>()
        .unwrap_or(5)
        .clamp(1, 50);

    let mut alphabet = Vec::new();
    alphabet.extend_from_slice(LOWER);
    alphabet.extend_from_slice(UPPER);
    alphabet.extend_from_slice(DIGITS);
    alphabet.extend_from_slice(SYMBOLS);

    let mut rng = rand::thread_rng();

    say(tx, "⟡ secret forge");
    say(tx, format!("length   {length}"));
    say(tx, format!("count    {count}"));
    say(tx, "");

    for index in 1..=count {
        let classes: [&[u8]; 4] = [LOWER, UPPER, DIGITS, SYMBOLS];
        let mut characters: Vec<char> = Vec::with_capacity(length);

        for class in classes.iter().take(length.min(classes.len())) {
            let byte = class[rng.gen_range(0..class.len())];
            characters.push(byte as char);
        }
        while characters.len() < length {
            let byte = alphabet[rng.gen_range(0..alphabet.len())];
            characters.push(byte as char);
        }
        characters.shuffle(&mut rng);

        let password: String = characters.into_iter().collect();
        say(tx, format!("{index:02}  {password}"));
    }

    let bits = length as f64 * (alphabet.len() as f64).log2();
    say(tx, "");
    say(
        tx,
        format!(
            "entropy  ~{bits:.1} bits over a {} character alphabet",
            alphabet.len()
        ),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    fn emit(
        run: fn(&[String], &Sender<Task>) -> Result<(), String>,
        inputs: &[&str],
    ) -> Vec<String> {
        let (sender, receiver) = channel();
        let inputs: Vec<String> = inputs.iter().map(|value| value.to_string()).collect();
        run(&inputs, &sender).expect("tool should succeed");

        let mut lines = Vec::new();
        while let Ok(Task::Line(line)) = receiver.try_recv() {
            lines.push(line);
        }
        lines
    }

    #[test]
    fn known_hash_values() {
        assert_eq!(
            digest::<Sha256>(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(digest::<Md5>(b"abc"), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(
            digest::<Sha1>(b"abc"),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
    }

    #[test]
    fn hash_id_recognises_known_digest() {
        let lines = emit(
            run_hash_id,
            &["ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"],
        );
        assert!(lines.iter().any(|line| line.contains("SHA-256 of \"abc\"")));
        assert!(lines.iter().any(|line| line.contains("SHA-256")));
    }

    #[test]
    fn caesar_rotates_and_wraps() {
        assert_eq!(rotate("xyz XYZ", 3), "abc ABC");
        assert_eq!(rotate("Hela", 0), "Hela");
    }

    #[test]
    fn base64_decodes_standard_and_urlsafe() {
        assert_eq!(decode_any("aGVsYQ=="), Some(b"hela".to_vec()));
        assert_eq!(decode_any("aGVsYQ"), Some(b"hela".to_vec()));
        assert_eq!(decode_any("!!!"), None);
    }

    #[test]
    fn wordlist_respects_limit() {
        let lines = emit(run_wordlist, &["hela", "16"]);
        let variants: Vec<&String> = lines
            .iter()
            .filter(|line| line.len() > 6 && line[..4].chars().all(|c| c.is_ascii_digit()))
            .collect();

        assert_eq!(variants.len(), 16);
        assert!(variants[0].ends_with("hela"));
        assert!(variants[15].ends_with("HELA"));

        let capped = emit(run_wordlist, &["hela", "6"]);
        let capped_variants = capped
            .iter()
            .filter(|line| line.len() > 6 && line[..4].chars().all(|c| c.is_ascii_digit()))
            .count();
        assert_eq!(capped_variants, 6);
    }

    #[test]
    fn password_length_and_count_are_applied() {
        let lines = emit(run_password, &["12", "3"]);
        let passwords: Vec<&String> = lines
            .iter()
            .filter(|line| {
                line.len() > 4
                    && line.as_bytes().get(2) == Some(&b' ')
                    && line.as_bytes().get(3) == Some(&b' ')
                    && line[..2].chars().all(|c| c.is_ascii_digit())
            })
            .collect();

        assert_eq!(passwords.len(), 3);
        for entry in passwords {
            let password = entry.split_once("  ").map(|(_, p)| p).unwrap_or("");
            assert_eq!(password.chars().count(), 12);
        }
    }
}
