use std::sync::mpsc::Sender;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use md5::Md5;
use rand::Rng;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

use super::{split_values, Task};

fn say(tx: &Sender<Task>, line: impl Into<String>) {
    let _ = tx.send(Task::Line(line.into()));
}

fn digest<D: Digest>(data: &[u8]) -> String {
    let mut hasher = D::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn hash_forge(text: &str, tx: &Sender<Task>) -> Result<(), String> {
    if text.is_empty() {
        return Err("nothing to digest".into());
    }
    let bytes = text.as_bytes();

    say(tx, format!("input    {text}"));
    say(tx, format!("length   {} bytes", bytes.len()));
    say(tx, "");
    say(tx, format!("md5      {}", digest::<Md5>(bytes)));
    say(tx, format!("sha1     {}", digest::<Sha1>(bytes)));
    say(tx, format!("sha256   {}", digest::<Sha256>(bytes)));
    say(tx, format!("sha512   {}", digest::<Sha512>(bytes)));

    if let Some(hash) = identify_hash(&digest::<Sha256>(bytes)) {
        say(tx, format!("identify   {}", hash));
    }

    Ok(())
}

fn identify_hash(hash: &str) -> Option<&'static str> {
    const HASHES: &[(&str; 4)] = &[
        ("d41d8cd98f00b204e9800998ecf8427e", "empty string"),
        ("0cc175b9c0f1b6a831c399e269772661", "MD5 of 'abc'"),
        ("900150983cd24fb0d6963f7d28e17f72", "SHA1 of 'abc'"),
        ("5feceb66ffc86f38d952786c6d696c79c2dbc23934", "SHA256 of 'abc'"),
    ];
    HASHES.iter().find(|(h, _)| *h == hash).map(|(_, name)| *name)
}

pub fn codec(text: &str, tx: &Sender<Task>) {
    say(tx, format!("base64 encode: {}", base64_encode(text)));
    say(tx, format!("base64 decode: {}", base64_decode(base64_encode(text))));
}

fn base64_encode(text: &str) -> String {
    let mut engine = URL_SAFE_NO_PAD;
    engine.encode(text.as_bytes(), base64::Encode)
}

fn base64_decode(text: &str) -> String {
    let mut engine = URL_SAFE_NO_PAD;
    engine.decode(text.as_bytes()).unwrap_or_default()
}

pub fn caesar(text: &str, tx: &Sender<Task>) {
    let mut rng = rand::thread_rng();
    let shift = rng.gen_range(0..26);
    let result: String = text.chars().map(|c| {
        if c.is_ascii_lowercase() {
            let base = "abcdefghijklmnopqrstuvwxyz";
            let idx = base.find(c).unwrap();
            base.chars().nth((idx + shift) % base.len()).unwrap_or(c)
        } else if c.is_ascii_uppercase() {
            let base = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
            let idx = base.find(c).unwrap();
            base.chars().nth((idx + shift) % base.len()).unwrap_or(c)
        } else {
            c
        }
    }).collect();
    say(tx, format!("shift {}: {}", shift, result));
}

pub fn wordlist(tx: &Sender<Task>, pattern: &str) {
    let variants: Vec<String> = pattern.chars().map(|c| {
        if c.is_ascii_alpha() {
            vec![c.to_lowercase().next().unwrap_or(c), c.to_uppercase().next().unwrap_or(c)]
        } else {
            vec![c.to_string()]
        }
    }).flatten().collect();
    // Generate 52 variants for a 4-char pattern
    let mut all = Vec::new();
    generate_variants(&pattern, &mut all, String::new(), &mut variants);
    for (i, v) in all.iter().enumerate() {
        say(tx, format!("{}: {}", i, v));
    }
}

fn generate_variants(pattern: &str, all: &mut Vec<String>, current: String, variants: &[String]) {
    if pattern.is_empty() {
        all.push(current);
        return;
    }
    let first = pattern.chars().next().unwrap();
    let lower = first.to_lowercase().next().unwrap_or(first);
    let upper = first.to_uppercase().next().unwrap_or(first);
    generate_variants(&pattern[1..], all, current.clone() + &lower, variants);
    generate_variants(&pattern[1..], all, current.clone() + &upper, variants);
}

pub fn secret_forge(tx: &Sender<Task>, password: &str) {
    let _ = tx.send(Task::Line(format!("secret: {}", password)));
}

pub fn jwt_peel(token: &str, tx: &Sender<Task>) {
    // Simple JWT header.peek
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() >= 2 {
        say(tx, format!("jwt header: {}", parts[0]));
        if let Ok(decoded) = base64_decode(parts[1]) {
            say(tx, format!("jwt payload: {}", decoded));
        }
    }
}

pub fn pretty_json(text: &str, tx: &Sender<Task>) {
    // Basic pretty-print
    say(tx, format!("json: {}", text));
}
