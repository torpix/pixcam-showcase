//! Loopback-only HTTP adapter. No CORS, arbitrary paths, tool proxy or CAD write surface.
use crate::{
    Result,
    bridge::{self, Client},
    recognition, showcase,
};
use serde_json::json;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    time::Duration,
};
const MAX_BODY: usize = 16 * 1024 * 1024;

pub fn authorized(
    method: &str,
    path: &str,
    headers: &[(String, String)],
    host: &str,
    session: &str,
) -> bool {
    let get = |name: &str| {
        headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    };
    if get("host") != Some(host) {
        return false;
    }
    if let Some(origin) = get("origin")
        && origin != format!("http://{host}")
    {
        return false;
    }
    if get("sec-fetch-site").is_some_and(|site| site != "same-origin" && site != "none") {
        return false;
    }
    if method == "GET" && path == "/" {
        return true;
    }
    method == "POST"
        && ["/api/refresh", "/api/analyze"].contains(&path)
        && get("origin") == Some(format!("http://{host}").as_str())
        && get("x-pixcam-session") == Some(session)
        && get("content-type") == Some("application/json")
}
fn reply(stream: &mut TcpStream, status: u16, kind: &str, body: &str) -> Result<()> {
    let reason = match status {
        200 => "OK",
        403 => "Forbidden",
        400 => "Bad Request",
        502 => "Bad Gateway",
        _ => "Not Found",
    };
    // Session key is HTTP adapter-only; it is unrelated to SolidCraft authentication.
    write!(stream,"HTTP/1.1 {status} {reason}\r\nContent-Type: {kind}; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nX-Frame-Options: DENY\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; img-src data: blob:; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'\r\n\r\n{body}",body.len()).map_err(|_|"HTTP write failed".into())
}
fn handle(
    mut stream: TcpStream,
    host: &str,
    session: &str,
    html: &str,
    config: &bridge::Config,
    client: &mut Option<Client>,
) -> Result<()> {
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|_| "HTTP timeout setup failed")?;
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .map_err(|_| "HTTP timeout setup failed")?;
    let mut head = Vec::new();
    let mut byte = [0];
    while !head.ends_with(b"\r\n\r\n") {
        if head.len() >= 8192 || stream.read(&mut byte).map_err(|_| "HTTP read timed out")? == 0 {
            return reply(&mut stream, 400, "text/plain", "Invalid request");
        }
        head.push(byte[0]);
    }
    let head = String::from_utf8(head).map_err(|_| "Invalid HTTP headers")?;
    let mut lines = head.split("\r\n");
    let parts: Vec<_> = lines.next().unwrap_or("").split_whitespace().collect();
    if parts.len() != 3 || parts[2] != "HTTP/1.1" {
        return reply(&mut stream, 400, "text/plain", "Invalid request");
    }
    let mut headers = Vec::new();
    for line in lines.filter(|l| !l.is_empty()) {
        let (key, value) = line.split_once(':').ok_or("Invalid header")?;
        let key = key.to_ascii_lowercase();
        if headers.iter().any(|(k, _)| *k == key) {
            return reply(&mut stream, 400, "text/plain", "Duplicate header");
        }
        headers.push((key, value.trim().to_string()));
    }
    if !authorized(parts[0], parts[1], &headers, host, session) {
        return reply(&mut stream, 403, "text/plain", "Forbidden");
    }
    if parts[1] == "/" {
        return reply(&mut stream, 200, "text/html", html);
    }
    if headers.iter().any(|(k, _)| k == "transfer-encoding") {
        return reply(&mut stream, 400, "text/plain", "Unsupported encoding");
    }
    let len = headers
        .iter()
        .find(|(k, _)| k == "content-length")
        .and_then(|(_, v)| v.parse::<usize>().ok());
    let Some(len) = len.filter(|len| *len <= MAX_BODY) else {
        return reply(&mut stream, 400, "text/plain", "Invalid content length");
    };
    let mut bytes = vec![0; len];
    stream
        .read_exact(&mut bytes)
        .map_err(|_| "HTTP body unavailable")?;
    let result = if parts[1] == "/api/refresh" {
        if len > 2 || (bytes != b"{}" && !bytes.is_empty()) {
            return reply(
                &mut stream,
                400,
                "text/plain",
                "Refresh takes no parameters",
            );
        }
        let mut result = Err("MCP unavailable".into());
        for _ in 0..2 {
            result = (|| {
                if client.is_none() {
                    *client = Some(Client::start(config)?);
                }
                client.as_mut().unwrap().snapshot()
            })();
            if result.is_ok() {
                break;
            }
            *client = None;
        }
        result.and_then(|s| recognition::analyze(&s.to_string()))
    } else {
        std::str::from_utf8(&bytes)
            .map_err(|_| "Invalid JSON encoding".into())
            .and_then(recognition::analyze)
    };
    match result {
        Ok(report) => reply(
            &mut stream,
            200,
            "application/json",
            &serde_json::to_string(&report).map_err(|_| "Report serialization failed")?,
        ),
        Err(error) => reply(
            &mut stream,
            502,
            "application/json",
            &json!({"connected":false,"error":error,"stale":true}).to_string(),
        ),
    }
}
pub fn serve(args: &[String]) -> Result<()> {
    let config = bridge::config(args)?;
    if config.input.is_none() && config.connect.is_none() {
        return Err("bridge requires --in isolated.solidcraft OR --connect 127.0.0.1:PORT".into());
    }
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .map_err(|_| "Cannot bind loopback adapter")?;
    let host = listener
        .local_addr()
        .map_err(|_| "Missing HTTP address")?
        .to_string();
    let mut random = [0u8; 32];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut random))
        .map_err(|_| "Secure session generation failed")?;
    let session = random
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let source = if config.connect.is_some() {
        "active SolidCraft application"
    } else {
        "isolated headless SolidCraft"
    };
    let html = showcase::render_html()?
        .replace("__LOCAL_SESSION__", &session)
        .replace("__LOCAL_SOURCE__", source);
    println!("pixCAM read-only adapter: http://{host}/ ({source})");
    std::io::stdout()
        .flush()
        .map_err(|_| "Cannot print adapter URL")?;
    let mut client = None;
    for stream in listener.incoming().flatten() {
        if stream.peer_addr().is_ok_and(|peer| peer.ip().is_loopback()) {
            // No request bodies, credentials, tool errors or geometry are logged.
            let _ = handle(stream, &host, &session, &html, &config, &mut client);
        }
    }
    Ok(())
}
