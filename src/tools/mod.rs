pub mod crypto;
pub mod files;
pub mod net;

use std::sync::mpsc::Sender;

use sha2::Digest;

use crate::app::Task;
use crate::system::SystemInfo;

pub fn say(tx: &Sender<Task>, line: impl Into<String>) {
    let _ = tx.send(Task::Line(line.into()));
}

pub fn value(inputs: &[String], index: usize) -> &str {
    inputs.get(index).map(String::as_str).unwrap_or("")
}

pub fn digest<D: Digest>(data: &[u8]) -> String {
    let mut hasher = D::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn run_sysinfo(_inputs: &[String], tx: &Sender<Task>) -> Result<(), String> {
    let sys = SystemInfo::collect();

    say(tx, "⟡ system ledger");
    say(tx, format!("operator {}", sys.user));
    say(tx, format!("host     {}", sys.host));
    say(tx, format!("os       {}", sys.os));
    say(tx, format!("kernel   {}", sys.kernel));
    say(tx, format!("arch     {}", sys.arch));
    say(tx, format!("cpu      {}", sys.cpu));
    say(tx, format!("memory   {}", sys.memory));
    say(tx, format!("uptime   {}", sys.uptime));
    say(tx, format!("shell    {}", sys.shell));
    say(tx, format!("term     {}", sys.term));
    say(tx, format!("ip       {}", sys.ip));
    say(tx, format!("cwd      {}", sys.cwd));

    Ok(())
}
