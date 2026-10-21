//! Read-only, bounded stdio MCP client. The CAD process owns control authentication.
use crate::Result;
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

pub const CLI: &str = "/home/torpix/Aplikacje/solidcraft-cli-0.17.0";
pub const READ_ONLY_TOOLS: &[&str] = &[
    "list_commands",
    "inspect_design",
    "measure",
    "body_topology",
];
const MAX_MESSAGE: usize = 16 * 1024 * 1024;

#[derive(Clone)]
pub struct Config {
    pub executable: PathBuf,
    pub input: Option<PathBuf>,
    pub connect: Option<String>,
    pub timeout: Duration,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            executable: CLI.into(),
            input: None,
            connect: None,
            timeout: Duration::from_secs(15),
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        if !self.executable.is_absolute() || self.timeout.is_zero() {
            return Err("Absolute CLI path and positive timeout required".into());
        }
        if self.input.is_some() && self.connect.is_some() {
            return Err("Choose an isolated input OR an active app connection".into());
        }
        if let Some(address) = &self.connect {
            let address: std::net::SocketAddr =
                address.parse().map_err(|_| "Use 127.0.0.1:PORT")?;
            if address.ip() != std::net::Ipv4Addr::LOCALHOST || address.port() == 0 {
                return Err(
                    "SolidCraft control connection must use 127.0.0.1 and a nonzero explicit port"
                        .into(),
                );
            }
        }
        Ok(())
    }
}

pub fn config(args: &[String]) -> Result<Config> {
    let mut c = Config::default();
    let mut i = 0;
    while i < args.len() {
        let value = args.get(i + 1).ok_or("Missing bridge option value")?;
        match args[i].as_str() {
            "--in" => {
                c.input = Some(std::fs::canonicalize(value).map_err(|_| "Input file unavailable")?)
            }
            "--connect" => c.connect = Some(value.clone()),
            "--timeout-ms" => {
                c.timeout = Duration::from_millis(value.parse().map_err(|_| "Invalid timeout")?)
            }
            _ => {
                return Err(
                    "Options: --in FILE OR --connect 127.0.0.1:PORT; --timeout-ms N".into(),
                );
            }
        }
        i += 2;
    }
    c.validate()?;
    Ok(c)
}

pub struct Client {
    child: Child,
    stdin: Option<ChildStdin>,
    messages: Receiver<Result<Value>>,
    timeout: Duration,
    next_id: u64,
    usable: bool,
}
impl Client {
    pub fn start(config: &Config) -> Result<Self> {
        config.validate()?;
        let mut cmd = Command::new(&config.executable);
        cmd.arg("mcp");
        if let Some(path) = &config.input {
            cmd.arg("--in").arg(path);
        }
        if let Some(address) = &config.connect {
            cmd.arg("--connect").arg(address);
        }
        // Never copy control tokens into arguments, diagnostics, snapshots or HTTP.
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "Cannot start SolidCraft CLI 0.17.0")?;
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().ok_or("Missing MCP stdout")?;
        let (tx, messages) = mpsc::sync_channel(8);
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut line = Vec::new();
                let read = reader
                    .by_ref()
                    .take((MAX_MESSAGE + 1) as u64)
                    .read_until(b'\n', &mut line);
                let value = match read {
                    Ok(0) => Err("MCP connection closed".into()),
                    Ok(_) if line.len() > MAX_MESSAGE => {
                        Err("MCP message exceeds size limit".into())
                    }
                    Ok(_) => serde_json::from_slice(&line).map_err(|_| "Invalid MCP JSON".into()),
                    Err(_) => Err("MCP read failed".into()),
                };
                let failed = value.is_err();
                if tx.send(value).is_err() || failed {
                    break;
                }
            }
        });
        let mut client = Self {
            child,
            stdin,
            messages,
            timeout: config.timeout,
            next_id: 0,
            usable: true,
        };
        let init = client.request("initialize", json!({"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"pixCAM read-only bridge","version":"0.3.0"}}))?;
        if init["serverInfo"]["name"] != "solidcraft"
            || init["serverInfo"]["version"] != "0.17.0"
            || init["protocolVersion"] != "2025-03-26"
        {
            return Err("Incompatible SolidCraft server/protocol; require 0.17.0".into());
        }
        client.send(&json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
        Ok(client)
    }
    fn send(&mut self, value: &Value) -> Result<()> {
        let stdin = self.stdin.as_mut().ok_or("MCP session closed")?;
        writeln!(stdin, "{value}")
            .and_then(|_| stdin.flush())
            .map_err(|_| "MCP write failed".into())
    }
    fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        if !self.usable {
            return Err("Restart required after failed MCP request".into());
        }
        self.next_id += 1;
        let id = self.next_id;
        self.send(&json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))?;
        let deadline = Instant::now() + self.timeout;
        let response = (|| {
            for _ in 0..64 {
                let message = self
                    .messages
                    .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                    .map_err(|_| "MCP timeout/disconnect")??;
                if message["jsonrpc"] != "2.0" {
                    return Err("Invalid JSON-RPC version".into());
                }
                if message.get("id").is_none() && message["method"].is_string() {
                    continue;
                }
                if message["id"] != id
                    || message.get("result").is_some() == message.get("error").is_some()
                {
                    return Err("MCP response id/envelope mismatch".into());
                }
                if message.get("error").is_some() {
                    return Err("MCP protocol error (details suppressed)".into());
                }
                return Ok(message["result"].clone());
            }
            Err("Excess MCP notifications".into())
        })();
        if response.is_err() {
            self.usable = false;
            let _ = self.child.kill();
        }
        response
    }
    pub fn call(&mut self, name: &str, arguments: Value) -> Result<Value> {
        if !READ_ONLY_TOOLS.contains(&name) {
            return Err("MCP tool denied by pixCAM read-only allowlist".into());
        }
        let r = self.request("tools/call", json!({"name":name,"arguments":arguments}))?;
        if r["isError"].as_bool().unwrap_or(false) {
            return Err("SolidCraft read-only tool failed (details suppressed)".into());
        }
        let text = r["content"]
            .as_array()
            .and_then(|a| a.iter().find(|v| v["type"] == "text"))
            .and_then(|v| v["text"].as_str())
            .ok_or("Missing MCP text result")?;
        serde_json::from_str(text).map_err(|_| "Invalid JSON tool result".into())
    }
    /// Retrying means a fresh initialize/initialized and a complete recapture.
    pub fn capture(config: &Config) -> Result<Value> {
        let mut last = Err("MCP capture failed".into());
        for _ in 0..2 {
            last = Self::start(config).and_then(|mut c| c.snapshot());
            if last.is_ok() {
                break;
            }
        }
        last
    }
    pub fn snapshot(&mut self) -> Result<Value> {
        self.call("list_commands", json!({"filter":"solid.hole"}))?;
        let inspect = self.call(
            "inspect_design",
            json!({"measure":true,"sketches":false,"joints":false}),
        )?;
        if inspect["units"] != "mm" {
            return Err("Only millimetre CAD documents are supported".into());
        }
        let mut topologies = Vec::new();
        let mut measurements = Vec::new();
        for body in inspect["bodies"].as_array().ok_or("Missing bodies")? {
            if topologies.len() >= 256 {
                return Err("Body limit exceeded".into());
            }
            let name = body["name"].as_str().ok_or("Missing body name")?;
            let topology = self.call("body_topology", json!({"body":name}))?;
            measurements.push(self.call("measure", json!({"body":name}))?);
            topologies.push(topology);
        }
        let after = self.call(
            "inspect_design",
            json!({"measure":true,"sketches":false,"joints":false}),
        )?;
        for field in ["revision", "name", "bodies", "timeline", "metadata"] {
            if inspect[field] != after[field] {
                return Err("CAD changed during capture; refresh again".into());
            }
        }
        let mut clean = json!({});
        for field in ["name", "units", "revision", "bodies", "timeline"] {
            clean[field] = inspect[field].clone();
        }
        // No arbitrary metadata, filesystem paths, control credentials or server diagnostics.
        let snapshot = json!({"schema_version":1,"solidcraft_version":"0.17.0","inspect_design":clean,"body_topology":topologies,"measurements":measurements});
        crate::recognition::analyze(&snapshot.to_string())?;
        Ok(snapshot)
    }
    pub fn close(&mut self) {
        self.stdin.take();
        self.usable = false;
        let until = Instant::now() + Duration::from_millis(200);
        while Instant::now() < until {
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        self.close();
    }
}
