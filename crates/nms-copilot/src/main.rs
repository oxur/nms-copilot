//! NMS Copilot -- interactive galactic REPL for No Man's Sky.
//!
//! Modes:
//! - **Normal** (default): REPL + MCP HTTP server sharing one `GalaxyModel`
//! - **Headless** (`--headless`): MCP server only, no REPL

use std::collections::BTreeMap;
use std::fmt;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use clap::{Parser, Subcommand};
use reedline::{FileBackedHistory, Reedline, Signal};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::RwLock;

use nms_copilot::banner;
use nms_copilot::completer::{CopilotCompleter, ModelCompletions};
use nms_copilot::config::Config;
use nms_copilot::mcp;
use nms_copilot::prompt::{CopilotPrompt, PromptState};
use nms_copilot::session::SessionState;
use nms_copilot::watch::drain_watch_events;
use nms_copilot::{commands, dispatch, paths};
use nms_graph::GalaxyModel;
use nms_watch::{WatchConfig, WatchHandle, start_watching};

const DEFAULT_MCP_HTTP_ADDR: &str = "127.0.0.1:5099";

#[derive(Debug, Parser)]
#[command(
    name = "nms-copilot",
    about = "Interactive NMS galactic copilot REPL with MCP server support",
    version
)]
struct Cli {
    /// Path to a specific NMS save file.
    #[arg(long)]
    save: Option<PathBuf>,

    /// Disable the rkyv cache and rebuild from the save file.
    #[arg(long)]
    no_cache: bool,

    /// Run only the MCP server without starting the REPL.
    #[arg(long)]
    headless: bool,

    /// Run the interactive setup questionnaire before loading the save file.
    #[arg(long)]
    setup: bool,

    /// Use MCP HTTP transport, optionally bound to ADDR.
    ///
    /// In normal REPL mode, HTTP is always enabled; this option overrides the
    /// configured bind address. In headless mode, omitting this option uses
    /// stdio transport for MCP clients.
    #[cfg(feature = "http")]
    #[arg(long, value_name = "ADDR", num_args = 0..=1, default_missing_value = DEFAULT_MCP_HTTP_ADDR)]
    http: Option<Option<SocketAddr>>,

    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    /// Verify an HTTP MCP endpoint with initialize and tools/list.
    McpSmoke {
        /// MCP endpoint URL. Defaults to the configured /mcp endpoint.
        #[arg(long)]
        url: Option<String>,

        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: could not load config: {e}");
            Config::default()
        }
    };

    if let Some(CliCommand::McpSmoke { url, json }) = &cli.command {
        let url = url
            .clone()
            .unwrap_or_else(|| mcp_http_url(config.mcp_http_addr(), "/mcp"));
        match run_mcp_smoke(&url, *json) {
            Ok(()) => return,
            Err(e) => {
                eprintln!("MCP smoke failed: {e}");
                std::process::exit(1);
            }
        }
    }

    // In headless mode, initialize logging for the MCP server
    if cli.headless {
        let mcp_config = mcp::config::McpConfig::load();
        if let Err(e) = twyg::setup(mcp_config.logging.to_twyg_opts()) {
            eprintln!("Warning: Failed to initialize logging: {e}");
        }
    }

    // Art banner (skip in headless mode)
    if !cli.headless {
        banner::print_banner(
            config.display.banner.as_deref(),
            config.display.show_banner,
            config.display.color,
        );
    }

    let save_path = resolve_save_path(&cli, &config);
    let no_cache = cli.no_cache || !config.cache_enabled();
    let cache_path = config.cache_path_for(save_path.as_deref());

    let (model, was_cached, save_version) =
        match load_model(save_path.clone(), &cache_path, no_cache) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error loading save: {e}");
                std::process::exit(1);
            }
        };

    let mut model = model;
    model.ensure_player_system();

    // ── Headless mode: MCP server only ──────────────────────────
    if cli.headless {
        let transport = parse_mcp_transport(&cli, &config);
        let model = Arc::new(RwLock::new(model));
        eprintln!(
            "NMS Copilot MCP server (headless) — {} systems, {} planets, {} bases",
            model.blocking_read().system_count(),
            model.blocking_read().planet_count(),
            model.blocking_read().bases.len(),
        );
        mcp::run_headless(model, transport, save_path);
        return;
    }

    // ── REPL mode ───────────────────────────────────────────────

    let source = if was_cached {
        "from cache"
    } else {
        "from save file"
    };

    banner::print_system_banner(
        config.display.show_system_banner,
        model.systems.len(),
        model.planets.len(),
        model.bases.len(),
        source,
    );

    // Start file watcher (optional)
    let watch_handle = if config.watch_enabled() {
        match start_watcher(&config, save_path.clone()) {
            Ok(handle) => {
                println!("Watching save file for live updates.\n");
                Some(handle)
            }
            Err(e) => {
                eprintln!("Warning: could not start file watcher: {e}\n");
                None
            }
        }
    } else {
        println!();
        None
    };

    let cache_for_watcher = if no_cache {
        None
    } else {
        Some(cache_path.as_path())
    };

    // Wrap model in Arc<RwLock> for sharing with MCP server
    let model = Arc::new(RwLock::new(model));

    // Start MCP HTTP server on background thread (shares the model)
    let mcp_addr = mcp_http_addr(&cli, &config);
    mcp::spawn_mcp_background(
        Arc::clone(&model),
        mcp::Transport::Http(mcp_addr),
        save_path,
    );
    let mcp_base = format!("http://{mcp_addr}");
    eprintln!("MCP server listening on {mcp_base}");
    eprintln!("MCP endpoint: {mcp_base}/mcp");
    eprintln!("Health check: {mcp_base}/health");
    eprintln!("MCP info: {mcp_base}/mcp-info");

    let completions = build_model_completions(&model.blocking_read());
    let completer = Box::new(CopilotCompleter::new(completions));
    let mut editor = build_editor(completer);
    let mut session = SessionState::from_model(&model.blocking_read());
    if let Some(warp_range) = config.defaults.warp_range {
        session.set_warp_range(warp_range);
    }
    let mut prompt = CopilotPrompt::new(PromptState::from_session(&session));

    loop {
        // Drain any pending watch events before showing prompt
        if let Some(ref handle) = watch_handle {
            let mut guard = model.blocking_write();
            drain_watch_events(
                &handle.receiver,
                &mut guard,
                &mut session,
                cache_for_watcher,
                save_version,
            );
        }

        prompt.update(PromptState::from_session(&session));
        match editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => match commands::parse_line(&line) {
                Ok(Some(action)) => {
                    if matches!(action, commands::Action::Exit | commands::Action::Quit) {
                        break;
                    }
                    if matches!(action, commands::Action::Map) {
                        let guard = model.blocking_read();
                        if let Err(e) = nms_copilot::map::run_map(&guard, &session) {
                            eprintln!("Map error: {e}");
                        }
                        continue;
                    }
                    {
                        let guard = model.blocking_read();
                        match dispatch::dispatch(&action, &guard, &mut session) {
                            Ok(output) => {
                                if !output.is_empty() {
                                    print!("{output}");
                                }
                            }
                            Err(e) => eprintln!("Error: {e}"),
                        }
                    }

                    // Also drain after command execution
                    if let Some(ref handle) = watch_handle {
                        let mut guard = model.blocking_write();
                        drain_watch_events(
                            &handle.receiver,
                            &mut guard,
                            &mut session,
                            cache_for_watcher,
                            save_version,
                        );
                    }
                }
                Ok(None) => {}
                Err(e) => eprintln!("{e}"),
            },
            Ok(Signal::CtrlD | Signal::CtrlC) => {
                break;
            }
            Err(e) => {
                eprintln!("Input error: {e}");
                break;
            }
        }
    }

    println!("Goodbye!");
}

fn build_editor(completer: Box<CopilotCompleter>) -> Reedline {
    if let Err(e) = paths::ensure_data_dir() {
        eprintln!("Warning: could not create data directory: {e}");
        return Reedline::create().with_completer(completer);
    }

    let history = match FileBackedHistory::with_file(1000, paths::history_path()) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Warning: could not load history: {e}");
            return Reedline::create().with_completer(completer);
        }
    };

    Reedline::create()
        .with_history(Box::new(history))
        .with_completer(completer)
}

fn build_model_completions(model: &GalaxyModel) -> ModelCompletions {
    let base_names: Vec<String> = model.bases.values().map(|b| b.name.clone()).collect();
    let system_names: Vec<String> = model
        .systems
        .values()
        .filter_map(|s| s.name.clone())
        .collect();

    ModelCompletions {
        base_names,
        system_names,
    }
}

fn resolve_save_path(cli: &Cli, config: &Config) -> Option<PathBuf> {
    // 1. CLI arg (explicit, highest priority)
    if let Some(p) = cli.save.clone() {
        return Some(p);
    }

    // 2. Explicit interactive setup
    if cli.setup && !cli.headless {
        match nms_copilot::setup::run_setup_wizard() {
            Ok(path) => return Some(path),
            Err(e) => {
                eprintln!("Setup failed: {e}");
                eprintln!(
                    "Configure manually in ~/.nms-copilot/config.toml \
                     or use: nms-copilot --save /path/to/save.hg"
                );
                std::process::exit(1);
            }
        }
    }

    // 3. ENV vars + config file (user has explicitly configured a save)
    if let Some(p) = config.effective_save_file() {
        return Some(p);
    }

    // 4. Non-interactive fallback: auto-detect most recent save
    if let Ok(save) = nms_save::locate::find_most_recent_save() {
        return Some(save.path().to_path_buf());
    }
    None
}

fn load_model(
    save_path: Option<PathBuf>,
    cache_path: &Path,
    no_cache: bool,
) -> Result<(GalaxyModel, bool, u32), Box<dyn std::error::Error>> {
    let save = save_path.ok_or("no save file path resolved")?;
    let result = nms_cache::load_or_rebuild(cache_path, &save, no_cache)?;
    Ok((result.model, result.was_cached, result.save_version))
}

fn start_watcher(
    config: &Config,
    save_path: Option<PathBuf>,
) -> Result<WatchHandle, Box<dyn std::error::Error>> {
    let path = match save_path {
        Some(p) => p,
        None => match config.save_path() {
            Some(p) => p,
            None => nms_save::locate::find_most_recent_save()?
                .path()
                .to_path_buf(),
        },
    };

    let watch_config = WatchConfig {
        save_path: path,
        debounce: config.watch_debounce(),
    };

    Ok(start_watching(watch_config)?)
}

/// Parse MCP transport for headless mode.
///
/// If `--http <addr>` is specified, uses HTTP. Otherwise defaults to stdio.
fn parse_mcp_transport(cli: &Cli, config: &Config) -> mcp::Transport {
    if cli.http.is_some() {
        mcp::Transport::Http(mcp_http_addr(cli, config))
    } else {
        mcp::Transport::Stdio
    }
}

fn mcp_http_addr(cli: &Cli, config: &Config) -> SocketAddr {
    match cli.http {
        Some(Some(addr)) => addr,
        Some(None) => DEFAULT_MCP_HTTP_ADDR
            .parse()
            .expect("default MCP HTTP address is valid"),
        None => config.mcp_http_addr(),
    }
}

fn mcp_http_url(addr: SocketAddr, path: &str) -> String {
    format!("http://{addr}{path}")
}

#[derive(Debug, Serialize)]
struct SmokeReport {
    url: String,
    health_url: String,
    health_status: u16,
    server_name: String,
    server_version: String,
    session_id: String,
    tool_count: usize,
    tools: Vec<String>,
}

fn run_mcp_smoke(url: &str, json: bool) -> Result<(), SmokeError> {
    let endpoint = HttpEndpoint::parse(url)?;
    let health_endpoint = endpoint.with_path("/health");
    let health = http_request(&health_endpoint, HttpMethod::Get, &[], None)?;
    if health.status != 200 {
        return Err(SmokeError::UnexpectedStatus {
            context: "health check",
            status: health.status,
            body: health.body_text(),
        });
    }

    let init_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "nms-copilot-smoke",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    })
    .to_string();
    let init = mcp_post(&endpoint, None, &init_body)?;
    if init.status != 200 {
        return Err(SmokeError::UnexpectedStatus {
            context: "initialize",
            status: init.status,
            body: init.body_text(),
        });
    }
    let session_id = init
        .header("mcp-session-id")
        .ok_or(SmokeError::MissingSessionId)?
        .to_string();
    let init_messages = sse_json_messages(&init.body_text())?;
    let init_result = init_messages
        .iter()
        .find(|message| message.get("id").and_then(Value::as_i64) == Some(1))
        .and_then(|message| message.get("result"))
        .ok_or(SmokeError::MissingJsonRpcResult("initialize"))?;
    let server_info = init_result
        .get("serverInfo")
        .ok_or(SmokeError::MissingField("serverInfo"))?;
    let server_name = json_string_field(server_info, "name")?.to_string();
    let server_version = json_string_field(server_info, "version")?.to_string();

    let initialized_body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    })
    .to_string();
    let initialized = mcp_post(&endpoint, Some(&session_id), &initialized_body)?;
    if initialized.status != 202 {
        return Err(SmokeError::UnexpectedStatus {
            context: "notifications/initialized",
            status: initialized.status,
            body: initialized.body_text(),
        });
    }

    let tools_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    })
    .to_string();
    let tools_response = mcp_post(&endpoint, Some(&session_id), &tools_body)?;
    if tools_response.status != 200 {
        return Err(SmokeError::UnexpectedStatus {
            context: "tools/list",
            status: tools_response.status,
            body: tools_response.body_text(),
        });
    }
    let tools_messages = sse_json_messages(&tools_response.body_text())?;
    let tools_result = tools_messages
        .iter()
        .find(|message| message.get("id").and_then(Value::as_i64) == Some(2))
        .and_then(|message| message.get("result"))
        .ok_or(SmokeError::MissingJsonRpcResult("tools/list"))?;
    let tools = tools_result
        .get("tools")
        .and_then(Value::as_array)
        .ok_or(SmokeError::MissingField("tools"))?
        .iter()
        .map(|tool| json_string_field(tool, "name").map(str::to_string))
        .collect::<Result<Vec<_>, _>>()?;

    let report = SmokeReport {
        url: endpoint.to_url(),
        health_url: health_endpoint.to_url(),
        health_status: health.status,
        server_name,
        server_version,
        session_id,
        tool_count: tools.len(),
        tools,
    };

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(SmokeError::Json)?
        );
    } else {
        println!(
            "MCP HTTP smoke passed: {} v{}",
            report.server_name, report.server_version
        );
        println!("Health: {} ({})", report.health_status, report.health_url);
        println!("Endpoint: {}", report.url);
        println!("Session: {}", report.session_id);
        println!("Tools: {}", report.tool_count);
        for tool in &report.tools {
            println!("  - {tool}");
        }
    }

    Ok(())
}

fn json_string_field<'a>(value: &'a Value, field: &'static str) -> Result<&'a str, SmokeError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or(SmokeError::MissingField(field))
}

fn mcp_post(
    endpoint: &HttpEndpoint,
    session_id: Option<&str>,
    body: &str,
) -> Result<HttpResponse, SmokeError> {
    let mut headers = vec![
        ("Content-Type", "application/json"),
        ("Accept", "application/json, text/event-stream"),
    ];
    if let Some(session_id) = session_id {
        headers.push(("Mcp-Session-Id", session_id));
    }
    http_request(endpoint, HttpMethod::Post, &headers, Some(body))
}

#[derive(Debug, Clone, Copy)]
enum HttpMethod {
    Get,
    Post,
}

impl HttpMethod {
    fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HttpEndpoint {
    host: String,
    port: u16,
    path: String,
}

impl HttpEndpoint {
    fn parse(url: &str) -> Result<Self, SmokeError> {
        let rest = url
            .strip_prefix("http://")
            .ok_or_else(|| SmokeError::InvalidUrl(url.to_string()))?;
        let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
        if authority.is_empty() {
            return Err(SmokeError::InvalidUrl(url.to_string()));
        }
        let (host, port) = authority
            .rsplit_once(':')
            .map(|(host, port)| {
                let port = port
                    .parse::<u16>()
                    .map_err(|_| SmokeError::InvalidUrl(url.to_string()))?;
                Ok((host, port))
            })
            .unwrap_or_else(|| Ok((authority, 80)))?;
        if host.is_empty() {
            return Err(SmokeError::InvalidUrl(url.to_string()));
        }
        Ok(Self {
            host: host.to_string(),
            port,
            path: format!("/{path}"),
        })
    }

    fn socket_addr(&self) -> Result<SocketAddr, SmokeError> {
        (self.host.as_str(), self.port)
            .to_socket_addrs()
            .map_err(SmokeError::Io)?
            .next()
            .ok_or_else(|| SmokeError::InvalidUrl(self.to_url()))
    }

    fn host_header(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    fn to_url(&self) -> String {
        format!("http://{}:{}{}", self.host, self.port, self.path)
    }

    fn with_path(&self, path: &str) -> Self {
        Self {
            host: self.host.clone(),
            port: self.port,
            path: path.to_string(),
        }
    }
}

#[derive(Debug)]
struct HttpResponse {
    status: u16,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

impl HttpResponse {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
    }

    fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

fn http_request(
    endpoint: &HttpEndpoint,
    method: HttpMethod,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> Result<HttpResponse, SmokeError> {
    let mut stream = TcpStream::connect_timeout(&endpoint.socket_addr()?, Duration::from_secs(5))
        .map_err(SmokeError::Io)?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(SmokeError::Io)?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(SmokeError::Io)?;

    let body = body.unwrap_or("");
    let mut request = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n",
        method.as_str(),
        endpoint.path,
        endpoint.host_header()
    );
    for (name, value) in headers {
        request.push_str(name);
        request.push_str(": ");
        request.push_str(value);
        request.push_str("\r\n");
    }
    if !body.is_empty() {
        request.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    request.push_str("\r\n");
    request.push_str(body);

    stream
        .write_all(request.as_bytes())
        .map_err(SmokeError::Io)?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).map_err(SmokeError::Io)?;
    parse_http_response(&response)
}

fn parse_http_response(response: &[u8]) -> Result<HttpResponse, SmokeError> {
    let split = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or(SmokeError::InvalidHttpResponse)?;
    let (head, body) = response.split_at(split + 4);
    let head = std::str::from_utf8(head).map_err(|_| SmokeError::InvalidHttpResponse)?;
    let mut lines = head.lines();
    let status_line = lines.next().ok_or(SmokeError::InvalidHttpResponse)?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .ok_or(SmokeError::InvalidHttpResponse)?
        .parse::<u16>()
        .map_err(|_| SmokeError::InvalidHttpResponse)?;

    let mut headers = BTreeMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.to_ascii_lowercase(), value.trim().to_string());
        }
    }

    let body = if headers
        .get("transfer-encoding")
        .is_some_and(|value| value.eq_ignore_ascii_case("chunked"))
    {
        decode_chunked(body)?
    } else {
        body.to_vec()
    };

    Ok(HttpResponse {
        status,
        headers,
        body,
    })
}

fn decode_chunked(mut body: &[u8]) -> Result<Vec<u8>, SmokeError> {
    let mut decoded = Vec::new();
    loop {
        let line_end = body
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or(SmokeError::InvalidChunkedResponse)?;
        let size_line = std::str::from_utf8(&body[..line_end])
            .map_err(|_| SmokeError::InvalidChunkedResponse)?;
        let size_hex = size_line.split(';').next().unwrap_or(size_line);
        let size = usize::from_str_radix(size_hex.trim(), 16)
            .map_err(|_| SmokeError::InvalidChunkedResponse)?;
        body = &body[line_end + 2..];
        if size == 0 {
            return Ok(decoded);
        }
        if body.len() < size + 2 || &body[size..size + 2] != b"\r\n" {
            return Err(SmokeError::InvalidChunkedResponse);
        }
        decoded.extend_from_slice(&body[..size]);
        body = &body[size + 2..];
    }
}

fn sse_json_messages(body: &str) -> Result<Vec<Value>, SmokeError> {
    let mut messages = Vec::new();
    for event in body.split("\n\n") {
        let mut data = String::new();
        for line in event.lines() {
            if let Some(value) = line.strip_prefix("data:") {
                data.push_str(value.trim_start());
            }
        }
        let trimmed = data.trim();
        if trimmed.starts_with('{') {
            messages.push(serde_json::from_str(trimmed).map_err(SmokeError::Json)?);
        }
    }
    Ok(messages)
}

#[derive(Debug)]
enum SmokeError {
    InvalidUrl(String),
    Io(std::io::Error),
    InvalidHttpResponse,
    InvalidChunkedResponse,
    Json(serde_json::Error),
    MissingSessionId,
    MissingJsonRpcResult(&'static str),
    MissingField(&'static str),
    UnexpectedStatus {
        context: &'static str,
        status: u16,
        body: String,
    },
}

impl fmt::Display for SmokeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUrl(url) => write!(f, "invalid HTTP URL: {url}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::InvalidHttpResponse => write!(f, "invalid HTTP response"),
            Self::InvalidChunkedResponse => write!(f, "invalid chunked HTTP response"),
            Self::Json(e) => write!(f, "invalid JSON: {e}"),
            Self::MissingSessionId => {
                write!(f, "initialize response did not include Mcp-Session-Id")
            }
            Self::MissingJsonRpcResult(method) => {
                write!(f, "{method} response did not include a JSON-RPC result")
            }
            Self::MissingField(field) => write!(f, "response is missing {field}"),
            Self::UnexpectedStatus {
                context,
                status,
                body,
            } => {
                write!(f, "{context} returned HTTP {status}")?;
                if !body.trim().is_empty() {
                    write!(f, ": {}", body.trim())?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for SmokeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_help_parses_before_startup() {
        let err = Cli::try_parse_from(["nms-copilot", "--help"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn test_setup_flag_parses() {
        let cli = Cli::try_parse_from(["nms-copilot", "--setup"]).unwrap();
        assert!(cli.setup);
    }

    #[test]
    fn test_mcp_smoke_command_parses() {
        let cli = Cli::try_parse_from([
            "nms-copilot",
            "mcp-smoke",
            "--url",
            "http://127.0.0.1:5055/mcp",
            "--json",
        ])
        .unwrap();

        assert!(matches!(
            cli.command,
            Some(CliCommand::McpSmoke { json: true, .. })
        ));
    }

    #[test]
    fn test_mcp_addr_defaults_to_config() {
        let cli = Cli::try_parse_from(["nms-copilot"]).unwrap();
        let mut config = Config::default();
        config.mcp.port = 5055;

        assert_eq!(mcp_http_addr(&cli, &config).to_string(), "127.0.0.1:5055");
    }

    #[test]
    fn test_mcp_addr_http_flag_uses_default() {
        let cli = Cli::try_parse_from(["nms-copilot", "--http"]).unwrap();

        assert_eq!(
            mcp_http_addr(&cli, &Config::default()).to_string(),
            DEFAULT_MCP_HTTP_ADDR
        );
    }

    #[test]
    fn test_mcp_addr_http_arg_overrides_config() {
        let cli = Cli::try_parse_from(["nms-copilot", "--http", "127.0.0.1:5055"]).unwrap();
        let config = Config::default();

        assert_eq!(mcp_http_addr(&cli, &config).to_string(), "127.0.0.1:5055");
    }

    #[test]
    fn test_mcp_http_url_uses_configured_addr() {
        let mut config = Config::default();
        config.mcp.port = 5055;

        assert_eq!(
            mcp_http_url(config.mcp_http_addr(), "/mcp"),
            "http://127.0.0.1:5055/mcp"
        );
    }

    #[test]
    fn test_http_endpoint_parse() {
        let endpoint = HttpEndpoint::parse("http://127.0.0.1:5055/mcp").unwrap();

        assert_eq!(endpoint.host, "127.0.0.1");
        assert_eq!(endpoint.port, 5055);
        assert_eq!(endpoint.path, "/mcp");
        assert_eq!(endpoint.to_url(), "http://127.0.0.1:5055/mcp");
        assert_eq!(
            endpoint.with_path("/health").to_url(),
            "http://127.0.0.1:5055/health"
        );
    }

    #[test]
    fn test_sse_json_messages_extracts_json_rpc_events() {
        let body = r#"data: 
id: 0
retry: 3000

data: {"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"health"}]}}
id: 0/0

"#;

        let messages = sse_json_messages(body).unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["id"], 2);
        assert_eq!(messages[0]["result"]["tools"][0]["name"], "health");
    }
}
