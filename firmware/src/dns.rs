use std::net::UdpSocket;
use std::thread;

/// IP address to resolve all DNS queries to (192.168.4.1).
const CAPTIVE_PORTAL_IP: [u8; 4] = [192, 168, 4, 1];

/// Minimum valid DNS query size: 12-byte header + at least 1 byte question.
const MIN_DNS_QUERY_LEN: usize = 13;

/// Start the captive-portal DNS server on a background thread.
///
/// Binds a UDP socket to port 53 and responds to every A-record query with
/// `192.168.4.1`. This forces all DNS resolution on the AP network to point
/// to the ESP32, enabling captive-portal detection on iOS, Android and Windows.
///
/// The thread is spawned as a daemon — caller does not need to hold a handle.
pub fn start() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:53")?;
    log::info!("DNS captive-portal server listening on :53");

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

                if let Some(response) = build_response(&buf[..len]) {
                    if let Err(e) = socket.send_to(&response, src) {
                        log::warn!("DNS send error: {e}");
                    }
                }
            }
        })?;

    Ok(())
}

/// Build a DNS response that answers every query with `CAPTIVE_PORTAL_IP`.
///
/// Returns `None` if the query is malformed.
fn build_response(query: &[u8]) -> Option<Vec<u8>> {
    // DNS header is 12 bytes:
    //   [0..2]  Transaction ID
    //   [2..4]  Flags
    //   [4..6]  QDCOUNT (questions)
    //   [6..8]  ANCOUNT (answers)
    //   [8..10] NSCOUNT
    //  [10..12] ARCOUNT
    if query.len() < 12 {
        return None;
    }

    let qdcount = u16::from_be_bytes([query[4], query[5]]);
    if qdcount == 0 {
        return None;
    }

    // Find the end of the question section so we can copy it verbatim.
    // Each question is: name (labels terminated by 0x00) + QTYPE(2) + QCLASS(2).
    let mut pos = 12;
    for _ in 0..qdcount {
        // Skip name labels
        while pos < query.len() {
            let label_len = query[pos] as usize;
            if label_len == 0 {
                pos += 1; // skip the terminating zero
                break;
            }
            // Pointer compression (0xC0 prefix) — shouldn't appear in a query's
            // question section, but handle it defensively.
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

    // Build response:
    //   - Copy header, set QR=1 (response), AA=1 (authoritative), RCODE=0 (no error)
    //   - Keep original QDCOUNT, set ANCOUNT = QDCOUNT
    //   - Copy question section verbatim
    //   - Append one A-record answer per question
    let question_section = &query[12..pos];

    let mut resp = Vec::with_capacity(pos + (qdcount as usize) * 16);

    // Transaction ID (copy from query)
    resp.extend_from_slice(&query[0..2]);

    // Flags: QR=1, Opcode=0, AA=1, TC=0, RD=1, RA=0, RCODE=0 → 0x8580
    // RD is copied from the query to be polite, but we always set AA.
    let rd = query[2] & 0x01; // Recursion Desired bit from query
    resp.push(0x84 | rd); // QR=1, AA=1, plus RD if set
    resp.push(0x00); // RA=0, RCODE=0

    // QDCOUNT (same as query)
    resp.extend_from_slice(&query[4..6]);
    // ANCOUNT = QDCOUNT (one answer per question)
    resp.extend_from_slice(&query[4..6]);
    // NSCOUNT = 0
    resp.extend_from_slice(&[0, 0]);
    // ARCOUNT = 0
    resp.extend_from_slice(&[0, 0]);

    // Question section (verbatim copy)
    resp.extend_from_slice(question_section);

    // Answer section: one A record per question
    // We use a name pointer (0xC00C) pointing back to offset 12 (first question name).
    // This is only fully correct for the first question, but virtually all DNS queries
    // contain exactly one question, so this is fine in practice.
    for _ in 0..qdcount {
        // Name: pointer to offset 0x000C
        resp.extend_from_slice(&[0xC0, 0x0C]);
        // Type: A (1)
        resp.extend_from_slice(&[0x00, 0x01]);
        // Class: IN (1)
        resp.extend_from_slice(&[0x00, 0x01]);
        // TTL: 60 seconds
        resp.extend_from_slice(&[0x00, 0x00, 0x00, 0x3C]);
        // RDLENGTH: 4 (IPv4 address)
        resp.extend_from_slice(&[0x00, 0x04]);
        // RDATA: 192.168.4.1
        resp.extend_from_slice(&CAPTIVE_PORTAL_IP);
    }

    Some(resp)
}
