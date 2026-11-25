use std::collections::{BTreeSet, VecDeque};
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::{say, value};
use crate::app::Task;

const MAX_PORTS: usize = 2048;

pub fn run_port_scan(inputs: &[String], tx: &Sender<Task>) -> Result<(), String> {
    let host = value(inputs, 0).trim();
    if host.is_empty() {
        return Err("a host is required".into());
    }

    let ports = parse_ports(value(inputs, 1))?;
    let timeout_ms = value(inputs, 2)
        .trim()
        .parse::<u64>()
        .unwrap_or(300)
        .clamp(50, 5000);

    let addresses: Vec<SocketAddr> = (host, 0)
        .to_socket_addrs()
        .map_err(|error| format!("cannot resolve {host}: {error}"))?
        .collect();

    let address = addresses
        .iter()
        .find(|address| address.is_ipv4())
        .or_else(|| addresses.first())
        .ok_or_else(|| format!("no addresses found for {host}"))?;

    let target_ip = address.ip();
    let worker_count = ports.len().clamp(1, 64);

    say(tx, "⟡ port probe");
    say(tx, format!("target   {host} ({target_ip})"));
    say(
        tx,
        format!(
            "ports    {} · timeout {timeout_ms}ms · {worker_count} workers",
            ports.len()
        ),
    );
    say(tx, "consent  only scan systems you own or may test");
    say(tx, "");

    let queue = Arc::new(Mutex::new(ports.iter().copied().collect::<VecDeque<u16>>()));
    let open = Arc::new(Mutex::new(Vec::<u16>::new()));
    let checked = Arc::new(AtomicUsize::new(0));

    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            let queue = Arc::clone(&queue);
            let open = Arc::clone(&open);
            let checked = Arc::clone(&checked);
            let sender = tx.clone();

            scope.spawn(move || loop {
                let port = {
                    let mut queue = queue.lock().expect("port queue poisoned");
                    queue.pop_front()
                };
                let Some(port) = port else {
                    break;
                };

                let socket = SocketAddr::new(target_ip, port);
                let reachable =
                    TcpStream::connect_timeout(&socket, Duration::from_millis(timeout_ms)).is_ok();

                checked.fetch_add(1, Ordering::Relaxed);

                if reachable {
                    open.lock().expect("open port list poisoned").push(port);
                    let service = service_hint(port);
                    let suffix = if service.is_empty() {
                        String::new()
                    } else {
                        format!("  {service}")
                    };
                    let _ = sender.send(Task::Line(format!("  open  {port:>5}{suffix}")));
                }
            });
        }
    });

    let mut open_ports = open.lock().expect("open port list poisoned").clone();
    open_ports.sort_unstable();

    let scanned = checked.load(Ordering::Relaxed);
    say(tx, "");
    if open_ports.is_empty() {
        say(tx, format!("done     {scanned} ports probed · none open"));
    } else {
        say(
            tx,
            format!(
                "done     {scanned} ports probed · {} open: {}",
                open_ports.len(),
                open_ports
                    .iter()
                    .map(u16::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
    }

    Ok(())
}

fn parse_ports(raw: &str) -> Result<Vec<u16>, String> {
    let mut ports = BTreeSet::new();

    for token in raw
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        if let Some((start, end)) = token.split_once('-') {
            let start = start
                .trim()
                .parse::<u16>()
                .map_err(|_| format!("invalid port range start in '{token}'"))?;
            let end = end
                .trim()
                .parse::<u16>()
                .map_err(|_| format!("invalid port range end in '{token}'"))?;

            if start == 0 || end == 0 || start > end {
                return Err(format!("invalid port range '{token}'"));
            }

            for port in start..=end {
                ports.insert(port);
                if ports.len() > MAX_PORTS {
                    return Err(format!("too many ports - the cap is {MAX_PORTS}"));
                }
            }
        } else {
            let port = token
                .parse::<u16>()
                .map_err(|_| format!("invalid port '{token}'"))?;
            if port == 0 {
                return Err("port 0 is not valid for this scan".into());
            }
            ports.insert(port);
        }

        if ports.len() > MAX_PORTS {
            return Err(format!("too many ports - the cap is {MAX_PORTS}"));
        }
    }

    if ports.is_empty() {
        return Err("no ports to scan - use 22,80,443 or a range like 1-1024".into());
    }

    Ok(ports.into_iter().collect())
}

fn service_hint(port: u16) -> &'static str {
    match port {
        20 | 21 => "ftp",
        22 => "ssh",
        23 => "telnet",
        25 => "smtp",
        53 => "dns",
        80 => "http",
        110 => "pop3",
        111 => "rpcbind",
        135 => "msrpc",
        139 => "netbios",
        143 => "imap",
        443 => "https",
        445 => "smb",
        993 => "imaps",
        995 => "pop3s",
        1433 => "mssql",
        1521 => "oracle",
        2049 => "nfs",
        2375 => "docker",
        3000 => "dev http",
        3306 => "mysql",
        3389 => "rdp",
        5432 => "postgres",
        5672 => "amqp",
        5900 => "vnc",
        6379 => "redis",
        8080 => "http-alt",
        8443 => "https-alt",
        9000 => "app",
        9200 => "elasticsearch",
        27017 => "mongodb",
        _ => "",
    }
}

pub fn run_dns(inputs: &[String], tx: &Sender<Task>) -> Result<(), String> {
    let host = value(inputs, 0).trim();
    if host.is_empty() {
        return Err("a hostname is required".into());
    }

    say(tx, "⟡ name reader");
    say(tx, format!("host     {host}"));

    let addresses: BTreeSet<IpAddr> = (host, 0)
        .to_socket_addrs()
        .map_err(|error| format!("lookup failed: {error}"))?
        .map(|address| address.ip())
        .collect();

    if addresses.is_empty() {
        return Err("the resolver returned no addresses".into());
    }

    for address in addresses {
        let family = if address.is_ipv6() { "ipv6" } else { "ipv4" };
        say(tx, format!("  {address:<40} {family}"));
    }

    Ok(())
}

pub fn run_http(inputs: &[String], tx: &Sender<Task>) -> Result<(), String> {
    let url = value(inputs, 0).trim();
    if url.is_empty() {
        return Err("a URL is required".into());
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL must start with http:// or https://".into());
    }

    say(tx, "⟡ door listener");
    say(tx, format!("url      {url}"));

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(12))
        .user_agent("Hela-TRM/0.2 (+https://github.com/prettycashless/Hela-TRM)")
        .build();

    let response = match agent.head(url).call() {
        Ok(response) => response,
        Err(ureq::Error::Status(405, _)) => {
            say(tx, "HEAD was refused - retrying with GET");
            match agent.get(url).call() {
                Ok(response) => response,
                Err(ureq::Error::Status(_, response)) => response,
                Err(error) => return Err(format!("request failed: {error}")),
            }
        }
        Err(ureq::Error::Status(_, response)) => response,
        Err(error) => return Err(format!("request failed: {error}")),
    };

    report(&response, tx);
    Ok(())
}

fn report(response: &ureq::Response, tx: &Sender<Task>) {
    let names = response.headers_names();
    say(
        tx,
        format!("status   {} {}", response.status(), response.status_text()),
    );
    say(tx, format!("headers  {}", names.len()));
    say(tx, "");

    for name in names {
        if let Some(value) = response.header(&name) {
            say(tx, format!("  {name}: {value}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ports_and_ranges() {
        assert_eq!(
            parse_ports("22, 80,443-445,22").expect("ports parse"),
            vec![22, 80, 443, 444, 445]
        );
        assert!(parse_ports("").is_err());
        assert!(parse_ports("0").is_err());
        assert!(parse_ports("90-80").is_err());
        assert!(parse_ports("not-a-port").is_err());
    }

    #[test]
    fn caps_port_count() {
        assert!(parse_ports("1-4096").is_err());
        assert_eq!(parse_ports("1-2048").expect("within cap").len(), 2048);
    }

    #[test]
    fn labels_common_services() {
        assert_eq!(service_hint(22), "ssh");
        assert_eq!(service_hint(443), "https");
        assert_eq!(service_hint(65000), "");
    }
}
