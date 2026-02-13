use std::net::{Ipv4Addr, UdpSocket};
use std::thread;

/// Minimum valid DNS query size: 12-byte header + at least 1 byte question.
const MIN_DNS_QUERY_LEN: usize = 13;

/// Start the captive-portal DNS server on a background thread.
///
/// Binds a UDP socket to port 53 and responds to every A-record query with
/// the given `ip`. This forces all DNS resolution on the AP network to point
/// to the ESP32, enabling captive-portal detection on iOS, Android and Windows.
///
/// The thread is spawned as a daemon — caller does not need to hold a handle.
pub fn start(ip: Ipv4Addr) -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:53")?;
    log::info!("DNS captive-portal server listening on :53 (-> {ip})");

    let ip_octets = ip.octets();

    thread::Builder::new()
        .name("dns-server".into())
        .stack_size(4096)
        .spawn(move || {
            let mut buf = [0u8; 512];
            loop {
                let (len, src) = match socket.recv_from(&mut buf) {
                    Ok(v) => v,
                    Err(e) => {
                        log::warn!("DNS recv error: {e}");
                        continue;
                    }
                };

                if len < MIN_DNS_QUERY_LEN {
                    continue;
                }

                if let Some(response) = build_response(&buf[..len], &ip_octets) {
                    if let Err(e) = socket.send_to(&response, src) {
                        log::warn!("DNS send error: {e}");
                    }
                }
            }
        })?;

    Ok(())
}

/// Build a DNS response that answers every query with the given IP.
///
/// Returns `None` if the query is malformed.
fn build_response(query: &[u8], ip: &[u8; 4]) -> Option<Vec<u8>> {
    if query.len() < 12 {
        return None;
    }

    let qdcount = u16::from_be_bytes([query[4], query[5]]);
    if qdcount == 0 {
        return None;
    }

    // Find the end of the question section.
    let mut pos = 12;
    for _ in 0..qdcount {
        while pos < query.len() {
            let label_len = query[pos] as usize;
            if label_len == 0 {
                pos += 1;
                break;
            }
            if label_len >= 0xC0 {
                pos += 2;
                break;
            }
            pos += 1 + label_len;
        }
        pos += 4; // QTYPE + QCLASS
    }

    if pos > query.len() {
        return None;
    }

    let question_section = &query[12..pos];
    let mut resp = Vec::with_capacity(pos + (qdcount as usize) * 16);

    // Transaction ID
    resp.extend_from_slice(&query[0..2]);

    // Flags: QR=1, AA=1, copy RD from query
    let rd = query[2] & 0x01;
    resp.push(0x84 | rd);
    resp.push(0x00);

    // QDCOUNT (same), ANCOUNT = QDCOUNT, NSCOUNT = 0, ARCOUNT = 0
    resp.extend_from_slice(&query[4..6]);
    resp.extend_from_slice(&query[4..6]);
    resp.extend_from_slice(&[0, 0, 0, 0]);

    // Question section (verbatim)
    resp.extend_from_slice(question_section);

    // Answer section: one A record per question
    for _ in 0..qdcount {
        resp.extend_from_slice(&[0xC0, 0x0C]); // Name pointer
        resp.extend_from_slice(&[0x00, 0x01]); // Type A
        resp.extend_from_slice(&[0x00, 0x01]); // Class IN
        resp.extend_from_slice(&[0x00, 0x00, 0x00, 0x3C]); // TTL 60s
        resp.extend_from_slice(&[0x00, 0x04]); // RDLENGTH 4
        resp.extend_from_slice(ip);
    }

    Some(resp)
}
