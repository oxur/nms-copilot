//! Native MCP child-process coverage for real save-file movement.

#[path = "support/live_mcp.rs"]
pub mod live_mcp;
#[path = "support/name_observations.rs"]
pub mod name_observations;

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::player::PlayerState;
use nms_core::system::{Planet, System};
use nms_graph::GalaxyModel;
use nms_watch::{SaveSnapshot, compute_delta};
use serde_json::{Value, json};
use tempfile::TempDir;

fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

type LineReceiver = Receiver<Result<String, String>>;
type TeedLineReceivers = (LineReceiver, LineReceiver);

const PLAYER_LOCATION_URI: &str = "nms://player/location";
const PHASE_TIMEOUT: Duration = Duration::from_secs(8);
const RPC_TIMEOUT: Duration = Duration::from_secs(5);

#[cfg(unix)]
#[test]
fn test_headless_stdio_child_applies_names_and_reuses_compatible_restart() {
    use name_observations::{ModelStage, SourcePhase};

    let dir = tempfile::tempdir().expect("name child fixture directory");
    let save = dir.path().join("save.json");
    let cache = dir.path().join("cache.rkyv");
    std::fs::write(&save, name_observations::source_bytes(SourcePhase::Initial))
        .expect("write fixed initial name source");
    let source_time = source_mtime();
    set_mtime(&save, source_time);
    nms_cache::write_cache(&name_observations::warm_seed_cache(), &cache)
        .expect("write independently literal warm cache");
    set_mtime(&cache, source_time + Duration::from_secs(100));
    let home = isolated_names_home(&cache, true);
    let config_path = home.path().join(".nms-copilot/config.toml");
    let enabled_config = std::fs::read(&config_path).expect("retain enabled child config");
    let initial_source = name_observations::source_bytes(SourcePhase::Initial);
    assert_eq!(
        std::fs::read(&save).expect("read initial source bytes"),
        initial_source
    );
    let (mut process, stdout_capture) =
        StdioProcess::spawn_with_cache_policy_captured(&save, Some(home), false);
    initialize_name_stdio(&mut process);
    assert_name_stdio_stage(&mut process, &cache, ModelStage::Initial);

    run_name_stdio_phase(
        &mut process,
        &save,
        &cache,
        "overwrite",
        SourcePhase::Overwrite,
        ModelStage::Overwrite,
        true,
    );
    run_name_stdio_phase(
        &mut process,
        &save,
        &cache,
        "promote",
        SourcePhase::Promote,
        ModelStage::Promote,
        true,
    );
    run_name_stdio_phase(
        &mut process,
        &save,
        &cache,
        "rename",
        SourcePhase::Rename,
        ModelStage::Rename,
        true,
    );
    run_name_stdio_phase(
        &mut process,
        &save,
        &cache,
        "clear",
        SourcePhase::Clear,
        ModelStage::Clear,
        true,
    );

    replace_name_source(&save, SourcePhase::Omit);
    process.wait_resource_update("nms://player/location", PHASE_TIMEOUT);
    process.assert_no_name_updates_for(Duration::from_millis(650));
    assert_name_stdio_stage(&mut process, &cache, ModelStage::Ignore);
    let omitted_cache = name_cache_checkpoint(&cache);

    for phase in [
        SourcePhase::Null,
        SourcePhase::NonString,
        SourcePhase::NonObject,
        SourcePhase::NonObjectDm,
    ] {
        let cache_before = name_cache_checkpoint(&cache);
        assert_eq!(
            cache_before, omitted_cache,
            "prior rejected form preserved accepted omission cache"
        );
        replace_name_source(&save, phase);
        process.assert_no_name_updates_for(Duration::from_millis(650));
        assert_name_stdio_stage(&mut process, &cache, ModelStage::Ignore);
        assert_name_cache_unchanged(&cache, &cache_before, "invalid metadata representation");
    }

    for attempt in 1..=5 {
        let cache_before = name_cache_checkpoint(&cache);
        replace_name_bytes(&save, &name_observations::invalid_address_attempt(attempt));
        process.assert_no_name_updates_for(Duration::from_millis(650));
        assert_name_stdio_stage(&mut process, &cache, ModelStage::Ignore);
        assert_name_cache_unchanged(&cache, &cache_before, "invalid packed address");
    }
    assert!(
        process.wait_for_stderr(
            "5 consecutive parse failures",
            Instant::now() + Duration::from_secs(8)
        ),
        "five invalid name candidates must produce the bounded parser warning: {}",
        process.stderr_text()
    );
    run_name_stdio_phase(
        &mut process,
        &save,
        &cache,
        "recover",
        SourcePhase::Recover,
        ModelStage::Recover,
        true,
    );
    let recovered = name_observations::source_bytes(SourcePhase::Recover);
    let recovered_cache = name_cache_checkpoint(&cache);
    replace_name_bytes(&save, &recovered);
    process.assert_no_name_updates_for(Duration::from_millis(650));
    assert_name_stdio_stage(&mut process, &cache, ModelStage::Recover);
    assert_name_cache_unchanged(&cache, &recovered_cache, "identical valid rewrite");
    assert_eq!(
        process
            .stderr_text()
            .matches("5 consecutive parse failures")
            .count(),
        1,
        "one five-failure warning is emitted for this uninterrupted failure sequence"
    );

    let lifecycle_source = recovered.clone();
    let lifecycle_source_time = std::fs::metadata(&save)
        .expect("read source metadata before normal EOF")
        .modified()
        .expect("read source mtime before normal EOF");
    let lifecycle_cache = name_cache_checkpoint(&cache);
    assert_stdio_name_file_boundary(
        &save,
        &lifecycle_source,
        lifecycle_source_time,
        &cache,
        &lifecycle_cache,
        &config_path,
        &enabled_config,
    );
    process.close_stdin_and_wait(Duration::from_secs(5));
    validate_complete_stdio_stdout(stdout_capture, Instant::now() + Duration::from_secs(5))
        .expect("complete child stdout contains JSON-RPC only through EOF");
    assert_stdio_name_file_boundary(
        &save,
        &lifecycle_source,
        lifecycle_source_time,
        &cache,
        &lifecycle_cache,
        &config_path,
        &enabled_config,
    );

    let (cache_bytes, cache_time) = lifecycle_cache.clone();
    replace_name_bytes(&save, br#"{"Version":slice23-warm-malformed"#);
    set_mtime(&save, cache_time - Duration::from_secs(1));
    let restart_home = process.take_home().expect("retain child HOME for restart");
    std::fs::write(
        restart_home.path().join(".nms-copilot/config.toml"),
        names_child_config(&cache, false),
    )
    .expect("disable watcher for compatible restart");
    let disabled_config = std::fs::read(&config_path).expect("retain exact disabled config");
    let restart_source = std::fs::read(&save).expect("retain malformed restart source");
    let restart_source_time = std::fs::metadata(&save)
        .expect("read malformed source metadata")
        .modified()
        .expect("read malformed source mtime");
    let restart_cache = (cache_bytes, cache_time);
    let (mut restarted, restart_stdout) =
        StdioProcess::spawn_with_cache_policy_captured(&save, Some(restart_home), false);
    initialize_name_stdio(&mut restarted);
    assert_name_stdio_stage(&mut restarted, &cache, ModelStage::Recover);
    restarted.assert_no_name_updates_for(Duration::from_millis(650));
    assert_stdio_name_file_boundary(
        &save,
        &restart_source,
        restart_source_time,
        &cache,
        &restart_cache,
        &config_path,
        &disabled_config,
    );
    restarted.close_stdin_and_wait(Duration::from_secs(5));
    validate_complete_stdio_stdout(restart_stdout, Instant::now() + Duration::from_secs(5))
        .expect("compatible restart stdout remains JSON-RPC through EOF");
    assert_stdio_name_file_boundary(
        &save,
        &restart_source,
        restart_source_time,
        &cache,
        &restart_cache,
        &config_path,
        &disabled_config,
    );
}

#[cfg(unix)]
fn isolated_names_home(cache: &Path, watch_enabled: bool) -> TempDir {
    let home = tempfile::tempdir().expect("isolated names child HOME");
    let config = home.path().join(".nms-copilot");
    std::fs::create_dir_all(&config).expect("create names child config");
    std::fs::write(
        config.join("config.toml"),
        names_child_config(cache, watch_enabled),
    )
    .expect("write exact names child config");
    home
}

#[cfg(all(unix, feature = "http"))]
#[test]
fn test_headless_http_child_name_lifecycle_and_fresh_bind_retries() {
    run_http_names_lifecycle(None);
    run_http_names_lifecycle(Some(NamesCollisionPoint::Startup));
    run_http_names_lifecycle(Some(NamesCollisionPoint::Restart));
}

#[cfg(all(unix, feature = "http"))]
fn prepare_names_child_files() -> (PathBuf, PathBuf, PathBuf, TempDir) {
    use name_observations::SourcePhase;

    let fixture = tempfile::tempdir().expect("HTTP name fixture").keep();
    let save = fixture.join("save.json");
    let cache = fixture.join("cache.rkyv");
    std::fs::write(&save, name_observations::source_bytes(SourcePhase::Initial))
        .expect("write literal HTTP name source");
    let source_time = source_mtime();
    set_mtime(&save, source_time);
    nms_cache::write_cache(&name_observations::warm_seed_cache(), &cache)
        .expect("write independent HTTP name cache seed");
    set_mtime(&cache, source_time + Duration::from_secs(100));
    let home = isolated_names_home(&cache, true);
    (fixture, save, cache, home)
}

#[cfg(all(unix, feature = "http"))]
struct NamesHttpFiles {
    fixture: PathBuf,
    save: PathBuf,
    cache: PathBuf,
    config: PathBuf,
}

#[cfg(all(unix, feature = "http"))]
struct NamesHttpSnapshot {
    source: Vec<u8>,
    source_time: SystemTime,
    cache: Vec<u8>,
    cache_time: SystemTime,
    config: Vec<u8>,
}

#[cfg(all(unix, feature = "http"))]
struct NamesHttpBindFailure {
    phase: String,
    address: SocketAddr,
    fixture: PathBuf,
    home: PathBuf,
    receipt: PathBuf,
}

#[cfg(all(unix, feature = "http"))]
#[derive(Clone, Copy, PartialEq, Eq)]
enum NamesCollisionPoint {
    Startup,
    Restart,
}

#[cfg(all(unix, feature = "http"))]
enum NamesHttpAttemptError {
    RetryBind(Box<NamesHttpBindFailure>),
    Failed(String),
}

#[cfg(all(unix, feature = "http"))]
fn is_address_already_in_use(diagnostic: &str) -> bool {
    diagnostic
        .to_ascii_lowercase()
        .contains("address already in use")
}

#[cfg(all(unix, feature = "http"))]
fn retain_names_bind_failure(
    phase: &str,
    address: SocketAddr,
    files: &NamesHttpFiles,
    snapshot: &NamesHttpSnapshot,
    child: &mut HttpChild,
    stdout: Receiver<Result<String, String>>,
) -> Result<NamesHttpBindFailure, String> {
    let status = child.wait_after_startup_failure_result()?;
    let diagnostic = child.stderr_text();
    let is_bind_failure = is_address_already_in_use(&diagnostic);
    let source = std::fs::read(&files.save).map_err(|e| e.to_string())?;
    let source_time = std::fs::metadata(&files.save)
        .and_then(|metadata| metadata.modified())
        .map_err(|e| e.to_string())?;
    let cache = std::fs::read(&files.cache).map_err(|e| e.to_string())?;
    let cache_time = std::fs::metadata(&files.cache)
        .and_then(|metadata| metadata.modified())
        .map_err(|e| e.to_string())?;
    let config = std::fs::read(&files.config).map_err(|e| e.to_string())?;
    let cache_temp_absent = !files.cache.with_extension("rkyv.tmp").exists();
    let source_unchanged = source == snapshot.source && source_time == snapshot.source_time;
    let cache_unchanged = cache == snapshot.cache && cache_time == snapshot.cache_time;
    let config_unchanged = config == snapshot.config;
    let home = child
        .take_home()
        .ok_or_else(|| "failed HTTP child HOME was not retained".to_owned())?;
    let home_path = home.keep();
    let stdout_deadline = Instant::now() + Duration::from_secs(5);
    let mut stdout_lines = Vec::new();
    let stdout_eof = loop {
        let remaining = stdout_deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break false;
        }
        match stdout.recv_timeout(remaining) {
            Err(RecvTimeoutError::Disconnected) => break true,
            Err(RecvTimeoutError::Timeout) => break false,
            Ok(Ok(line)) => stdout_lines.push(line),
            Ok(Err(error)) => stdout_lines.push(format!("<stdout read error: {error}>")),
        }
    };
    let stdout_empty_through_eof = stdout_eof && stdout_lines.is_empty();
    let receipt_path = files.fixture.join("http-bind-failure.json");
    let receipt = json!({
        "phase": phase,
        "address": address.to_string(),
        "fixture": files.fixture.display().to_string(),
        "save_path": files.save.display().to_string(),
        "cache_path": files.cache.display().to_string(),
        "home": home_path.display().to_string(),
        "config_path": files.config.display().to_string(),
        "config_hex_before": hex_bytes(&snapshot.config),
        "config_hex_after": hex_bytes(&config),
        "stderr_capture": "line-wise available at reap; not byte-exact or stderr-EOF proof",
        "diagnostic": diagnostic,
        "bind_failure": is_bind_failure,
        "child_status": {"debug": format!("{status:?}"), "code": status.code(), "success": status.success()},
        "stdout_capture": "line-wise through EOF or bounded timeout",
        "stdout_lines_through_eof": stdout_lines,
        "source_hex_before": hex_bytes(&snapshot.source),
        "source_hex_after": hex_bytes(&source),
        "source_modified_nanos_before": snapshot.source_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos(),
        "source_modified_nanos_after": source_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos(),
        "source_unchanged": source_unchanged,
        "cache_hex_before": hex_bytes(&snapshot.cache),
        "cache_hex_after": hex_bytes(&cache),
        "cache_modified_nanos_before": snapshot.cache_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos(),
        "cache_modified_nanos_after": cache_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos(),
        "cache_unchanged": cache_unchanged,
        "config_unchanged": config_unchanged,
        "cache_temp_absent": cache_temp_absent,
        "stdout_empty_through_eof": stdout_empty_through_eof,
    });
    std::fs::write(
        &receipt_path,
        serde_json::to_vec(&receipt).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if !source_unchanged || !cache_unchanged || !config_unchanged || !cache_temp_absent {
        return Err(format!(
            "HTTP {phase} failure changed a protected boundary: {receipt}"
        ));
    }
    if !status.success() {
        return Err(format!(
            "HTTP {phase} child exited unsuccessfully: {receipt}"
        ));
    }
    if !stdout_empty_through_eof {
        return Err(format!(
            "HTTP {phase} child stdout was not empty through EOF: {receipt}"
        ));
    }
    if !is_bind_failure {
        return Err(format!(
            "HTTP {phase} failure was not an observed bind collision: {receipt}"
        ));
    }
    Ok(NamesHttpBindFailure {
        phase: phase.to_owned(),
        address,
        fixture: files.fixture.clone(),
        home: home_path,
        receipt: receipt_path,
    })
}

#[cfg(all(unix, feature = "http"))]
fn assert_names_bind_failure_record(failure: &NamesHttpBindFailure) {
    let receipt: Value = serde_json::from_slice(
        &std::fs::read(&failure.receipt).expect("read retained HTTP names bind receipt"),
    )
    .expect("parse retained HTTP names bind receipt");
    assert_eq!(
        failure.receipt,
        failure.fixture.join("http-bind-failure.json")
    );
    assert_eq!(receipt["phase"].as_str(), Some(failure.phase.as_str()));
    assert_eq!(
        receipt["address"].as_str(),
        Some(failure.address.to_string().as_str())
    );
    assert_eq!(
        receipt["fixture"].as_str(),
        Some(failure.fixture.display().to_string().as_str())
    );
    assert_eq!(
        receipt["save_path"].as_str(),
        Some(
            failure
                .fixture
                .join("save.json")
                .display()
                .to_string()
                .as_str()
        )
    );
    assert_eq!(
        receipt["cache_path"].as_str(),
        Some(
            failure
                .fixture
                .join("cache.rkyv")
                .display()
                .to_string()
                .as_str()
        )
    );
    assert_eq!(
        receipt["home"].as_str(),
        Some(failure.home.display().to_string().as_str())
    );
    assert!(failure.fixture.is_dir());
    assert!(failure.home.is_dir());
    assert!(failure.home.join(".nms-copilot/config.toml").is_file());
    assert!(receipt["bind_failure"].as_bool().unwrap_or(false));
    assert!(
        receipt["diagnostic"]
            .as_str()
            .is_some_and(is_address_already_in_use)
    );
    assert_eq!(receipt["child_status"]["success"], json!(true));
    assert_eq!(receipt["source_unchanged"], json!(true));
    assert_eq!(receipt["cache_unchanged"], json!(true));
    assert_eq!(receipt["config_unchanged"], json!(true));
    assert_eq!(receipt["cache_temp_absent"], json!(true));
    assert_eq!(receipt["stdout_empty_through_eof"], json!(true));
}

#[cfg(all(unix, feature = "http"))]
fn run_http_names_lifecycle(controlled_collision: Option<NamesCollisionPoint>) {
    let mut failures = Vec::new();
    for attempt in 1..=3 {
        let controlled_collision = (attempt == 1).then_some(controlled_collision).flatten();
        match run_http_names_lifecycle_attempt(controlled_collision) {
            Ok(()) => {
                assert_eq!(attempt > 1, !failures.is_empty());
                return;
            }
            Err(NamesHttpAttemptError::RetryBind(failure)) => {
                assert_names_bind_failure_record(&failure);
                failures.push(failure);
                assert!(
                    attempt < 3,
                    "HTTP names scenario exhausted three fresh bind attempts"
                );
            }
            Err(NamesHttpAttemptError::Failed(error)) => {
                panic!("HTTP names lifecycle failed without a retryable bind collision: {error}")
            }
        }
    }
    unreachable!("the third bind collision panics before another attempt")
}

#[cfg(all(unix, feature = "http"))]
fn run_http_names_lifecycle_attempt(
    controlled_collision: Option<NamesCollisionPoint>,
) -> Result<(), NamesHttpAttemptError> {
    use name_observations::{ModelStage, SourcePhase};

    let (fixture, save, cache, home) = prepare_names_child_files();
    let source_before_start = std::fs::read(&save).expect("retain HTTP prelaunch source");
    let source_time_before_start = std::fs::metadata(&save).unwrap().modified().unwrap();
    let cache_before_start = std::fs::read(&cache).expect("retain HTTP prelaunch cache");
    let cache_time_before_start = std::fs::metadata(&cache).unwrap().modified().unwrap();
    let config_path = home.path().join(".nms-copilot/config.toml");
    let config_before_start = std::fs::read(&config_path).expect("retain HTTP prelaunch config");
    let startup_listener = if controlled_collision == Some(NamesCollisionPoint::Startup) {
        Some(TcpListener::bind(("127.0.0.1", 0)).expect("hold controlled startup port"))
    } else {
        None
    };
    let address = if let Some(listener) = &startup_listener {
        listener.local_addr().expect("read held startup address")
    } else {
        let reservation = TcpListener::bind(("127.0.0.1", 0)).expect("reserve ordinary HTTP port");
        let address = reservation.local_addr().unwrap();
        drop(reservation);
        address
    };
    let (mut child, stdout) =
        HttpChild::spawn_with_cache_policy_captured(&save, address, Some(home), false);
    if let Err(_diagnostic) = wait_health(address, &mut child) {
        let failure = retain_names_bind_failure(
            "startup",
            address,
            &NamesHttpFiles {
                fixture: fixture.clone(),
                save: save.clone(),
                cache: cache.clone(),
                config: config_path.clone(),
            },
            &NamesHttpSnapshot {
                source: source_before_start.clone(),
                source_time: source_time_before_start,
                cache: cache_before_start.clone(),
                cache_time: cache_time_before_start,
                config: config_before_start.clone(),
            },
            &mut child,
            stdout,
        );
        drop(startup_listener);
        return match failure {
            Ok(failure) => Err(NamesHttpAttemptError::RetryBind(Box::new(failure))),
            Err(error) => Err(NamesHttpAttemptError::Failed(error)),
        };
    }
    if startup_listener.is_some() {
        return Err(NamesHttpAttemptError::Failed(
            "controlled startup port was not observed as a bind collision".into(),
        ));
    }
    let mut client = HttpClient::new(address);
    let initialized = client.post_without_session(initialize_request());
    assert_eq!(initialized.status, 200);
    let session = initialized
        .header("mcp-session-id")
        .expect("HTTP session")
        .to_owned();
    let message = http_json_messages(&initialized.text())
        .into_iter()
        .find(|message| message["id"] == 1)
        .expect("HTTP initialize response");
    assert_initialized(&message["result"]);
    client.post(json!({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}));
    let mut sse = live_mcp::subscribe_sse_uris(
        address,
        &session,
        &["nms://player/location", "nms://galaxy/stats"],
    )
    .expect("subscribe actual HTTP child to both resources");
    assert_name_http_stage(&mut client, &cache, ModelStage::Initial);

    http_names_phase(
        &mut client,
        &mut sse,
        &save,
        &cache,
        SourcePhase::Overwrite,
        ModelStage::Overwrite,
        true,
    );
    http_names_phase(
        &mut client,
        &mut sse,
        &save,
        &cache,
        SourcePhase::Promote,
        ModelStage::Promote,
        true,
    );
    http_names_phase(
        &mut client,
        &mut sse,
        &save,
        &cache,
        SourcePhase::Rename,
        ModelStage::Rename,
        true,
    );
    http_names_phase(
        &mut client,
        &mut sse,
        &save,
        &cache,
        SourcePhase::Clear,
        ModelStage::Clear,
        true,
    );

    let cursor = sse.cursor();
    replace_name_source(&save, SourcePhase::Omit);
    let location_end = sse
        .wait_for_resource_updated_since(cursor, "nms://player/location", PHASE_TIMEOUT)
        .expect("omitted-map event still invalidates location");
    sse.wait_quiet_uris_since(
        location_end,
        &["nms://player/location", "nms://galaxy/stats"],
        Duration::from_millis(650),
    )
    .expect("omitted-map transition has no stats update");
    assert_name_http_stage(&mut client, &cache, ModelStage::Ignore);
    let omitted_cache = name_cache_checkpoint(&cache);
    for phase in [
        SourcePhase::Null,
        SourcePhase::NonString,
        SourcePhase::NonObject,
        SourcePhase::NonObjectDm,
    ] {
        let cursor = sse.cursor();
        let cache_before = name_cache_checkpoint(&cache);
        assert_eq!(
            cache_before, omitted_cache,
            "prior rejected form preserved accepted omission cache"
        );
        replace_name_source(&save, phase);
        sse.wait_quiet_uris_since(
            cursor,
            &["nms://player/location", "nms://galaxy/stats"],
            Duration::from_millis(650),
        )
        .expect("equivalent invalid CN representation is quiet");
        assert_name_http_stage(&mut client, &cache, ModelStage::Ignore);
        assert_name_cache_unchanged(&cache, &cache_before, "invalid metadata representation");
    }
    for attempt in 1..=5 {
        let cursor = sse.cursor();
        let cache_before = name_cache_checkpoint(&cache);
        replace_name_bytes(&save, &name_observations::invalid_address_attempt(attempt));
        sse.wait_quiet_uris_since(
            cursor,
            &["nms://player/location", "nms://galaxy/stats"],
            Duration::from_millis(650),
        )
        .expect("invalid typed address keeps both HTTP resources quiet");
        assert_name_http_stage(&mut client, &cache, ModelStage::Ignore);
        assert_name_cache_unchanged(&cache, &cache_before, "invalid packed address");
    }
    assert!(
        child.wait_for_stderr(
            "5 consecutive parse failures",
            Instant::now() + Duration::from_secs(8)
        ),
        "HTTP child did not report five invalid saves: {}",
        child.stderr_text()
    );
    http_names_phase(
        &mut client,
        &mut sse,
        &save,
        &cache,
        SourcePhase::Recover,
        ModelStage::Recover,
        true,
    );
    let recovered = name_observations::source_bytes(SourcePhase::Recover);
    let cursor = sse.cursor();
    let recovered_cache = name_cache_checkpoint(&cache);
    replace_name_bytes(&save, &recovered);
    sse.wait_quiet_uris_since(
        cursor,
        &["nms://player/location", "nms://galaxy/stats"],
        Duration::from_millis(650),
    )
    .expect("identical valid rewrite is quiet for both resources");
    assert_name_http_stage(&mut client, &cache, ModelStage::Recover);
    assert_name_cache_unchanged(&cache, &recovered_cache, "identical valid rewrite");
    assert_eq!(
        child
            .stderr_text()
            .matches("5 consecutive parse failures")
            .count(),
        1,
        "one five-failure warning in the uninterrupted HTTP sequence"
    );
    let source_after_lifecycle = std::fs::read(&save).expect("retain source at normal EOF");
    let source_time_after_lifecycle = std::fs::metadata(&save).unwrap().modified().unwrap();
    let cache_after_lifecycle = std::fs::read(&cache).expect("retain cache at normal EOF");
    let cache_time_after_lifecycle = std::fs::metadata(&cache).unwrap().modified().unwrap();
    let config_after_lifecycle = std::fs::read(&config_path).expect("retain config at normal EOF");
    child.kill_and_wait();
    sse.wait_closed_without_resource_updates(
        Duration::from_secs(5),
        &["nms://player/location", "nms://galaxy/stats"],
    )
    .expect("HTTP child EOF has no late name-resource updates");
    sse.close_and_join()
        .expect("join HTTP lifecycle SSE reader");
    assert_http_stdout_empty_to_eof(stdout, Instant::now() + Duration::from_secs(5));
    assert_eq!(std::fs::read(&save).unwrap(), source_after_lifecycle);
    assert_eq!(
        std::fs::metadata(&save).unwrap().modified().unwrap(),
        source_time_after_lifecycle
    );
    assert_eq!(std::fs::read(&cache).unwrap(), cache_after_lifecycle);
    assert_eq!(
        std::fs::metadata(&cache).unwrap().modified().unwrap(),
        cache_time_after_lifecycle
    );
    assert_eq!(std::fs::read(&config_path).unwrap(), config_after_lifecycle);
    assert!(!cache.with_extension("rkyv.tmp").exists());
    assert!(!save.with_extension("replacement").exists());

    let cache_bytes = std::fs::read(&cache).expect("retain warm cache before restart");
    let cache_time = std::fs::metadata(&cache).unwrap().modified().unwrap();
    replace_name_bytes(&save, br#"{"Version":slice23-warm-malformed"#);
    set_mtime(&save, cache_time - Duration::from_secs(1));
    let home = child.take_home().expect("reuse actual HTTP child HOME");
    std::fs::write(
        home.path().join(".nms-copilot/config.toml"),
        names_child_config(&cache, false),
    )
    .expect("disable watcher for compatible HTTP restart");
    let config_bytes = std::fs::read(&config_path).expect("retain restart config bytes");
    let restart_source = std::fs::read(&save).expect("retain exact restart source before spawn");
    let restart_source_time = std::fs::metadata(&save).unwrap().modified().unwrap();
    let restart_snapshot = NamesHttpSnapshot {
        source: restart_source,
        source_time: restart_source_time,
        cache: cache_bytes.clone(),
        cache_time,
        config: config_bytes.clone(),
    };
    let files = NamesHttpFiles {
        fixture: fixture.clone(),
        save: save.clone(),
        cache: cache.clone(),
        config: config_path.clone(),
    };
    if controlled_collision == Some(NamesCollisionPoint::Restart) {
        let listener = TcpListener::bind(address).expect("hold old HTTP child port at restart");
        let (mut failed, failed_stdout) =
            HttpChild::spawn_with_cache_policy_captured(&save, address, Some(home), false);
        let failure = retain_names_bind_failure(
            "compatible restart",
            address,
            &files,
            &restart_snapshot,
            &mut failed,
            failed_stdout,
        );
        drop(listener);
        return match failure {
            Ok(failure) => Err(NamesHttpAttemptError::RetryBind(Box::new(failure))),
            Err(error) => Err(NamesHttpAttemptError::Failed(error)),
        };
    } else {
        let (mut restarted, restart_stdout) =
            HttpChild::spawn_with_cache_policy_captured(&save, address, Some(home), false);
        if let Err(_diagnostic) = wait_health(address, &mut restarted) {
            let failure = retain_names_bind_failure(
                "compatible restart",
                address,
                &files,
                &restart_snapshot,
                &mut restarted,
                restart_stdout,
            );
            return match failure {
                Ok(failure) => Err(NamesHttpAttemptError::RetryBind(Box::new(failure))),
                Err(error) => Err(NamesHttpAttemptError::Failed(error)),
            };
        }
        let mut restart_client = HttpClient::new(address);
        let response = restart_client.post_without_session(initialize_request());
        assert_eq!(response.status, 200);
        let restart_session = response.header("mcp-session-id").unwrap().to_owned();
        let initialized = http_json_messages(&response.text())
            .into_iter()
            .find(|message| message["id"] == 1)
            .unwrap();
        assert_initialized(&initialized["result"]);
        restart_client
            .post(json!({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}));
        assert_name_http_stage(&mut restart_client, &cache, ModelStage::Recover);
        let mut restart_sse = live_mcp::subscribe_sse_uris(
            address,
            &restart_session,
            &["nms://player/location", "nms://galaxy/stats"],
        )
        .expect("subscribe compatible HTTP restart");
        let restart_cursor = restart_sse.cursor();
        restart_sse
            .wait_quiet_uris_since(
                restart_cursor,
                &["nms://player/location", "nms://galaxy/stats"],
                Duration::from_millis(650),
            )
            .expect("disabled watcher restart remains quiet");
        restarted.kill_and_wait();
        restart_sse
            .wait_closed_without_resource_updates(
                Duration::from_secs(5),
                &["nms://player/location", "nms://galaxy/stats"],
            )
            .expect("HTTP restart EOF has no late name-resource updates");
        restart_sse.close_and_join().expect("join restart SSE");
        assert_http_stdout_empty_to_eof(restart_stdout, Instant::now() + Duration::from_secs(5));
        assert_eq!(std::fs::read(&cache).unwrap(), cache_bytes);
        assert_eq!(
            std::fs::metadata(&cache).unwrap().modified().unwrap(),
            cache_time
        );
        assert_eq!(
            std::fs::read(&save).unwrap(),
            br#"{"Version":slice23-warm-malformed"#
        );
        assert_eq!(
            std::fs::metadata(&save).unwrap().modified().unwrap(),
            restart_snapshot.source_time
        );
        assert_eq!(
            std::fs::read(&config_path).unwrap(),
            restart_snapshot.config
        );
        assert!(!cache.with_extension("rkyv.tmp").exists());
        assert!(!save.with_extension("replacement").exists());
    }
    Ok(())
}

#[cfg(all(unix, feature = "http"))]
fn http_names_phase(
    client: &mut HttpClient,
    sse: &mut live_mcp::SseStream,
    save: &Path,
    cache: &Path,
    source: name_observations::SourcePhase,
    stage: name_observations::ModelStage,
    expect_stats: bool,
) {
    let cursor = sse.cursor();
    replace_name_source(save, source);
    let location_end = sse
        .wait_for_resource_updated_since(cursor, "nms://player/location", PHASE_TIMEOUT)
        .expect("HTTP child location event");
    let end = if expect_stats {
        sse.wait_for_resource_updated_since(location_end, "nms://galaxy/stats", PHASE_TIMEOUT)
            .expect("HTTP child stats event")
    } else {
        location_end
    };
    sse.wait_quiet_uris_since(
        end,
        &["nms://player/location", "nms://galaxy/stats"],
        Duration::from_millis(650),
    )
    .expect("HTTP child stage has exactly one paired update");
    assert_name_http_stage(client, cache, stage);
}

#[cfg(all(unix, feature = "http"))]
fn assert_name_http_stage(
    client: &mut HttpClient,
    cache: &Path,
    stage: name_observations::ModelStage,
) {
    let address = client.addr;
    let session = client.session.as_deref().expect("initialized HTTP session");
    let (tool, location) =
        live_mcp::location_rpc_pair(address, session).expect("full HTTP location pair");
    let expected_location = name_observations::expected_location(stage, false);
    assert_eq!(tool, expected_location);
    assert_eq!(location, expected_location);
    let stats = live_mcp::resource_json(address, session, "nms://galaxy/stats")
        .expect("HTTP stats resource");
    assert_eq!(
        name_observations::normalize_stats_biomes(stats),
        name_observations::normalize_stats_biomes(name_observations::expected_stats(stage, false))
    );
    let response = client.post(json!({
        "jsonrpc":"2.0", "id":400, "method":"tools/call",
        "params":{"name":"search_planets", "arguments":{"biome":"Toxic", "infested":true, "named_only":true, "from_base":"Anchor"}}
    }));
    assert_eq!(response.status, 200);
    let message = http_json_messages(&response.text())
        .into_iter()
        .find(|message| message["id"] == 400)
        .expect("HTTP named query response");
    let query: Value = serde_json::from_str(
        message["result"]["content"][0]["text"]
            .as_str()
            .expect("search result text"),
    )
    .expect("search result JSON");
    let expected_name = match stage {
        name_observations::ModelStage::Promote => Some(("Current", "Current Moon")),
        name_observations::ModelStage::Rename => Some(("Current Renamed", "Renamed Moon")),
        name_observations::ModelStage::Recover => Some(("Current Recovered", "Recovered Moon")),
        _ => None,
    };
    assert_eq!(query["count"], usize::from(expected_name.is_some()));
    let rows = query["results"].as_array().expect("HTTP named query rows");
    assert_eq!(rows.len(), usize::from(expected_name.is_some()));
    if let (Some(row), Some((system, planet))) = (rows.first(), expected_name) {
        assert_eq!(
            row,
            &json!({
                "system": system,
                "planet": planet,
                "biome": "Toxic",
                "infested": true,
                "distance": "< 1 ly",
                "distance_ly": 0.0,
                "portal_glyphs_hex": "000700000064",
                "portal_glyphs_emoji": nms_query::display::hex_to_emoji("000700000064"),
                "discoverer": "Explorer"
            }),
            "complete named search row from the literal Anchor base"
        );
    }
    let actual = nms_cache::read_cache(cache).expect("HTTP child raw cache");
    let rebuilt = nms_cache::rebuild_model(&actual);
    live_mcp::compare_cache_model(
        &actual,
        &rebuilt,
        &name_observations::expected_cache(stage, 7),
        &name_observations::expected_model(stage),
    )
    .expect("HTTP raw cache and rebuilt graph match the complete literal state");
}

#[cfg(unix)]
fn names_child_config(cache: &Path, watch_enabled: bool) -> String {
    format!(
        "[display]\nshow_banner = false\nshow_system_banner = false\ncolor = false\n\n[watch]\nenabled = {watch_enabled}\ndebounce_ms = 137\n\n[cache]\nenabled = true\npath = {:?}\n",
        cache.to_string_lossy()
    )
}

#[cfg(unix)]
fn initialize_name_stdio(process: &mut StdioProcess) {
    process.send(initialize_request());
    assert_initialized(&process.response(1, RPC_TIMEOUT)["result"]);
    process.send(json!({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}));
    for (id, uri) in [(20, "nms://player/location"), (21, "nms://galaxy/stats")] {
        process.send(json!({
            "jsonrpc":"2.0", "id":id, "method":"resources/subscribe", "params":{"uri":uri}
        }));
        assert!(process.response(id, RPC_TIMEOUT)["result"].is_object());
    }
}

#[cfg(unix)]
fn replace_name_source(path: &Path, phase: name_observations::SourcePhase) {
    replace_name_bytes(path, &name_observations::source_bytes(phase));
}

#[cfg(unix)]
fn replace_name_bytes(path: &Path, bytes: &[u8]) {
    atomic_replace(path, bytes);
}

#[cfg(unix)]
fn name_cache_checkpoint(path: &Path) -> (Vec<u8>, SystemTime) {
    (
        std::fs::read(path).expect("read name-cache bytes"),
        std::fs::metadata(path)
            .expect("read name-cache metadata")
            .modified()
            .expect("read name-cache mtime"),
    )
}

#[cfg(unix)]
fn assert_name_cache_unchanged(path: &Path, expected: &(Vec<u8>, SystemTime), phase: &str) {
    assert_eq!(
        &name_cache_checkpoint(path),
        expected,
        "{phase} must preserve cache bytes and mtime"
    );
}

#[cfg(unix)]
fn assert_stdio_name_file_boundary(
    save: &Path,
    expected_source: &[u8],
    expected_source_time: SystemTime,
    cache: &Path,
    expected_cache: &(Vec<u8>, SystemTime),
    config: &Path,
    expected_config: &[u8],
) {
    assert_eq!(
        std::fs::read(save).expect("read source bytes at EOF boundary"),
        expected_source
    );
    assert_eq!(
        std::fs::metadata(save)
            .expect("read source metadata at EOF boundary")
            .modified()
            .expect("read source mtime at EOF boundary"),
        expected_source_time
    );
    assert_eq!(
        &name_cache_checkpoint(cache),
        expected_cache,
        "cache bytes and mtime remain fixed at EOF boundary"
    );
    assert_eq!(
        std::fs::read(config).expect("read config at EOF boundary"),
        expected_config
    );
    assert!(cache.parent().expect("cache parent").is_dir());
    assert!(!cache.with_extension("rkyv.tmp").exists());
    assert!(!save.with_extension("replacement").exists());
}

#[cfg(unix)]
fn run_name_stdio_phase(
    process: &mut StdioProcess,
    save: &Path,
    cache: &Path,
    label: &str,
    source: name_observations::SourcePhase,
    stage: name_observations::ModelStage,
    expect_stats: bool,
) {
    replace_name_source(save, source);
    process.wait_resource_update("nms://player/location", PHASE_TIMEOUT);
    if expect_stats {
        process.wait_resource_update("nms://galaxy/stats", PHASE_TIMEOUT);
    }
    process.assert_no_name_updates_for(Duration::from_millis(650));
    assert_name_stdio_stage(process, cache, stage);
    eprintln!("SLICE23_STDIO_NAME_PHASE {label}=pass");
}

#[cfg(unix)]
fn assert_name_stdio_stage(
    process: &mut StdioProcess,
    cache: &Path,
    stage: name_observations::ModelStage,
) {
    let id = 300;
    process.send(json!({
        "jsonrpc":"2.0", "id":id, "method":"tools/call",
        "params":{"name":"where_am_i", "arguments":{}}
    }));
    let tool_response = process.response(id, RPC_TIMEOUT);
    let tool: Value = serde_json::from_str(
        tool_response["result"]["content"][0]["text"]
            .as_str()
            .expect("location tool text"),
    )
    .expect("location tool JSON");
    process.send(json!({
        "jsonrpc":"2.0", "id":id + 1, "method":"resources/read",
        "params":{"uri":"nms://player/location"}
    }));
    let location_response = process.response(id + 1, RPC_TIMEOUT);
    let location: Value = serde_json::from_str(
        location_response["result"]["contents"][0]["text"]
            .as_str()
            .expect("location resource text"),
    )
    .expect("location resource JSON");
    let expected_location = name_observations::expected_location(stage, false);
    assert_eq!(tool, expected_location, "complete where_am_i tool object");
    assert_eq!(
        location, expected_location,
        "complete location resource object"
    );

    process.send(json!({
        "jsonrpc":"2.0", "id":id + 2, "method":"resources/read",
        "params":{"uri":"nms://galaxy/stats"}
    }));
    let stats_response = process.response(id + 2, RPC_TIMEOUT);
    let stats: Value = serde_json::from_str(
        stats_response["result"]["contents"][0]["text"]
            .as_str()
            .expect("stats resource text"),
    )
    .expect("stats resource JSON");
    assert_eq!(
        name_observations::normalize_stats_biomes(stats),
        name_observations::normalize_stats_biomes(name_observations::expected_stats(stage, false))
    );

    process.send(json!({
        "jsonrpc":"2.0", "id":id + 3, "method":"tools/call",
        "params":{"name":"search_planets", "arguments":{"biome":"Toxic", "infested":true, "named_only":true, "from_base":"Anchor"}}
    }));
    let query_response = process.response(id + 3, RPC_TIMEOUT);
    let query: Value = serde_json::from_str(
        query_response["result"]["content"][0]["text"]
            .as_str()
            .expect("search tool text"),
    )
    .expect("search tool JSON");
    let named = matches!(
        stage,
        name_observations::ModelStage::Promote
            | name_observations::ModelStage::Rename
            | name_observations::ModelStage::Recover
    );
    assert_eq!(query["count"], if named { 1 } else { 0 });
    let results = query["results"].as_array().expect("search results array");
    assert_eq!(results.len(), usize::from(named));
    if let Some(result) = results.first() {
        let name = match stage {
            name_observations::ModelStage::Promote => ("Current", "Current Moon"),
            name_observations::ModelStage::Rename => ("Current Renamed", "Renamed Moon"),
            _ => ("Current Recovered", "Recovered Moon"),
        };
        assert_eq!(
            result,
            &json!({
                "system": name.0,
                "planet": name.1,
                "biome": "Toxic",
                "infested": true,
                "distance": "< 1 ly",
                "distance_ly": 0.0,
                "portal_glyphs_hex": "000700000064",
                "portal_glyphs_emoji": nms_query::display::hex_to_emoji("000700000064"),
                "discoverer": "Explorer"
            }),
            "complete named search row from the literal Anchor base"
        );
    }

    let actual = nms_cache::read_cache(cache).expect("read complete name cache");
    let rebuilt = nms_cache::rebuild_model(&actual);
    let expected = name_observations::expected_cache(stage, 7);
    let model = name_observations::expected_model(stage);
    live_mcp::compare_cache_model(&actual, &rebuilt, &expected, &model)
        .expect("complete raw and rebuilt model match the independent name oracle");
}

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
    nms_save::parse_validated_save(&bytes)
        .expect("fixture is accepted by the checked application save parser");
    bytes
}

#[cfg(unix)]
fn unknown_lifecycle_save(
    version: u32,
    solar_system_index: u16,
    units: u64,
    nanites: u64,
    specials: u64,
    previous: Option<(u8, i64, i64, i64, u16, u8)>,
    promoted: bool,
) -> Vec<u8> {
    let previous = previous.map(|(reality, x, y, z, system, planet)| {
        GalacticAddress::new(x as i16, y as i8, z as i16, system, planet, reality)
    });
    crate::live_mcp::unknown_lifecycle_source(
        version,
        solar_system_index,
        units,
        nanites,
        specials,
        previous,
        promoted,
    )
}

#[cfg(unix)]
fn assert_unknown_lifecycle_parser_promotion() {
    let before = nms_save::parse_validated_save(&unknown_lifecycle_save(
        4721,
        3,
        70,
        80,
        90,
        Some((0, 0, 0, 0, 2, 5)),
        false,
    ))
    .expect("movement source validates before projection");
    let after = nms_save::parse_validated_save(&unknown_lifecycle_save(
        4722,
        3,
        700,
        800,
        900,
        Some((0, 0, 0, 0, 2, 5)),
        true,
    ))
    .expect("real discovery source validates before projection");
    let delta = compute_delta(
        &SaveSnapshot::from_validated_save(&before),
        &SaveSnapshot::from_validated_save(&after),
    );
    let u3 = GalacticAddress::new(0, 0, 0, 3, 0, 0);
    let system = System::new(
        u3,
        None,
        Some("Explorer".into()),
        Some(
            "2023-11-14T22:13:20Z"
                .parse()
                .expect("literal RFC3339 timestamp"),
        ),
        vec![Planet::new(
            5,
            Some(Biome::Toxic),
            None,
            false,
            None,
            Some(171),
        )],
    );
    assert_eq!(delta.new_systems, vec![system.clone()]);
    assert_eq!(
        delta.new_planets,
        vec![(
            nms_core::system::SystemId::from_address(&u3),
            system.planets[0].clone()
        )]
    );
}

#[cfg(unix)]
fn assert_unknown_file_boundary(
    save: &Path,
    expected_source: &[u8],
    expected_source_time: std::time::SystemTime,
    cache: &Path,
    config: (&Path, &[u8]),
    stage: crate::live_mcp::UnknownLifecycleStage,
    cache_checkpoint: Option<(&[u8], std::time::SystemTime)>,
) {
    assert_eq!(std::fs::read(save).unwrap(), expected_source);
    assert_eq!(std::fs::read(config.0).unwrap(), config.1);
    assert_eq!(
        std::fs::metadata(save).unwrap().modified().unwrap(),
        expected_source_time
    );
    crate::live_mcp::assert_unknown_lifecycle_cache(cache, stage)
        .expect("whole checked cache and rebuilt model match lifecycle stage");
    let parent = cache.parent().expect("cache parent");
    assert!(std::fs::symlink_metadata(parent).unwrap().is_dir());
    assert!(!cache.with_extension("rkyv.tmp").exists());
    if let Some((expected_bytes, expected_modified)) = cache_checkpoint {
        assert_eq!(std::fs::read(cache).unwrap(), expected_bytes);
        assert_eq!(
            std::fs::metadata(cache).unwrap().modified().unwrap(),
            expected_modified
        );
    }
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

#[cfg(unix)]
fn isolated_watch_home(enabled: bool, debounce_ms: u64) -> TempDir {
    let home = tempfile::tempdir().expect("isolated child home directory");
    let config_dir = home.path().join(".nms-copilot");
    std::fs::create_dir_all(&config_dir).expect("create isolated config directory");
    std::fs::write(
        config_dir.join("config.toml"),
        format!("[watch]\nenabled = {enabled}\ndebounce_ms = {debounce_ms}\n"),
    )
    .expect("write isolated watcher configuration");
    home
}

#[cfg(unix)]
fn isolated_cache_home(cache_path: &Path) -> TempDir {
    let home = isolated_watch_home(false, 137);
    let config_path = home.path().join(".nms-copilot/config.toml");
    let quoted_path = serde_json::to_string(
        cache_path
            .to_str()
            .expect("temporary cache path is valid UTF-8"),
    )
    .expect("serialize cache path as a TOML-compatible string");
    let mut config = std::fs::read_to_string(&config_path).expect("read isolated config");
    config.push_str(&format!(
        "\n[cache]\nenabled = true\npath = {quoted_path}\n"
    ));
    let value: toml::Value = toml::from_str(&config).expect("parse complete child config");
    assert_eq!(value["cache"]["enabled"].as_bool(), Some(true));
    assert_eq!(
        value["cache"]["path"].as_str(),
        Some(cache_path.to_str().unwrap())
    );
    assert_eq!(value["watch"]["enabled"].as_bool(), Some(false));
    assert_eq!(value["watch"]["debounce_ms"].as_integer(), Some(137));
    std::fs::write(config_path, config).expect("write verified isolated cache config");
    home
}

#[cfg(unix)]
fn isolated_live_cache_home(cache_path: &Path) -> TempDir {
    isolated_lifecycle_home(cache_path, true)
}

#[cfg(unix)]
fn isolated_lifecycle_home(cache_path: &Path, watch_enabled: bool) -> TempDir {
    let home = tempfile::tempdir().expect("isolated live-cache child home");
    let config_dir = home.path().join(".nms-copilot");
    std::fs::create_dir_all(&config_dir).expect("create live config directory");
    let quoted_path = serde_json::to_string(cache_path.to_str().expect("cache path UTF-8"))
        .expect("quote configured cache path");
    std::fs::write(
        config_dir.join("config.toml"),
        format!("[watch]\nenabled = {watch_enabled}\ndebounce_ms = 137\n\n[cache]\nenabled = true\npath = {quoted_path}\n"),
    ).expect("write live cache and watcher configuration");
    let config = std::fs::read_to_string(config_dir.join("config.toml")).unwrap();
    let parsed: toml::Value = toml::from_str(&config).expect("parse live config");
    assert_eq!(parsed["watch"]["enabled"].as_bool(), Some(watch_enabled));
    assert_eq!(parsed["watch"]["debounce_ms"].as_integer(), Some(137));
    assert_eq!(parsed["cache"]["enabled"].as_bool(), Some(true));
    assert_eq!(
        parsed["cache"]["path"].as_str(),
        Some(cache_path.to_str().unwrap())
    );
    home
}

#[cfg(unix)]
fn live_save_bytes(
    version: u64,
    reality: u8,
    voxel_x: i64,
    units: u64,
    nanites: u64,
    specials: u64,
    previous: Option<(u8, i64, i64, i64, u16, u8)>,
) -> Vec<u8> {
    let mut value = save_json(PHASE_A);
    value["Version"] = json!(version);
    value["BaseContext"]["PlayerStateData"]["UniverseAddress"]["RealityIndex"] = json!(reality);
    value["BaseContext"]["PlayerStateData"]["UniverseAddress"]["GalacticAddress"]["VoxelX"] =
        json!(voxel_x);
    value["BaseContext"]["PlayerStateData"]["Units"] = json!(units);
    value["BaseContext"]["PlayerStateData"]["Nanites"] = json!(nanites);
    value["BaseContext"]["PlayerStateData"]["Specials"] = json!(specials);
    let previous = previous.unwrap_or((0, 0, 0, 0, 0, 0));
    value["BaseContext"]["PlayerStateData"]["PreviousUniverseAddress"] = json!({
        "RealityIndex":previous.0, "GalacticAddress":{"VoxelX":previous.1,"VoxelY":previous.2,"VoxelZ":previous.3,"SolarSystemIndex":previous.4,"PlanetIndex":previous.5}
    });
    let bytes = serde_json::to_vec(&value).expect("serialize live save");
    nms_save::parse_save(&bytes).expect("production parser accepts live state fixture");
    bytes
}

#[cfg(unix)]
fn imported_live_model() -> GalaxyModel {
    let mut model = GalaxyModel::new();
    for (reality, name, biome, planet_name) in [
        (0, "Origin-0", nms_core::biome::Biome::Lush, "Leaf-0"),
        (9, "Imported-9", nms_core::biome::Biome::Toxic, "Leaf-9"),
    ] {
        let address = GalacticAddress::new(0, 0, 0, 1, 0, reality);
        model.insert_system(System::new(
            address,
            Some(name.into()),
            None,
            None,
            vec![nms_core::system::Planet::new(
                0,
                Some(biome),
                None,
                false,
                Some(planet_name.into()),
                None,
            )],
        ));
        model.insert_base(nms_core::player::PlayerBase::new(
            format!("Base-{reality}"),
            nms_core::player::BaseType::HomePlanetBase,
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

#[cfg(unix)]
#[derive(Clone, Copy)]
struct LiveStatePhase {
    version: u32,
    reality: u8,
    voxel_x: i16,
    units: u64,
    nanites: u64,
    specials: u64,
    previous: Option<GalacticAddress>,
    selected: u8,
}

#[cfg(unix)]
fn assert_live_cache_state(cache: &Path, phase: LiveStatePhase) {
    let data = nms_cache::read_cache(cache).expect("checked-read full live cache");
    let expected = PlayerState::new(
        GalacticAddress::new(phase.voxel_x, 0, 0, 1, 0, phase.reality),
        phase.reality,
        phase.previous,
        None,
        phase.units,
        phase.nanites,
        phase.specials,
    );
    assert_eq!(data.player_state, Some(expected.clone()));
    assert_eq!(data.save_version, phase.version);
    assert_eq!(data.active_galaxy, phase.selected);
    let mut identities: Vec<_> = data
        .systems
        .iter()
        .map(|s| {
            (
                s.address.reality_index,
                s.name.as_deref(),
                s.planets.len(),
                s.is_player_placeholder,
            )
        })
        .collect();
    identities.sort_by_key(|entry| entry.0);
    assert_eq!(
        identities,
        vec![
            (0, Some("Origin-0"), 1, false),
            (9, Some("Imported-9"), 1, false)
        ]
    );
    let mut base_names: Vec<_> = data.bases.iter().map(|base| base.name.as_str()).collect();
    base_names.sort();
    assert_eq!(base_names, vec!["Base-0", "Base-9"]);
    let rebuilt = nms_cache::rebuild_model(&data);
    let expected_model = imported_live_model();
    let marker_ids = rebuilt
        .systems
        .keys()
        .copied()
        .filter(|id| rebuilt.is_player_placeholder(id))
        .collect::<std::collections::HashSet<_>>();
    assert!(marker_ids.is_empty());
    for address in [
        GalacticAddress::new(0, 0, 0, 1, 0, 0),
        GalacticAddress::new(0, 0, 0, 1, 0, 9),
    ] {
        let id = nms_core::system::SystemId::from_address(&address);
        assert!(!rebuilt.is_player_placeholder(&id));
    }
    assert_eq!(rebuilt.systems, expected_model.systems);
    assert_eq!(rebuilt.planets, expected_model.planets);
    assert_eq!(rebuilt.bases, expected_model.bases);
    assert_eq!(rebuilt.name_index, expected_model.name_index);
    assert_eq!(rebuilt.biome_index, expected_model.biome_index);
    assert_eq!(rebuilt.address_to_id, expected_model.address_to_id);
    assert_eq!(rebuilt.player_state, Some(expected));
    assert_eq!(rebuilt.active_galaxy, phase.selected);
}

#[cfg(unix)]
fn assert_cold_source_cache(cache: &Path) {
    let data = nms_cache::read_cache(cache).expect("checked-read cache built from cold source");
    assert_eq!(
        data.player_state,
        Some(PlayerState::new(
            GalacticAddress::new(0, 0, 0, 1, 0, 0),
            0,
            None,
            None,
            7,
            8,
            9,
        ))
    );
    assert_eq!(data.save_version, 4720);
    assert_eq!(data.active_galaxy, 0);
    assert_eq!(data.systems.len(), 1);
    let marker = &data.systems[0];
    assert_eq!(marker.address, GalacticAddress::new(0, 0, 0, 1, 0, 0));
    assert!(marker.is_player_placeholder);
    assert!(marker.name.is_none());
    assert!(marker.discoverer.is_none());
    assert!(marker.timestamp_secs.is_none());
    assert!(marker.planets.is_empty());
    assert!(data.bases.is_empty());
}

#[cfg(unix)]
fn assert_cold_live_cache(cache: &Path) {
    let data = nms_cache::read_cache(cache).expect("checked-read cache after cold live update");
    assert_eq!(
        data.player_state,
        Some(PlayerState::new(
            GalacticAddress::new(0, 0, 0, 1, 0, 9),
            9,
            None,
            None,
            700,
            800,
            900,
        ))
    );
    assert_eq!(data.save_version, 4721);
    assert_eq!(data.active_galaxy, 9);
    assert_eq!(data.systems.len(), 2);
    let markers = data
        .systems
        .iter()
        .map(|system| (system.address.reality_index, system.is_player_placeholder))
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(
        markers,
        std::collections::HashMap::from([(0, true), (9, true)])
    );
    assert!(data.systems.iter().all(|system| {
        system.name.is_none()
            && system.discoverer.is_none()
            && system.timestamp_secs.is_none()
            && system.planets.is_empty()
    }));
    assert!(data.bases.is_empty());
    let rebuilt = nms_cache::rebuild_model(&data);
    let marker_ids = rebuilt
        .systems
        .keys()
        .copied()
        .filter(|id| rebuilt.is_player_placeholder(id))
        .collect::<std::collections::HashSet<_>>();
    let id0 = nms_core::system::SystemId::from_address(&GalacticAddress::new(0, 0, 0, 1, 0, 0));
    let id9 = nms_core::system::SystemId::from_address(&GalacticAddress::new(0, 0, 0, 1, 0, 9));
    assert_eq!(marker_ids, std::collections::HashSet::from([id0, id9]));
    assert_eq!(
        rebuilt
            .systems
            .keys()
            .copied()
            .collect::<std::collections::HashSet<_>>(),
        std::collections::HashSet::from([id0, id9])
    );
}

#[cfg(unix)]
fn live_phases() -> [LiveStatePhase; 6] {
    [
        LiveStatePhase {
            version: 4720,
            reality: 0,
            voxel_x: 0,
            units: 70,
            nanites: 80,
            specials: 90,
            previous: None,
            selected: 7,
        },
        LiveStatePhase {
            version: 4720,
            reality: 0,
            voxel_x: 0,
            units: 70,
            nanites: 80,
            specials: 90,
            previous: Some(GalacticAddress::new(3, 3, 4, 1, 1, 9)),
            selected: 7,
        },
        LiveStatePhase {
            version: 4721,
            reality: 0,
            voxel_x: 0,
            units: 70,
            nanites: 80,
            specials: 90,
            previous: Some(GalacticAddress::new(3, 3, 4, 1, 1, 9)),
            selected: 7,
        },
        LiveStatePhase {
            version: 4722,
            reality: 9,
            voxel_x: 0,
            units: 700,
            nanites: 800,
            specials: 900,
            previous: Some(GalacticAddress::new(2, 3, 4, 1, 1, 0)),
            selected: 9,
        },
        LiveStatePhase {
            version: 4723,
            reality: 9,
            voxel_x: 0,
            units: 701,
            nanites: 801,
            specials: 901,
            previous: Some(GalacticAddress::new(2, 3, 4, 1, 1, 0)),
            selected: 9,
        },
        LiveStatePhase {
            version: 4724,
            reality: 0,
            voxel_x: 0,
            units: 900,
            nanites: 901,
            specials: 902,
            previous: Some(GalacticAddress::new(0, 3, 4, 1, 1, 9)),
            selected: 0,
        },
    ]
}

#[cfg(unix)]
#[test]
fn test_live_previous_address_fixtures_keep_planet_and_reality_fields_distinct() {
    let phases = live_phases();
    let expected = [
        None,
        Some((9, 1)),
        Some((9, 1)),
        Some((0, 1)),
        Some((0, 1)),
        Some((9, 1)),
    ];
    for (phase, expected) in phases.into_iter().zip(expected) {
        let actual = phase
            .previous
            .map(|address| (address.reality_index, address.planet_index()));
        assert_eq!(
            actual, expected,
            "phase {} previous reality/planet",
            phase.version
        );
    }
}

#[cfg(unix)]
fn prepare_live_cache_fixture() -> (TempDir, PathBuf, PathBuf, Vec<u8>) {
    let directory = tempfile::tempdir().expect("live matrix fixture directory");
    let save = directory.path().join("save.json");
    let cache = directory.path().join("cache.rkyv");
    let source = live_save_bytes(4720, 0, 0, 7, 8, 9, None);
    std::fs::write(&save, &source).expect("write live source fixture");
    set_mtime(&save, source_mtime());
    let model = imported_live_model();
    let data = nms_cache::extract_cache_data(&model, 4720);
    nms_cache::write_cache(&data, &cache).expect("write warm imported cache");
    set_mtime(&cache, source_mtime() + Duration::from_secs(100));
    assert!(nms_cache::is_cache_fresh(&cache, &save));
    let checked = nms_cache::read_cache(&cache).expect("warm imported cache precondition");
    assert_eq!(checked.active_galaxy, 7);
    assert_eq!(checked.systems.len(), 2);
    assert_eq!(checked.bases.len(), 2);
    (directory, save, cache, source)
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
    let resource_system = assert_location(&value, phase);
    assert_eq!(resource_system, system, "tool and resource system labels");
    eprintln!(
        "MCP_LIVE_OBSERVATION transport=stdio phase={} tool={} resource={}",
        phase.label,
        serde_json::to_string(&tool_location).expect("serialize actual tool location"),
        serde_json::to_string(&value).expect("serialize actual resource location")
    );
    system
}

#[cfg(unix)]
fn call_unknown_stdio_pair(
    process: &mut StdioProcess,
    id: u64,
    address: GalacticAddress,
    system: &str,
    planets: usize,
    portal_hex: &str,
    signal_booster: &str,
) {
    let deadline = Instant::now() + RPC_TIMEOUT;
    process.send(json!({
        "jsonrpc":"2.0", "id":id, "method":"tools/call",
        "params":{"name":"where_am_i", "arguments":{}}
    }));
    let tool_response = process.response(id, remaining(deadline));
    let tool = parse_text_json(&tool_response["result"]);
    process.send(json!({
        "jsonrpc":"2.0", "id":id + 1, "method":"resources/read",
        "params":{"uri":PLAYER_LOCATION_URI}
    }));
    let resource_response = process.response(id + 1, remaining(deadline));
    let resource: Value = serde_json::from_str(
        resource_response["result"]["contents"][0]["text"]
            .as_str()
            .expect("location resource text"),
    )
    .expect("location resource JSON");
    crate::live_mcp::assert_location_pair_at(
        &tool,
        &resource,
        address,
        system,
        planets,
        portal_hex,
        signal_booster,
    )
    .unwrap_or_else(|error| panic!("full physical tool/resource oracle: {error}"));
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
    assert_eq!(tool.status, 200, "tools/call response: {}", tool.text());
    let response = http_json_messages(&tool.text())
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
        resource.text()
    );
    let response = http_json_messages(&resource.text())
        .into_iter()
        .find(|message| message["id"] == id + 1)
        .expect("HTTP resources/read response ID");
    let contents = &response["result"]["contents"][0]["text"];
    let value: Value = serde_json::from_str(contents.as_str().expect("location text exists"))
        .expect("location resource text is JSON");
    let resource_system = assert_location(&value, phase);
    assert_eq!(resource_system, system, "tool and resource system labels");
    eprintln!(
        "MCP_LIVE_OBSERVATION transport=http phase={} tool={} resource={}",
        phase.label,
        serde_json::to_string(&tool_location).expect("serialize actual tool location"),
        serde_json::to_string(&value).expect("serialize actual resource location")
    );
    system
}

#[cfg(all(unix, feature = "http"))]
fn call_unknown_http_pair(
    client: &mut HttpClient,
    id: u64,
    address: GalacticAddress,
    system: &str,
    planets: usize,
    portal_hex: &str,
    signal_booster: &str,
) {
    let deadline = Instant::now() + RPC_TIMEOUT;
    let tool_response = client.post_with_timeout(
        json!({
            "jsonrpc":"2.0", "id":id, "method":"tools/call",
            "params":{"name":"where_am_i", "arguments":{}}
        }),
        remaining(deadline),
    );
    assert_eq!(tool_response.status, 200, "HTTP tools/call");
    let tool_message = http_json_messages(&tool_response.text())
        .into_iter()
        .find(|message| message["id"] == id)
        .expect("HTTP tools/call response ID");
    let tool = parse_text_json(&tool_message["result"]);

    let resource_response = client.post_with_timeout(
        json!({
            "jsonrpc":"2.0", "id":id + 1, "method":"resources/read",
            "params":{"uri":PLAYER_LOCATION_URI}
        }),
        remaining(deadline),
    );
    assert_eq!(resource_response.status, 200, "HTTP resources/read");
    let resource_message = http_json_messages(&resource_response.text())
        .into_iter()
        .find(|message| message["id"] == id + 1)
        .expect("HTTP resources/read response ID");
    let resource: Value = serde_json::from_str(
        resource_message["result"]["contents"][0]["text"]
            .as_str()
            .expect("HTTP location resource text"),
    )
    .expect("HTTP location resource JSON");
    crate::live_mcp::assert_location_pair_at(
        &tool,
        &resource,
        address,
        system,
        planets,
        portal_hex,
        signal_booster,
    )
    .unwrap_or_else(|error| panic!("full HTTP physical tool/resource oracle: {error}"));
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

#[cfg(unix)]
#[test]
fn test_warm_missing_physical_system_is_repaired_then_promoted_and_reused() {
    use crate::live_mcp::UnknownLifecycleStage as Stage;

    assert_unknown_lifecycle_parser_promotion();
    let dir = tempfile::tempdir().expect("owned lifecycle fixture directory");
    let save = dir.path().join("save.json");
    let cache = dir.path().join("cache.rkyv");
    let initial = unknown_lifecycle_save(4720, 2, 7, 8, 9, Some((9, 2, 3, 4, 1, 1)), false);
    std::fs::write(&save, &initial).expect("write initial opposed save");
    let initial_time = source_mtime();
    set_mtime(&save, initial_time);
    let seed = crate::live_mcp::unknown_lifecycle_seed_cache();
    nms_cache::write_cache(&seed, &cache).expect("write compatible fresh cache seed");
    set_mtime(&cache, source_mtime() + Duration::from_secs(100));
    assert!(nms_cache::is_cache_fresh(&cache, &save));
    crate::live_mcp::compare_cache(&nms_cache::read_cache(&cache).unwrap(), &seed)
        .expect("prelaunch seed is the exact missing-U2 three-key cache");
    assert_eq!(std::fs::read(&save).unwrap(), initial.as_slice());
    assert_eq!(
        std::fs::metadata(&save).unwrap().modified().unwrap(),
        initial_time
    );
    assert!(!cache.with_extension("rkyv.tmp").exists());

    let home = isolated_lifecycle_home(&cache, true);
    let config_path = home.path().join(".nms-copilot/config.toml");
    let config_before = std::fs::read(&config_path).expect("read verified prelaunch config");
    let (mut process, stdout_capture) =
        StdioProcess::spawn_with_cache_policy_captured(&save, Some(home), false);
    process.send(initialize_request());
    assert_initialized(&process.response(1, RPC_TIMEOUT)["result"]);
    process.send(json!({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}));
    process.send(json!({
        "jsonrpc":"2.0", "id":20, "method":"resources/subscribe",
        "params":{"uri":PLAYER_LOCATION_URI}
    }));
    assert!(process.response(20, RPC_TIMEOUT).get("result").is_some());

    let u2 = GalacticAddress::new(0, 0, 0, 2, 5, 0);
    call_unknown_stdio_pair(
        &mut process,
        30,
        u2,
        "(unknown)",
        0,
        "500200000000",
        "07FF:007F:07FF:0002",
    );
    assert_unknown_file_boundary(
        &save,
        initial.as_slice(),
        initial_time,
        &cache,
        (&config_path, &config_before),
        Stage::Startup,
        None,
    );
    let startup_cache = std::fs::read(&cache).unwrap();
    let startup_cache_mtime = std::fs::metadata(&cache).unwrap().modified().unwrap();

    let moved = unknown_lifecycle_save(4721, 3, 70, 80, 90, Some((0, 0, 0, 0, 2, 5)), false);
    let moved_time = initial_time + Duration::from_secs(1);
    assert_unknown_file_boundary(
        &save,
        initial.as_slice(),
        initial_time,
        &cache,
        (&config_path, &config_before),
        Stage::Startup,
        Some((&startup_cache, startup_cache_mtime)),
    );
    atomic_replace(&save, &moved);
    set_mtime(&save, moved_time);
    process.wait_location_update(PHASE_TIMEOUT);
    thread::sleep(Duration::from_millis(650));
    process.assert_no_location_update_for(Duration::from_millis(650));
    let u3 = GalacticAddress::new(0, 0, 0, 3, 5, 0);
    call_unknown_stdio_pair(
        &mut process,
        32,
        u3,
        "(unknown)",
        0,
        "500300000000",
        "07FF:007F:07FF:0003",
    );
    assert_unknown_file_boundary(
        &save,
        moved.as_slice(),
        moved_time,
        &cache,
        (&config_path, &config_before),
        Stage::Moved,
        None,
    );
    let moved_cache = std::fs::read(&cache).unwrap();
    let moved_mtime = std::fs::metadata(&cache).unwrap().modified().unwrap();

    let malformed = br#"{"Version":broken"#;
    let malformed_time = moved_time + Duration::from_secs(1);
    atomic_replace(&save, malformed);
    set_mtime(&save, malformed_time);
    process.assert_no_location_update_for(Duration::from_millis(650));
    call_unknown_stdio_pair(
        &mut process,
        34,
        u3,
        "(unknown)",
        0,
        "500300000000",
        "07FF:007F:07FF:0003",
    );
    assert_unknown_file_boundary(
        &save,
        malformed,
        malformed_time,
        &cache,
        (&config_path, &config_before),
        Stage::Moved,
        Some((&moved_cache, moved_mtime)),
    );

    let invalid_address = crate::live_mcp::unknown_lifecycle_invalid_address_source();
    let invalid_address_time = malformed_time + Duration::from_secs(1);
    atomic_replace(&save, &invalid_address);
    set_mtime(&save, invalid_address_time);
    process.assert_no_location_update_for(Duration::from_millis(650));
    call_unknown_stdio_pair(
        &mut process,
        35,
        u3,
        "(unknown)",
        0,
        "500300000000",
        "07FF:007F:07FF:0003",
    );
    assert_unknown_file_boundary(
        &save,
        &invalid_address,
        invalid_address_time,
        &cache,
        (&config_path, &config_before),
        Stage::Moved,
        Some((&moved_cache, moved_mtime)),
    );

    let promoted = unknown_lifecycle_save(4722, 3, 700, 800, 900, Some((0, 0, 0, 0, 2, 5)), true);
    let promoted_time = invalid_address_time + Duration::from_secs(1);
    atomic_replace(&save, &promoted);
    set_mtime(&save, promoted_time);
    process.wait_location_update(PHASE_TIMEOUT);
    thread::sleep(Duration::from_millis(650));
    process.assert_no_location_update_for(Duration::from_millis(650));
    call_unknown_stdio_pair(
        &mut process,
        36,
        u3,
        "-",
        1,
        "500300000000",
        "07FF:007F:07FF:0003",
    );
    assert_unknown_file_boundary(
        &save,
        promoted.as_slice(),
        promoted_time,
        &cache,
        (&config_path, &config_before),
        Stage::Promoted,
        None,
    );
    let promoted_cache = std::fs::read(&cache).unwrap();
    let fixed_cache_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_777);
    set_mtime(&cache, fixed_cache_time);
    let identical_time = fixed_cache_time - Duration::from_secs(1);
    atomic_replace(&save, &promoted);
    set_mtime(&save, identical_time);
    process.assert_no_location_update_for(Duration::from_millis(650));
    call_unknown_stdio_pair(
        &mut process,
        38,
        u3,
        "-",
        1,
        "500300000000",
        "07FF:007F:07FF:0003",
    );
    assert_unknown_file_boundary(
        &save,
        promoted.as_slice(),
        identical_time,
        &cache,
        (&config_path, &config_before),
        Stage::Promoted,
        Some((&promoted_cache, fixed_cache_time)),
    );
    assert_eq!(std::fs::read(&save).unwrap(), promoted);
    process.close_stdin_and_wait(Duration::from_secs(5));
    let first_messages =
        validate_complete_stdio_stdout(stdout_capture, Instant::now() + Duration::from_secs(5))
            .expect("complete stdio lifecycle stdout remains JSON-RPC through EOF");
    assert_eq!(
        first_messages
            .iter()
            .filter(|message| is_location_update(message))
            .count(),
        2,
        "first lifecycle has exactly the move and promotion location updates"
    );
    thread::sleep(Duration::from_millis(650));
    assert_unknown_file_boundary(
        &save,
        promoted.as_slice(),
        identical_time,
        &cache,
        (&config_path, &config_before),
        Stage::Promoted,
        Some((&promoted_cache, fixed_cache_time)),
    );

    atomic_replace(&save, malformed);
    let restart_time = fixed_cache_time - Duration::from_secs(1);
    set_mtime(&save, restart_time);
    let disabled_home = isolated_lifecycle_home(&cache, false);
    let config_path = disabled_home.path().join(".nms-copilot/config.toml");
    let config_before = std::fs::read(&config_path).expect("read verified restart config");
    let (mut restarted, restart_stdout) =
        StdioProcess::spawn_with_cache_policy_captured(&save, Some(disabled_home), false);
    restarted.send(initialize_request());
    assert_initialized(&restarted.response(1, RPC_TIMEOUT)["result"]);
    restarted.send(json!({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}));
    restarted.send(json!({
        "jsonrpc":"2.0", "id":20, "method":"resources/subscribe",
        "params":{"uri":PLAYER_LOCATION_URI}
    }));
    assert!(restarted.response(20, RPC_TIMEOUT).get("result").is_some());
    call_unknown_stdio_pair(
        &mut restarted,
        30,
        u3,
        "-",
        1,
        "500300000000",
        "07FF:007F:07FF:0003",
    );
    assert_unknown_file_boundary(
        &save,
        malformed,
        restart_time,
        &cache,
        (&config_path, &config_before),
        Stage::Promoted,
        Some((&promoted_cache, fixed_cache_time)),
    );
    assert_eq!(std::fs::read(&save).unwrap(), malformed);
    assert_eq!(
        std::fs::metadata(&save).unwrap().modified().unwrap(),
        restart_time
    );
    assert_eq!(
        std::fs::metadata(&cache).unwrap().modified().unwrap(),
        fixed_cache_time
    );
    crate::live_mcp::assert_unknown_lifecycle_cache(&cache, Stage::Promoted)
        .expect("compatible restart served the cached genuine unnamed U3");
    restarted.close_stdin_and_wait(Duration::from_secs(5));
    let restart_messages =
        validate_complete_stdio_stdout(restart_stdout, Instant::now() + Duration::from_secs(5))
            .expect("restart stdio stdout remains JSON-RPC through EOF");
    assert_eq!(
        restart_messages
            .iter()
            .filter(|message| is_location_update(message))
            .count(),
        0,
        "watcher-disabled restart emits no location updates through EOF"
    );
    thread::sleep(Duration::from_millis(650));
    assert_unknown_file_boundary(
        &save,
        malformed,
        restart_time,
        &cache,
        (&config_path, &config_before),
        Stage::Promoted,
        Some((&promoted_cache, fixed_cache_time)),
    );
    assert_eq!(std::fs::read(&save).unwrap(), malformed);
    assert_eq!(std::fs::read(&cache).unwrap(), promoted_cache);
    assert!(!cache.with_extension("rkyv.tmp").exists());
}

#[cfg(all(unix, feature = "http"))]
#[test]
fn test_http_child_repairs_missing_physical_system_and_promotes_real_discovery() {
    run_http_unknown_lifecycle(None);
}

#[cfg(all(unix, feature = "http"))]
fn run_http_unknown_lifecycle(mut first_held: Option<TcpListener>) {
    let mut collision_seen = false;
    let mut failed_fixture: Option<PathBuf> = None;
    let mut failed_homes = Vec::new();
    let mut failed_receipt: Option<PathBuf> = None;
    for attempt in 0..2 {
        let address = if attempt == 0 {
            if let Some(listener) = first_held.as_ref() {
                listener.local_addr().expect("read held HTTP address")
            } else {
                let listener = TcpListener::bind(("127.0.0.1", 0)).expect("reserve HTTP port");
                let address = listener.local_addr().unwrap();
                drop(listener);
                address
            }
        } else {
            let listener = TcpListener::bind(("127.0.0.1", 0)).expect("reserve retry HTTP port");
            let address = listener.local_addr().unwrap();
            drop(listener);
            address
        };
        match run_http_unknown_lifecycle_attempt(address, failed_fixture.as_deref(), &failed_homes)
        {
            Ok(()) => {
                if first_held.is_some() {
                    assert!(
                        collision_seen,
                        "controlled listener must cause a real bind failure"
                    );
                }
                assert_eq!(attempt > 0, collision_seen);
                if collision_seen {
                    let receipt_path = failed_receipt
                        .as_ref()
                        .expect("observed collision has a durable receipt");
                    let receipt: Value = serde_json::from_slice(
                        &std::fs::read(receipt_path)
                            .expect("read retained bind receipt after retry"),
                    )
                    .expect("parse retained bind receipt after retry");
                    let fixture = failed_fixture
                        .as_ref()
                        .expect("observed collision has a retained fixture");
                    assert!(receipt_path.is_file());
                    assert!(fixture.is_dir());
                    assert!(fixture.join("save.json").is_file());
                    assert!(fixture.join("cache.rkyv").is_file());
                    assert_eq!(receipt_path, &fixture.join("http-bind-failure.json"));
                    assert_eq!(
                        receipt["fixture"].as_str(),
                        Some(fixture.display().to_string().as_str())
                    );
                    assert_eq!(
                        receipt["save_path"].as_str(),
                        Some(fixture.join("save.json").display().to_string().as_str())
                    );
                    assert_eq!(
                        receipt["cache_path"].as_str(),
                        Some(fixture.join("cache.rkyv").display().to_string().as_str())
                    );
                    for home in &failed_homes {
                        assert!(home.is_dir(), "retained child HOME must survive retry");
                        assert!(home.join(".nms-copilot/config.toml").is_file());
                    }
                    let receipt_homes = receipt["homes"].as_array().expect("receipt HOME links");
                    assert_eq!(receipt_homes.len(), failed_homes.len());
                    for (recorded, home) in receipt_homes.iter().zip(&failed_homes) {
                        assert_eq!(recorded.as_str(), Some(home.display().to_string().as_str()));
                    }
                }
                return;
            }
            Err(failure)
                if failure
                    .evidence
                    .to_ascii_lowercase()
                    .contains("address already in use") =>
            {
                let failure = *failure;
                assert_eq!(
                    attempt, 0,
                    "only the first HTTP lifecycle attempt may be retried"
                );
                let receipt: Value = serde_json::from_slice(
                    &std::fs::read(&failure.receipt).expect("read failed-attempt bind receipt"),
                )
                .expect("parse failed-attempt bind receipt");
                assert_eq!(
                    failure.receipt,
                    failure.fixture.join("http-bind-failure.json")
                );
                assert!(matches!(
                    failure.phase,
                    "startup repair" | "promoted restart"
                ));
                assert_eq!(receipt["phase"].as_str(), Some(failure.phase));
                assert_eq!(
                    receipt["address"].as_str(),
                    Some(failure.address.to_string().as_str())
                );
                assert_eq!(
                    receipt["fixture"].as_str(),
                    Some(failure.fixture.display().to_string().as_str())
                );
                assert_eq!(
                    receipt["save_path"].as_str(),
                    Some(
                        failure
                            .fixture
                            .join("save.json")
                            .display()
                            .to_string()
                            .as_str()
                    )
                );
                assert_eq!(
                    receipt["cache_path"].as_str(),
                    Some(
                        failure
                            .fixture
                            .join("cache.rkyv")
                            .display()
                            .to_string()
                            .as_str()
                    )
                );
                assert_eq!(
                    receipt["child_status"]["debug"].as_str(),
                    Some(failure.status_debug.as_str())
                );
                assert_eq!(receipt["child_status"]["code"], json!(failure.status_code));
                assert_eq!(
                    receipt["child_status"]["success"],
                    json!(failure.status_success)
                );
                assert_eq!(
                    receipt["diagnostic"].as_str(),
                    Some(failure.evidence.as_str())
                );
                assert_eq!(receipt["stderr_lines"], json!(failure.stderr_lines));
                assert_eq!(
                    receipt["stderr_capture"].as_str(),
                    Some("line-wise available-at-reap; not byte-exact or stderr-EOF proof")
                );
                assert_eq!(receipt["stdout_empty_through_eof"], json!(true));
                let receipt_homes = receipt["homes"].as_array().expect("receipt HOME links");
                assert_eq!(receipt_homes.len(), failure.homes.len());
                for (recorded, home) in receipt_homes.iter().zip(&failure.homes) {
                    assert_eq!(recorded.as_str(), Some(home.display().to_string().as_str()));
                    assert!(home.is_dir());
                }
                collision_seen = true;
                failed_fixture = Some(failure.fixture);
                failed_homes = failure.homes;
                failed_receipt = Some(failure.receipt);
                drop(first_held.take());
            }
            Err(failure) => {
                panic!(
                    "HTTP lifecycle {} failed without bind collision: {}",
                    failure.phase, failure.evidence
                )
            }
        }
    }
    panic!("fresh HTTP lifecycle retry did not complete");
}

#[cfg(all(unix, feature = "http"))]
fn run_http_unknown_lifecycle_attempt(
    address: SocketAddr,
    previous_fixture: Option<&std::path::Path>,
    previous_homes: &[PathBuf],
) -> Result<(), Box<HttpLifecycleFailure>> {
    use crate::live_mcp::UnknownLifecycleStage as Stage;

    assert_unknown_lifecycle_parser_promotion();
    let dir = tempfile::tempdir().expect("HTTP lifecycle fixture directory");
    if let Some(previous_fixture) = previous_fixture {
        assert_ne!(
            dir.path(),
            previous_fixture,
            "HTTP retry must use a fresh fixture"
        );
    }
    let save = dir.path().join("save.json");
    let cache = dir.path().join("cache.rkyv");
    let initial = unknown_lifecycle_save(4720, 2, 7, 8, 9, Some((9, 2, 3, 4, 1, 1)), false);
    let initial_time = source_mtime();
    std::fs::write(&save, &initial).expect("write HTTP lifecycle source");
    set_mtime(&save, initial_time);
    let seed = crate::live_mcp::unknown_lifecycle_seed_cache();
    nms_cache::write_cache(&seed, &cache).expect("write HTTP lifecycle cache seed");
    set_mtime(&cache, source_mtime() + Duration::from_secs(100));
    assert!(nms_cache::is_cache_fresh(&cache, &save));
    assert_eq!(std::fs::read(&save).unwrap(), initial.as_slice());
    assert_eq!(
        std::fs::metadata(&save).unwrap().modified().unwrap(),
        initial_time
    );
    crate::live_mcp::compare_cache(&nms_cache::read_cache(&cache).unwrap(), &seed)
        .expect("before launch, exact three-key seed omits physical U2");
    assert_eq!(
        std::fs::metadata(&cache).unwrap().modified().unwrap(),
        initial_time + Duration::from_secs(100)
    );
    assert!(!cache.with_extension("rkyv.tmp").exists());

    let home = isolated_lifecycle_home(&cache, true);
    let config_path = home.path().join(".nms-copilot/config.toml");
    let config_before = std::fs::read(&config_path).expect("read verified startup config");
    assert!(
        previous_homes
            .iter()
            .all(|previous| !home.path().starts_with(previous)),
        "HTTP retry must use a fresh child HOME"
    );
    let (mut child, stdout_capture) =
        HttpChild::spawn_with_cache_policy_captured(&save, address, Some(home), false);
    if let Err(error) = wait_health(address, &mut child) {
        let evidence = error.to_string();
        if evidence
            .to_ascii_lowercase()
            .contains("address already in use")
        {
            let homes = vec![
                child
                    .take_home()
                    .expect("startup failure retains its owned child HOME")
                    .keep(),
            ];
            let preserved_failed_fixture = dir.keep();
            child.wait_after_startup_failure();
            let status = child
                .child
                .try_wait()
                .expect("read cached startup-failure child status")
                .expect("startup-failure child was reaped");
            assert_http_stdout_empty_to_eof(
                stdout_capture,
                Instant::now() + Duration::from_secs(5),
            );
            let diagnostic = child.stderr_text();
            let stderr_lines = child.stderr_lines.clone();
            let receipt = persist_http_lifecycle_failure(
                &preserved_failed_fixture,
                "startup repair",
                address,
                &status,
                &stderr_lines,
                &homes,
            );
            let failure = HttpLifecycleFailure::from_observed(
                diagnostic,
                ("startup repair", address),
                preserved_failed_fixture,
                homes,
                receipt,
                stderr_lines,
                &status,
            );
            assert!(
                failure.evidence.contains("Address already in use"),
                "collision stderr must retain the actual bind error"
            );
            crate::live_mcp::assert_unknown_lifecycle_cache(&cache, Stage::Startup)
                .expect("loader repair remains observable before the failed bind");
            assert_eq!(std::fs::read(&save).unwrap(), initial);
            assert_eq!(
                std::fs::metadata(&save).unwrap().modified().unwrap(),
                initial_time
            );
            assert!(!cache.with_extension("rkyv.tmp").exists());
            assert_eq!(std::fs::read(&config_path).unwrap(), config_before);
            return Err(Box::new(failure));
        }
        panic!("HTTP child health failed without the controlled bind error: {evidence}");
    }
    let mut client = HttpClient::new(address);
    let initialize = client.post_without_session(initialize_request());
    assert_eq!(initialize.status, 200, "HTTP lifecycle initialize");
    let session = initialize
        .header("mcp-session-id")
        .expect("HTTP lifecycle session")
        .to_owned();
    let initialized = client.post(json!({
        "jsonrpc":"2.0", "method":"notifications/initialized", "params":{}
    }));
    assert_eq!(initialized.status, 202, "HTTP lifecycle initialized");
    let mut events = live_mcp::subscribe_sse(address, &session)
        .expect("subscribe and connect the actual location SSE stream");

    let u2 = GalacticAddress::new(0, 0, 0, 2, 5, 0);
    call_unknown_http_pair(
        &mut client,
        30,
        u2,
        "(unknown)",
        0,
        "500200000000",
        "07FF:007F:07FF:0002",
    );
    assert_unknown_file_boundary(
        &save,
        initial.as_slice(),
        initial_time,
        &cache,
        (&config_path, &config_before),
        Stage::Startup,
        None,
    );
    let startup_cache = std::fs::read(&cache).unwrap();
    let startup_cache_mtime = std::fs::metadata(&cache).unwrap().modified().unwrap();

    let moved = unknown_lifecycle_save(4721, 3, 70, 80, 90, Some((0, 0, 0, 0, 2, 5)), false);
    let moved_time = source_mtime() + Duration::from_secs(1);
    assert_unknown_file_boundary(
        &save,
        initial.as_slice(),
        initial_time,
        &cache,
        (&config_path, &config_before),
        Stage::Startup,
        Some((&startup_cache, startup_cache_mtime)),
    );
    let moved_cursor = events.cursor();
    atomic_replace(&save, &moved);
    set_mtime(&save, moved_time);
    let moved_event = events
        .wait_for_resource_updated_since(moved_cursor, "nms://player/location", PHASE_TIMEOUT)
        .unwrap();
    events
        .wait_quiet_since(moved_event, Duration::from_millis(650))
        .unwrap();
    let u3 = GalacticAddress::new(0, 0, 0, 3, 5, 0);
    call_unknown_http_pair(
        &mut client,
        32,
        u3,
        "(unknown)",
        0,
        "500300000000",
        "07FF:007F:07FF:0003",
    );
    assert_unknown_file_boundary(
        &save,
        moved.as_slice(),
        moved_time,
        &cache,
        (&config_path, &config_before),
        Stage::Moved,
        None,
    );
    let moved_cache = std::fs::read(&cache).unwrap();
    let moved_cache_mtime = std::fs::metadata(&cache).unwrap().modified().unwrap();

    let malformed = br#"{"Version":broken"#;
    let malformed_time = moved_time + Duration::from_secs(1);
    let malformed_cursor = events.cursor();
    atomic_replace(&save, malformed);
    set_mtime(&save, malformed_time);
    events
        .wait_quiet_since(malformed_cursor, Duration::from_millis(650))
        .unwrap();
    call_unknown_http_pair(
        &mut client,
        33,
        u3,
        "(unknown)",
        0,
        "500300000000",
        "07FF:007F:07FF:0003",
    );
    assert_unknown_file_boundary(
        &save,
        malformed,
        malformed_time,
        &cache,
        (&config_path, &config_before),
        Stage::Moved,
        Some((&moved_cache, moved_cache_mtime)),
    );

    let invalid_address = crate::live_mcp::unknown_lifecycle_invalid_address_source();
    let invalid_address_time = malformed_time + Duration::from_secs(1);
    let invalid_cursor = events.cursor();
    atomic_replace(&save, &invalid_address);
    set_mtime(&save, invalid_address_time);
    events
        .wait_quiet_since(invalid_cursor, Duration::from_millis(650))
        .unwrap();
    call_unknown_http_pair(
        &mut client,
        35,
        u3,
        "(unknown)",
        0,
        "500300000000",
        "07FF:007F:07FF:0003",
    );
    assert_unknown_file_boundary(
        &save,
        &invalid_address,
        invalid_address_time,
        &cache,
        (&config_path, &config_before),
        Stage::Moved,
        Some((&moved_cache, moved_cache_mtime)),
    );

    let promoted = unknown_lifecycle_save(4722, 3, 700, 800, 900, Some((0, 0, 0, 0, 2, 5)), true);
    let promotion_cursor = events.cursor();
    atomic_replace(&save, &promoted);
    let promoted_time = invalid_address_time + Duration::from_secs(1);
    set_mtime(&save, promoted_time);
    let promoted_event = events
        .wait_for_resource_updated_since(promotion_cursor, "nms://player/location", PHASE_TIMEOUT)
        .unwrap();
    events
        .wait_quiet_since(promoted_event, Duration::from_millis(650))
        .unwrap();
    call_unknown_http_pair(
        &mut client,
        36,
        u3,
        "-",
        1,
        "500300000000",
        "07FF:007F:07FF:0003",
    );
    assert_unknown_file_boundary(
        &save,
        promoted.as_slice(),
        promoted_time,
        &cache,
        (&config_path, &config_before),
        Stage::Promoted,
        None,
    );
    let promoted_cache = std::fs::read(&cache).unwrap();
    let fixed_cache_mtime = source_mtime() + Duration::from_secs(777);
    set_mtime(&cache, fixed_cache_mtime);
    let identical_time = fixed_cache_mtime - Duration::from_secs(1);
    let identical_cursor = events.cursor();
    atomic_replace(&save, &promoted);
    set_mtime(&save, identical_time);
    events
        .wait_quiet_since(identical_cursor, Duration::from_millis(650))
        .unwrap();
    call_unknown_http_pair(
        &mut client,
        38,
        u3,
        "-",
        1,
        "500300000000",
        "07FF:007F:07FF:0003",
    );
    assert_unknown_file_boundary(
        &save,
        promoted.as_slice(),
        identical_time,
        &cache,
        (&config_path, &config_before),
        Stage::Promoted,
        Some((&promoted_cache, fixed_cache_mtime)),
    );
    child.kill_and_wait();
    events
        .wait_closed_without_resource_updated(Duration::from_secs(5), "nms://player/location")
        .unwrap();
    events.close_and_join().unwrap();
    assert_http_stdout_empty_to_eof(stdout_capture, Instant::now() + Duration::from_secs(5));
    let mut initial_home = child.take_home();
    thread::sleep(Duration::from_millis(650));
    assert_unknown_file_boundary(
        &save,
        promoted.as_slice(),
        identical_time,
        &cache,
        (&config_path, &config_before),
        Stage::Promoted,
        Some((&promoted_cache, fixed_cache_mtime)),
    );
    let malformed_restart = br#"{"Version":restart-opposed"#;
    let restart_time = fixed_cache_mtime - Duration::from_secs(1);
    atomic_replace(&save, malformed_restart);
    set_mtime(&save, restart_time);
    let restart_home = isolated_lifecycle_home(&cache, false);
    let restart_config_path = restart_home.path().join(".nms-copilot/config.toml");
    let restart_config_before =
        std::fs::read(&restart_config_path).expect("read verified watcher-disabled restart config");
    assert!(
        previous_homes
            .iter()
            .all(|previous| !restart_home.path().starts_with(previous)),
        "HTTP restart HOME must be fresh after collision"
    );
    if let Some(initial_home) = initial_home.as_ref() {
        assert_ne!(
            initial_home.path(),
            restart_home.path(),
            "HTTP restart must use its own watcher-disabled HOME"
        );
    }
    let restart_listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let restart_address = restart_listener.local_addr().unwrap();
    drop(restart_listener);
    let (mut restarted, restart_stdout) = HttpChild::spawn_with_cache_policy_captured(
        &save,
        restart_address,
        Some(restart_home),
        false,
    );
    if let Err(error) = wait_health(restart_address, &mut restarted) {
        let evidence = error.to_string();
        if evidence
            .to_ascii_lowercase()
            .contains("address already in use")
        {
            let homes = vec![
                restarted
                    .take_home()
                    .expect("restart failure retains its owned child HOME")
                    .keep(),
                initial_home
                    .take()
                    .expect("restart failure retains the initial child HOME")
                    .keep(),
            ];
            let preserved_failed_fixture = dir.keep();
            restarted.wait_after_startup_failure();
            let status = restarted
                .child
                .try_wait()
                .expect("read cached restart-failure child status")
                .expect("restart-failure child was reaped");
            assert_http_stdout_empty_to_eof(
                restart_stdout,
                Instant::now() + Duration::from_secs(5),
            );
            let diagnostic = restarted.stderr_text();
            let stderr_lines = restarted.stderr_lines.clone();
            let receipt = persist_http_lifecycle_failure(
                &preserved_failed_fixture,
                "promoted restart",
                restart_address,
                &status,
                &stderr_lines,
                &homes,
            );
            let failure = HttpLifecycleFailure::from_observed(
                diagnostic,
                ("promoted restart", restart_address),
                preserved_failed_fixture,
                homes,
                receipt,
                stderr_lines,
                &status,
            );
            assert!(failure.evidence.contains("Address already in use"));
            assert_unknown_file_boundary(
                &save,
                malformed_restart,
                restart_time,
                &cache,
                (&restart_config_path, &restart_config_before),
                Stage::Promoted,
                Some((&promoted_cache, fixed_cache_mtime)),
            );
            assert!(!cache.with_extension("rkyv.tmp").exists());
            return Err(Box::new(failure));
        }
        panic!("HTTP restart health failed without bind collision: {evidence}");
    }
    let mut restart_client = HttpClient::new(restart_address);
    let restart_init = restart_client.post_without_session(initialize_request());
    assert_eq!(restart_init.status, 200);
    let restart_session = restart_init.header("mcp-session-id").unwrap().to_owned();
    restart_client.post(json!({
        "jsonrpc":"2.0", "method":"notifications/initialized", "params":{}
    }));
    let mut restart_events = live_mcp::subscribe_sse(restart_address, &restart_session)
        .expect("subscribe and connect the restart location SSE stream");
    call_unknown_http_pair(
        &mut restart_client,
        40,
        u3,
        "-",
        1,
        "500300000000",
        "07FF:007F:07FF:0003",
    );
    assert_unknown_file_boundary(
        &save,
        malformed_restart,
        restart_time,
        &cache,
        (&restart_config_path, &restart_config_before),
        Stage::Promoted,
        Some((&promoted_cache, fixed_cache_mtime)),
    );
    restarted.kill_and_wait();
    restart_events
        .wait_closed_without_resource_updated(Duration::from_secs(5), "nms://player/location")
        .unwrap();
    restart_events.close_and_join().unwrap();
    assert_http_stdout_empty_to_eof(restart_stdout, Instant::now() + Duration::from_secs(5));
    thread::sleep(Duration::from_millis(650));
    assert_unknown_file_boundary(
        &save,
        malformed_restart,
        restart_time,
        &cache,
        (&restart_config_path, &restart_config_before),
        Stage::Promoted,
        Some((&promoted_cache, fixed_cache_mtime)),
    );
    assert!(!cache.with_extension("rkyv.tmp").exists());
    Ok(())
}

#[cfg(all(unix, feature = "http"))]
struct HttpLifecycleFailure {
    evidence: String,
    phase: &'static str,
    address: SocketAddr,
    fixture: PathBuf,
    homes: Vec<PathBuf>,
    receipt: PathBuf,
    stderr_lines: Vec<String>,
    status_debug: String,
    status_code: Option<i32>,
    status_success: bool,
}

#[cfg(all(unix, feature = "http"))]
impl HttpLifecycleFailure {
    fn from_observed(
        evidence: String,
        stage: (&'static str, SocketAddr),
        fixture: PathBuf,
        homes: Vec<PathBuf>,
        receipt: PathBuf,
        stderr_lines: Vec<String>,
        status: &ExitStatus,
    ) -> Self {
        Self {
            evidence,
            phase: stage.0,
            address: stage.1,
            fixture,
            homes,
            receipt,
            stderr_lines,
            status_debug: format!("{status:?}"),
            status_code: status.code(),
            status_success: status.success(),
        }
    }
}

#[cfg(all(unix, feature = "http"))]
fn persist_http_lifecycle_failure(
    fixture: &Path,
    phase: &str,
    address: SocketAddr,
    status: &ExitStatus,
    stderr_lines: &[String],
    homes: &[PathBuf],
) -> PathBuf {
    let receipt = fixture.join("http-bind-failure.json");
    let diagnostic = stderr_lines.join("\n");
    let document = json!({
        "phase": phase,
        "address": address.to_string(),
        "child_status": {
            "debug": format!("{status:?}"),
            "code": status.code(),
            "success": status.success(),
        },
        "stderr_capture": "line-wise available-at-reap; not byte-exact or stderr-EOF proof",
        "stderr_lines": stderr_lines,
        "diagnostic": diagnostic,
        "stdout_empty_through_eof": true,
        "fixture": fixture.display().to_string(),
        "save_path": fixture.join("save.json").display().to_string(),
        "cache_path": fixture.join("cache.rkyv").display().to_string(),
        "homes": homes
            .iter()
            .map(|home| home.display().to_string())
            .collect::<Vec<_>>(),
    });
    std::fs::write(
        &receipt,
        serde_json::to_vec_pretty(&document).expect("serialize bind-failure receipt"),
    )
    .expect("persist fixture-local bind-failure receipt");
    receipt
}

#[cfg(all(unix, feature = "http"))]
#[test]
fn test_http_unknown_bind_retry_uses_fresh_fixture_after_observed_repair() {
    let held = TcpListener::bind(("127.0.0.1", 0)).expect("hold controlled HTTP bind address");
    run_http_unknown_lifecycle(Some(held));
}

#[test]
fn test_stdio_mcp_child_observes_atomic_save_replacements_and_recovers() {
    test_stdio_child_tracks_real_save_and_recovers_after_parse_failures();
}

#[cfg(unix)]
#[test]
fn test_stdio_mcp_child_disabled_watcher_keeps_initial_model_after_save_change() {
    let (_dir, save) = temp_save();
    let mut process = StdioProcess::spawn_with_watch_settings(&save, false, 137);

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

    atomic_replace(&save, &save_bytes(PHASE_C));
    let observation_ends = Instant::now() + Duration::from_millis(1500);
    let mut observed_before_end = false;
    let mut observed_after_end = false;
    let mut id = 30;
    while !observed_after_end {
        let sample_started = Instant::now();
        let system = call_where_am_i_stdio(&mut process, id, PHASE_A, Instant::now() + RPC_TIMEOUT);
        let sample_finished = Instant::now();
        assert_eq!(
            system, initial_system,
            "disabled watcher keeps phase A system"
        );
        observed_before_end |= sample_finished < observation_ends;
        observed_after_end |= sample_started >= observation_ends;
        id += 2;
        if !observed_after_end {
            thread::sleep(Duration::from_millis(100));
        }
    }
    assert!(
        observed_before_end,
        "sampled tool and resource before 1500ms"
    );
    assert!(observed_after_end, "sampled tool and resource after 1500ms");
    assert!(process.stdout_is_json_only());
    process.close_stdin_and_wait(Duration::from_secs(5));
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
    assert_eq!(initialize.status, 200, "initialize: {}", initialize.text());
    let session = initialize
        .header("mcp-session-id")
        .expect("server returned a real MCP session ID")
        .to_owned();
    let result = http_json_messages(&initialize.text())
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
    let tools = http_json_messages(&tools.text())
        .into_iter()
        .find(|message| message["id"] == 2)
        .expect("tools/list response ID");
    let resources = client.post(rpc("resources/list", 3));
    assert_eq!(resources.status, 200);
    let resources = http_json_messages(&resources.text())
        .into_iter()
        .find(|message| message["id"] == 3)
        .expect("resources/list response ID");
    assert_lists(&tools["result"], &resources["result"]);
    let initial_system =
        call_where_am_i_http(&mut client, 10, PHASE_A, Instant::now() + RPC_TIMEOUT);

    let subscribed = client.post(json!({
        "jsonrpc":"2.0", "id":20, "method":"resources/subscribe",
        "params":{"uri":PLAYER_LOCATION_URI}
    }));
    assert_eq!(
        subscribed.status,
        200,
        "subscription response: {}",
        subscribed.text()
    );
    let subscription_response = http_json_messages(&subscribed.text())
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
        let system = call_where_am_i_http(&mut client, id, phase, phase_deadline);
        if phase.label == PHASE_D.label {
            assert_eq!(system, initial_system, "phase D returns to phase A system");
        }
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

#[cfg(all(unix, feature = "http"))]
#[test]
fn test_http_mcp_child_disabled_watcher_keeps_initial_model_after_save_change() {
    let (_dir, save) = temp_save();
    let mut last_error = String::new();
    let mut client = None;
    let mut process = None;

    // Retry only when this child's actual startup diagnostic identifies a bind collision.
    for _attempt in 0..3 {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("reserve loopback port");
        let addr = listener.local_addr().expect("reserved socket address");
        drop(listener);
        let mut child = HttpChild::spawn_with_watch_settings(&save, addr, false, 137);
        match wait_health(addr, &mut child) {
            Ok(()) => {
                process = Some(child);
                client = Some(HttpClient::new(addr));
                break;
            }
            Err(message) if message.contains("bind") => last_error = message,
            Err(message) => panic!("HTTP child did not become healthy: {message}"),
        }
    }
    let mut process =
        process.unwrap_or_else(|| panic!("three loopback bind collisions: {last_error}"));
    let mut client = client.expect("HTTP client established after health readiness");

    let initialized = client.post_without_session(initialize_request());
    assert_eq!(
        initialized.status,
        200,
        "initialize: {}",
        initialized.text()
    );
    let session = initialized
        .header("mcp-session-id")
        .expect("server returned an MCP session ID")
        .to_owned();
    let response = http_json_messages(&initialized.text())
        .into_iter()
        .find(|message| message["id"] == 1)
        .expect("initialize response ID");
    assert_initialized(&response["result"]);
    let initialized = client.post(json!({
        "jsonrpc":"2.0", "method":"notifications/initialized", "params":{}
    }));
    assert_eq!(initialized.status, 202, "initialized notification");
    assert_eq!(client.session.as_deref(), Some(session.as_str()));

    let tools = client.post(rpc("tools/list", 2));
    assert_eq!(tools.status, 200);
    let tools = http_json_messages(&tools.text())
        .into_iter()
        .find(|message| message["id"] == 2)
        .expect("tools/list response ID");
    let resources = client.post(rpc("resources/list", 3));
    assert_eq!(resources.status, 200);
    let resources = http_json_messages(&resources.text())
        .into_iter()
        .find(|message| message["id"] == 3)
        .expect("resources/list response ID");
    assert_lists(&tools["result"], &resources["result"]);
    let initial_system =
        call_where_am_i_http(&mut client, 10, PHASE_A, Instant::now() + RPC_TIMEOUT);

    atomic_replace(&save, &save_bytes(PHASE_C));
    let observation_ends = Instant::now() + Duration::from_millis(1500);
    let mut observed_before_end = false;
    let mut observed_after_end = false;
    let mut id = 30;
    while !observed_after_end {
        let sample_started = Instant::now();
        let system = call_where_am_i_http(&mut client, id, PHASE_A, Instant::now() + RPC_TIMEOUT);
        let sample_finished = Instant::now();
        assert_eq!(
            system, initial_system,
            "disabled watcher keeps phase A system"
        );
        observed_before_end |= sample_finished < observation_ends;
        observed_after_end |= sample_started >= observation_ends;
        id += 2;
        if !observed_after_end {
            thread::sleep(Duration::from_millis(100));
        }
    }
    assert!(
        observed_before_end,
        "sampled tool and resource before 1500ms"
    );
    assert!(observed_after_end, "sampled tool and resource after 1500ms");
    assert!(process.stdout_is_empty(), "HTTP child stdout remains empty");
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
    _home: Option<TempDir>,
}

impl StdioProcess {
    fn spawn(save: &Path) -> Self {
        #[cfg(unix)]
        {
            Self::spawn_with_watch_settings(save, true, 137)
        }
        #[cfg(not(unix))]
        {
            Self::spawn_with_home(save, None)
        }
    }

    #[cfg(unix)]
    fn spawn_with_watch_settings(save: &Path, enabled: bool, debounce_ms: u64) -> Self {
        Self::spawn_with_home(save, Some(isolated_watch_home(enabled, debounce_ms)))
    }

    fn spawn_with_home(save: &Path, home: Option<TempDir>) -> Self {
        Self::spawn_with_cache_policy(save, home, true)
    }

    #[cfg(unix)]
    fn spawn_cached_startup(save: &Path, cache_path: &Path, no_cache: bool) -> Self {
        Self::spawn_with_cache_policy(save, Some(isolated_cache_home(cache_path)), no_cache)
    }

    fn spawn_with_cache_policy(save: &Path, home: Option<TempDir>, no_cache: bool) -> Self {
        Self::spawn_with_cache_policy_inner(save, home, no_cache, false).0
    }

    fn spawn_with_cache_policy_captured(
        save: &Path,
        home: Option<TempDir>,
        no_cache: bool,
    ) -> (Self, Receiver<Result<String, String>>) {
        let (process, capture) = Self::spawn_with_cache_policy_inner(save, home, no_cache, true);
        (process, capture.expect("stdout EOF capture enabled"))
    }

    fn spawn_with_cache_policy_inner(
        save: &Path,
        home: Option<TempDir>,
        no_cache: bool,
        capture_stdout: bool,
    ) -> (Self, Option<Receiver<Result<String, String>>>) {
        let mut command = Command::new(assert_cmd::cargo::cargo_bin!("nms-copilot"));
        if let Some(home) = &home {
            command.env("HOME", home.path());
        }
        command.args(["--headless"]);
        if no_cache {
            command.arg("--no-cache");
        }
        let mut child = command
            .arg("--save")
            .arg(save)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn compiled nms-copilot child");
        let stdin = child.stdin.take().expect("piped stdin");
        let (stdout, stdout_eof_capture) = if capture_stdout {
            let (protocol, capture) = lines_channel_tee(child.stdout.take().expect("piped stdout"));
            (protocol, Some(capture))
        } else {
            (
                lines_channel(child.stdout.take().expect("piped stdout")),
                None,
            )
        };
        let stderr = lines_channel(child.stderr.take().expect("piped stderr"));
        let process = Self {
            child,
            stdin: Some(stdin),
            stdout,
            stderr,
            stderr_lines: Vec::new(),
            pending: Vec::new(),
            last_id: 100,
            _home: home,
        };
        (process, stdout_eof_capture)
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
        self.wait_resource_update(PLAYER_LOCATION_URI, timeout);
    }

    fn wait_resource_update(&mut self, uri: &str, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(index) = self
                .pending
                .iter()
                .position(|message| is_resource_update_for(message, uri))
            {
                self.pending.remove(index);
                return;
            }
            let message = self.read_message(deadline, &format!("resources/updated for {uri}"));
            if is_resource_update_for(&message, uri) {
                return;
            }
            self.pending.push(message);
        }
    }

    fn assert_no_name_updates_for(&mut self, duration: Duration) {
        let uris = ["nms://player/location", "nms://galaxy/stats"];
        let deadline = Instant::now() + duration;
        loop {
            if let Some(index) = self
                .pending
                .iter()
                .position(|message| uris.iter().any(|uri| is_resource_update_for(message, uri)))
            {
                panic!(
                    "unexpected queued name-resource update: {}",
                    self.pending.remove(index)
                );
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return;
            }
            match self
                .stdout
                .recv_timeout(remaining.min(Duration::from_millis(25)))
            {
                Ok(Ok(line)) => {
                    let message: Value = serde_json::from_str(&line).expect("JSON-RPC output");
                    assert!(
                        !uris.iter().any(|uri| is_resource_update_for(&message, uri)),
                        "unexpected name-resource update: {message}"
                    );
                    self.pending.push(message);
                }
                Ok(Err(error)) => panic!("read stdout during quiet window: {error}"),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("stdout closed during name-resource quiet window")
                }
            }
        }
    }

    fn assert_no_location_update_for(&mut self, duration: Duration) {
        let deadline = Instant::now() + duration;
        loop {
            if let Some(index) = self.pending.iter().position(is_location_update) {
                panic!(
                    "unexpected queued location update: {}",
                    self.pending.remove(index)
                );
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return;
            }
            match self
                .stdout
                .recv_timeout(remaining.min(Duration::from_millis(25)))
            {
                Ok(Ok(line)) => {
                    let message: Value = serde_json::from_str(&line).expect("JSON-RPC output");
                    assert!(
                        !is_location_update(&message),
                        "unexpected location update: {message}"
                    );
                    self.pending.push(message);
                }
                Ok(Err(error)) => panic!("read stdout while checking no update: {error}"),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("stdout closed before no-update window")
                }
            }
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

    fn take_home(&mut self) -> Option<TempDir> {
        self._home.take()
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
    _home: Option<TempDir>,
}

impl HttpChild {
    fn spawn(save: &Path, addr: SocketAddr) -> Self {
        #[cfg(unix)]
        {
            Self::spawn_with_watch_settings(save, addr, true, 137)
        }
        #[cfg(not(unix))]
        {
            Self::spawn_with_home(save, addr, None)
        }
    }

    #[cfg(unix)]
    fn spawn_with_watch_settings(
        save: &Path,
        addr: SocketAddr,
        enabled: bool,
        debounce_ms: u64,
    ) -> Self {
        Self::spawn_with_home(save, addr, Some(isolated_watch_home(enabled, debounce_ms)))
    }

    fn spawn_with_home(save: &Path, addr: SocketAddr, home: Option<TempDir>) -> Self {
        Self::spawn_with_cache_policy(save, addr, home, true)
    }

    #[cfg(all(unix, feature = "http"))]
    fn spawn_cached_startup(
        save: &Path,
        cache_path: &Path,
        addr: SocketAddr,
        no_cache: bool,
    ) -> Self {
        Self::spawn_with_cache_policy(save, addr, Some(isolated_cache_home(cache_path)), no_cache)
    }

    fn spawn_with_cache_policy(
        save: &Path,
        addr: SocketAddr,
        home: Option<TempDir>,
        no_cache: bool,
    ) -> Self {
        Self::spawn_with_cache_policy_inner(save, addr, home, no_cache, false).0
    }

    fn spawn_with_cache_policy_captured(
        save: &Path,
        addr: SocketAddr,
        home: Option<TempDir>,
        no_cache: bool,
    ) -> (Self, Receiver<Result<String, String>>) {
        let (process, capture) =
            Self::spawn_with_cache_policy_inner(save, addr, home, no_cache, true);
        (process, capture.expect("stdout EOF capture enabled"))
    }

    fn spawn_with_cache_policy_inner(
        save: &Path,
        addr: SocketAddr,
        home: Option<TempDir>,
        no_cache: bool,
        capture_stdout: bool,
    ) -> (Self, Option<Receiver<Result<String, String>>>) {
        let mut command = Command::new(assert_cmd::cargo::cargo_bin!("nms-copilot"));
        if let Some(home) = &home {
            command.env("HOME", home.path());
        }
        command.args(["--headless"]);
        if no_cache {
            command.arg("--no-cache");
        }
        let mut child = command
            .arg("--save")
            .arg(save)
            .args(["--http", &addr.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn compiled nms-copilot HTTP child");
        let (stdout, stdout_eof_capture) = if capture_stdout {
            let (protocol, capture) = lines_channel_tee(child.stdout.take().expect("piped stdout"));
            (protocol, Some(capture))
        } else {
            (
                lines_channel(child.stdout.take().expect("piped stdout")),
                None,
            )
        };
        let stderr = lines_channel(child.stderr.take().expect("piped stderr"));
        let process = Self {
            child,
            stdout,
            stderr,
            stdout_lines: Vec::new(),
            stderr_lines: Vec::new(),
            _home: home,
        };
        (process, stdout_eof_capture)
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

    fn take_home(&mut self) -> Option<TempDir> {
        self._home.take()
    }

    fn stdout_is_empty(&mut self) -> bool {
        self.collect();
        self.stdout_lines.is_empty()
    }

    fn wait_for_stderr(&mut self, needle: &str, deadline: Instant) -> bool {
        loop {
            self.collect();
            if self.stderr_lines.iter().any(|line| line.contains(needle)) {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(25));
        }
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

    fn wait_after_startup_failure(&mut self) -> ExitStatus {
        let deadline = Instant::now() + Duration::from_secs(5);
        let status = loop {
            if let Some(status) = self.child.try_wait().expect("poll failed HTTP child") {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "HTTP child did not exit after startup failure; stderr: {}",
                self.stderr_text()
            );
            thread::sleep(Duration::from_millis(20));
        };
        eprintln!("MCP_BIND_FAILURE_CHILD_EXIT status={status:?}");
        assert!(
            status.success(),
            "failed HTTP server startup returns from headless run with status 0: {status}"
        );
        self.collect();
        assert!(
            self.stdout_lines.is_empty(),
            "failed HTTP child stdout: {:?}",
            self.stdout_lines
        );
        status
    }

    fn wait_after_startup_failure_result(&mut self) -> Result<ExitStatus, String> {
        let deadline = Instant::now() + Duration::from_secs(5);
        let status = loop {
            if let Some(status) = self.child.try_wait().map_err(|error| error.to_string())? {
                break status;
            }
            if Instant::now() >= deadline {
                return Err(format!(
                    "HTTP child did not exit after startup failure; stderr: {}",
                    self.stderr_text()
                ));
            }
            thread::sleep(Duration::from_millis(20));
        };
        eprintln!("MCP_BIND_FAILURE_CHILD_EXIT status={status:?}");
        self.collect();
        Ok(status)
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

fn lines_channel_tee<R: Read + Send + 'static>(reader: R) -> TeedLineReceivers {
    let (protocol_tx, protocol_rx) = mpsc::channel();
    let (capture_tx, capture_rx) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(reader).lines() {
            let item = line.map_err(|error| error.to_string());
            let _ = capture_tx.send(item.clone());
            if protocol_tx.send(item).is_err() {
                break;
            }
        }
    });
    (protocol_rx, capture_rx)
}

fn validate_complete_stdio_stdout(
    capture: Receiver<Result<String, String>>,
    deadline: Instant,
) -> Result<Vec<Value>, String> {
    let mut messages = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("stdout reader did not reach EOF before deadline".into());
        }
        match capture.recv_timeout(remaining) {
            Ok(Ok(line)) => {
                let value: Value = serde_json::from_str(&line)
                    .map_err(|error| format!("invalid stdout JSON {line:?}: {error}"))?;
                if !value.is_object() || value["jsonrpc"] != "2.0" {
                    return Err(format!("stdout line is not a JSON-RPC 2.0 object: {value}"));
                }
                messages.push(value);
            }
            Ok(Err(error)) => return Err(format!("stdout read failed: {error}")),
            Err(RecvTimeoutError::Timeout) => {
                return Err("stdout reader did not reach EOF before deadline".into());
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    if messages.is_empty() {
        return Err("stdout contained no JSON-RPC messages".into());
    }
    Ok(messages)
}

fn assert_http_stdout_empty_to_eof(capture: Receiver<Result<String, String>>, deadline: Instant) {
    let remaining = deadline.saturating_duration_since(Instant::now());
    assert!(!remaining.is_zero(), "HTTP stdout reader did not reach EOF");
    match capture.recv_timeout(remaining) {
        Ok(Ok(line)) => panic!("HTTP child emitted stdout through EOF: {line:?}"),
        Ok(Err(error)) => panic!("HTTP stdout read failed: {error}"),
        Err(RecvTimeoutError::Timeout) => panic!("HTTP stdout reader did not reach EOF"),
        Err(RecvTimeoutError::Disconnected) => {}
    }
}

#[test]
fn test_complete_stdio_stdout_validator_rejects_late_noise_and_non_rpc_json() {
    let valid = json!({"jsonrpc":"2.0", "id":1, "result":{}}).to_string();
    let (sender, receiver) = mpsc::channel();
    sender.send(Ok(valid.clone())).unwrap();
    drop(sender);
    assert_eq!(
        validate_complete_stdio_stdout(receiver, Instant::now() + Duration::from_secs(1))
            .unwrap()
            .len(),
        1
    );

    let (sender, receiver) = mpsc::channel();
    sender.send(Ok(valid.clone())).unwrap();
    sender.send(Ok("late non-json noise".into())).unwrap();
    drop(sender);
    assert!(
        validate_complete_stdio_stdout(receiver, Instant::now() + Duration::from_secs(1)).is_err()
    );

    let (sender, receiver) = mpsc::channel();
    sender.send(Ok(valid)).unwrap();
    sender.send(Ok(json!({"result":{}}).to_string())).unwrap();
    drop(sender);
    assert!(
        validate_complete_stdio_stdout(receiver, Instant::now() + Duration::from_secs(1)).is_err()
    );
}

fn is_location_update(message: &Value) -> bool {
    is_resource_update_for(message, PLAYER_LOCATION_URI)
}

fn is_resource_update_for(message: &Value, uri: &str) -> bool {
    message["method"] == "notifications/resources/updated" && message["params"]["uri"] == uri
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

    fn post_without_session(&mut self, body: Value) -> live_mcp::HttpResponse {
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

    fn post(&mut self, body: Value) -> live_mcp::HttpResponse {
        self.post_with_timeout(body, RPC_TIMEOUT)
    }

    fn post_with_timeout(&mut self, body: Value, timeout: Duration) -> live_mcp::HttpResponse {
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
) -> Result<live_mcp::HttpResponse, String> {
    let mut headers = Vec::with_capacity(extra_headers.len() + 2);
    if method == "POST" {
        headers.push(("Content-Type", "application/json"));
        headers.push(("Accept", "application/json, text/event-stream"));
    }
    headers.extend_from_slice(extra_headers);
    live_mcp::http_request_with_timeout(addr, method, path, &headers, body, timeout)
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

    fn assert_no_location_update_for(&self, duration: Duration) {
        let deadline = Instant::now() + duration;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return;
            }
            match self
                .messages
                .recv_timeout(remaining.min(Duration::from_millis(25)))
            {
                Ok(message) => assert!(
                    !is_location_update(&message),
                    "unexpected HTTP location update: {message}"
                ),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("HTTP SSE closed during no-update window")
                }
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

#[cfg(unix)]
#[derive(Clone, Copy, Debug)]
enum StartupCacheCase {
    ColdMissing,
    FreshCurrent,
    StaleCurrent,
    FreshSchema1,
    FreshSemantic1,
    FreshSemantic3,
    FreshSchema3Semantic5,
    FreshCorrupt,
    BypassExisting,
    BypassMissing,
}

#[cfg(unix)]
impl StartupCacheCase {
    const ALL: [Self; 10] = [
        Self::ColdMissing,
        Self::FreshCurrent,
        Self::StaleCurrent,
        Self::FreshSchema1,
        Self::FreshSemantic1,
        Self::FreshSemantic3,
        Self::FreshSchema3Semantic5,
        Self::FreshCorrupt,
        Self::BypassExisting,
        Self::BypassMissing,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::ColdMissing => "cold_missing",
            Self::FreshCurrent => "fresh_current",
            Self::StaleCurrent => "stale_current",
            Self::FreshSchema1 => "fresh_schema1",
            Self::FreshSemantic1 => "fresh_semantic1",
            Self::FreshSemantic3 => "fresh_semantic3",
            Self::FreshSchema3Semantic5 => "fresh_schema3_semantic5",
            Self::FreshCorrupt => "fresh_corrupt",
            Self::BypassExisting => "bypass_existing",
            Self::BypassMissing => "bypass_missing",
        }
    }

    fn source(self) -> Phase {
        match self {
            Self::ColdMissing | Self::FreshSchema1 | Self::FreshCorrupt | Self::BypassMissing => {
                PHASE_C
            }
            Self::FreshCurrent
            | Self::StaleCurrent
            | Self::FreshSemantic1
            | Self::FreshSemantic3
            | Self::FreshSchema3Semantic5
            | Self::BypassExisting => PHASE_A,
        }
    }

    fn no_cache(self) -> bool {
        matches!(self, Self::BypassExisting | Self::BypassMissing)
    }

    fn served(self) -> Phase {
        if matches!(self, Self::FreshCurrent) {
            PHASE_B
        } else {
            self.source()
        }
    }

    fn served_system(self) -> &'static str {
        if matches!(self, Self::FreshCurrent) {
            "Cached-9"
        } else {
            "(unknown)"
        }
    }

    fn rebuilds_cache(self) -> bool {
        matches!(
            self,
            Self::ColdMissing
                | Self::StaleCurrent
                | Self::FreshSchema1
                | Self::FreshSemantic1
                | Self::FreshSemantic3
                | Self::FreshSchema3Semantic5
                | Self::FreshCorrupt
        )
    }
}

#[cfg(unix)]
struct StartupCaseFiles {
    _dir: TempDir,
    save: PathBuf,
    cache: PathBuf,
    source_bytes: Vec<u8>,
    source_model: GalaxyModel,
    initial_cache: Option<Vec<u8>>,
}

#[cfg(unix)]
fn literal_player(phase: Phase) -> PlayerState {
    let address = GalacticAddress::new(phase.voxel_x, 0, 0, 1, 0, phase.reality_index);
    PlayerState::new(address, phase.reality_index, None, None, 7, 8, 9)
}

#[cfg(unix)]
fn assert_source_model_literals(model: &GalaxyModel, phase: Phase) {
    assert_eq!(model.active_galaxy, phase.reality_index);
    assert_eq!(model.player_state.as_ref(), Some(&literal_player(phase)));
    assert!(model.systems.is_empty(), "source fixture has no systems");
    assert!(model.planets.is_empty(), "source fixture has no planets");
    assert!(model.bases.is_empty(), "source fixture has no bases");
}

#[cfg(unix)]
fn sentinel_model() -> GalaxyModel {
    let address = GalacticAddress::new(0, 0, 0, 1, 0, 9);
    let mut model = GalaxyModel::new();
    model.insert_system(System::new(
        address,
        Some("Cached-9".into()),
        None,
        None,
        vec![],
    ));
    model.player_state = Some(PlayerState::new(address, 9, None, None, 700, 800, 900));
    model.set_active_galaxy(7);
    model
}

#[cfg(unix)]
fn sentinel_bytes() -> Vec<u8> {
    let model = sentinel_model();
    let data = nms_cache::extract_cache_data(&model, 9999);
    let bytes = nms_cache::serialize::serialize(&data).expect("serialize current sentinel");
    assert_eq!(&bytes[..16], b"NMSCACHE\x04\0\0\0\x06\0\0\0");
    let dir = tempfile::tempdir().expect("sentinel verification directory");
    let path = dir.path().join("sentinel.rkyv");
    nms_cache::write_cache(&data, &path).expect("write sentinel through cache API");
    assert_eq!(
        std::fs::read(&path).expect("read written sentinel bytes"),
        bytes
    );
    let checked = nms_cache::read_cache(&path).expect("checked-read current sentinel");
    let rebuilt = nms_cache::rebuild_model(&checked);
    assert_eq!(
        checked.player_state,
        Some(PlayerState::new(
            GalacticAddress::new(0, 0, 0, 1, 0, 9),
            9,
            None,
            None,
            700,
            800,
            900,
        ))
    );
    assert_eq!(checked.active_galaxy, 7);
    assert_eq!(checked.save_version, 9999);
    assert_eq!(rebuilt.systems, model.systems);
    assert_eq!(rebuilt.planets, model.planets);
    assert_eq!(rebuilt.bases, model.bases);
    assert_eq!(rebuilt.player_state, model.player_state);
    assert_eq!(rebuilt.active_galaxy, 7);
    assert_eq!(
        rebuilt
            .systems
            .values()
            .next()
            .and_then(|system| system.name.as_deref()),
        Some("Cached-9")
    );
    bytes
}

#[cfg(unix)]
fn set_mtime(path: &Path, time: SystemTime) {
    std::fs::File::options()
        .write(true)
        .open(path)
        .expect("open fixture to set mtime")
        .set_times(std::fs::FileTimes::new().set_modified(time))
        .expect("set exact fixture mtime");
}

#[cfg(unix)]
fn source_mtime() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

#[cfg(unix)]
fn prepare_startup_case(case: StartupCacheCase) -> StartupCaseFiles {
    let dir = tempfile::tempdir().expect("case save/cache directory");
    let save = dir.path().join("save.json");
    let cache = if matches!(case, StartupCacheCase::BypassMissing) {
        dir.path().join("absent/cache.rkyv")
    } else {
        dir.path().join("cache.rkyv")
    };
    let source_bytes = save_bytes(case.source());
    std::fs::write(&save, &source_bytes).expect("write literal source fixture");
    set_mtime(&save, source_mtime());
    let parsed = nms_save::parse_save(&source_bytes).expect("parse literal source fixture");
    let source_model = GalaxyModel::from_save(&parsed);
    assert_source_model_literals(&source_model, case.source());

    let initial_cache = match case {
        StartupCacheCase::ColdMissing | StartupCacheCase::BypassMissing => None,
        StartupCacheCase::FreshCurrent
        | StartupCacheCase::StaleCurrent
        | StartupCacheCase::BypassExisting => {
            let bytes = sentinel_bytes();
            std::fs::write(&cache, &bytes).expect("write healthy current sentinel");
            let cache_time = if matches!(case, StartupCacheCase::StaleCurrent) {
                source_mtime() - Duration::from_secs(100)
            } else {
                source_mtime() + Duration::from_secs(100)
            };
            set_mtime(&cache, cache_time);
            assert_eq!(
                nms_cache::is_cache_fresh(&cache, &save),
                !matches!(case, StartupCacheCase::StaleCurrent),
                "{} actual freshness precondition",
                case.label()
            );
            Some(bytes)
        }
        StartupCacheCase::FreshSchema1 => {
            let bytes =
                include_bytes!("../../nms-cache/tests/fixtures/predecessor-schema1-semantic1.rkyv")
                    .to_vec();
            assert_eq!(bytes.len(), 216);
            assert_eq!(&bytes[..16], b"NMSCACHE\x01\0\0\0\x01\0\0\0");
            std::fs::write(&cache, &bytes).expect("write immutable schema1 predecessor fixture");
            set_mtime(&cache, source_mtime() + Duration::from_secs(100));
            assert!(nms_cache::is_cache_fresh(&cache, &save));
            assert!(matches!(
                nms_cache::read_cache(&cache),
                Err(nms_cache::CacheError::Incompatible(
                    nms_cache::CacheCompatibilityError::SchemaMismatch {
                        found: 1,
                        expected: 4
                    }
                ))
            ));
            Some(bytes)
        }
        StartupCacheCase::FreshSemantic1 => {
            let mut bytes = sentinel_bytes();
            bytes[12..16].copy_from_slice(&1_u32.to_le_bytes());
            std::fs::write(&cache, &bytes).expect("write synthetic semantic1 witness");
            set_mtime(&cache, source_mtime() + Duration::from_secs(100));
            assert!(nms_cache::is_cache_fresh(&cache, &save));
            assert!(matches!(
                nms_cache::read_cache(&cache),
                Err(nms_cache::CacheError::Incompatible(
                    nms_cache::CacheCompatibilityError::SemanticMismatch {
                        found: 1,
                        expected: 6
                    }
                ))
            ));
            Some(bytes)
        }
        StartupCacheCase::FreshSemantic3 => {
            let bytes =
                include_bytes!("../../nms-cache/tests/fixtures/predecessor-schema3-semantic3.rkyv")
                    .to_vec();
            assert_eq!(bytes.len(), 752);
            assert_eq!(&bytes[..16], b"NMSCACHE\x03\0\0\0\x03\0\0\0");
            std::fs::write(&cache, &bytes).expect("write actual schema3 semantic3 predecessor");
            set_mtime(&cache, source_mtime() + Duration::from_secs(100));
            assert!(nms_cache::is_cache_fresh(&cache, &save));
            assert!(matches!(
                nms_cache::read_cache(&cache),
                Err(nms_cache::CacheError::Incompatible(
                    nms_cache::CacheCompatibilityError::SchemaMismatch {
                        found: 3,
                        expected: 4
                    }
                ))
            ));
            Some(bytes)
        }
        StartupCacheCase::FreshSchema3Semantic5 => {
            let bytes =
                include_bytes!("../../nms-cache/tests/fixtures/predecessor-schema3-semantic5.rkyv")
                    .to_vec();
            assert_eq!(bytes.len(), 632);
            assert_eq!(&bytes[..16], b"NMSCACHE\x03\0\0\0\x05\0\0\0");
            std::fs::write(&cache, &bytes).expect("write authentic schema3 semantic5 predecessor");
            set_mtime(&cache, source_mtime() + Duration::from_secs(100));
            assert!(nms_cache::is_cache_fresh(&cache, &save));
            assert!(matches!(
                nms_cache::read_cache(&cache),
                Err(nms_cache::CacheError::Incompatible(
                    nms_cache::CacheCompatibilityError::SchemaMismatch {
                        found: 3,
                        expected: 4
                    }
                ))
            ));
            Some(bytes)
        }
        StartupCacheCase::FreshCorrupt => {
            let bytes = b"NMSCACHE\x04\0\0\0\x06\0\0\0".to_vec();
            std::fs::write(&cache, &bytes).expect("write literal empty current archive");
            set_mtime(&cache, source_mtime() + Duration::from_secs(100));
            assert!(nms_cache::is_cache_fresh(&cache, &save));
            assert!(matches!(
                nms_cache::read_cache(&cache),
                Err(nms_cache::CacheError::Deserialize(_))
            ));
            Some(bytes)
        }
    };
    StartupCaseFiles {
        _dir: dir,
        save,
        cache,
        source_bytes,
        source_model,
        initial_cache,
    }
}

#[cfg(unix)]
fn assert_startup_freshness_precondition(case: StartupCacheCase, files: &StartupCaseFiles) {
    match files.initial_cache.as_ref() {
        Some(_) => assert_eq!(
            nms_cache::is_cache_fresh(&files.cache, &files.save),
            !matches!(case, StartupCacheCase::StaleCurrent),
            "{} freshness immediately before child launch",
            case.label()
        ),
        None => {
            assert!(
                !files.cache.exists(),
                "{} cache is absent immediately before child launch",
                case.label()
            );
            if matches!(case, StartupCacheCase::BypassMissing) {
                assert!(
                    !files.cache.parent().unwrap().exists(),
                    "bypass_missing cache parent is absent immediately before child launch"
                );
            }
        }
    }
}

#[cfg(unix)]
fn assert_cache_effect(case: StartupCacheCase, files: &StartupCaseFiles) -> &'static str {
    if case.rebuilds_cache() {
        let bytes = std::fs::read(&files.cache).expect("startup wrote current cache");
        assert_eq!(&bytes[..16], b"NMSCACHE\x04\0\0\0\x06\0\0\0");
        let data = nms_cache::read_cache(&files.cache).expect("checked-read rebuilt cache");
        assert_eq!(data.save_version, 4720, "{} source version", case.label());
        assert_eq!(data.active_galaxy, case.source().reality_index);
        assert_eq!(data.player_state, Some(literal_player(case.source())));
        let address = GalacticAddress::new(
            case.source().voxel_x,
            0,
            0,
            1,
            0,
            case.source().reality_index,
        );
        let id = nms_core::system::SystemId::from_address(&address);
        assert_eq!(data.systems.len(), 1, "one canonical physical cache key");
        let marker = &data.systems[0];
        assert_eq!(
            marker.address,
            GalacticAddress::from_packed(id.system_address(), id.reality_index())
        );
        assert!(marker.name.is_none());
        assert!(marker.discoverer.is_none());
        assert!(marker.timestamp_secs.is_none());
        assert!(marker.planets.is_empty());
        assert!(marker.is_player_placeholder);
        assert!(data.bases.is_empty(), "persisted source bases stay empty");
        let rebuilt = nms_cache::rebuild_model(&data);
        let source_cache = nms_cache::extract_cache_data(&files.source_model, 4720);
        let mut expected_model = nms_cache::rebuild_model(&source_cache);
        expected_model.ensure_player_system();
        assert_eq!(rebuilt.systems, expected_model.systems);
        assert_eq!(rebuilt.planets, expected_model.planets);
        assert_eq!(rebuilt.bases, expected_model.bases);
        assert_eq!(rebuilt.player_state, expected_model.player_state);
        assert_eq!(rebuilt.active_galaxy, expected_model.active_galaxy);
        assert_eq!(rebuilt.biome_index, expected_model.biome_index);
        assert_eq!(rebuilt.name_index, expected_model.name_index);
        assert_eq!(rebuilt.address_to_id, expected_model.address_to_id);
        assert!(rebuilt.is_player_placeholder(&id));
        "rewritten_current"
    } else if matches!(case, StartupCacheCase::BypassMissing) {
        assert!(!files.cache.exists(), "bypass leaves absent cache absent");
        assert!(
            !files.cache.parent().unwrap().exists(),
            "bypass creates no cache parent"
        );
        "absent_preserved"
    } else {
        assert_eq!(
            std::fs::read(&files.cache).expect("existing cache remains present"),
            files
                .initial_cache
                .as_deref()
                .expect("case starts with cache"),
            "{} existing cache bytes",
            case.label()
        );
        "exact_bytes_preserved"
    }
}

#[cfg(unix)]
fn assert_source_bytes_unchanged(files: &StartupCaseFiles) {
    assert_eq!(
        std::fs::read(&files.save).expect("read source after startup"),
        files.source_bytes,
        "startup must not modify source save"
    );
}

#[cfg(unix)]
fn run_stdio_cache_case(case: StartupCacheCase) {
    let files = prepare_startup_case(case);
    assert_startup_freshness_precondition(case, &files);
    let mut process =
        StdioProcess::spawn_cached_startup(&files.save, &files.cache, case.no_cache());
    process.send(initialize_request());
    let initialized = process.response(1, RPC_TIMEOUT);
    assert_initialized(&initialized["result"]);
    process.send(json!({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}));
    process.send(rpc("tools/list", 2));
    let tools = process.response(2, RPC_TIMEOUT);
    process.send(rpc("resources/list", 3));
    let resources = process.response(3, RPC_TIMEOUT);
    assert_lists(&tools["result"], &resources["result"]);
    let system = call_where_am_i_stdio(
        &mut process,
        10,
        case.served(),
        Instant::now() + RPC_TIMEOUT,
    );
    assert_eq!(
        system,
        case.served_system(),
        "{} actual system",
        case.label()
    );
    let effect = assert_cache_effect(case, &files);
    eprintln!(
        "MCP_CACHE_STARTUP transport=stdio case={} source={} served={} cache_effect={}",
        case.label(),
        case.source().label,
        case.served().label,
        effect
    );
    assert!(process.stdout_is_json_only());
    process.close_stdin_and_wait(Duration::from_secs(5));
    assert_cache_effect(case, &files);
    assert_source_bytes_unchanged(&files);
}

#[cfg(all(unix, feature = "http"))]
fn run_http_cache_case(case: StartupCacheCase) {
    run_http_cache_case_with_first_addr(case, None);
}

#[cfg(all(unix, feature = "http"))]
fn run_http_cache_case_with_first_addr(
    case: StartupCacheCase,
    first_addr: Option<SocketAddr>,
) -> bool {
    let mut last_error = String::new();
    let mut winning = None;
    let mut failed_attempt_path: Option<PathBuf> = None;
    let mut forced_collision_seen = false;
    for attempt in 0..3 {
        let files = prepare_startup_case(case);
        let attempt_path = files._dir.path().to_path_buf();
        if let Some(failed_path) = &failed_attempt_path {
            assert_ne!(
                &attempt_path,
                failed_path,
                "{} bind retry owns newly prepared case files",
                case.label()
            );
        }
        let addr = if attempt == 0 {
            if let Some(addr) = first_addr {
                addr
            } else {
                let listener = TcpListener::bind(("127.0.0.1", 0)).expect("reserve loopback port");
                let addr = listener.local_addr().expect("reserved socket address");
                drop(listener);
                addr
            }
        } else {
            let listener = TcpListener::bind(("127.0.0.1", 0)).expect("reserve loopback port");
            let addr = listener.local_addr().expect("reserved socket address");
            drop(listener);
            addr
        };
        assert_startup_freshness_precondition(case, &files);
        let mut child =
            HttpChild::spawn_cached_startup(&files.save, &files.cache, addr, case.no_cache());
        match wait_health(addr, &mut child) {
            Ok(()) => {
                winning = Some((files, child, HttpClient::new(addr)));
                break;
            }
            Err(message) if message.contains("bind") => {
                if attempt == 0 && first_addr.is_some() {
                    assert!(
                        matches!(case, StartupCacheCase::StaleCurrent),
                        "controlled first-bind collision uses stale_current"
                    );
                    assert!(
                        message.contains("bind"),
                        "forced first-bind attempt reports actual bind diagnostic: {message}"
                    );
                    let effect = assert_cache_effect(case, &files);
                    assert!(
                        nms_cache::is_cache_fresh(&files.cache, &files.save),
                        "failed first-bind child rewrote stale cache before listener bind"
                    );
                    assert_source_bytes_unchanged(&files);
                    eprintln!(
                        "MCP_CACHE_BIND_RETRY transport=http case={} stage=forced_collision diagnostic={:?} cache_effect={} rewritten_cache_fresh=true",
                        case.label(),
                        message,
                        effect
                    );
                    failed_attempt_path = Some(attempt_path);
                    forced_collision_seen = true;
                }
                last_error = message;
                drop(child);
                drop(files);
            }
            Err(message) => panic!(
                "{} HTTP child did not become healthy: {message}",
                case.label()
            ),
        }
    }
    let (files, mut process, mut client) =
        winning.unwrap_or_else(|| panic!("{} HTTP bind collisions: {last_error}", case.label()));
    assert_eq!(
        forced_collision_seen,
        first_addr.is_some(),
        "{} forced first-bind collision observed before successful startup",
        case.label()
    );
    let initialized = client.post_without_session(initialize_request());
    assert_eq!(initialized.status, 200, "{} initialize", case.label());
    let session = initialized
        .header("mcp-session-id")
        .expect("HTTP initialize returns session")
        .to_owned();
    let response = http_json_messages(&initialized.text())
        .into_iter()
        .find(|message| message["id"] == 1)
        .expect("HTTP initialize response ID");
    assert_initialized(&response["result"]);
    let notification =
        client.post(json!({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}));
    assert_eq!(notification.status, 202);
    assert_eq!(client.session.as_deref(), Some(session.as_str()));
    let tools_response = client.post(rpc("tools/list", 2));
    assert_eq!(tools_response.status, 200);
    let tools = http_json_messages(&tools_response.text())
        .into_iter()
        .find(|message| message["id"] == 2)
        .expect("HTTP tools/list response ID");
    let resources_response = client.post(rpc("resources/list", 3));
    assert_eq!(resources_response.status, 200);
    let resources = http_json_messages(&resources_response.text())
        .into_iter()
        .find(|message| message["id"] == 3)
        .expect("HTTP resources/list response ID");
    assert_lists(&tools["result"], &resources["result"]);
    let system = call_where_am_i_http(&mut client, 10, case.served(), Instant::now() + RPC_TIMEOUT);
    assert_eq!(
        system,
        case.served_system(),
        "{} actual system",
        case.label()
    );
    let effect = assert_cache_effect(case, &files);
    eprintln!(
        "MCP_CACHE_STARTUP transport=http case={} source={} served={} cache_effect={}",
        case.label(),
        case.source().label,
        case.served().label,
        effect
    );
    assert!(process.stdout_is_empty(), "HTTP stdout stays empty");
    process.kill_and_wait();
    assert_cache_effect(case, &files);
    assert_source_bytes_unchanged(&files);
    forced_collision_seen
}

#[cfg(unix)]
#[test]
fn test_headless_stdio_cache_startup_matrix() {
    for case in StartupCacheCase::ALL {
        run_stdio_cache_case(case);
    }
}

#[cfg(all(unix, feature = "http"))]
#[test]
fn test_headless_http_cache_startup_matrix() {
    for case in StartupCacheCase::ALL {
        run_http_cache_case(case);
    }
}

#[cfg(all(unix, feature = "http"))]
#[test]
fn test_headless_http_stale_cache_retries_with_original_inputs_after_bind_collision() {
    let occupied = TcpListener::bind(("127.0.0.1", 0)).expect("occupy controlled MCP port");
    let addr = occupied
        .local_addr()
        .expect("read controlled occupied MCP address");
    let collision_seen =
        run_http_cache_case_with_first_addr(StartupCacheCase::StaleCurrent, Some(addr));
    assert!(
        collision_seen,
        "controlled first-bind failure occurred before successful stale-cache retry"
    );
    drop(occupied);
}

#[cfg(unix)]
fn test_live_update_case_phase() {
    let (_dir, save, cache, initial_source) = prepare_live_cache_fixture();
    let home = isolated_live_cache_home(&cache);
    let (mut process, stdout_capture) =
        StdioProcess::spawn_with_cache_policy_captured(&save, Some(home), false);
    process.send(initialize_request());
    let initialized = process.response(1, RPC_TIMEOUT);
    assert_initialized(&initialized["result"]);
    process.send(json!({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}));
    process.send(json!({"jsonrpc":"2.0", "id":20, "method":"resources/subscribe", "params":{"uri":PLAYER_LOCATION_URI}}));
    assert!(process.response(20, RPC_TIMEOUT).get("result").is_some());
    assert_live_cache_state(
        &cache,
        LiveStatePhase {
            version: 4720,
            reality: 0,
            voxel_x: 0,
            units: 7,
            nanites: 8,
            specials: 9,
            previous: None,
            selected: 7,
        },
    );
    assert_eq!(std::fs::read(&save).unwrap(), initial_source);

    for (index, phase) in live_phases().into_iter().enumerate() {
        if index == 5 {
            let retained_cache = std::fs::read(&cache).expect("cache after same-reality phase");
            let fixed_mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_123);
            set_mtime(&cache, fixed_mtime);
            let retained_mtime = std::fs::metadata(&cache)
                .expect("cache metadata before malformed writes")
                .modified()
                .expect("cache mtime before malformed writes");
            assert_eq!(
                retained_mtime, fixed_mtime,
                "fixed cache mtime is observable"
            );
            for attempt in 1..=5 {
                let invalid = format!("{{{{ invalid live cache retry {attempt} }}}}").into_bytes();
                atomic_replace(&save, &invalid);
                process.assert_no_location_update_for(Duration::from_millis(650));
                assert_eq!(
                    std::fs::read(&save).unwrap(),
                    invalid,
                    "malformed source remains exact"
                );
                call_where_am_i_stdio(
                    &mut process,
                    800 + (attempt as u64) * 2,
                    PHASE_B,
                    Instant::now() + RPC_TIMEOUT,
                );
                assert_eq!(std::fs::read(&cache).unwrap(), retained_cache);
                assert_eq!(
                    std::fs::metadata(&cache).unwrap().modified().unwrap(),
                    retained_mtime
                );
                assert_live_cache_state(
                    &cache,
                    LiveStatePhase {
                        version: 4723,
                        reality: 9,
                        voxel_x: 0,
                        units: 701,
                        nanites: 801,
                        specials: 901,
                        previous: Some(GalacticAddress::new(2, 3, 4, 1, 1, 0)),
                        selected: 9,
                    },
                );
            }
            assert!(process.wait_for_stderr(
                "5 consecutive parse failures",
                Instant::now() + Duration::from_secs(12)
            ));
            let retained =
                call_where_am_i_stdio(&mut process, 80, PHASE_B, Instant::now() + RPC_TIMEOUT);
            assert!(!retained.is_empty());
            assert_eq!(std::fs::read(&cache).unwrap(), retained_cache);
            assert_eq!(
                std::fs::metadata(&cache).unwrap().modified().unwrap(),
                retained_mtime
            );
        }
        let previous_tuple = phase.previous.map(|address| {
            (
                address.reality_index,
                i64::from(address.voxel_x()),
                i64::from(address.voxel_y()),
                i64::from(address.voxel_z()),
                address.solar_system_index(),
                address.planet_index(),
            )
        });
        let bytes = live_save_bytes(
            u64::from(phase.version),
            phase.reality,
            i64::from(phase.voxel_x),
            phase.units,
            phase.nanites,
            phase.specials,
            previous_tuple,
        );
        atomic_replace(&save, &bytes);
        let phase_deadline = Instant::now() + PHASE_TIMEOUT;
        process.wait_location_update(remaining(phase_deadline));
        thread::sleep(Duration::from_millis(650));
        let phase_info = if phase.reality == 9 { PHASE_B } else { PHASE_A };
        let system = call_where_am_i_stdio(
            &mut process,
            30 + (index as u64) * 2,
            phase_info,
            phase_deadline,
        );
        assert!(!system.is_empty());
        assert_live_cache_state(&cache, phase);
        assert_eq!(std::fs::read(&save).unwrap(), bytes);
        eprintln!(
            "SLICE16_LIVE transport=stdio phase={} version={} reality={} selected={}",
            index + 1,
            phase.version,
            phase.reality,
            phase.selected
        );
    }
    let final_cache = std::fs::read(&cache).expect("cache after recovery");
    let recovery_control_mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_777);
    set_mtime(&cache, recovery_control_mtime);
    let final_mtime = std::fs::metadata(&cache).unwrap().modified().unwrap();
    assert_eq!(
        final_mtime, recovery_control_mtime,
        "recovery-control mtime is distinct and observable"
    );
    let final_source = std::fs::read(&save).expect("final recovered source");
    atomic_replace(&save, &final_source);
    process.assert_no_location_update_for(Duration::from_millis(650));
    assert_eq!(std::fs::read(&cache).unwrap(), final_cache);
    assert_eq!(
        std::fs::metadata(&cache).unwrap().modified().unwrap(),
        final_mtime
    );
    assert!(!cache.with_extension("rkyv.tmp").exists());
    assert!(process.stdout_is_json_only());
    process.close_stdin_and_wait(Duration::from_secs(5));
    validate_complete_stdio_stdout(stdout_capture, Instant::now() + Duration::from_secs(5))
        .expect("complete stdio stdout remains JSON-RPC through EOF");
    assert_eq!(
        std::fs::read(&save).unwrap(),
        final_source,
        "warm stdio source bytes remain exact through EOF"
    );
    assert_eq!(
        std::fs::read(&cache).unwrap(),
        final_cache,
        "warm stdio cache bytes remain exact through EOF"
    );
    assert_eq!(
        std::fs::metadata(&cache).unwrap().modified().unwrap(),
        final_mtime,
        "warm stdio fixed cache mtime remains exact through EOF"
    );
    assert!(!cache.with_extension("rkyv.tmp").exists());
    assert_live_cache_state(&cache, *live_phases().last().expect("final live phase"));
}

#[cfg(unix)]
#[test]
fn test_headless_stdio_complete_live_update_matrix() {
    test_live_update_case_phase();
}

#[cfg(all(unix, feature = "http"))]
#[test]
fn test_headless_http_complete_live_update_matrix() {
    run_headless_http_complete_live_update_matrix(None);
}

#[cfg(all(unix, feature = "http"))]
fn run_headless_http_complete_live_update_matrix(first_addr: Option<SocketAddr>) {
    let mut failed_fixture: Option<PathBuf> = None;
    let mut attempts = 0;
    let (_dir, save, cache, addr, mut process, stdout_capture) = loop {
        assert!(
            attempts < 3,
            "HTTP warm retry exhausted three fresh attempts"
        );
        attempts += 1;
        let (dir, save, cache, initial_source) = prepare_live_cache_fixture();
        let fixture_path = dir.path().to_path_buf();
        assert_eq!(
            std::fs::read(&save).unwrap(),
            initial_source,
            "warm HTTP fixture starts with its exact authored source"
        );
        assert_eq!(
            std::fs::metadata(&save).unwrap().modified().unwrap(),
            source_mtime(),
            "warm HTTP fixture starts with its exact source mtime"
        );
        assert_live_cache_state(
            &cache,
            LiveStatePhase {
                version: 4720,
                reality: 0,
                voxel_x: 0,
                units: 7,
                nanites: 8,
                specials: 9,
                previous: None,
                selected: 7,
            },
        );
        let initial_cache_bytes = std::fs::read(&cache).expect("initial warm HTTP cache bytes");
        let initial_source_mtime = std::fs::metadata(&save)
            .expect("initial warm HTTP source metadata")
            .modified()
            .expect("initial warm HTTP source mtime");
        let initial_cache_mtime = std::fs::metadata(&cache)
            .expect("initial warm HTTP cache metadata")
            .modified()
            .expect("initial warm HTTP cache mtime");
        assert_eq!(initial_source_mtime, source_mtime());
        assert_eq!(
            initial_cache_mtime,
            source_mtime() + Duration::from_secs(100),
            "warm HTTP cache begins with the authored fresh-cache mtime"
        );
        assert!(nms_cache::is_cache_fresh(&cache, &save));
        assert!(!cache.with_extension("rkyv.tmp").exists());
        let home = isolated_live_cache_home(&cache);
        let addr = if failed_fixture.is_none() {
            if let Some(addr) = first_addr {
                addr
            } else {
                let listener =
                    TcpListener::bind(("127.0.0.1", 0)).expect("reserve live HTTP address");
                let addr = listener.local_addr().unwrap();
                drop(listener);
                addr
            }
        } else {
            let listener =
                TcpListener::bind(("127.0.0.1", 0)).expect("reserve retry live HTTP address");
            let addr = listener.local_addr().unwrap();
            drop(listener);
            addr
        };
        let (mut process, stdout_capture) =
            HttpChild::spawn_with_cache_policy_captured(&save, addr, Some(home), false);
        match wait_health(addr, &mut process) {
            Ok(()) => {
                assert_eq!(std::fs::read(&save).unwrap(), initial_source);
                assert_eq!(std::fs::read(&cache).unwrap(), initial_cache_bytes);
                assert_eq!(
                    std::fs::metadata(&save).unwrap().modified().unwrap(),
                    initial_source_mtime
                );
                assert_eq!(
                    std::fs::metadata(&cache).unwrap().modified().unwrap(),
                    initial_cache_mtime
                );
                assert_live_cache_state(
                    &cache,
                    LiveStatePhase {
                        version: 4720,
                        reality: 0,
                        voxel_x: 0,
                        units: 7,
                        nanites: 8,
                        specials: 9,
                        previous: None,
                        selected: 7,
                    },
                );
                assert!(!cache.with_extension("rkyv.tmp").exists());
                if let Some(previous) = &failed_fixture {
                    assert_ne!(&fixture_path, previous, "HTTP retry uses a fresh fixture");
                }
                break (dir, save, cache, addr, process, stdout_capture);
            }
            Err(error) => {
                let observed_stderr = process.stderr_text();
                let evidence = format!("{error}\n{observed_stderr}");
                if !evidence
                    .to_ascii_lowercase()
                    .contains("address already in use")
                {
                    panic!("HTTP startup failed without a bind collision: {evidence}");
                }
                process.wait_after_startup_failure();
                assert_http_stdout_empty_to_eof(
                    stdout_capture,
                    Instant::now() + Duration::from_secs(5),
                );
                let stderr = process.stderr_text();
                assert!(
                    stderr.contains(&format!("MCP server error: bind {addr} failed:")),
                    "bind stderr identifies exact listener/address: {stderr:?}"
                );
                assert!(
                    stderr.contains("Address already in use"),
                    "bind stderr includes the operating system collision: {stderr:?}"
                );
                assert!(
                    !stderr.contains("MCP server listening on http://"),
                    "failed bind emitted no HTTP readiness line: {stderr:?}"
                );
                assert_eq!(
                    std::fs::read(&save).unwrap(),
                    initial_source,
                    "failed warm HTTP child leaves exact authored source through exit"
                );
                assert_eq!(
                    std::fs::read(&cache).unwrap(),
                    initial_cache_bytes,
                    "failed warm HTTP child leaves exact initial cache through exit"
                );
                assert_eq!(
                    std::fs::metadata(&save).unwrap().modified().unwrap(),
                    initial_source_mtime,
                    "failed warm HTTP child leaves exact source mtime through exit"
                );
                assert_eq!(
                    std::fs::metadata(&cache).unwrap().modified().unwrap(),
                    initial_cache_mtime,
                    "failed warm HTTP child leaves exact cache mtime through exit"
                );
                assert_live_cache_state(
                    &cache,
                    LiveStatePhase {
                        version: 4720,
                        reality: 0,
                        voxel_x: 0,
                        units: 7,
                        nanites: 8,
                        specials: 9,
                        previous: None,
                        selected: 7,
                    },
                );
                assert!(!cache.with_extension("rkyv.tmp").exists());
                let preserved = dir.keep();
                if let Some(previous) = &failed_fixture {
                    assert_ne!(&preserved, previous, "each retry preserves a fresh fixture");
                }
                eprintln!(
                    "SLICE16_WARM_HTTP_BIND_RETRY attempt={} address={} stderr={:?} preserved_fixture={}",
                    attempts,
                    addr,
                    stderr,
                    preserved.display()
                );
                failed_fixture = Some(preserved);
            }
        }
    };
    let mut client = HttpClient::new(addr);
    let initialized = client.post_without_session(initialize_request());
    assert_eq!(initialized.status, 200);
    let session = initialized.header("mcp-session-id").unwrap().to_owned();
    let response = http_json_messages(&initialized.text())
        .into_iter()
        .find(|message| message["id"] == 1)
        .unwrap();
    assert_initialized(&response["result"]);
    client.post(json!({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}));
    let events = SseStream::connect(addr, &session);
    let subscribed = client.post(json!({"jsonrpc":"2.0", "id":20, "method":"resources/subscribe", "params":{"uri":PLAYER_LOCATION_URI}}));
    assert_eq!(subscribed.status, 200);
    call_where_am_i_http(&mut client, 10, PHASE_A, Instant::now() + RPC_TIMEOUT);

    for (index, phase) in live_phases().into_iter().enumerate() {
        if index == 5 {
            let retained_cache = std::fs::read(&cache).expect("cache after same-reality phase");
            let retained_mtime = std::fs::metadata(&cache)
                .expect("cache metadata before malformed writes")
                .modified()
                .expect("cache mtime before malformed writes");
            for attempt in 1..=5 {
                let invalid = format!("{{{{ invalid live cache retry {attempt} }}}}").into_bytes();
                atomic_replace(&save, &invalid);
                events.assert_no_location_update_for(Duration::from_millis(650));
                assert_eq!(
                    std::fs::read(&save).unwrap(),
                    invalid,
                    "malformed source remains exact"
                );
                call_where_am_i_http(
                    &mut client,
                    800 + (attempt as u64) * 2,
                    PHASE_B,
                    Instant::now() + RPC_TIMEOUT,
                );
                assert_eq!(std::fs::read(&cache).unwrap(), retained_cache);
                assert_eq!(
                    std::fs::metadata(&cache).unwrap().modified().unwrap(),
                    retained_mtime
                );
                assert_live_cache_state(
                    &cache,
                    LiveStatePhase {
                        version: 4723,
                        reality: 9,
                        voxel_x: 0,
                        units: 701,
                        nanites: 801,
                        specials: 901,
                        previous: Some(GalacticAddress::new(2, 3, 4, 1, 1, 0)),
                        selected: 9,
                    },
                );
            }
            assert!(process.wait_for_stderr(
                "5 consecutive parse failures",
                Instant::now() + Duration::from_secs(12)
            ));
            let retained =
                call_where_am_i_http(&mut client, 80, PHASE_B, Instant::now() + RPC_TIMEOUT);
            assert!(!retained.is_empty());
            assert_eq!(std::fs::read(&cache).unwrap(), retained_cache);
            assert_eq!(
                std::fs::metadata(&cache).unwrap().modified().unwrap(),
                retained_mtime
            );
        }
        let previous_tuple = phase.previous.map(|address| {
            (
                address.reality_index,
                i64::from(address.voxel_x()),
                i64::from(address.voxel_y()),
                i64::from(address.voxel_z()),
                address.solar_system_index(),
                address.planet_index(),
            )
        });
        let bytes = live_save_bytes(
            u64::from(phase.version),
            phase.reality,
            i64::from(phase.voxel_x),
            phase.units,
            phase.nanites,
            phase.specials,
            previous_tuple,
        );
        atomic_replace(&save, &bytes);
        let phase_deadline = Instant::now() + PHASE_TIMEOUT;
        events.wait_location_update(phase_deadline);
        thread::sleep(Duration::from_millis(650));
        let phase_info = if phase.reality == 9 { PHASE_B } else { PHASE_A };
        call_where_am_i_http(
            &mut client,
            30 + (index as u64) * 2,
            phase_info,
            phase_deadline,
        );
        assert_live_cache_state(&cache, phase);
        assert_eq!(std::fs::read(&save).unwrap(), bytes);
        eprintln!(
            "SLICE16_LIVE transport=http phase={} version={} reality={} selected={}",
            index + 1,
            phase.version,
            phase.reality,
            phase.selected
        );
    }
    let final_cache = std::fs::read(&cache).expect("cache after recovery");
    let recovery_control_mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_777);
    set_mtime(&cache, recovery_control_mtime);
    let final_mtime = std::fs::metadata(&cache).unwrap().modified().unwrap();
    assert_eq!(
        final_mtime, recovery_control_mtime,
        "recovery-control mtime is distinct and observable"
    );
    let final_source = std::fs::read(&save).expect("final recovered source");
    atomic_replace(&save, &final_source);
    events.assert_no_location_update_for(Duration::from_millis(650));
    assert_eq!(std::fs::read(&cache).unwrap(), final_cache);
    assert_eq!(
        std::fs::metadata(&cache).unwrap().modified().unwrap(),
        final_mtime
    );
    assert!(!cache.with_extension("rkyv.tmp").exists());
    assert!(process.stdout_is_empty());
    process.kill_and_wait();
    assert_http_stdout_empty_to_eof(stdout_capture, Instant::now() + Duration::from_secs(5));
    assert_eq!(std::fs::read(&save).unwrap(), final_source);
    assert_eq!(std::fs::read(&cache).unwrap(), final_cache);
    assert_eq!(
        std::fs::metadata(&cache).unwrap().modified().unwrap(),
        final_mtime
    );
    assert!(!cache.with_extension("rkyv.tmp").exists());
    assert_live_cache_state(&cache, *live_phases().last().expect("final live phase"));
}

#[cfg(all(unix, feature = "http"))]
#[test]
fn test_headless_http_complete_live_update_retries_after_first_bind_collision() {
    let occupied = TcpListener::bind(("127.0.0.1", 0)).expect("occupy first warm HTTP address");
    run_headless_http_complete_live_update_matrix(Some(occupied.local_addr().unwrap()));
}

#[cfg(unix)]
#[derive(Clone, Copy, Debug)]
enum LiveStartupCase {
    Cold,
    BypassExisting,
    BypassMissing,
    DisabledCurrent,
    WriteError,
}

#[cfg(unix)]
impl LiveStartupCase {
    fn label(self) -> &'static str {
        match self {
            Self::Cold => "Cold",
            Self::BypassExisting => "BypassExisting",
            Self::BypassMissing => "BypassMissing",
            Self::DisabledCurrent => "DisabledCurrent",
            Self::WriteError => "WriteError",
        }
    }
}

#[cfg(unix)]
struct LiveControlFixture {
    directory: TempDir,
    save: PathBuf,
    cache: PathBuf,
    source: Vec<u8>,
    home: TempDir,
    cache_exists: bool,
    blocker: Option<(PathBuf, Vec<u8>)>,
}

#[cfg(unix)]
fn prepare_control_case(case: LiveStartupCase) -> LiveControlFixture {
    let (directory, save, mut cache, source) = prepare_live_cache_fixture();
    if matches!(case, LiveStartupCase::BypassExisting) {
        std::fs::write(&cache, sentinel_bytes()).expect("seed existing bypass sentinel bytes");
        let fixed = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_321);
        set_mtime(&cache, fixed);
        assert_eq!(
            std::fs::metadata(&cache).unwrap().modified().unwrap(),
            fixed
        );
    }
    let cache_exists = !matches!(
        case,
        LiveStartupCase::Cold | LiveStartupCase::BypassMissing | LiveStartupCase::WriteError
    );
    if matches!(case, LiveStartupCase::Cold | LiveStartupCase::BypassMissing) {
        std::fs::remove_file(&cache).expect("remove seeded cache for missing case");
    }
    if matches!(case, LiveStartupCase::BypassMissing) {
        cache = directory.path().join("absent/cache.rkyv");
        assert!(
            !cache.parent().unwrap().exists(),
            "bypass-missing parent starts absent"
        );
    }
    let blocker = if matches!(case, LiveStartupCase::WriteError) {
        let path = directory.path().join("cache-parent-is-a-file");
        let bytes = b"slice16 cache parent blocker\n".to_vec();
        std::fs::write(&path, &bytes).expect("create regular-file cache parent blocker");
        cache = path.join("cache.rkyv");
        Some((path, bytes))
    } else {
        None
    };
    let enabled = !matches!(case, LiveStartupCase::DisabledCurrent);
    let home = if enabled {
        isolated_live_cache_home(&cache)
    } else {
        isolated_cache_home(&cache)
    };
    LiveControlFixture {
        directory,
        save,
        cache,
        source,
        home,
        cache_exists,
        blocker,
    }
}

#[cfg(unix)]
fn run_stdio_live_control(case: LiveStartupCase) {
    let LiveControlFixture {
        directory: _directory,
        save,
        cache,
        source,
        home,
        cache_exists,
        blocker,
    } = prepare_control_case(case);
    let initial_bytes = if cache_exists {
        Some(std::fs::read(&cache).expect("initial cache bytes"))
    } else {
        None
    };
    let initial_mtime = if cache_exists {
        Some(std::fs::metadata(&cache).unwrap().modified().unwrap())
    } else {
        None
    };
    let no_cache = matches!(
        case,
        LiveStartupCase::BypassExisting | LiveStartupCase::BypassMissing
    );
    let (mut process, stdout_capture) =
        StdioProcess::spawn_with_cache_policy_captured(&save, Some(home), no_cache);
    if matches!(case, LiveStartupCase::WriteError) {
        assert!(
            process.wait_for_stderr(
                "could not write cache",
                Instant::now() + Duration::from_secs(2)
            ),
            "startup cache write failure is best-effort and reported"
        );
        assert!(!cache.exists());
        assert!(!cache.with_extension("rkyv.tmp").exists());
    }
    assert_eq!(
        std::fs::read(&save).unwrap(),
        source,
        "control starts with exact authored source"
    );
    process.send(initialize_request());
    assert_initialized(&process.response(1, RPC_TIMEOUT)["result"]);
    process.send(json!({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}));
    if matches!(case, LiveStartupCase::Cold) {
        assert_cold_source_cache(&cache);
        assert!(!cache.with_extension("rkyv.tmp").exists());
    } else if matches!(case, LiveStartupCase::DisabledCurrent) {
        assert_live_cache_state(
            &cache,
            LiveStatePhase {
                version: 4720,
                reality: 0,
                voxel_x: 0,
                units: 7,
                nanites: 8,
                specials: 9,
                previous: None,
                selected: 7,
            },
        );
    }
    process.send(json!({"jsonrpc":"2.0", "id":20, "method":"resources/subscribe", "params":{"uri":PLAYER_LOCATION_URI}}));
    assert!(process.response(20, RPC_TIMEOUT).get("result").is_some());
    thread::sleep(Duration::from_millis(500));

    let live_reality = 9;
    let bytes = live_save_bytes(4721, live_reality, 0, 700, 800, 900, None);
    atomic_replace(&save, &bytes);
    if !matches!(case, LiveStartupCase::DisabledCurrent) {
        process.wait_location_update(PHASE_TIMEOUT);
    }
    thread::sleep(Duration::from_millis(500));
    let served_phase = if matches!(case, LiveStartupCase::DisabledCurrent) {
        PHASE_A
    } else {
        PHASE_B
    };
    call_where_am_i_stdio(&mut process, 30, served_phase, Instant::now() + RPC_TIMEOUT);
    if matches!(case, LiveStartupCase::WriteError) {
        assert!(
            process.wait_for_stderr(
                "could not update cache",
                Instant::now() + Duration::from_secs(2)
            ),
            "coordinator logs live cache failure: {}",
            process.stderr_text()
        );
        assert!(!cache.exists(), "blocked cache target remains absent");
        assert!(!cache.with_extension("rkyv.tmp").exists());
        let (path, expected) = blocker.as_ref().expect("regular-file parent blocker");
        assert_eq!(std::fs::read(path).unwrap(), *expected);
        let warnings = process.stderr_text();
        assert!(warnings.matches("could not write cache").count() >= 1);
        assert!(warnings.matches("could not update cache").count() >= 1);
    } else if matches!(
        case,
        LiveStartupCase::BypassExisting
            | LiveStartupCase::BypassMissing
            | LiveStartupCase::DisabledCurrent
    ) {
        if let Some(expected) = initial_bytes.as_ref() {
            assert_eq!(
                std::fs::read(&cache).unwrap(),
                *expected,
                "{} preserves existing cache bytes",
                case.label()
            );
            assert_eq!(
                std::fs::metadata(&cache).unwrap().modified().unwrap(),
                *initial_mtime.as_ref().unwrap(),
                "{} preserves cache mtime",
                case.label()
            );
        } else {
            assert!(
                !cache.exists(),
                "{} leaves absent cache absent",
                case.label()
            );
            if matches!(case, LiveStartupCase::BypassMissing) {
                assert!(
                    !cache.parent().unwrap().exists(),
                    "bypass missing leaves parent absent"
                );
            }
        }
    } else {
        assert_cold_live_cache(&cache);
    }
    eprintln!("SLICE16_CONTROL transport=stdio case={}", case.label());
    assert_eq!(
        std::fs::read(&save).unwrap(),
        live_save_bytes(4721, 9, 0, 700, 800, 900, None)
    );
    if let Some((path, expected)) = blocker.as_ref() {
        assert_eq!(std::fs::read(path).unwrap(), *expected);
    }
    assert!(!cache.with_extension("rkyv.tmp").exists());
    assert!(process.stdout_is_json_only());
    process.close_stdin_and_wait(Duration::from_secs(5));
    validate_complete_stdio_stdout(stdout_capture, Instant::now() + Duration::from_secs(5))
        .expect("complete stdio control stdout remains JSON-RPC through EOF");
    assert_eq!(
        std::fs::read(&save).unwrap(),
        live_save_bytes(4721, 9, 0, 700, 800, 900, None)
    );
    if let Some((path, expected)) = blocker.as_ref() {
        assert_eq!(
            std::fs::read(path).unwrap(),
            *expected,
            "blocker remains through EOF"
        );
    }
    if matches!(
        case,
        LiveStartupCase::BypassExisting | LiveStartupCase::DisabledCurrent
    ) {
        assert_eq!(
            std::fs::read(&cache).unwrap(),
            *initial_bytes.as_ref().unwrap()
        );
        assert_eq!(
            std::fs::metadata(&cache).unwrap().modified().unwrap(),
            *initial_mtime.as_ref().unwrap()
        );
    }
    match case {
        LiveStartupCase::Cold => assert_cold_live_cache(&cache),
        LiveStartupCase::BypassExisting => {
            assert!(!cache.with_extension("rkyv.tmp").exists());
        }
        LiveStartupCase::BypassMissing => {
            assert!(!cache.exists());
            assert!(!cache.parent().unwrap().exists());
            assert!(!cache.with_extension("rkyv.tmp").exists());
        }
        LiveStartupCase::DisabledCurrent => {
            assert_live_cache_state(
                &cache,
                LiveStatePhase {
                    version: 4720,
                    reality: 0,
                    voxel_x: 0,
                    units: 7,
                    nanites: 8,
                    specials: 9,
                    previous: None,
                    selected: 7,
                },
            );
            assert!(!cache.with_extension("rkyv.tmp").exists());
        }
        LiveStartupCase::WriteError => {
            assert!(!cache.exists());
            assert!(!cache.with_extension("rkyv.tmp").exists());
            let warnings = process.stderr_text();
            assert!(warnings.matches("could not write cache").count() >= 1);
            assert!(warnings.matches("could not update cache").count() >= 1);
        }
    }
    assert!(!cache.with_extension("rkyv.tmp").exists());
}

#[cfg(all(unix, feature = "http"))]
fn run_http_live_control(case: LiveStartupCase) {
    run_http_live_control_with_first_addr(case, None);
}

#[cfg(all(unix, feature = "http"))]
fn run_http_live_control_with_first_addr(case: LiveStartupCase, first_addr: Option<SocketAddr>) {
    let mut failed_fixture: Option<PathBuf> = None;
    let mut collision_seen = false;
    for attempt in 0..3 {
        let addr = if attempt == 0 {
            if let Some(addr) = first_addr {
                addr
            } else {
                let listener =
                    TcpListener::bind(("127.0.0.1", 0)).expect("reserve live control HTTP port");
                let addr = listener.local_addr().unwrap();
                drop(listener);
                addr
            }
        } else {
            let listener = TcpListener::bind(("127.0.0.1", 0)).expect("reserve retry HTTP port");
            let addr = listener.local_addr().unwrap();
            drop(listener);
            addr
        };
        match run_http_live_control_attempt(case, addr) {
            Ok(fixture) => {
                if let Some(failed) = &failed_fixture {
                    assert_ne!(
                        &fixture, failed,
                        "HTTP retry uses a fresh source/cache/HOME fixture"
                    );
                }
                assert_eq!(
                    collision_seen,
                    first_addr.is_some(),
                    "controlled bind witness observed"
                );
                return;
            }
            Err((fixture, evidence))
                if evidence
                    .to_ascii_lowercase()
                    .contains("address already in use") =>
            {
                if let Some(previous) = &failed_fixture {
                    assert_ne!(
                        &fixture, previous,
                        "each HTTP bind retry gets a fresh fixture"
                    );
                }
                if attempt == 0 && first_addr.is_some() {
                    assert!(
                        matches!(case, LiveStartupCase::Cold),
                        "forced live collision uses cache-mutating Cold case"
                    );
                    collision_seen = true;
                }
                eprintln!(
                    "SLICE16_HTTP_BIND_RETRY case={} attempt={} fixture={} evidence={:?}",
                    case.label(),
                    attempt + 1,
                    fixture.display(),
                    evidence
                );
                failed_fixture = Some(fixture);
            }
            Err((fixture, evidence)) => panic!(
                "{} HTTP control failed in fixture {}: {evidence}",
                case.label(),
                fixture.display()
            ),
        }
    }
    panic!("{} HTTP control exhausted bind retries", case.label());
}

#[cfg(all(unix, feature = "http"))]
fn run_http_live_control_attempt(
    case: LiveStartupCase,
    addr: SocketAddr,
) -> Result<PathBuf, (PathBuf, String)> {
    let LiveControlFixture {
        directory: _directory,
        save,
        cache,
        source,
        home,
        cache_exists,
        blocker,
    } = prepare_control_case(case);
    let fixture_path = _directory.path().to_path_buf();
    let initial_bytes = if cache_exists {
        Some(std::fs::read(&cache).expect("initial cache bytes"))
    } else {
        None
    };
    let initial_mtime = if cache_exists {
        Some(std::fs::metadata(&cache).unwrap().modified().unwrap())
    } else {
        None
    };
    let no_cache = matches!(
        case,
        LiveStartupCase::BypassExisting | LiveStartupCase::BypassMissing
    );
    let (mut process, stdout_capture) =
        HttpChild::spawn_with_cache_policy_captured(&save, addr, Some(home), no_cache);
    if let Err(error) = wait_health(addr, &mut process) {
        let stderr = process.stderr_text();
        let evidence = format!("{error}\n{stderr}");
        if evidence
            .to_ascii_lowercase()
            .contains("address already in use")
        {
            assert_eq!(
                std::fs::read(&save).unwrap(),
                source,
                "failed bind leaves exact source intact"
            );
            if matches!(case, LiveStartupCase::Cold) {
                assert_cold_source_cache(&cache);
            }
            if matches!(
                case,
                LiveStartupCase::BypassExisting | LiveStartupCase::DisabledCurrent
            ) {
                assert_eq!(
                    std::fs::read(&cache).unwrap(),
                    *initial_bytes.as_ref().unwrap()
                );
                assert_eq!(
                    std::fs::metadata(&cache).unwrap().modified().unwrap(),
                    *initial_mtime.as_ref().unwrap()
                );
            }
            if matches!(case, LiveStartupCase::BypassMissing) {
                assert!(!cache.exists());
                assert!(!cache.parent().unwrap().exists());
                assert!(!cache.with_extension("rkyv.tmp").exists());
            }
            if matches!(case, LiveStartupCase::WriteError) {
                assert!(!cache.exists());
                assert!(!cache.with_extension("rkyv.tmp").exists());
            }
            if let Some((path, expected)) = &blocker {
                assert_eq!(std::fs::read(path).unwrap(), *expected);
            }
            process.wait_after_startup_failure();
            assert_http_stdout_empty_to_eof(
                stdout_capture,
                Instant::now() + Duration::from_secs(5),
            );
            assert!(!cache.with_extension("rkyv.tmp").exists());
            let preserved_path = _directory.keep();
            return Err((preserved_path, evidence));
        }
        panic!("live control HTTP server failed without a bind collision: {error}; {stderr}");
    }
    if matches!(case, LiveStartupCase::WriteError) {
        assert!(
            process.wait_for_stderr(
                "could not write cache",
                Instant::now() + Duration::from_secs(2)
            ),
            "startup cache write failure is best-effort and reported"
        );
        assert!(!cache.exists());
        assert!(!cache.with_extension("rkyv.tmp").exists());
    }
    assert_eq!(
        std::fs::read(&save).unwrap(),
        source,
        "control starts with exact authored source"
    );
    if matches!(case, LiveStartupCase::Cold) {
        assert_cold_source_cache(&cache);
        assert!(!cache.with_extension("rkyv.tmp").exists());
    } else if matches!(case, LiveStartupCase::DisabledCurrent) {
        assert_live_cache_state(
            &cache,
            LiveStatePhase {
                version: 4720,
                reality: 0,
                voxel_x: 0,
                units: 7,
                nanites: 8,
                specials: 9,
                previous: None,
                selected: 7,
            },
        );
    }
    let mut client = HttpClient::new(addr);
    let initialized = client.post_without_session(initialize_request());
    assert_eq!(initialized.status, 200);
    let session = initialized.header("mcp-session-id").unwrap().to_owned();
    let response = http_json_messages(&initialized.text())
        .into_iter()
        .find(|message| message["id"] == 1)
        .unwrap();
    assert_initialized(&response["result"]);
    client.post(json!({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}));
    let events = SseStream::connect(addr, &session);
    let subscribed = client.post(json!({"jsonrpc":"2.0", "id":20, "method":"resources/subscribe", "params":{"uri":PLAYER_LOCATION_URI}}));
    assert_eq!(subscribed.status, 200);
    call_where_am_i_http(&mut client, 10, PHASE_A, Instant::now() + RPC_TIMEOUT);
    thread::sleep(Duration::from_millis(500));

    let live_reality = 9;
    let bytes = live_save_bytes(4721, live_reality, 0, 700, 800, 900, None);
    atomic_replace(&save, &bytes);
    if !matches!(case, LiveStartupCase::DisabledCurrent) {
        events.wait_location_update(Instant::now() + PHASE_TIMEOUT);
    }
    thread::sleep(Duration::from_millis(500));
    let served_phase = if matches!(case, LiveStartupCase::DisabledCurrent) {
        PHASE_A
    } else {
        PHASE_B
    };
    call_where_am_i_http(&mut client, 30, served_phase, Instant::now() + RPC_TIMEOUT);
    if matches!(case, LiveStartupCase::WriteError) {
        assert!(
            process.wait_for_stderr(
                "could not update cache",
                Instant::now() + Duration::from_secs(2)
            ),
            "coordinator logs live cache failure"
        );
        assert!(!cache.exists());
        assert!(!cache.with_extension("rkyv.tmp").exists());
        let (path, expected) = blocker.as_ref().expect("regular-file parent blocker");
        assert_eq!(std::fs::read(path).unwrap(), *expected);
        assert!(
            process
                .stderr_text()
                .matches("could not update cache")
                .count()
                >= 1
        );
    } else if matches!(
        case,
        LiveStartupCase::BypassExisting
            | LiveStartupCase::BypassMissing
            | LiveStartupCase::DisabledCurrent
    ) {
        if let Some(expected) = initial_bytes.as_ref() {
            assert_eq!(std::fs::read(&cache).unwrap(), *expected);
            assert_eq!(
                std::fs::metadata(&cache).unwrap().modified().unwrap(),
                *initial_mtime.as_ref().unwrap()
            );
        } else {
            assert!(!cache.exists());
            if matches!(case, LiveStartupCase::BypassMissing) {
                assert!(!cache.parent().unwrap().exists());
            }
        }
    } else {
        assert_cold_live_cache(&cache);
    }
    eprintln!("SLICE16_CONTROL transport=http case={}", case.label());
    assert_eq!(
        std::fs::read(&save).unwrap(),
        live_save_bytes(4721, 9, 0, 700, 800, 900, None)
    );
    if let Some((path, expected)) = blocker.as_ref() {
        assert_eq!(std::fs::read(path).unwrap(), *expected);
    }
    assert!(!cache.with_extension("rkyv.tmp").exists());
    assert!(process.stdout_is_empty());
    process.kill_and_wait();
    assert_http_stdout_empty_to_eof(stdout_capture, Instant::now() + Duration::from_secs(5));
    assert_eq!(
        std::fs::read(&save).unwrap(),
        live_save_bytes(4721, 9, 0, 700, 800, 900, None)
    );
    if let Some((path, expected)) = blocker.as_ref() {
        assert_eq!(
            std::fs::read(path).unwrap(),
            *expected,
            "blocker remains through HTTP teardown"
        );
    }
    if matches!(case, LiveStartupCase::BypassExisting) {
        assert_eq!(
            std::fs::read(&cache).unwrap(),
            *initial_bytes.as_ref().unwrap()
        );
        assert_eq!(
            std::fs::metadata(&cache).unwrap().modified().unwrap(),
            *initial_mtime.as_ref().unwrap()
        );
    }
    if matches!(case, LiveStartupCase::BypassMissing) {
        assert!(!cache.exists());
        assert!(!cache.parent().unwrap().exists());
        assert!(!cache.with_extension("rkyv.tmp").exists());
    }
    if matches!(case, LiveStartupCase::DisabledCurrent) {
        assert_eq!(
            std::fs::read(&cache).unwrap(),
            *initial_bytes.as_ref().unwrap()
        );
        assert_eq!(
            std::fs::metadata(&cache).unwrap().modified().unwrap(),
            *initial_mtime.as_ref().unwrap()
        );
        assert_live_cache_state(
            &cache,
            LiveStatePhase {
                version: 4720,
                reality: 0,
                voxel_x: 0,
                units: 7,
                nanites: 8,
                specials: 9,
                previous: None,
                selected: 7,
            },
        );
    }
    if matches!(case, LiveStartupCase::Cold) {
        assert_cold_live_cache(&cache);
    }
    if matches!(case, LiveStartupCase::WriteError) {
        assert!(!cache.exists());
        assert!(!cache.with_extension("rkyv.tmp").exists());
        assert!(
            process
                .stderr_text()
                .matches("could not write cache")
                .count()
                >= 1
        );
        assert!(
            process
                .stderr_text()
                .matches("could not update cache")
                .count()
                >= 1
        );
    }
    if matches!(case, LiveStartupCase::BypassMissing) {
        assert!(!cache.with_extension("rkyv.tmp").exists());
    }
    if matches!(
        case,
        LiveStartupCase::BypassExisting | LiveStartupCase::DisabledCurrent
    ) {
        assert!(!cache.with_extension("rkyv.tmp").exists());
    }
    assert!(!cache.with_extension("rkyv.tmp").exists());
    Ok(fixture_path)
}

#[cfg(unix)]
#[test]
fn test_headless_stdio_live_cache_control_matrix() {
    for case in [
        LiveStartupCase::Cold,
        LiveStartupCase::BypassExisting,
        LiveStartupCase::BypassMissing,
        LiveStartupCase::DisabledCurrent,
        LiveStartupCase::WriteError,
    ] {
        run_stdio_live_control(case);
    }
}

#[cfg(all(unix, feature = "http"))]
#[test]
fn test_headless_http_live_cache_control_matrix() {
    for case in [
        LiveStartupCase::Cold,
        LiveStartupCase::BypassExisting,
        LiveStartupCase::BypassMissing,
        LiveStartupCase::DisabledCurrent,
        LiveStartupCase::WriteError,
    ] {
        run_http_live_control(case);
    }
}

#[cfg(all(unix, feature = "http"))]
#[test]
fn test_headless_http_live_cold_case_retries_after_first_bind_collision() {
    let occupied = TcpListener::bind(("127.0.0.1", 0)).expect("occupy controlled live HTTP port");
    let addr = occupied
        .local_addr()
        .expect("read occupied live HTTP address");
    run_http_live_control_with_first_addr(LiveStartupCase::Cold, Some(addr));
    drop(occupied);
}
