//! Native MCP child-process coverage for real save-file movement.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tempfile::TempDir;

const PLAYER_LOCATION_URI: &str = "nms://player/location";
const PHASE_TIMEOUT: Duration = Duration::from_secs(8);
const RPC_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy)]
struct Phase {
    label: &'static str,
    reality_index: u8,
    voxel_x: i16,
    galaxy: &'static str,
    glyph_hex: &'static str,
}

const PHASE_A: Phase = Phase {
    label: "A",
    reality_index: 0,
    voxel_x: 0,
    galaxy: "Euclid",
    glyph_hex: "000100000000",
};
const PHASE_B: Phase = Phase {
    label: "B",
    reality_index: 9,
    voxel_x: 0,
    galaxy: "Eissentam",
    glyph_hex: "000100000000",
};
const PHASE_C: Phase = Phase {
    label: "C",
    reality_index: 9,
    voxel_x: 1,
    galaxy: "Eissentam",
    glyph_hex: "000100000001",
};
const PHASE_D: Phase = Phase {
    label: "D",
    reality_index: 0,
    voxel_x: 0,
    galaxy: "Euclid",
    glyph_hex: "000100000000",
};

fn save_json(phase: Phase) -> Value {
    json!({
        "Version": 4720,
        "Platform": "Mac|Final",
        "ActiveContext": "Main",
        "CommonStateData": {"SaveName": "LiveWatch", "TotalPlayTime": 100},
        "BaseContext": {
            "GameMode": 1,
            "PlayerStateData": {
                "UniverseAddress": {
                    "RealityIndex": phase.reality_index,
                    "GalacticAddress": {
                        "VoxelX": phase.voxel_x,
                        "VoxelY": 0,
                        "VoxelZ": 0,
                        "SolarSystemIndex": 1,
                        "PlanetIndex": 0
                    }
                },
                "Units": 7,
                "Nanites": 8,
                "Specials": 9,
                "PersistentPlayerBases": []
            }
        },
        "ExpeditionContext": {
            "GameMode": 6,
            "PlayerStateData": {
                "UniverseAddress": {
                    "RealityIndex": 0,
                    "GalacticAddress": {
                        "VoxelX": 0,
                        "VoxelY": 0,
                        "VoxelZ": 0,
                        "SolarSystemIndex": 0,
                        "PlanetIndex": 0
                    }
                },
                "Units": 0,
                "Nanites": 0,
                "Specials": 0,
                "PersistentPlayerBases": []
            }
        },
        "DiscoveryManagerData": {
            "DiscoveryData-v1": {
                "ReserveStore": 0,
                "ReserveManaged": 0,
                "Store": {"Record": []}
            }
        }
    })
}

fn save_bytes(phase: Phase) -> Vec<u8> {
    let bytes = serde_json::to_vec(&save_json(phase)).expect("fixture serializes");
    nms_save::parse_save(&bytes).expect("fixture is accepted by the production save parser");
    bytes
}

fn atomic_replace(path: &Path, bytes: &[u8]) {
    let replacement = path.with_extension("replacement");
    std::fs::write(&replacement, bytes).expect("write replacement save beside target");
    std::fs::rename(&replacement, path).expect("atomically replace save on the same filesystem");
}

fn temp_save() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("temporary save directory");
    let path = dir.path().join("save.json");
    std::fs::write(&path, save_bytes(PHASE_A)).expect("write initial save");
    (dir, path)
}

fn assert_location(location: &Value, phase: Phase) -> String {
    assert_eq!(
        location["galaxy"], phase.galaxy,
        "phase {} galaxy",
        phase.label
    );
    assert_eq!(
        location["voxel_x"], phase.voxel_x,
        "phase {} X",
        phase.label
    );
    assert_eq!(location["voxel_y"], 0, "phase {} Y", phase.label);
    assert_eq!(location["voxel_z"], 0, "phase {} Z", phase.label);
    assert_eq!(
        location["solar_system_index"], 1,
        "phase {} SSI",
        phase.label
    );
    assert_eq!(
        location["portal_glyphs_hex"], phase.glyph_hex,
        "phase {} glyphs",
        phase.label
    );
    let system = location["system"]
        .as_str()
        .expect("location includes its existing system label")
        .to_owned();
    assert!(location["portal_glyphs_emoji"].is_string());
    system
}

fn result_text(result: &Value) -> &str {
    result["content"][0]["text"]
        .as_str()
        .expect("MCP result contains text")
}

fn parse_text_json(result: &Value) -> Value {
    serde_json::from_str(result_text(result)).expect("MCP text content is JSON")
}

fn rpc(method: &str, id: u64) -> Value {
    json!({"jsonrpc":"2.0", "id":id, "method":method, "params":{}})
}

fn initialize_request() -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {"resources": {"subscribe": true, "listChanged": false}},
            "clientInfo": {"name": "nms-live-watch-integration", "version": "1.0"}
        }
    })
}

fn assert_initialized(result: &Value) {
    assert!(
        result["protocolVersion"].as_str().is_some(),
        "server negotiated a protocol version"
    );
    assert_eq!(result["serverInfo"]["name"], "nms-copilot");
}

fn assert_lists(tools: &Value, resources: &Value) {
    assert!(
        tools["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool["name"] == "where_am_i")
    );
    assert!(
        resources["resources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|resource| resource["uri"] == PLAYER_LOCATION_URI)
    );
}

fn call_where_am_i_stdio(
    process: &mut StdioProcess,
    id: u64,
    phase: Phase,
    deadline: Instant,
) -> String {
    process.send(json!({
        "jsonrpc":"2.0", "id":id, "method":"tools/call",
        "params":{"name":"where_am_i", "arguments":{}}
    }));
    let response = process.response(id, remaining(deadline));
    let tool_location = parse_text_json(&response["result"]);
    let system = assert_location(&tool_location, phase);

    process.send(json!({
        "jsonrpc":"2.0", "id":id + 1, "method":"resources/read",
        "params":{"uri":PLAYER_LOCATION_URI}
    }));
    let resource = process.response(id + 1, remaining(deadline));
    let contents = &resource["result"]["contents"][0]["text"];
    let value: Value = serde_json::from_str(contents.as_str().expect("location text exists"))
        .expect("location resource text is JSON");
    let _ = assert_location(&value, phase);
    eprintln!(
        "MCP_LIVE_OBSERVATION transport=stdio phase={} tool={} resource={}",
        phase.label,
        serde_json::to_string(&tool_location).expect("serialize actual tool location"),
        serde_json::to_string(&value).expect("serialize actual resource location")
    );
    system
}

fn call_where_am_i_http(
    client: &mut HttpClient,
    id: u64,
    phase: Phase,
    deadline: Instant,
) -> String {
    let tool = client.post_with_timeout(
        json!({
            "jsonrpc":"2.0", "id":id, "method":"tools/call",
            "params":{"name":"where_am_i", "arguments":{}}
        }),
        remaining(deadline),
    );
    assert_eq!(
        tool.status,
        200,
        "tools/call response: {}",
        tool.body_text()
    );
    let response = http_json_messages(&tool.body_text())
        .into_iter()
        .find(|message| message["id"] == id)
        .expect("HTTP tools/call response ID");
    let tool_location = parse_text_json(&response["result"]);
    let system = assert_location(&tool_location, phase);

    let resource = client.post_with_timeout(
        json!({
            "jsonrpc":"2.0", "id":id + 1, "method":"resources/read",
            "params":{"uri":PLAYER_LOCATION_URI}
        }),
        remaining(deadline),
    );
    assert_eq!(
        resource.status,
        200,
        "resources/read response: {}",
        resource.body_text()
    );
    let response = http_json_messages(&resource.body_text())
        .into_iter()
        .find(|message| message["id"] == id + 1)
        .expect("HTTP resources/read response ID");
    let contents = &response["result"]["contents"][0]["text"];
    let value: Value = serde_json::from_str(contents.as_str().expect("location text exists"))
        .expect("location resource text is JSON");
    let _ = assert_location(&value, phase);
    eprintln!(
        "MCP_LIVE_OBSERVATION transport=http phase={} tool={} resource={}",
        phase.label,
        serde_json::to_string(&tool_location).expect("serialize actual tool location"),
        serde_json::to_string(&value).expect("serialize actual resource location")
    );
    system
}

fn test_stdio_child_tracks_real_save_and_recovers_after_parse_failures() {
    let (_dir, save) = temp_save();
    let mut process = StdioProcess::spawn(&save);

    process.send(initialize_request());
    let initialized = process.response(1, RPC_TIMEOUT);
    assert_initialized(&initialized["result"]);
    process.send(json!({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}));

    process.send(rpc("tools/list", 2));
    let tools = process.response(2, RPC_TIMEOUT);
    process.send(rpc("resources/list", 3));
    let resources = process.response(3, RPC_TIMEOUT);
    assert_lists(&tools["result"], &resources["result"]);
    let initial_system =
        call_where_am_i_stdio(&mut process, 10, PHASE_A, Instant::now() + RPC_TIMEOUT);

    process.send(json!({
        "jsonrpc":"2.0", "id":20, "method":"resources/subscribe",
        "params":{"uri":PLAYER_LOCATION_URI}
    }));
    let subscribed = process.response(20, RPC_TIMEOUT);
    assert!(
        subscribed.get("result").is_some(),
        "subscription succeeded: {subscribed}"
    );

    let phase_b_deadline = Instant::now() + PHASE_TIMEOUT;
    atomic_replace(&save, &save_bytes(PHASE_B));
    process.wait_location_update(remaining(phase_b_deadline));
    let galaxy9_system = call_where_am_i_stdio(&mut process, 30, PHASE_B, phase_b_deadline);

    // Drive distinct filesystem events at >500ms debounce +100ms stability.
    // The existing five-failure warning is the evidence that parsing occurred.
    let warning_deadline = Instant::now() + Duration::from_secs(12);
    for attempt in 1..=12 {
        assert!(
            Instant::now() < warning_deadline,
            "warning deadline elapsed; stderr: {}",
            process.stderr_text()
        );
        let invalid = format!("{{{{ invalid save phase B attempt {attempt} }}}}").into_bytes();
        atomic_replace(&save, &invalid);
        if process.stderr_contains("5 consecutive parse failures") {
            break;
        }
        let delay = Duration::from_millis(650);
        if Instant::now() + delay >= warning_deadline {
            break;
        }
        thread::sleep(delay);
    }
    assert!(
        process.wait_for_stderr("5 consecutive parse failures", warning_deadline),
        "watcher did not report five actual parse failures by the 12s deadline; stderr: {}",
        process.stderr_text()
    );
    let retained_system =
        call_where_am_i_stdio(&mut process, 40, PHASE_B, Instant::now() + RPC_TIMEOUT);
    assert_eq!(retained_system, galaxy9_system);

    let phase_c_deadline = Instant::now() + PHASE_TIMEOUT;
    atomic_replace(&save, &save_bytes(PHASE_C));
    process.wait_location_update(remaining(phase_c_deadline));
    let system_c = call_where_am_i_stdio(&mut process, 50, PHASE_C, phase_c_deadline);
    assert_eq!(system_c, galaxy9_system);

    let phase_d_deadline = Instant::now() + PHASE_TIMEOUT;
    atomic_replace(&save, &save_bytes(PHASE_D));
    process.wait_location_update(remaining(phase_d_deadline));
    let system_d = call_where_am_i_stdio(&mut process, 60, PHASE_D, phase_d_deadline);
    assert_eq!(system_d, initial_system);
    assert!(process.stdout_is_json_only());
    process.close_stdin_and_wait(Duration::from_secs(5));
}

#[test]
fn test_stdio_mcp_child_observes_atomic_save_replacements_and_recovers() {
    test_stdio_child_tracks_real_save_and_recovers_after_parse_failures();
}

#[cfg(feature = "http")]
#[test]
fn test_http_mcp_child_tracks_save_with_session_and_live_sse() {
    let (_dir, save) = temp_save();
    let mut last_error = String::new();
    let mut client = None;
    let mut process = None;

    // The reservation is closed before the child binds. Retry only when the
    // child's actual diagnostic identifies a bind collision.
    for _attempt in 0..3 {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("reserve loopback port");
        let addr = listener.local_addr().expect("reserved socket address");
        drop(listener);
        let mut child = HttpChild::spawn(&save, addr);
        match wait_health(addr, &mut child) {
            Ok(()) => {
                process = Some(child);
                client = Some(HttpClient::new(addr));
                break;
            }
            Err(message) if message.contains("bind") => {
                last_error = message;
            }
            Err(message) => panic!("HTTP child did not become healthy: {message}"),
        }
    }
    let mut process =
        process.unwrap_or_else(|| panic!("three loopback bind collisions: {last_error}"));
    let mut client = client.expect("HTTP client established after health readiness");

    let initialize = client.post_without_session(initialize_request());
    assert_eq!(
        initialize.status,
        200,
        "initialize: {}",
        initialize.body_text()
    );
    let session = initialize
        .header("mcp-session-id")
        .expect("server returned a real MCP session ID")
        .to_owned();
    let result = http_json_messages(&initialize.body_text())
        .into_iter()
        .find(|message| message["id"] == 1)
        .expect("initialize response ID");
    assert_initialized(&result["result"]);

    let initialized =
        client.post(json!({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}));
    assert_eq!(initialized.status, 202, "initialized notification");
    let mut event_stream = SseStream::connect(client.addr, &session);

    let tools = client.post(rpc("tools/list", 2));
    assert_eq!(tools.status, 200);
    let tools = http_json_messages(&tools.body_text())
        .into_iter()
        .find(|message| message["id"] == 2)
        .expect("tools/list response ID");
    let resources = client.post(rpc("resources/list", 3));
    assert_eq!(resources.status, 200);
    let resources = http_json_messages(&resources.body_text())
        .into_iter()
        .find(|message| message["id"] == 3)
        .expect("resources/list response ID");
    assert_lists(&tools["result"], &resources["result"]);
    let _ = call_where_am_i_http(&mut client, 10, PHASE_A, Instant::now() + RPC_TIMEOUT);

    let subscribed = client.post(json!({
        "jsonrpc":"2.0", "id":20, "method":"resources/subscribe",
        "params":{"uri":PLAYER_LOCATION_URI}
    }));
    assert_eq!(
        subscribed.status,
        200,
        "subscription response: {}",
        subscribed.body_text()
    );
    let subscription_response = http_json_messages(&subscribed.body_text())
        .into_iter()
        .find(|message| message["id"] == 20)
        .expect("HTTP subscription response ID");
    assert!(
        subscription_response["result"].is_object() && subscription_response.get("error").is_none(),
        "HTTP subscription succeeded: {subscription_response}"
    );

    for (phase, id) in [(PHASE_B, 30), (PHASE_C, 40), (PHASE_D, 50)] {
        let phase_started = Instant::now();
        let phase_deadline = phase_started + PHASE_TIMEOUT;
        atomic_replace(&save, &save_bytes(phase));
        event_stream.wait_location_update(phase_deadline);
        let _ = call_where_am_i_http(&mut client, id, phase, phase_deadline);
        assert!(
            phase_started.elapsed() < PHASE_TIMEOUT,
            "phase {} exceeded 8s",
            phase.label
        );
    }

    let smoke = Command::new(assert_cmd::cargo::cargo_bin!("nms-copilot"))
        .args([
            "mcp-smoke",
            "--url",
            &format!("http://{}/mcp", client.addr),
            "--json",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run mcp-smoke against the real child");
    assert!(
        smoke.status.success(),
        "mcp-smoke stderr: {}",
        String::from_utf8_lossy(&smoke.stderr)
    );
    let smoke_report: Value = serde_json::from_slice(&smoke.stdout).expect("mcp-smoke JSON output");
    assert_eq!(smoke_report["health_status"], 200);
    assert!(
        smoke_report["tools"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    );
    assert!(
        process.stdout_is_empty(),
        "HTTP child wrote protocol/log bytes to stdout"
    );
    event_stream.close();
    process.kill_and_wait();
}

struct StdioProcess {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: Receiver<Result<String, String>>,
    stderr: Receiver<Result<String, String>>,
    stderr_lines: Vec<String>,
    pending: Vec<Value>,
    last_id: u64,
}

impl StdioProcess {
    fn spawn(save: &Path) -> Self {
        let mut child = Command::new(assert_cmd::cargo::cargo_bin!("nms-copilot"))
            .args(["--headless", "--no-cache", "--save"])
            .arg(save)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn compiled nms-copilot child");
        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = lines_channel(child.stdout.take().expect("piped stdout"));
        let stderr = lines_channel(child.stderr.take().expect("piped stderr"));
        Self {
            child,
            stdin: Some(stdin),
            stdout,
            stderr,
            stderr_lines: Vec::new(),
            pending: Vec::new(),
            last_id: 100,
        }
    }

    fn send(&mut self, value: Value) {
        self.last_id = self.last_id.max(value["id"].as_u64().unwrap_or(0));
        let stdin = self.stdin.as_mut().expect("stdio remains open");
        serde_json::to_writer(&mut *stdin, &value).expect("write JSON-RPC request");
        stdin.write_all(b"\n").expect("write JSON-RPC newline");
        stdin.flush().expect("flush JSON-RPC request");
    }

    fn read_message(&mut self, deadline: Instant, expected: &str) -> Value {
        self.drain_stderr();
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(
            !remaining.is_zero(),
            "stdio RPC deadline elapsed waiting for {expected}; stderr: {}",
            self.stderr_text()
        );
        match self.stdout.recv_timeout(remaining) {
            Ok(Ok(line)) => serde_json::from_str(&line).unwrap_or_else(|error| {
                panic!(
                    "stdout is not one JSON-RPC object ({error}): {line:?}; stderr: {}",
                    self.stderr_text()
                )
            }),
            Ok(Err(error)) => panic!("read stdout: {error}; stderr: {}", self.stderr_text()),
            Err(error) => panic!(
                "waiting for stdio JSON-RPC {expected}: {error}; stderr: {}",
                self.stderr_text()
            ),
        }
    }

    fn response(&mut self, id: u64, timeout: Duration) -> Value {
        if let Some(index) = self.pending.iter().position(|message| message["id"] == id) {
            return self.pending.remove(index);
        }
        let deadline = Instant::now() + timeout;
        loop {
            let message = self.read_message(deadline, &format!("response id {id}"));
            if message["id"] == id {
                return message;
            }
            self.pending.push(message);
        }
    }

    fn wait_location_update(&mut self, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(index) = self.pending.iter().position(is_location_update) {
                self.pending.remove(index);
                return;
            }
            let message = self.read_message(deadline, "resources/updated notification");
            if is_location_update(&message) {
                return;
            }
            self.pending.push(message);
        }
    }

    fn drain_stderr(&mut self) {
        while let Ok(item) = self.stderr.try_recv() {
            self.stderr_lines
                .push(item.unwrap_or_else(|error| format!("stderr read error: {error}")));
        }
    }

    fn stderr_contains(&mut self, needle: &str) -> bool {
        self.drain_stderr();
        self.stderr_lines.iter().any(|line| line.contains(needle))
    }

    fn wait_for_stderr(&mut self, needle: &str, deadline: Instant) -> bool {
        loop {
            if self.stderr_contains(needle) {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            match self.stderr.recv_timeout(Duration::from_millis(50)) {
                Ok(item) => self
                    .stderr_lines
                    .push(item.unwrap_or_else(|error| format!("stderr read error: {error}"))),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return false,
            }
        }
    }

    fn stderr_text(&mut self) -> String {
        self.drain_stderr();
        self.stderr_lines.join("\n")
    }

    fn stdout_is_json_only(&mut self) -> bool {
        self.drain_stderr();
        self.pending
            .iter()
            .all(|message| message.get("jsonrpc").is_some())
    }

    fn close_stdin_and_wait(&mut self, timeout: Duration) {
        drop(self.stdin.take());
        let deadline = Instant::now() + timeout;
        loop {
            self.drain_stderr();
            if let Some(status) = self.child.try_wait().expect("poll stdio child") {
                assert!(
                    status.success(),
                    "stdio child exit {status}; stderr: {}",
                    self.stderr_text()
                );
                return;
            }
            assert!(
                Instant::now() < deadline,
                "stdio child did not exit within 5s; stderr: {}",
                self.stderr_text()
            );
            thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for StdioProcess {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

struct HttpChild {
    child: Child,
    stdout: Receiver<Result<String, String>>,
    stderr: Receiver<Result<String, String>>,
    stdout_lines: Vec<String>,
    stderr_lines: Vec<String>,
}

impl HttpChild {
    fn spawn(save: &Path, addr: SocketAddr) -> Self {
        let mut child = Command::new(assert_cmd::cargo::cargo_bin!("nms-copilot"))
            .args(["--headless", "--no-cache", "--save"])
            .arg(save)
            .args(["--http", &addr.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn compiled nms-copilot HTTP child");
        let stdout = lines_channel(child.stdout.take().expect("piped stdout"));
        let stderr = lines_channel(child.stderr.take().expect("piped stderr"));
        Self {
            child,
            stdout,
            stderr,
            stdout_lines: Vec::new(),
            stderr_lines: Vec::new(),
        }
    }

    fn collect(&mut self) {
        while let Ok(Ok(line)) = self.stdout.try_recv() {
            self.stdout_lines.push(line);
        }
        while let Ok(item) = self.stderr.try_recv() {
            self.stderr_lines
                .push(item.unwrap_or_else(|error| format!("stderr read error: {error}")));
        }
    }

    fn stderr_text(&mut self) -> String {
        self.collect();
        self.stderr_lines.join("\n")
    }

    fn stdout_is_empty(&mut self) -> bool {
        self.collect();
        self.stdout_lines.is_empty()
    }

    fn kill_and_wait(&mut self) {
        if self.child.try_wait().expect("poll HTTP child").is_none() {
            self.child.kill().expect("kill HTTP child");
        }
        let status = self.child.wait().expect("wait HTTP child");
        assert!(
            !status.success(),
            "HTTP test child was expected to be explicitly killed"
        );
        self.collect();
        assert!(
            self.stdout_lines.is_empty(),
            "HTTP child stdout: {:?}",
            self.stdout_lines
        );
    }
}

impl Drop for HttpChild {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn lines_channel<R: Read + Send + 'static>(reader: R) -> Receiver<Result<String, String>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(reader).lines() {
            let item = line.map_err(|error| error.to_string());
            if sender.send(item).is_err() {
                break;
            }
        }
    });
    receiver
}

fn is_location_update(message: &Value) -> bool {
    message["method"] == "notifications/resources/updated"
        && message["params"]["uri"] == PLAYER_LOCATION_URI
}

fn wait_health(addr: SocketAddr, child: &mut HttpChild) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Ok(response) = http_request(
            addr,
            "GET",
            "/health",
            &[],
            None,
            Duration::from_millis(150),
        ) && response.status == 200
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(child.stderr_text());
        }
        thread::sleep(Duration::from_millis(30));
    }
}

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

struct HttpClient {
    addr: SocketAddr,
    session: Option<String>,
}

impl HttpClient {
    fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            session: None,
        }
    }

    fn post_without_session(&mut self, body: Value) -> HttpResponse {
        let response = http_request(
            self.addr,
            "POST",
            "/mcp",
            &[],
            Some(&body.to_string()),
            RPC_TIMEOUT,
        )
        .expect("HTTP POST");
        if let Some(session) = response.header("mcp-session-id") {
            self.session = Some(session.to_owned());
        }
        response
    }

    fn post(&mut self, body: Value) -> HttpResponse {
        self.post_with_timeout(body, RPC_TIMEOUT)
    }

    fn post_with_timeout(&mut self, body: Value, timeout: Duration) -> HttpResponse {
        let session = self.session.as_deref().expect("initialized HTTP session");
        let encoded = body.to_string();
        http_request(
            self.addr,
            "POST",
            "/mcp",
            &[("Mcp-Session-Id", session)],
            Some(&encoded),
            timeout,
        )
        .expect("session HTTP POST")
    }
}

fn remaining(deadline: Instant) -> Duration {
    let remaining = deadline.saturating_duration_since(Instant::now());
    assert!(
        !remaining.is_zero(),
        "shared 8-second protocol phase deadline elapsed"
    );
    remaining
}

fn http_request(
    addr: SocketAddr,
    method: &str,
    path: &str,
    extra_headers: &[(&str, &str)],
    body: Option<&str>,
    timeout: Duration,
) -> Result<HttpResponse, String> {
    let mut stream =
        TcpStream::connect_timeout(&addr, timeout).map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|error| error.to_string())?;
    let body = body.unwrap_or("");
    let mut request = format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n");
    if method == "POST" {
        request.push_str(
            "Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\n",
        );
    }
    for (name, value) in extra_headers {
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
        .map_err(|error| error.to_string())?;
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|error| error.to_string())?;
    parse_http_response(&response)
}

fn parse_http_response(response: &[u8]) -> Result<HttpResponse, String> {
    let split = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| "HTTP response has no header terminator".to_owned())?;
    let head = std::str::from_utf8(&response[..split]).map_err(|error| error.to_string())?;
    let mut lines = head.lines();
    let status = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or_else(|| "HTTP response has no status".to_owned())?
        .parse::<u16>()
        .map_err(|error| error.to_string())?;
    let mut headers = BTreeMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.to_ascii_lowercase(), value.trim().to_owned());
        }
    }
    let body = &response[split + 4..];
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

fn decode_chunked(mut input: &[u8]) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    loop {
        let end = input
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or_else(|| "chunk size line missing".to_owned())?;
        let size_line = std::str::from_utf8(&input[..end]).map_err(|error| error.to_string())?;
        let size =
            usize::from_str_radix(size_line.split(';').next().unwrap_or(size_line).trim(), 16)
                .map_err(|error| error.to_string())?;
        input = &input[end + 2..];
        if size == 0 {
            return Ok(output);
        }
        if input.len() < size + 2 || &input[size..size + 2] != b"\r\n" {
            return Err("truncated chunk".to_owned());
        }
        output.extend_from_slice(&input[..size]);
        input = &input[size + 2..];
    }
}

fn http_json_messages(body: &str) -> Vec<Value> {
    if let Ok(value) = serde_json::from_str::<Value>(body) {
        return vec![value];
    }
    parse_sse_events(body)
}

fn parse_sse_events(body: &str) -> Vec<Value> {
    body.replace("\r\n", "\n")
        .split("\n\n")
        .filter_map(|event| {
            let data = event
                .lines()
                .filter_map(|line| line.strip_prefix("data:"))
                .map(str::trim_start)
                .collect::<String>();
            serde_json::from_str::<Value>(&data).ok()
        })
        .collect()
}

struct SseStream {
    messages: Receiver<Value>,
    socket: Option<TcpStream>,
}

impl SseStream {
    fn connect(addr: SocketAddr, session: &str) -> Self {
        let mut stream =
            TcpStream::connect_timeout(&addr, RPC_TIMEOUT).expect("connect SSE stream");
        stream
            .set_write_timeout(Some(RPC_TIMEOUT))
            .expect("set SSE write timeout");
        write!(
            stream,
            "GET /mcp HTTP/1.1\r\nHost: {addr}\r\nConnection: keep-alive\r\nAccept: text/event-stream\r\nMcp-Session-Id: {session}\r\n\r\n"
        ).expect("write actual GET event-stream request");
        let reader_stream = stream.try_clone().expect("clone SSE socket for reader");
        let (status_tx, status_rx) = mpsc::channel();
        let (message_tx, message_rx) = mpsc::channel();
        thread::spawn(move || read_sse_stream(reader_stream, status_tx, message_tx));
        let status = status_rx
            .recv_timeout(RPC_TIMEOUT)
            .expect("SSE response headers");
        assert_eq!(status, 200, "GET /mcp event stream status");
        Self {
            messages: message_rx,
            socket: Some(stream),
        }
    }

    fn wait_location_update(&self, deadline: Instant) {
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            assert!(
                !remaining.is_zero(),
                "HTTP location update deadline elapsed"
            );
            match self.messages.recv_timeout(remaining) {
                Ok(message) if is_location_update(&message) => return,
                Ok(_) => {}
                Err(error) => panic!("waiting for HTTP SSE notification: {error}"),
            }
        }
    }

    fn close(&mut self) {
        if let Some(socket) = self.socket.take() {
            let _ = socket.shutdown(std::net::Shutdown::Both);
        }
    }
}

fn read_sse_stream(mut stream: TcpStream, status_tx: Sender<u16>, message_tx: Sender<Value>) {
    let mut reader = BufReader::new(&mut stream);
    let mut status_line = String::new();
    if reader.read_line(&mut status_line).is_err() {
        let _ = status_tx.send(0);
        return;
    }
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let mut headers = BTreeMap::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            let _ = status_tx.send(0);
            return;
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.to_ascii_lowercase(), value.trim().to_owned());
        }
    }
    if status_tx.send(status).is_err() || status != 200 {
        return;
    }
    if headers
        .get("transfer-encoding")
        .is_some_and(|value| value.eq_ignore_ascii_case("chunked"))
    {
        read_chunked_sse(&mut reader, &message_tx);
    } else {
        read_plain_sse(&mut reader, &message_tx);
    }
}

fn read_chunked_sse(reader: &mut impl BufRead, message_tx: &Sender<Value>) {
    let mut event_buffer = Vec::new();
    loop {
        let mut line = Vec::new();
        match reader.read_until(b'\n', &mut line) {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
        let size_text = String::from_utf8_lossy(&line);
        let size = match usize::from_str_radix(
            size_text.trim().split(';').next().unwrap_or("").trim(),
            16,
        ) {
            Ok(size) => size,
            Err(_) => return,
        };
        if size == 0 {
            return;
        }
        let mut chunk = vec![0; size];
        if reader.read_exact(&mut chunk).is_err() {
            return;
        }
        let mut ending = [0; 2];
        if reader.read_exact(&mut ending).is_err() {
            return;
        }
        event_buffer.extend_from_slice(&chunk);
        emit_complete_sse_events(&mut event_buffer, message_tx);
    }
}

fn read_plain_sse(reader: &mut impl BufRead, message_tx: &Sender<Value>) {
    let mut buffer = Vec::new();
    loop {
        let mut chunk = [0; 2048];
        match reader.read(&mut chunk) {
            Ok(0) | Err(_) => return,
            Ok(count) => buffer.extend_from_slice(&chunk[..count]),
        }
        emit_complete_sse_events(&mut buffer, message_tx);
    }
}

fn emit_complete_sse_events(buffer: &mut Vec<u8>, message_tx: &Sender<Value>) {
    loop {
        let lf_end = buffer.windows(2).position(|window| window == b"\n\n");
        let crlf_end = buffer.windows(4).position(|window| window == b"\r\n\r\n");
        let Some((end, delimiter)) = (match (lf_end, crlf_end) {
            (Some(lf), Some(crlf)) if crlf <= lf => Some((crlf, 4)),
            (Some(lf), _) => Some((lf, 2)),
            (None, Some(crlf)) => Some((crlf, 4)),
            (None, None) => None,
        }) else {
            return;
        };
        let event = String::from_utf8_lossy(&buffer[..end]);
        for message in parse_sse_events(&format!("{event}\n\n")) {
            if message_tx.send(message).is_err() {
                return;
            }
        }
        buffer.drain(..end + delimiter);
    }
}
