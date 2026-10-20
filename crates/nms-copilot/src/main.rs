//! NMS Copilot -- interactive galactic REPL for No Man's Sky.
//!
//! Modes:
//! - **Normal** (default): REPL + MCP HTTP server sharing one `GalaxyModel`
//! - **Headless** (`--headless`): MCP server only, no REPL

use std::collections::BTreeMap;
use std::fmt;
use std::io::{self, Read, Write};
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
use nms_copilot::coordinator::LiveSaveCoordinator;
use nms_copilot::mcp;
use nms_copilot::prompt::{CopilotPrompt, PromptState};
use nms_copilot::session::SessionState;
use nms_copilot::watch::drain_applied_notifications;
use nms_copilot::{commands, dispatch, paths};
use nms_graph::GalaxyModel;
use nms_watch::WatchConfig;

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
        let mut mcp_config = mcp::config::McpConfig::load();
        mcp_config.logging.output = "stderr".into();
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

    let (model, was_cached, _save_version) =
        match load_model(save_path.clone(), &cache_path, no_cache) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error loading save: {e}");
                std::process::exit(1);
            }
        };

    let mut model = model;
    model.ensure_player_system();
    let watch_config = configured_watcher(&config, save_path.as_deref());

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
        let coordinator = watch_config.and_then(|watch| {
            let live_cache = (!no_cache).then(|| cache_path.clone());
            match LiveSaveCoordinator::start(Arc::clone(&model), watch, live_cache) {
                Ok(coordinator) => Some(coordinator),
                Err(error) => {
                    eprintln!("Warning: live save coordinator unavailable: {error}");
                    None
                }
            }
        });
        let updates = coordinator.as_ref().map(LiveSaveCoordinator::subscribe);
        mcp::run_headless_with_updates(model, transport, updates);
        drop(coordinator);
        return;
    }

    // ── REPL mode ───────────────────────────────────────────────
    let mcp_addr = mcp_http_addr(&cli, &config);
    if let Err(error) = run_repl(
        model,
        config,
        was_cached,
        watch_config,
        cache_path,
        no_cache,
        mcp_addr,
    ) {
        eprintln!("MCP server failed: {error}");
        std::process::exit(1);
    }
}

/// Own normal-mode resources in a scope that unwinds on startup errors.
#[cfg(feature = "http")]
fn run_repl(
    model: GalaxyModel,
    config: Config,
    was_cached: bool,
    watch_config: Option<WatchConfig>,
    cache_path: PathBuf,
    no_cache: bool,
    mcp_addr: SocketAddr,
) -> Result<(), mcp::McpBackgroundError> {
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

    // Wrap model in Arc<RwLock> for sharing with MCP server
    let model = Arc::new(RwLock::new(model));

    // One coordinator owns watching, full-state apply and serialized cache
    // persistence. REPL and MCP receive read-only observer subscriptions.
    let coordinator = watch_config.and_then(|watch| {
        let live_cache = (!no_cache).then(|| cache_path.clone());
        match LiveSaveCoordinator::start(Arc::clone(&model), watch, live_cache) {
            Ok(coordinator) => Some(coordinator),
            Err(error) => {
                eprintln!("Warning: live save coordinator unavailable: {error}");
                None
            }
        }
    });
    let repl_updates = coordinator.as_ref().map(LiveSaveCoordinator::subscribe);
    let mcp_updates = coordinator.as_ref().map(LiveSaveCoordinator::subscribe);
    if coordinator.is_some() {
        println!("Watching save file for live updates.\n");
    } else {
        println!();
    }

    // Start an owned MCP HTTP observer on the same model and coordinator.
    let mcp_server =
        mcp::spawn_mcp_background_with_updates(Arc::clone(&model), mcp_addr, mcp_updates)?;
    let mcp_addr = mcp_server.local_addr();
    let mcp_base = format!("http://{mcp_addr}");
    eprintln!("MCP server listening on {mcp_base}");
    eprintln!("MCP endpoint: {mcp_base}/mcp");
    eprintln!("Health check: {mcp_base}/health");
    eprintln!("MCP info: {mcp_base}/mcp-info");

    let completions = build_model_completions(&model.blocking_read());
    let completer = Box::new(CopilotCompleter::new(completions));
    let mut editor = build_editor(completer);
    let mut session = SessionState::from_model(&model.blocking_read());
    session.sync_from_model(&model.blocking_read());
    if let Some(warp_range) = config.defaults.warp_range {
        session.set_warp_range(warp_range);
    }
    let mut prompt = CopilotPrompt::new(PromptState::from_session(&session));

    loop {
        if let Some(ref receiver) = repl_updates {
            for note in drain_applied_notifications(receiver) {
                println!("{note}");
            }
        }

        sync_session_before_prompt(&model, &mut session);
        prompt.update(PromptState::from_session(&session));
        match editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => match commands::parse_line(&line) {
                Ok(Some(action)) => {
                    if matches!(action, commands::Action::Exit | commands::Action::Quit) {
                        break;
                    }
                    if matches!(action, commands::Action::Map) {
                        let guard = model.blocking_read();
                        session.sync_from_model(&guard);
                        if let Err(e) = nms_copilot::map::run_map(&guard, &session) {
                            eprintln!("Map error: {e}");
                        }
                        continue;
                    }
                    let result = {
                        let mut guard = model.blocking_write();
                        dispatch_repl_action(&action, &mut guard, &mut session)
                    };
                    match result {
                        Ok(output) if !output.is_empty() => print!("{output}"),
                        Ok(_) => {}
                        Err(error) => eprintln!("Error: {error}"),
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

    if let Err(error) = mcp_server.shutdown() {
        let _ = writeln!(io::stderr().lock(), "Warning: MCP shutdown failed: {error}");
    }
    drop(coordinator);
    println!("Goodbye!");
    Ok(())
}

fn dispatch_repl_action(
    action: &commands::Action,
    model: &mut GalaxyModel,
    session: &mut SessionState,
) -> Result<String, String> {
    session.sync_from_model(model);
    dispatch::dispatch(action, model, session)
}

fn sync_session_before_prompt(model: &RwLock<GalaxyModel>, session: &mut SessionState) {
    let guard = model.blocking_read();
    session.sync_from_model(&guard);
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

fn configured_watcher(config: &Config, save_path: Option<&Path>) -> Option<WatchConfig> {
    if !config.watch_enabled() {
        return None;
    }

    Some(WatchConfig {
        save_path: save_path?.to_path_buf(),
        debounce: config.watch_debounce(),
    })
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
    use nms_copilot::session::PositionContext;
    use nms_core::address::GalacticAddress;
    use nms_core::biome::Biome;
    use nms_core::player::{BaseType, PlayerBase, PlayerState};
    use nms_core::system::{Planet, System};

    #[test]
    fn test_configured_watcher_missing_path_returns_none() {
        assert!(configured_watcher(&Config::default(), None).is_none());
    }

    #[test]
    fn test_configured_watcher_disabled_ignores_present_path_without_io() {
        let mut config = Config::default();
        config.watch.enabled = false;
        let path = Path::new("a/path/that/does/not/exist/save.hg");

        assert!(configured_watcher(&config, Some(path)).is_none());
    }

    #[test]
    fn test_configured_watcher_default_preserves_exact_path_and_debounce() {
        let config = Config::default();
        let path = Path::new("relative/../save.hg");

        let watch = configured_watcher(&config, Some(path)).expect("default watch enabled");

        assert_eq!(watch.save_path, path);
        assert_eq!(watch.debounce, Duration::from_millis(500));
    }

    #[test]
    fn test_configured_watcher_nondefault_and_zero_debounce_are_literal() {
        let path = Path::new("relative/../save.hg");
        for (debounce_ms, expected) in [(137, 137), (0, 0)] {
            let mut config = Config::default();
            config.watch.debounce_ms = debounce_ms;

            let watch = configured_watcher(&config, Some(path)).expect("watch enabled");

            assert_eq!(watch.save_path, path);
            assert_eq!(watch.debounce, Duration::from_millis(expected));
        }
    }

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

    fn repl_live_model() -> GalaxyModel {
        let mut model = GalaxyModel::new();
        for (reality, system_name, biome, planet_name) in [
            (0, "Origin-0", Biome::Lush, "Leaf-0"),
            (9, "Imported-9", Biome::Toxic, "Leaf-9"),
        ] {
            let address = GalacticAddress::new(0, 0, 0, 1, 0, reality);
            model.insert_system(System::new(
                address,
                Some(system_name.into()),
                None,
                None,
                vec![Planet::new(
                    0,
                    Some(biome),
                    None,
                    false,
                    Some(planet_name.into()),
                    None,
                )],
            ));
            model.insert_base(PlayerBase::new(
                format!("Base-{reality}"),
                BaseType::HomePlanetBase,
                address,
                [0.0; 3],
                Some("fixture-owner".into()),
            ));
        }
        model.player_state = Some(PlayerState::new(
            GalacticAddress::new(0, 0, 0, 1, 0, 0),
            0,
            None,
            None,
            7,
            8,
            9,
        ));
        model.set_active_galaxy(7);
        model
    }

    #[test]
    fn test_dispatch_repl_action_refreshes_stale_session_and_preserves_manual_anchor() {
        let mut model = repl_live_model();
        let mut anchored = SessionState::from_model(&model);
        anchored.set_position_address(GalacticAddress::new(0, 0, 0, 1, 0, 0));
        anchored.set_biome_filter(Biome::Toxic);
        anchored.set_warp_range(2500.0);

        model.player_state = Some(PlayerState::new(
            GalacticAddress::new(0, 0, 0, 1, 0, 9),
            9,
            Some(GalacticAddress::new(3, 3, 4, 1, 1, 9)),
            None,
            700,
            800,
            900,
        ));
        model.set_active_galaxy(9);

        let status = dispatch_repl_action(&commands::Action::Status, &mut model, &mut anchored)
            .expect("status after idle sync");
        assert_eq!(
            status,
            "Galaxy:      Eissentam (9; Lush)\nModel:       2 systems, 2 planets\nPosition:    0x000100000000 (Euclid, galaxy 0)\nBiome:       Toxic\nWarp range:  2500 ly\n"
        );
        assert!(matches!(
            anchored.position,
            Some(PositionContext::Address(address))
                if address == GalacticAddress::new(0, 0, 0, 1, 0, 0)
        ));
        assert_eq!(
            (
                anchored.galaxy.index,
                anchored.system_count,
                anchored.planet_count
            ),
            (9, 2, 2)
        );
        assert_eq!(
            model.player_state,
            Some(PlayerState::new(
                GalacticAddress::new(0, 0, 0, 1, 0, 9),
                9,
                Some(GalacticAddress::new(3, 3, 4, 1, 1, 9)),
                None,
                700,
                800,
                900,
            ))
        );

        let mut base_control = SessionState::from_model(&repl_live_model());
        base_control.set_position_base("Base-0", &model).unwrap();
        base_control.set_biome_filter(Biome::Toxic);
        base_control.set_warp_range(2500.0);
        let base_status =
            dispatch_repl_action(&commands::Action::Status, &mut model, &mut base_control)
                .expect("Base anchor remains supported");
        assert_eq!(
            base_status,
            "Galaxy:      Eissentam (9; Lush)\nModel:       2 systems, 2 planets\nPosition:    Base-0 (Euclid, galaxy 0)\nBiome:       Toxic\nWarp range:  2500 ly\n"
        );
        assert!(matches!(
            base_control.position,
            Some(PositionContext::Base { ref name, address })
                if name == "Base-0" && address == GalacticAddress::new(0, 0, 0, 1, 0, 0)
        ));
        assert_eq!(
            (
                base_control.galaxy.index,
                base_control.system_count,
                base_control.planet_count
            ),
            (9, 2, 2)
        );
        assert_eq!(base_control.biome_filter, Some(Biome::Toxic));
        assert_eq!(base_control.warp_range, Some(2500.0));

        let mut following = SessionState::from_model(&repl_live_model());
        following.set_biome_filter(Biome::Toxic);
        let find = commands::Action::Find {
            biome: None,
            infested: false,
            within: None,
            nearest: None,
            named: true,
            discoverer: None,
            from: None,
        };
        let found = dispatch_repl_action(&find, &mut model, &mut following)
            .expect("current-position find uses the refreshed galaxy and position");
        assert!(found.contains("Leaf-9"));
        assert!(!found.contains("Leaf-0"));
        assert_eq!(following.galaxy.index, 9);
        assert_eq!(
            *following.position.unwrap().address(),
            GalacticAddress::new(0, 0, 0, 1, 0, 9)
        );
    }

    #[test]
    fn test_prompt_sync_includes_new_named_galaxy_and_latest_player_state() {
        let model = tokio::sync::RwLock::new(repl_live_model());
        let mut manual = SessionState::from_model(&model.blocking_read());
        manual
            .set_position_base("Base-0", &model.blocking_read())
            .unwrap();
        manual.set_biome_filter(Biome::Toxic);
        manual.set_warp_range(2500.0);
        let mut following = SessionState::from_model(&model.blocking_read());
        {
            let mut latest = model.blocking_write();
            latest.insert_system(System::new(
                GalacticAddress::new(12, 13, 14, 5, 0, 9),
                Some("Third-9".into()),
                None,
                None,
                vec![Planet::new(
                    0,
                    Some(Biome::Frozen),
                    None,
                    false,
                    Some("Frost-9".into()),
                    None,
                )],
            ));
            latest.player_state = Some(PlayerState::new(
                GalacticAddress::new(12, 13, 14, 5, 0, 9),
                9,
                None,
                None,
                70,
                80,
                90,
            ));
            latest.set_active_galaxy(9);
        }

        sync_session_before_prompt(&model, &mut manual);
        sync_session_before_prompt(&model, &mut following);
        let manual_prompt = PromptState::from_session(&manual);
        let following_prompt = PromptState::from_session(&following);
        assert_eq!(manual_prompt.galaxy_name, "Eissentam");
        assert_eq!(manual_prompt.biome_filter, Some("Toxic".to_owned()));
        assert_eq!(manual_prompt.planet_count, 3);
        assert_eq!(following_prompt.galaxy_name, "Eissentam");
        assert_eq!(following_prompt.planet_count, 3);
        assert_eq!(
            (
                manual.system_count,
                manual.planet_count,
                manual.galaxy.index
            ),
            (3, 3, 9)
        );
        assert!(
            matches!(manual.position, Some(PositionContext::Base { ref name, address })
            if name == "Base-0" && address == GalacticAddress::new(0, 0, 0, 1, 0, 0))
        );
        assert_eq!(manual.biome_filter, Some(Biome::Toxic));
        assert_eq!(manual.warp_range, Some(2500.0));
        assert_eq!(
            (
                following.system_count,
                following.planet_count,
                following.galaxy.index
            ),
            (3, 3, 9)
        );
        assert!(matches!(
            following.position,
            Some(PositionContext::PlayerPosition(address))
                if address == GalacticAddress::new(12, 13, 14, 5, 0, 9)
        ));
        assert_eq!(
            model.blocking_read().player_state,
            Some(PlayerState::new(
                GalacticAddress::new(12, 13, 14, 5, 0, 9),
                9,
                None,
                None,
                70,
                80,
                90,
            ))
        );
    }

    #[test]
    fn test_prompt_sync_preserves_typed_address_anchor() {
        let model = tokio::sync::RwLock::new(repl_live_model());
        let mut session = SessionState::from_model(&model.blocking_read());
        let anchor = GalacticAddress::new(-8, 12, 4, 3, 1, 9);
        session.set_position_address(anchor);
        {
            let mut latest = model.blocking_write();
            latest.player_state = Some(PlayerState::new(
                GalacticAddress::new(1, 2, 3, 4, 0, 0),
                0,
                None,
                None,
                70,
                80,
                90,
            ));
            latest.set_active_galaxy(0);
        }

        sync_session_before_prompt(&model, &mut session);
        assert!(
            matches!(session.position, Some(PositionContext::Address(address)) if address == anchor)
        );
        assert_eq!(session.galaxy.index, 0);
        assert_eq!(session.system_count, 2);
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
