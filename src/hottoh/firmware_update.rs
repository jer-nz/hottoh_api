//! Looks for a newer Wi-Fi module firmware on the HottoH update server.
//!
//! The module cannot tell by itself (`UPG R` does nothing). AppFire compares the installed
//! version with the one written in the application, then has the module download
//! `update.hottoh.it/update/upgrade_<major>_<minor>_<patch>.bin`. The bridge instead asks the
//! server whether the files of the next versions exist, with `HEAD` requests. Plain HTTP is
//! enough: nothing is downloaded, and the HTTPS certificate comes from a private HottoH CA.

use crate::hottoh::module_data::Version;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub const UPDATE_HOST: &str = "update.hottoh.it";
const TIMEOUT: Duration = Duration::from_secs(5);
/// Most `HEAD` requests for one check
const MAX_PROBES: usize = 12;

/// URL of the firmware file of a version
pub fn firmware_url(version: Version) -> String {
    format!(
        "http://{}/update/upgrade_{}_{}_{}.bin",
        UPDATE_HOST, version.0, version.1, version.2
    )
}

/// `true` for a 200 status line, `false` for 404, an error otherwise
fn exists_from_status_line(line: &str) -> Result<bool, String> {
    match line.split_whitespace().nth(1) {
        Some("200") => Ok(true),
        Some("404") => Ok(false),
        _ => Err(format!(
            "unexpected answer from {}: {:?}",
            UPDATE_HOST, line
        )),
    }
}

/// Asks the update server whether the firmware file of a version exists
pub fn firmware_exists(version: Version) -> Result<bool, String> {
    let address = (UPDATE_HOST, 80)
        .to_socket_addrs()
        .map_err(|e| format!("cannot resolve {}: {}", UPDATE_HOST, e))?
        .next()
        .ok_or_else(|| format!("cannot resolve {}", UPDATE_HOST))?;
    let mut stream = TcpStream::connect_timeout(&address, TIMEOUT)
        .map_err(|e| format!("cannot connect to {}: {}", UPDATE_HOST, e))?;
    stream
        .set_read_timeout(Some(TIMEOUT))
        .and_then(|_| stream.set_write_timeout(Some(TIMEOUT)))
        .map_err(|e| e.to_string())?;
    let path = firmware_url(version).replacen(&format!("http://{}", UPDATE_HOST), "", 1);
    write!(
        stream,
        "HEAD {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: hottoh_api/{}\r\nConnection: close\r\n\r\n",
        path,
        UPDATE_HOST,
        env!("CARGO_PKG_VERSION")
    )
    .map_err(|e| format!("request to {} failed: {}", UPDATE_HOST, e))?;
    let mut head = [0u8; 256];
    let mut len = 0;
    while len < head.len() && !head[..len].contains(&b'\n') {
        match stream.read(&mut head[len..]) {
            Ok(0) => break,
            Ok(n) => len += n,
            Err(e) => return Err(format!("no answer from {}: {}", UPDATE_HOST, e)),
        }
    }
    let text = String::from_utf8_lossy(&head[..len]);
    exists_from_status_line(text.lines().next().unwrap_or_default())
}

/// Newest version reachable from `installed` by following the published successors
pub fn latest_firmware(
    installed: Version,
    mut exists: impl FnMut(Version) -> Result<bool, String>,
) -> Result<Version, String> {
    let mut latest = installed;
    let mut probes = 0;
    'search: loop {
        for candidate in latest.successors() {
            if probes == MAX_PROBES {
                break 'search;
            }
            probes += 1;
            if exists(candidate)? {
                latest = candidate;
                continue 'search;
            }
        }
        break;
    }
    Ok(latest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_lines() {
        assert_eq!(exists_from_status_line("HTTP/1.1 200 OK"), Ok(true));
        assert_eq!(exists_from_status_line("HTTP/1.1 404 Not Found"), Ok(false));
        assert!(exists_from_status_line("HTTP/1.1 500 Oops").is_err());
        assert!(exists_from_status_line("").is_err());
    }

    #[test]
    fn urls_follow_appfire() {
        assert_eq!(
            firmware_url(Version(10, 5, 0)),
            "http://update.hottoh.it/update/upgrade_10_5_0.bin"
        );
    }

    #[test]
    fn latest_follows_successors() {
        let published = [Version(10, 5, 1), Version(10, 6, 0), Version(10, 6, 1)];
        let latest = latest_firmware(Version(10, 5, 0), |v| Ok(published.contains(&v))).unwrap();
        assert_eq!(latest, Version(10, 6, 1));

        let latest = latest_firmware(Version(10, 5, 0), |_| Ok(false)).unwrap();
        assert_eq!(latest, Version(10, 5, 0));

        assert!(latest_firmware(Version(10, 5, 0), |_| Err("offline".into())).is_err());

        // A server answering 200 to everything does not loop forever
        let mut calls = 0;
        latest_firmware(Version(1, 0, 0), |_| {
            calls += 1;
            Ok(true)
        })
        .unwrap();
        assert_eq!(calls, MAX_PROBES);
    }
}
