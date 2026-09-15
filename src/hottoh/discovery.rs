//! Search of the stove on the local network, when no address is configured.
//!
//! The local IPv4 address is the one the system would use to reach the Internet; every other
//! host of its /24 network is tried on the module port, and a host is kept when it answers an
//! `INF` request like a HottoH module (`HOTTOH32;<version>;<signal>;`). Nothing is sent to hosts
//! that do not accept the connection.

use crate::hottoh::hottoh_const::{Command, CommandType};
use crate::hottoh::tcp_client_structs::{FrameBuffer, Request, Response};
use log::{debug, info};
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, UdpSocket};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

/// Delay to accept a connection: modules answer within a few milliseconds on a home network
const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
/// Delay for the `INF` answer
const ANSWER_TIMEOUT: Duration = Duration::from_secs(2);
/// Hosts tried in parallel
const THREADS: usize = 48;

/// A HottoH Wi-Fi module found on the network
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub address: SocketAddr,
    pub hostname: String,
    pub version: String,
}

/// Local IPv4 address used to reach other networks (no packet is sent)
pub fn local_ipv4() -> Result<Ipv4Addr, String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("no network: {}", e))?;
    // Documentation address (TEST-NET-1): connecting a UDP socket only picks the route
    socket
        .connect("192.0.2.1:9")
        .map_err(|e| format!("no network route: {}", e))?;
    match socket.local_addr().map(|a| a.ip()) {
        Ok(IpAddr::V4(ip)) if !ip.is_loopback() && !ip.is_unspecified() => Ok(ip),
        Ok(ip) => Err(format!("no usable IPv4 address (got {})", ip)),
        Err(e) => Err(format!("no local address: {}", e)),
    }
}

/// `/24` network of an address, as text
pub fn network_of(ip: Ipv4Addr) -> String {
    let [a, b, c, _] = ip.octets();
    format!("{}.{}.{}.0/24", a, b, c)
}

/// Other hosts of the `/24` network of `ip`
fn hosts(ip: Ipv4Addr) -> Vec<Ipv4Addr> {
    let [a, b, c, own] = ip.octets();
    (1..=254)
        .filter(|&d| d != own)
        .map(|d| Ipv4Addr::new(a, b, c, d))
        .collect()
}

/// Asks a host for its `INF` page; `Some` when it answers like a HottoH module
pub fn probe(address: SocketAddr) -> Option<Found> {
    let mut stream = TcpStream::connect_timeout(&address, CONNECT_TIMEOUT).ok()?;
    debug!("Discovery: {} accepts connections, asking INF", address);
    stream
        .set_read_timeout(Some(Duration::from_millis(200)))
        .ok()?;
    stream.set_write_timeout(Some(ANSWER_TIMEOUT)).ok()?;
    let request = Request::new(1, Command::Inf, CommandType::Read, vec![]);
    stream.write_all(&request.build_message()).ok()?;
    let mut buffer = FrameBuffer::new();
    let mut chunk = [0u8; 512];
    let deadline = Instant::now() + ANSWER_TIMEOUT;
    while Instant::now() < deadline {
        match stream.read(&mut chunk) {
            Ok(0) => return None,
            Ok(n) => buffer.extend(&chunk[..n]),
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(_) => return None,
        }
        while let Some(frame) = buffer.next_frame() {
            let Ok(response) = Response::from_message(&frame) else {
                continue;
            };
            let params = response.get_params();
            if response.get_command() == Command::Inf
                && params.len() == 3
                && params[0].starts_with("HOTTOH")
            {
                return Some(Found {
                    address,
                    hostname: params[0].clone(),
                    version: params[1].clone(),
                });
            }
        }
    }
    None
}

/// Tries `candidates` in parallel and returns the modules found, in address order
fn scan_hosts(
    candidates: &[SocketAddr],
    probe: impl Fn(SocketAddr) -> Option<Found> + Sync,
) -> Vec<Found> {
    let next = AtomicUsize::new(0);
    let found = Mutex::new(Vec::new());
    thread::scope(|scope| {
        for _ in 0..THREADS.min(candidates.len()) {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&address) = candidates.get(index) else {
                        break;
                    };
                    if let Some(module) = probe(address) {
                        found.lock().unwrap_or_else(|p| p.into_inner()).push(module);
                    }
                }
            });
        }
    });
    let mut found = found.into_inner().unwrap_or_else(|p| p.into_inner());
    found.sort_by_key(|m| m.address);
    found
}

/// Searches the local network for HottoH modules listening on `port`
pub fn scan(port: u16) -> Result<Vec<Found>, String> {
    let ip = local_ipv4()?;
    let started = Instant::now();
    let candidates: Vec<SocketAddr> = hosts(ip)
        .into_iter()
        .map(|host| SocketAddr::new(IpAddr::V4(host), port))
        .collect();
    let found = scan_hosts(&candidates, probe);
    info!(
        "Discovery: {} scanned in {} ms, {} HottoH module(s) found{}",
        network_of(ip),
        started.elapsed().as_millis(),
        found.len(),
        found
            .iter()
            .map(|m| format!(" {} ({} {})", m.address, m.hostname, m.version))
            .collect::<String>()
    );
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hottoh::hottoh_structs::calculate_checksum;
    use std::net::TcpListener;

    /// Fake module answering one INF request with `hostname`
    fn fake_module(hostname: &'static str) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut request = [0u8; 64];
                let _ = stream.read(&mut request);
                let params = format!("{};10.5.0;194;", hostname);
                let body = format!("00001C---{:04X}INFR{}", params.len(), params);
                let frame = format!("#{}{}\n", body, calculate_checksum(&body));
                let _ = stream.write_all(frame.as_bytes());
            }
        });
        address
    }

    #[test]
    fn module_is_recognised_by_its_inf_answer() {
        let module = fake_module("HOTTOH32");
        let found = probe(module).unwrap();
        assert_eq!(found.hostname, "HOTTOH32");
        assert_eq!(found.version, "10.5.0");
        assert!(probe(fake_module("OTHERDEV")).is_none());
        // Closed port
        let closed = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap();
        assert!(probe(closed).is_none());
    }

    #[test]
    fn scan_keeps_modules_in_address_order() {
        let modules = [fake_module("HOTTOH32"), fake_module("HOTTOH32")];
        let mut candidates = modules.to_vec();
        candidates.push(fake_module("OTHERDEV"));
        let found = scan_hosts(&candidates, probe);
        assert_eq!(found.len(), 2);
        assert!(found[0].address < found[1].address);
    }

    #[test]
    fn network_hosts() {
        let ip = Ipv4Addr::new(192, 168, 1, 73);
        assert_eq!(network_of(ip), "192.168.1.0/24");
        let all = hosts(ip);
        assert_eq!(all.len(), 253);
        assert!(!all.contains(&ip));
    }
}
