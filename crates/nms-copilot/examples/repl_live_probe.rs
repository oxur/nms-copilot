//! Disposable fixture and observation utility for normal REPL live-cache checks.
//! This program never launches or controls the REPL; the existing native PTY
//! tools own that part of the acceptance matrix.

#[path = "../tests/support/live_mcp.rs"]
pub mod live_mcp;
#[path = "../tests/support/name_observations.rs"]
pub mod name_observations;

use std::fs::{self, File, FileTimes};
use std::io::{self, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, UNIX_EPOCH};

use live_mcp::{
    assert_location_pair, expected_cache, initialize_http, location_rpc_pair, subscribe_sse,
};
use name_observations::{ModelStage, SourcePhase};
use nms_cache::CacheData;
use nms_copilot::mcp;
use nms_core::address::GalacticAddress;

const MARKER: &str = ".slice19-owned-fixture";
const NAME_MARKER: &str = ".slice23-name-fixture";
type PreviousFields = (u8, i16, i8, i16, u16, u8);
type PhaseState = (u32, u8, u64, u64, u64, Option<PreviousFields>, u8);

fn main() {
    if let Err(error) = run() {
        eprintln!("repl_live_probe: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let command = args
        .next()
        .ok_or("expected prepare, inspect-cache, self-check, write, phase, malformed, expect-cache, monitor, or hold-port")?;
    match command.as_str() {
        "prepare-unknown" => {
            let dir = tempfile::Builder::new()
                .prefix("slice19-unknown-")
                .tempdir_in(std::env::temp_dir())?;
            let path = dir.keep();
            fs::write(path.join(MARKER), b"Project01 Arc06 Slice19 fixture\n")?;
            let home = path.join("home");
            let config_dir = home.join(".nms-copilot");
            fs::create_dir_all(&config_dir)?;
            let save = path.join("save.json");
            let cache = path.join("cache.rkyv");
            let source = live_mcp::unknown_lifecycle_source(
                4720,
                2,
                7,
                8,
                9,
                Some(GalacticAddress::new(2, 3, 4, 1, 1, 9)),
                false,
            );
            fs::write(&save, &source)?;
            let source_time = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
            File::options()
                .write(true)
                .open(&save)?
                .set_times(FileTimes::new().set_modified(source_time))?;
            let cache_seed = live_mcp::unknown_lifecycle_seed_cache();
            nms_cache::write_cache(&cache_seed, &cache)?;
            let cache_time = source_time + Duration::from_secs(100);
            File::options()
                .write(true)
                .open(&cache)?
                .set_times(FileTimes::new().set_modified(cache_time))?;
            let config = format!(
                "[display]\nshow_banner = false\nshow_system_banner = false\ncolor = false\n\n[watch]\nenabled = true\ndebounce_ms = 137\n\n[cache]\nenabled = true\npath = {:?}\n",
                cache.to_string_lossy()
            );
            fs::write(config_dir.join("config.toml"), config)?;
            fs::write(path.join("cache-path.txt"), b"cache.rkyv")?;
            fs::write(path.join("monitor.commands"), b"")?;
            fs::write(path.join("restart.commands"), b"")?;
            fs::write(path.join("mode.txt"), b"unknown\nenabled\n")?;
            fs::write(
                path.join("initial-state.json"),
                serde_json::to_vec(&serde_json::json!({
                    "source_hex": hex_bytes(&source),
                    "source_modified_nanos": source_time.duration_since(UNIX_EPOCH)?.as_nanos(),
                    "cache": {
                        "bytes_hex": hex_bytes(&fs::read(&cache)?),
                        "modified_nanos": cache_time.duration_since(UNIX_EPOCH)?.as_nanos(),
                    }
                }))?,
            )?;
            println!(
                "{}",
                serde_json::json!({
                    "fixture": path,
                    "marker": path.join(MARKER),
                    "home": home,
                    "save": save,
                    "cache": cache,
                    "mode": "unknown",
                    "watch": "enabled",
                })
            );
        }
        "prepare" => {
            let mode = args.next().unwrap_or_else(|| "warm".into());
            let watch = args.next().unwrap_or_else(|| "enabled".into());
            let dir = tempfile::Builder::new()
                .prefix("slice19-")
                .tempdir_in(std::env::temp_dir())?;
            let path = dir.keep();
            fs::write(path.join(MARKER), b"Project01 Arc06 Slice19 fixture\n")?;
            let home = path.join("home");
            let config_dir = home.join(".nms-copilot");
            fs::create_dir_all(&config_dir)?;
            let save = path.join("save.json");
            let cache = match mode.as_str() {
                "bypass-missing" => path.join("absent-parent/cache.rkyv"),
                "blocker" => path.join("cache-parent/cache.rkyv"),
                "cold" => path.join("cold-parent/cache.rkyv"),
                _ => path.join("cache.rkyv"),
            };
            fs::write(&save, save_bytes(4720, 0, 7, 8, 9, None))?;
            let fixed = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
            File::options()
                .write(true)
                .open(&save)?
                .set_times(FileTimes::new().set_modified(fixed))?;
            let config = format!(
                "[display]\nshow_banner = false\nshow_system_banner = false\ncolor = false\n\n[watch]\nenabled = {}\ndebounce_ms = 137\n\n[cache]\nenabled = {}\npath = {:?}\n",
                watch != "disabled",
                mode != "nocache",
                cache.to_string_lossy()
            );
            fs::write(config_dir.join("config.toml"), config)?;
            let relative_cache = cache.strip_prefix(&path)?;
            fs::write(
                path.join("cache-path.txt"),
                relative_cache.to_string_lossy().as_bytes(),
            )?;
            fs::write(path.join("monitor.commands"), b"")?;
            fs::write(path.join("mode.txt"), format!("{mode}\n{watch}\n"))?;
            if mode == "warm" {
                let golden = expected_cache(4720, 0, 7, 8, 9, None, 7);
                nms_cache::write_cache(&golden, &cache)?;
                let newer = fixed + Duration::from_secs(100);
                File::options()
                    .write(true)
                    .open(&cache)?
                    .set_times(FileTimes::new().set_modified(newer))?;
            } else if mode == "sentinel" {
                fs::write(&cache, b"slice19-no-cache-sentinel\n")?;
                File::options()
                    .write(true)
                    .open(&cache)?
                    .set_times(FileTimes::new().set_modified(fixed + Duration::from_secs(77)))?;
            } else if mode == "blocker" {
                let blocker = cache.parent().ok_or("blocker parent missing")?;
                fs::write(blocker, b"slice19-cache-write-blocker\n")?;
                File::options()
                    .write(true)
                    .open(blocker)?
                    .set_times(FileTimes::new().set_modified(fixed + Duration::from_secs(77)))?;
            } else if mode == "cold" {
                if cache.exists() || cache.parent().is_some_and(Path::exists) {
                    return Err("cold fixture must begin without cache parent or file".into());
                }
            } else if mode == "predecessor" {
                let bytes = include_bytes!(
                    "../../nms-cache/tests/fixtures/predecessor-schema3-semantic3.rkyv"
                );
                if bytes.len() != 752 || &bytes[..16] != b"NMSCACHE\x03\0\0\0\x03\0\0\0" {
                    return Err("canonical predecessor fixture bytes or header differ".into());
                }
                fs::write(&cache, bytes)?;
                File::options()
                    .write(true)
                    .open(&cache)?
                    .set_times(FileTimes::new().set_modified(fixed + Duration::from_secs(100)))?;
            } else if mode != "nocache" && mode != "bypass-missing" {
                return Err(format!("unknown fixture mode {mode}").into());
            }
            let source_meta = fs::metadata(&save)?;
            let source_modified = source_meta
                .modified()?
                .duration_since(UNIX_EPOCH)?
                .as_nanos();
            let cache_state = if cache.exists() {
                Some(serde_json::json!({
                    "bytes_hex": hex_bytes(&fs::read(&cache)?),
                    "modified_nanos": fs::metadata(&cache)?.modified()?.duration_since(UNIX_EPOCH)?.as_nanos(),
                }))
            } else {
                None
            };
            fs::write(
                path.join("initial-state.json"),
                serde_json::to_vec(&serde_json::json!({
                    "source_hex": hex_bytes(&fs::read(&save)?),
                    "source_modified_nanos": source_modified,
                    "cache": cache_state,
                }))?,
            )?;
            println!(
                "{}",
                serde_json::json!({
                    "fixture": path,
                    "marker": path.join(MARKER),
                    "home": home,
                    "save": save,
                    "cache": cache,
                    "mode": mode,
                    "watch": watch,
                })
            );
        }
        "inspect-cache" => {
            let fixture = owned_path(args.next().ok_or("fixture path required")?)?;
            let cache = fixture_cache_path(&fixture)?;
            let actual = nms_cache::read_cache(&cache)?;
            println!(
                "systems={} planets={} bases={} version={} selected={} player={:?}",
                actual.systems.len(),
                actual
                    .systems
                    .iter()
                    .map(|s| s.planets.len())
                    .sum::<usize>(),
                actual.bases.len(),
                actual.save_version,
                actual.active_galaxy,
                actual.player_state
            );
        }
        "verify-unknown" => {
            let fixture = owned_path(args.next().ok_or("fixture path required")?)?;
            verify_unknown_fixture(&fixture)?;
            println!("unknown-fixture=pass source+fresh-compatible-seed+config");
        }
        "prepare-names" => {
            let profile = args.next().unwrap_or_else(|| "warm".into());
            prepare_names(&profile)?;
        }
        "verify-names" => {
            let fixture = owned_names_path(args.next().ok_or("fixture path required")?)?;
            verify_names_fixture(&fixture)?;
            println!("name-fixture=pass exact-source-cache-config");
        }
        "prepare-names-restart" => {
            let fixture = owned_names_path(args.next().ok_or("fixture path required")?)?;
            prepare_names_restart(&fixture)?;
            println!("name-restart=pass exact-cache-source-mode");
        }
        "verify-names-restart" => {
            let fixture = owned_names_path(args.next().ok_or("fixture path required")?)?;
            verify_names_restart(&fixture)?;
            println!("name-restart-checkpoint=pass");
        }
        "monitor-names" => {
            let fixture = owned_names_path(args.next().ok_or("fixture path required")?)?;
            let address: SocketAddr = args.next().ok_or("MCP socket address required")?.parse()?;
            run_names_monitor(&fixture, address)?;
        }
        "monitor-names-restart" => {
            let fixture = owned_names_path(args.next().ok_or("fixture path required")?)?;
            let address: SocketAddr = args.next().ok_or("MCP socket address required")?.parse()?;
            run_names_restart_monitor(&fixture, address)?;
        }
        "name-phase" => {
            let fixture = owned_names_path(args.next().ok_or("fixture path required")?)?;
            let phase = args.next().ok_or("name phase required")?;
            write_name_phase(&fixture, &phase)?;
            println!("name-phase={phase} written");
        }
        "prepare-restart" => {
            let fixture = owned_path(args.next().ok_or("fixture path required")?)?;
            prepare_unknown_restart(&fixture)?;
            println!("restart-source=pass malformed+older-than-cache");
        }
        "self-check" => self_check()?,
        "verify-bind-failure" => {
            let fixture = owned_path(args.next().ok_or("fixture path required")?)?;
            let initial = load_initial_fixture(&fixture)?;
            check_fixture_files_before_startup(
                &fixture,
                &initial.source,
                initial.source_modified,
                &initial.cache,
                initial.semantic,
            )?;
            println!(
                "initial-bind-failure=pass source+cache bytes/mtime, parent and temp state retained"
            );
        }
        "write" => {
            let fixture = owned_path(args.next().ok_or("fixture path required")?)?;
            let version: u32 = args.next().ok_or("version required")?.parse()?;
            let reality: u8 = args.next().ok_or("reality required")?.parse()?;
            let units: u64 = args.next().ok_or("units required")?.parse()?;
            let nanites: u64 = args.next().ok_or("nanites required")?.parse()?;
            let specials: u64 = args.next().ok_or("specials required")?.parse()?;
            let bytes = save_bytes(version, reality, units, nanites, specials, None);
            atomic_replace(&fixture.join("save.json"), &bytes)?;
            println!("source-bytes-length={}", bytes.len());
        }
        "phase" => {
            let fixture = owned_path(args.next().ok_or("fixture path required")?)?;
            let phase: u8 = args.next().ok_or("phase number required")?.parse()?;
            let (version, reality, units, nanites, specials, previous) = match phase {
                1 => (4720, 0, 70, 80, 90, None),
                2 => (4720, 0, 70, 80, 90, Some((9, 3, 3, 4, 1, 1))),
                3 => (4721, 0, 70, 80, 90, Some((9, 3, 3, 4, 1, 1))),
                4 => (4722, 9, 700, 800, 900, Some((0, 2, 3, 4, 1, 1))),
                5 => (4723, 9, 701, 801, 901, Some((0, 2, 3, 4, 1, 1))),
                6 => (4724, 0, 900, 901, 902, Some((9, 0, 3, 4, 1, 1))),
                _ => return Err(format!("unknown phase {phase}").into()),
            };
            let bytes = save_bytes(version, reality, units, nanites, specials, previous);
            atomic_replace(&fixture.join("save.json"), &bytes)?;
            println!(
                "phase={phase} version={version} physical={reality} source-bytes-length={}",
                bytes.len()
            );
        }
        "malformed" => {
            let fixture = owned_path(args.next().ok_or("fixture path required")?)?;
            let bytes = br#"{"Version":broken"#;
            atomic_replace(&fixture.join("save.json"), bytes)?;
            println!("malformed-source-bytes={}", bytes.len());
        }
        "expect-cache" => {
            let fixture = owned_path(args.next().ok_or("fixture path required")?)?;
            let version: u32 = args.next().ok_or("version required")?.parse()?;
            let reality: u8 = args.next().ok_or("physical reality required")?.parse()?;
            let units: u64 = args.next().ok_or("units required")?.parse()?;
            let nanites: u64 = args.next().ok_or("nanites required")?.parse()?;
            let specials: u64 = args.next().ok_or("specials required")?.parse()?;
            let selected: u8 = args.next().ok_or("selected galaxy required")?.parse()?;
            let previous = if let Some(value) = args.next() {
                let values: Vec<u8> = value.split(',').map(str::parse).collect::<Result<_, _>>()?;
                if values.len() != 6 {
                    return Err("previous address needs reality,x,y,z,system,planet".into());
                }
                Some(nms_core::address::GalacticAddress::new(
                    values[1] as i8 as i16,
                    values[2] as i8,
                    values[3] as i8 as i16,
                    u16::from(values[4]),
                    values[5],
                    values[0],
                ))
            } else {
                None
            };
            let expected = expected_cache(
                version, reality, units, nanites, specials, previous, selected,
            );
            let expected_model =
                live_mcp::expected_model(reality, units, nanites, specials, previous, selected);
            live_mcp::assert_cache_oracle(
                &fixture_cache_path(&fixture)?,
                &expected,
                &expected_model,
            )
            .map_err(|error| format!("whole raw+rebuilt cache mismatch: {error}"))?;
            println!("whole-cache=pass version={version} selected={selected}");
        }
        "monitor" => {
            let fixture = owned_path(args.next().ok_or("fixture path required")?)?;
            let address: SocketAddr = args.next().ok_or("MCP socket address required")?.parse()?;
            run_monitor(&fixture, address)?;
        }
        "monitor-unknown" => {
            let fixture = owned_path(args.next().ok_or("fixture path required")?)?;
            let address: SocketAddr = args.next().ok_or("MCP socket address required")?.parse()?;
            run_unknown_monitor(&fixture, address)?;
        }
        "verify-restart" => {
            let fixture = owned_path(args.next().ok_or("fixture path required")?)?;
            let address: SocketAddr = args.next().ok_or("MCP socket address required")?.parse()?;
            run_unknown_restart_monitor(&fixture, address)?;
        }
        "hold-port" => {
            let address: SocketAddr = args.next().ok_or("socket address required")?.parse()?;
            let fixture = owned_path(args.next().ok_or("fixture path required")?)?;
            let listener = TcpListener::bind(address)?;
            let occupied = listener.local_addr()?;
            let ready = fixture.join("hold-port.ready");
            fs::write(&ready, format!("occupied={occupied}\n"))?;
            File::open(&ready)?.sync_all()?;
            println!("occupied={occupied}");
            io::stdout().flush()?;
            let control = fixture.join("monitor.commands");
            if wait_control_line(&control, 0, Duration::from_secs(60))?.0 != "release" {
                return Err("hold-port requires explicit release command".into());
            }
        }
        _ => return Err(format!("unknown command {command}").into()),
    }
    Ok(())
}

fn owned_path(path: String) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = PathBuf::from(path);
    let marker = path.join(MARKER);
    if !marker.is_file() || fs::read(marker)? != b"Project01 Arc06 Slice19 fixture\n" {
        return Err(format!("refusing unowned fixture path {}", path.display()).into());
    }
    Ok(path)
}

fn owned_names_path(path: String) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = PathBuf::from(path);
    let marker = path.join(NAME_MARKER);
    if !marker.is_file() || fs::read(marker)? != b"Project01 Arc06 Slice23 name fixture\n" {
        return Err(format!("refusing unowned name fixture path {}", path.display()).into());
    }
    Ok(path)
}

fn names_config(cache: &Path, watch_enabled: bool) -> String {
    format!(
        "[display]\nshow_banner = false\nshow_system_banner = false\ncolor = false\n\n[watch]\nenabled = {watch_enabled}\ndebounce_ms = 137\n\n[cache]\nenabled = true\npath = {:?}\n",
        cache.to_string_lossy()
    )
}

fn name_cache_path(fixture: &Path) -> PathBuf {
    fixture.join("cache.rkyv")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NamesProfile {
    Warm,
    Predecessor,
    PredecessorSource,
}

impl NamesProfile {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "warm" => Ok(Self::Warm),
            "predecessor" => Ok(Self::Predecessor),
            "predecessor-source" => Ok(Self::PredecessorSource),
            _ => Err(format!("unknown name profile {value:?}")),
        }
    }

    fn is_predecessor(self) -> bool {
        matches!(self, Self::Predecessor | Self::PredecessorSource)
    }

    fn predecessor_cache_bytes(self) -> Option<&'static [u8]> {
        match self {
            Self::Warm => None,
            Self::Predecessor => Some(include_bytes!(
                "../../nms-cache/tests/fixtures/predecessor-schema3-semantic4.rkyv"
            )),
            Self::PredecessorSource => Some(include_bytes!(
                "../../nms-cache/tests/fixtures/predecessor-schema3-semantic5.rkyv"
            )),
        }
    }

    fn watch_enabled(self) -> bool {
        self == Self::Warm
    }

    fn source_phase(self) -> SourcePhase {
        if self.is_predecessor() {
            SourcePhase::Recover
        } else {
            SourcePhase::Initial
        }
    }
}

fn prepare_names(profile: &str) -> Result<(), Box<dyn std::error::Error>> {
    let profile_kind = NamesProfile::parse(profile)?;
    let dir = tempfile::Builder::new()
        .prefix("slice23-names-")
        .tempdir_in(std::env::temp_dir())?;
    let fixture = dir.keep();
    fs::write(
        fixture.join(NAME_MARKER),
        b"Project01 Arc06 Slice23 name fixture\n",
    )?;
    let home = fixture.join("home");
    let config_dir = home.join(".nms-copilot");
    fs::create_dir_all(&config_dir)?;
    let source_path = fixture.join("save.json");
    let cache_path = name_cache_path(&fixture);
    let source_phase = profile_kind.source_phase();
    let source = name_observations::source_bytes(source_phase);
    fs::write(&source_path, &source)?;
    let source_time = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    File::options()
        .write(true)
        .open(&source_path)?
        .set_times(FileTimes::new().set_modified(source_time))?;
    let cache_bytes = if profile_kind == NamesProfile::Warm {
        nms_cache::write_cache(&name_observations::warm_seed_cache(), &cache_path)?;
        let modified = source_time + Duration::from_secs(100);
        File::options()
            .write(true)
            .open(&cache_path)?
            .set_times(FileTimes::new().set_modified(modified))?;
        fs::read(&cache_path)?
    } else {
        let predecessor = profile_kind
            .predecessor_cache_bytes()
            .ok_or("predecessor profile has no predecessor cache")?;
        let expected_header = if profile_kind == NamesProfile::PredecessorSource {
            b"NMSCACHE\x03\0\0\0\x05\0\0\0".as_slice()
        } else {
            b"NMSCACHE\x03\0\0\0\x04\0\0\0".as_slice()
        };
        let expected_len = if profile_kind == NamesProfile::PredecessorSource {
            632
        } else {
            224
        };
        if predecessor.len() != expected_len || &predecessor[..16] != expected_header {
            return Err(format!("canonical cache predecessor for {profile:?} differs").into());
        }
        fs::write(&cache_path, predecessor)?;
        let modified = source_time + Duration::from_secs(100);
        File::options()
            .write(true)
            .open(&cache_path)?
            .set_times(FileTimes::new().set_modified(modified))?;
        predecessor.to_vec()
    };
    let watch_enabled = profile_kind.watch_enabled();
    fs::write(
        config_dir.join("config.toml"),
        names_config(&cache_path, watch_enabled),
    )?;
    fs::write(fixture.join("cache-path.txt"), b"cache.rkyv\n")?;
    fs::write(
        fixture.join("mode.txt"),
        format!(
            "names-{profile}\n{}\n",
            if watch_enabled { "enabled" } else { "disabled" }
        ),
    )?;
    fs::write(fixture.join("monitor.names.commands"), b"")?;
    fs::write(fixture.join("restart.names.commands"), b"")?;
    let cache_modified = fs::metadata(&cache_path)?.modified()?;
    fs::write(
        fixture.join("names-initial-state.json"),
        serde_json::to_vec(&serde_json::json!({
            "profile": profile,
            "source_hex": hex_bytes(&source),
            "source_modified_nanos": source_time.duration_since(UNIX_EPOCH)?.as_nanos(),
            "cache_hex": hex_bytes(&cache_bytes),
            "cache_modified_nanos": cache_modified.duration_since(UNIX_EPOCH)?.as_nanos(),
        }))?,
    )?;
    println!(
        "{}",
        serde_json::json!({
            "fixture": fixture,
            "marker": fixture.join(NAME_MARKER),
            "home": home,
            "save": source_path,
            "cache": cache_path,
            "mode": profile,
            "watch": watch_enabled,
        })
    );
    Ok(())
}

fn verify_names_fixture(fixture: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let checkpoint: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.join("names-initial-state.json"))?)?;
    let profile = checkpoint["profile"]
        .as_str()
        .ok_or("name profile missing")?;
    let profile_kind = NamesProfile::parse(profile)?;
    let watch_enabled = profile_kind.watch_enabled();
    let source_path = fixture.join("save.json");
    let cache_path = name_cache_path(fixture);
    let expected_source = decode_hex(
        checkpoint["source_hex"]
            .as_str()
            .ok_or("initial source missing")?,
    )?;
    let expected_cache = decode_hex(
        checkpoint["cache_hex"]
            .as_str()
            .ok_or("initial cache missing")?,
    )?;
    let source_time = UNIX_EPOCH
        + Duration::from_nanos(
            checkpoint["source_modified_nanos"]
                .as_u64()
                .ok_or("initial source time missing")?,
        );
    let cache_time = UNIX_EPOCH
        + Duration::from_nanos(
            checkpoint["cache_modified_nanos"]
                .as_u64()
                .ok_or("initial cache time missing")?,
        );
    let expected_mode = format!(
        "names-{profile}\n{}\n",
        if watch_enabled { "enabled" } else { "disabled" }
    );
    if !fixture.join(NAME_MARKER).is_file()
        || fs::read(fixture.join(NAME_MARKER))? != b"Project01 Arc06 Slice23 name fixture\n"
        || fs::read_to_string(fixture.join("mode.txt"))? != expected_mode
        || fs::read_to_string(fixture.join("cache-path.txt"))? != "cache.rkyv\n"
        || fs::read_to_string(fixture.join("home/.nms-copilot/config.toml"))?
            != names_config(&cache_path, watch_enabled)
        || fs::read(&source_path)? != expected_source
        || fs::metadata(&source_path)?.modified()? != source_time
        || fs::read(&cache_path)? != expected_cache
        || fs::metadata(&cache_path)?.modified()? != cache_time
    {
        return Err("name fixture prelaunch source/cache/config checkpoint differs".into());
    }
    let profile_kind = NamesProfile::parse(profile)?;
    if profile_kind == NamesProfile::Warm {
        live_mcp::compare_cache(
            &nms_cache::read_cache(&cache_path)?,
            &name_observations::warm_seed_cache(),
        )?;
    } else if profile_kind == NamesProfile::PredecessorSource {
        let predecessor = profile_kind
            .predecessor_cache_bytes()
            .ok_or("schema5 predecessor bytes missing")?;
        if expected_cache.as_slice() != predecessor
            || !matches!(
                nms_cache::read_cache(&cache_path),
                Err(nms_cache::CacheError::Incompatible(
                    nms_cache::CacheCompatibilityError::SchemaMismatch {
                        found: 3,
                        expected: 4
                    }
                ))
            )
        {
            return Err(
                "authentic schema3/semantic5 cache must fail schema compatibility before startup"
                    .into(),
            );
        }
    } else {
        let predecessor = profile_kind
            .predecessor_cache_bytes()
            .ok_or("schema4 predecessor bytes missing")?;
        if expected_cache.as_slice() != predecessor
            || !matches!(
                nms_cache::read_cache(&cache_path),
                Err(nms_cache::CacheError::Incompatible(
                    nms_cache::CacheCompatibilityError::SchemaMismatch {
                        found: 3,
                        expected: 4
                    }
                ))
            )
        {
            return Err("authentic schema3/semantic4 prelaunch cache check differs".into());
        }
    }
    Ok(())
}

fn prepare_names_restart(fixture: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let checkpoint: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.join("names-initial-state.json"))?)?;
    let profile = checkpoint["profile"]
        .as_str()
        .ok_or("name profile missing")?;
    let profile_kind = NamesProfile::parse(profile)?;
    let source_path = fixture.join("save.json");
    let cache_path = name_cache_path(fixture);
    let current_cache = cache_stamp(&cache_path)?.ok_or("name cache missing before restart")?;
    if fs::read(&source_path)? != name_observations::source_bytes(SourcePhase::Recover) {
        return Err("name restart requires the completed recovery source".into());
    }
    let actual = nms_cache::read_cache(&cache_path)?;
    let rebuilt = nms_cache::rebuild_model(&actual);
    if !profile_kind.is_predecessor() {
        live_mcp::compare_cache_model(
            &actual,
            &rebuilt,
            &name_observations::expected_cache(ModelStage::Recover, 7),
            &name_observations::expected_model(ModelStage::Recover),
        )?;
    } else {
        live_mcp::compare_cache_model(
            &actual,
            &rebuilt,
            &name_observations::expected_predecessor_cache(),
            &name_observations::expected_predecessor_model(),
        )?;
    }
    let malformed = br#"{"Version":slice23-restart-opposed"#;
    atomic_replace(&source_path, malformed)?;
    File::options().write(true).open(&source_path)?.set_times(
        FileTimes::new().set_modified(current_cache.modified - Duration::from_secs(1)),
    )?;
    let config = names_config(&cache_path, false);
    fs::write(
        fixture.join("mode.txt"),
        format!("names-{profile}\ndisabled\n"),
    )?;
    fs::write(fixture.join("home/.nms-copilot/config.toml"), &config)?;
    let source_time = fs::metadata(&source_path)?.modified()?;
    fs::write(
        fixture.join("names-restart-checkpoint.json"),
        serde_json::to_vec(&serde_json::json!({
            "profile": profile,
            "cache_hex": hex_bytes(&fs::read(&cache_path)?),
            "cache_modified_nanos": fs::metadata(&cache_path)?.modified()?.duration_since(UNIX_EPOCH)?.as_nanos(),
            "source_hex": hex_bytes(&fs::read(&source_path)?),
            "source_modified_nanos": source_time.duration_since(UNIX_EPOCH)?.as_nanos(),
            "config": config,
            "mode": fs::read_to_string(fixture.join("mode.txt"))?,
        }))?,
    )?;
    File::open(fixture.join("names-restart-checkpoint.json"))?.sync_all()?;
    Ok(())
}

fn verify_names_restart(fixture: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let checkpoint: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.join("names-restart-checkpoint.json"))?)?;
    let profile = checkpoint["profile"]
        .as_str()
        .ok_or("restart profile missing")?;
    let profile_kind = NamesProfile::parse(profile)?;
    let source_path = fixture.join("save.json");
    let cache_path = name_cache_path(fixture);
    let expected_source = decode_hex(
        checkpoint["source_hex"]
            .as_str()
            .ok_or("restart source missing")?,
    )?;
    let expected_cache = decode_hex(
        checkpoint["cache_hex"]
            .as_str()
            .ok_or("restart cache missing")?,
    )?;
    let source_time = UNIX_EPOCH
        + Duration::from_nanos(
            checkpoint["source_modified_nanos"]
                .as_u64()
                .ok_or("restart source time missing")?,
        );
    let cache_time = UNIX_EPOCH
        + Duration::from_nanos(
            checkpoint["cache_modified_nanos"]
                .as_u64()
                .ok_or("restart cache time missing")?,
        );
    let malformed = br#"{"Version":slice23-restart-opposed"#;
    if expected_source.as_slice() != malformed
        || cache_time <= source_time
        || !nms_cache::is_cache_fresh(&cache_path, &source_path)
    {
        return Err("restart requires the literal older malformed source and fresh cache".into());
    }
    if fs::read(&source_path)? != expected_source
        || fs::metadata(&source_path)?.modified()? != source_time
        || fs::read(&cache_path)? != expected_cache
        || fs::metadata(&cache_path)?.modified()? != cache_time
        || fs::read_to_string(fixture.join("mode.txt"))?
            != "names-".to_owned() + profile + "\ndisabled\n"
        || fs::read_to_string(fixture.join("home/.nms-copilot/config.toml"))?
            != checkpoint["config"]
                .as_str()
                .ok_or("restart config missing")?
    {
        return Err("name restart exact source/cache/config checkpoint differs".into());
    }
    let actual = nms_cache::read_cache(&cache_path)?;
    let rebuilt = nms_cache::rebuild_model(&actual);
    if !profile_kind.is_predecessor() {
        live_mcp::compare_cache_model(
            &actual,
            &rebuilt,
            &name_observations::expected_cache(ModelStage::Recover, 7),
            &name_observations::expected_model(ModelStage::Recover),
        )?;
    } else {
        live_mcp::compare_cache_model(
            &actual,
            &rebuilt,
            &name_observations::expected_predecessor_cache(),
            &name_observations::expected_predecessor_model(),
        )?;
    }
    Ok(())
}

fn write_name_phase(fixture: &Path, phase: &str) -> Result<(), Box<dyn std::error::Error>> {
    let source = match phase {
        "overwrite" => name_observations::source_bytes(SourcePhase::Overwrite),
        "promote" => name_observations::source_bytes(SourcePhase::Promote),
        "rename" => name_observations::source_bytes(SourcePhase::Rename),
        "clear" => name_observations::source_bytes(SourcePhase::Clear),
        "omit" => name_observations::source_bytes(SourcePhase::Omit),
        "null" => name_observations::source_bytes(SourcePhase::Null),
        "nonstring" => name_observations::source_bytes(SourcePhase::NonString),
        "nonobject" => name_observations::source_bytes(SourcePhase::NonObject),
        "nonobject-dm" => name_observations::source_bytes(SourcePhase::NonObjectDm),
        "recover" => name_observations::source_bytes(SourcePhase::Recover),
        _ if phase.starts_with("invalid") => {
            let attempt = phase
                .strip_prefix("invalid")
                .ok_or("invalid phase suffix missing")?
                .parse::<u8>()?;
            if !(1..=5).contains(&attempt) {
                return Err(format!("invalid attempt is outside 1..=5: {attempt}").into());
            }
            name_observations::invalid_address_attempt(attempt)
        }
        _ => return Err(format!("unknown name phase {phase:?}").into()),
    };
    atomic_replace(&fixture.join("save.json"), &source)?;
    Ok(())
}

fn check_names_rpc(
    fixture: &Path,
    address: SocketAddr,
    session: &str,
    stage: ModelStage,
    predecessor: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tool, location) = location_rpc_pair(address, session)?;
    let expected_location = name_observations::expected_location(stage, predecessor);
    if tool != location || tool != expected_location {
        return Err(format!(
            "full where_am_i tool/resource mismatch; expected {expected_location}, tool {tool}, resource {location}"
        )
        .into());
    }
    let stats = live_mcp::resource_json(address, session, "nms://galaxy/stats")?;
    let expected_stats = name_observations::normalize_stats_biomes(
        name_observations::expected_stats(stage, predecessor),
    );
    let actual_stats = name_observations::normalize_stats_biomes(stats);
    if actual_stats != expected_stats {
        return Err(format!(
            "complete galaxy stats mismatch; expected {expected_stats}, observed {actual_stats}"
        )
        .into());
    }
    if !fixture.join("monitor.names.commands").is_file() {
        return Err("name monitor command file disappeared".into());
    }
    Ok(())
}

struct NamesBoundary<'a> {
    source: &'a [u8],
    source_time: std::time::SystemTime,
    cache_stamp: &'a CacheStamp,
    stage: ModelStage,
    profile: &'a str,
    watch_enabled: bool,
}

fn check_names_boundary(
    fixture: &Path,
    address: SocketAddr,
    session: &str,
    boundary: NamesBoundary<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    check_names_file_boundary(fixture, &boundary)?;
    let profile = NamesProfile::parse(boundary.profile)?;
    check_names_rpc(
        fixture,
        address,
        session,
        boundary.stage,
        profile.is_predecessor(),
    )?;
    Ok(())
}

fn check_names_file_boundary(
    fixture: &Path,
    boundary: &NamesBoundary<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = fixture.join("save.json");
    let cache = name_cache_path(fixture);
    let profile = NamesProfile::parse(boundary.profile)?;
    let expected_cache = if profile.is_predecessor() {
        name_observations::expected_predecessor_cache()
    } else {
        name_observations::expected_cache(boundary.stage, 7)
    };
    let expected_model = if profile.is_predecessor() {
        name_observations::expected_predecessor_model()
    } else {
        name_observations::expected_model(boundary.stage)
    };
    let raw = nms_cache::read_cache(&cache)?;
    let rebuilt = nms_cache::rebuild_model(&raw);
    live_mcp::compare_cache_model(&raw, &rebuilt, &expected_cache, &expected_model)
        .map_err(|error| format!("complete raw+rebuilt discovery-name oracle: {error}"))?;
    let actual_stamp = cache_stamp(&cache)?.ok_or("name cache disappeared")?;
    if fs::read(&source)? != boundary.source
        || fs::metadata(&source)?.modified()? != boundary.source_time
        || actual_stamp.bytes != boundary.cache_stamp.bytes
        || actual_stamp.modified != boundary.cache_stamp.modified
        || fs::read_to_string(fixture.join("home/.nms-copilot/config.toml"))?
            != names_config(&cache, boundary.watch_enabled)
        || fs::read_to_string(fixture.join("mode.txt"))?
            != format!(
                "names-{}\n{}\n",
                boundary.profile,
                if boundary.watch_enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            )
        || fs::read_to_string(fixture.join("cache-path.txt"))? != "cache.rkyv\n"
        || cache.with_extension("rkyv.tmp").exists()
        || source.with_extension("replacement").exists()
    {
        return Err("name lifecycle source/cache/config/mtime/temp boundary differs".into());
    }
    Ok(())
}

fn wait_names_positive(
    sse: &mut live_mcp::SseStream,
    cursor: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let location_end = sse.wait_for_resource_updated_since(
        cursor,
        "nms://player/location",
        Duration::from_secs(8),
    )?;
    let stats_end = sse.wait_for_resource_updated_since(
        location_end,
        "nms://galaxy/stats",
        Duration::from_secs(8),
    )?;
    sse.wait_quiet_uris_since(
        stats_end,
        &["nms://player/location", "nms://galaxy/stats"],
        Duration::from_millis(650),
    )?;
    Ok(())
}

fn wait_names_location_only(
    sse: &mut live_mcp::SseStream,
    cursor: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let location_end = sse.wait_for_resource_updated_since(
        cursor,
        "nms://player/location",
        Duration::from_secs(8),
    )?;
    sse.wait_quiet_uris_since(
        location_end,
        &["nms://player/location", "nms://galaxy/stats"],
        Duration::from_millis(650),
    )?;
    Ok(())
}

fn run_names_monitor(
    fixture: &Path,
    address: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let checkpoint: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.join("names-initial-state.json"))?)?;
    let profile = checkpoint["profile"]
        .as_str()
        .ok_or("name profile missing")?;
    let profile_kind = NamesProfile::parse(profile)?;
    let predecessor = profile_kind.is_predecessor();
    let mut expected_source = name_observations::source_bytes(if predecessor {
        SourcePhase::Recover
    } else {
        SourcePhase::Initial
    });
    let mut expected_source_time = UNIX_EPOCH
        + Duration::from_nanos(
            checkpoint["source_modified_nanos"]
                .as_u64()
                .ok_or("initial source mtime missing")?,
        );
    let mut expected_cache_stamp =
        cache_stamp(&name_cache_path(fixture))?.ok_or("name cache missing at monitor start")?;
    let (session, _) = initialize_http(address)?;
    let mut sse = live_mcp::subscribe_sse_uris(
        address,
        &session,
        &["nms://player/location", "nms://galaxy/stats"],
    )?;
    let mut stage = if predecessor {
        ModelStage::Recover
    } else {
        ModelStage::Initial
    };
    check_names_boundary(
        fixture,
        address,
        &session,
        NamesBoundary {
            source: &expected_source,
            source_time: expected_source_time,
            cache_stamp: &expected_cache_stamp,
            stage,
            profile,
            watch_enabled: profile_kind.watch_enabled(),
        },
    )?;
    if predecessor {
        let cursor = sse.cursor();
        sse.wait_quiet_uris_since(
            cursor,
            &["nms://player/location", "nms://galaxy/stats"],
            Duration::from_millis(650),
        )?;
    }
    fs::write(
        fixture.join("monitor.names.ready"),
        format!("address={address} session={session}\n"),
    )?;
    File::open(fixture.join("monitor.names.ready"))?.sync_all()?;
    let control = fixture.join("monitor.names.commands");
    let warm_commands = [
        "overwrite",
        "promote",
        "rename",
        "clear",
        "omit",
        "null",
        "nonstring",
        "nonobject",
        "nonobject-dm",
        "invalid1",
        "invalid2",
        "invalid3",
        "invalid4",
        "invalid5",
        "recover",
        "identical",
        "pre-exit",
        "finish",
    ];
    let predecessor_commands = ["pre-exit", "finish"];
    let commands: &[&str] = if predecessor {
        &predecessor_commands
    } else {
        &warm_commands
    };
    let mut offset = 0;
    for (index, required) in commands.iter().copied().enumerate() {
        let (line, next_offset) = wait_control_line(&control, offset, Duration::from_secs(60))?;
        offset = next_offset;
        if line != required {
            return Err(format!(
                "name lifecycle command {index} requires {required:?}, received {line:?}"
            )
            .into());
        }
        if required == "finish" {
            check_names_file_boundary(
                fixture,
                &NamesBoundary {
                    source: &expected_source,
                    source_time: expected_source_time,
                    cache_stamp: &expected_cache_stamp,
                    stage,
                    profile,
                    watch_enabled: profile_kind.watch_enabled(),
                },
            )?;
            sse.wait_closed_without_resource_updates(
                Duration::from_secs(8),
                &["nms://player/location", "nms://galaxy/stats"],
            )?;
            sse.close_and_join()?;
            check_names_file_boundary(
                fixture,
                &NamesBoundary {
                    source: &expected_source,
                    source_time: expected_source_time,
                    cache_stamp: &expected_cache_stamp,
                    stage,
                    profile,
                    watch_enabled: profile_kind.watch_enabled(),
                },
            )?;
            println!("name-lifecycle=pass profile={profile} both-resource-eof+pure-file-boundary");
            return Ok(());
        }
        check_names_boundary(
            fixture,
            address,
            &session,
            NamesBoundary {
                source: &expected_source,
                source_time: expected_source_time,
                cache_stamp: &expected_cache_stamp,
                stage,
                profile,
                watch_enabled: profile_kind.watch_enabled(),
            },
        )?;
        if required == "pre-exit" {
            println!("name-pre-exit=pass");
            io::stdout().flush()?;
            continue;
        }
        if required == "identical" {
            let cursor = sse.cursor();
            expected_source_time = atomic_replace(&fixture.join("save.json"), &expected_source)?;
            sse.wait_quiet_uris_since(
                cursor,
                &["nms://player/location", "nms://galaxy/stats"],
                Duration::from_millis(650),
            )?;
            check_names_boundary(
                fixture,
                address,
                &session,
                NamesBoundary {
                    source: &expected_source,
                    source_time: expected_source_time,
                    cache_stamp: &expected_cache_stamp,
                    stage,
                    profile,
                    watch_enabled: profile_kind.watch_enabled(),
                },
            )?;
            println!("name-identical=pass both-resource-quiet+whole-cache");
            io::stdout().flush()?;
            continue;
        }
        let cursor = sse.cursor();
        write_name_phase(fixture, required)?;
        expected_source = fs::read(fixture.join("save.json"))?;
        expected_source_time = fs::metadata(fixture.join("save.json"))?.modified()?;
        let (next_stage, notifications) = match required {
            "overwrite" => (ModelStage::Overwrite, Some(true)),
            "promote" => (ModelStage::Promote, Some(true)),
            "rename" => (ModelStage::Rename, Some(true)),
            "clear" => (ModelStage::Clear, Some(true)),
            "omit" => (ModelStage::Ignore, Some(false)),
            "recover" => (ModelStage::Recover, Some(true)),
            "invalid1" | "invalid2" | "invalid3" | "invalid4" | "invalid5" => {
                (ModelStage::Ignore, None)
            }
            "null" | "nonstring" | "nonobject" | "nonobject-dm" => (ModelStage::Ignore, None),
            _ => return Err(format!("unexpected name phase {required:?}").into()),
        };
        match notifications {
            Some(true) => wait_names_positive(&mut sse, cursor)?,
            Some(false) => wait_names_location_only(&mut sse, cursor)?,
            None => {
                sse.wait_quiet_uris_since(
                    cursor,
                    &["nms://player/location", "nms://galaxy/stats"],
                    Duration::from_millis(650),
                )?;
            }
        }
        stage = next_stage;
        if notifications.is_some() {
            expected_cache_stamp = cache_stamp(&name_cache_path(fixture))?
                .ok_or("name cache disappeared after an accepted update")?;
        }
        check_names_boundary(
            fixture,
            address,
            &session,
            NamesBoundary {
                source: &expected_source,
                source_time: expected_source_time,
                cache_stamp: &expected_cache_stamp,
                stage,
                profile,
                watch_enabled: profile_kind.watch_enabled(),
            },
        )?;
        println!("name-phase-{required}=pass");
        io::stdout().flush()?;
    }
    Err("name lifecycle controller ended without finish".into())
}

fn run_names_restart_monitor(
    fixture: &Path,
    address: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    verify_names_restart(fixture)?;
    let checkpoint: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.join("names-restart-checkpoint.json"))?)?;
    let profile = checkpoint["profile"]
        .as_str()
        .ok_or("restart profile missing")?;
    let profile_kind = NamesProfile::parse(profile)?;
    let expected_source = decode_hex(
        checkpoint["source_hex"]
            .as_str()
            .ok_or("restart source bytes missing")?,
    )?;
    let expected_source_time = UNIX_EPOCH
        + Duration::from_nanos(
            checkpoint["source_modified_nanos"]
                .as_u64()
                .ok_or("restart source mtime missing")?,
        );
    let expected_cache_stamp = CacheStamp {
        bytes: decode_hex(
            checkpoint["cache_hex"]
                .as_str()
                .ok_or("restart cache bytes missing")?,
        )?,
        modified: UNIX_EPOCH
            + Duration::from_nanos(
                checkpoint["cache_modified_nanos"]
                    .as_u64()
                    .ok_or("restart cache mtime missing")?,
            ),
    };
    let stage = ModelStage::Recover;
    let predecessor = profile_kind.is_predecessor();
    let (session, _) = initialize_http(address)?;
    let mut sse = live_mcp::subscribe_sse_uris(
        address,
        &session,
        &["nms://player/location", "nms://galaxy/stats"],
    )?;
    check_names_boundary(
        fixture,
        address,
        &session,
        NamesBoundary {
            source: &expected_source,
            source_time: expected_source_time,
            cache_stamp: &expected_cache_stamp,
            stage,
            profile,
            watch_enabled: false,
        },
    )?;
    let cursor = sse.cursor();
    sse.wait_quiet_uris_since(
        cursor,
        &["nms://player/location", "nms://galaxy/stats"],
        Duration::from_millis(650),
    )?;
    fs::write(
        fixture.join("restart.names.ready"),
        format!("address={address} session={session} predecessor={predecessor}\n"),
    )?;
    File::open(fixture.join("restart.names.ready"))?.sync_all()?;
    let (line, offset) = wait_control_line(
        &fixture.join("restart.names.commands"),
        0,
        Duration::from_secs(60),
    )?;
    if line != "pre-exit" {
        return Err(format!("name restart monitor expected pre-exit, got {line:?}").into());
    }
    check_names_boundary(
        fixture,
        address,
        &session,
        NamesBoundary {
            source: &expected_source,
            source_time: expected_source_time,
            cache_stamp: &expected_cache_stamp,
            stage,
            profile,
            watch_enabled: false,
        },
    )?;
    println!("name-restart-pre-exit=pass");
    io::stdout().flush()?;
    let (line, _) = wait_control_line(
        &fixture.join("restart.names.commands"),
        offset,
        Duration::from_secs(60),
    )?;
    if line != "finish" {
        return Err(format!("name restart monitor expected finish, got {line:?}").into());
    }
    sse.wait_closed_without_resource_updates(
        Duration::from_secs(8),
        &["nms://player/location", "nms://galaxy/stats"],
    )?;
    sse.close_and_join()?;
    check_names_file_boundary(
        fixture,
        &NamesBoundary {
            source: &expected_source,
            source_time: expected_source_time,
            cache_stamp: &expected_cache_stamp,
            stage,
            profile,
            watch_enabled: false,
        },
    )?;
    println!("name-restart=pass disabled-watch+both-resource-eof+whole-cache");
    Ok(())
}

fn fixture_cache_path(fixture: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let relative = fs::read_to_string(fixture.join("cache-path.txt"))?;
    let relative = PathBuf::from(relative.trim());
    if relative.is_absolute()
        || relative
            .components()
            .any(|part| part == std::path::Component::ParentDir)
    {
        return Err("fixture cache path escapes its owned directory".into());
    }
    Ok(fixture.join(relative))
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex(value: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if !value.len().is_multiple_of(2) {
        return Err("hex fixture bytes have odd length".into());
    }
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair)?;
            Ok(u8::from_str_radix(text, 16)?)
        })
        .collect()
}

fn atomic_replace(
    path: &Path,
    bytes: &[u8],
) -> Result<std::time::SystemTime, Box<dyn std::error::Error>> {
    let replacement = path.with_extension("replacement");
    fs::write(&replacement, bytes)?;
    fs::rename(replacement, path)?;
    let observed = fs::read(path)?;
    if observed != bytes {
        return Err("atomic source replacement bytes differ immediately after rename".into());
    }
    Ok(fs::metadata(path)?.modified()?)
}

fn save_bytes(
    version: u32,
    reality: u8,
    units: u64,
    nanites: u64,
    specials: u64,
    previous: Option<(u8, i64, i64, i64, u16, u8)>,
) -> Vec<u8> {
    let mut value = serde_json::json!({
        "Version": version, "Platform": "Mac|Final", "ActiveContext": "Main",
        "CommonStateData": {"SaveName":"Slice19", "TotalPlayTime":100},
        "BaseContext": {"GameMode":1,"PlayerStateData":{
            "UniverseAddress":{"RealityIndex":reality,"GalacticAddress":{"VoxelX":0,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":1,"PlanetIndex":0}},
            "Units":units,"Nanites":nanites,"Specials":specials,"PersistentPlayerBases":[]
        }},
        "ExpeditionContext":{"GameMode":6,"PlayerStateData":{"UniverseAddress":{"RealityIndex":0,"GalacticAddress":{"VoxelX":0,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":0,"PlanetIndex":0}},"Units":0,"Nanites":0,"Specials":0,"PersistentPlayerBases":[]}},
        "DiscoveryManagerData":{"DiscoveryData-v1":{"ReserveStore":0,"ReserveManaged":0,"Store":{"Record":[]}}}
    });
    if let Some(previous) = previous {
        value["BaseContext"]["PlayerStateData"]["PreviousUniverseAddress"] = serde_json::json!({
            "RealityIndex":previous.0,"GalacticAddress":{"VoxelX":previous.1,"VoxelY":previous.2,"VoxelZ":previous.3,"SolarSystemIndex":previous.4,"PlanetIndex":previous.5}
        });
    }
    serde_json::to_vec(&value).expect("fixture serializes")
}

fn self_check() -> Result<(), Box<dyn std::error::Error>> {
    use nms_core::address::GalacticAddress;
    use nms_core::system::DiscoverySource;
    let expected = expected_cache(4720, 0, 7, 8, 9, None, 7);
    let expected_model = live_mcp::expected_model(0, 7, 8, 9, None, 7);
    let rebuilt_expected = nms_cache::rebuild_model(&expected);
    live_mcp::compare_cache_model(&expected, &rebuilt_expected, &expected, &expected_model)?;
    let checked_files = tempfile::tempdir()?;
    let predecessor =
        include_bytes!("../../nms-cache/tests/fixtures/predecessor-schema3-semantic3.rkyv");
    if predecessor.len() != 752 || &predecessor[..16] != b"NMSCACHE\x03\0\0\0\x03\0\0\0" {
        return Err("self-check predecessor fixture size/header differs".into());
    }
    let predecessor_path = checked_files.path().join("predecessor-semantic3.rkyv");
    fs::write(&predecessor_path, predecessor)?;
    if !matches!(
        nms_cache::read_cache(&predecessor_path),
        Err(nms_cache::CacheError::Incompatible(
            nms_cache::CacheCompatibilityError::SchemaMismatch {
                found: 3,
                expected: 4
            }
        ))
    ) {
        return Err("self-check did not reject the genuine semantic3/schema3 predecessor".into());
    }
    let name_predecessor =
        include_bytes!("../../nms-cache/tests/fixtures/predecessor-schema3-semantic4.rkyv");
    if name_predecessor.len() != 224 || &name_predecessor[..16] != b"NMSCACHE\x03\0\0\0\x04\0\0\0" {
        return Err("self-check authentic name predecessor fixture size/header differs".into());
    }
    let name_predecessor_path = checked_files.path().join("predecessor-semantic4.rkyv");
    fs::write(&name_predecessor_path, name_predecessor)?;
    if !matches!(
        nms_cache::read_cache(&name_predecessor_path),
        Err(nms_cache::CacheError::Incompatible(
            nms_cache::CacheCompatibilityError::SchemaMismatch {
                found: 3,
                expected: 4
            }
        ))
    ) {
        return Err(
            "self-check did not reject authentic schema3/semantic4 names predecessor".into(),
        );
    }
    let source_predecessor =
        include_bytes!("../../nms-cache/tests/fixtures/predecessor-schema3-semantic5.rkyv");
    if source_predecessor.len() != 632
        || &source_predecessor[..16] != b"NMSCACHE\x03\0\0\0\x05\0\0\0"
    {
        return Err(
            "self-check authentic schema3/semantic5 source predecessor fixture differs".into(),
        );
    }
    let source_predecessor_path = checked_files.path().join("predecessor-semantic5.rkyv");
    fs::write(&source_predecessor_path, source_predecessor)?;
    if !matches!(
        nms_cache::read_cache(&source_predecessor_path),
        Err(nms_cache::CacheError::Incompatible(
            nms_cache::CacheCompatibilityError::SchemaMismatch {
                found: 3,
                expected: 4
            }
        ))
    ) {
        return Err(
            "self-check did not reject authentic schema3/semantic5 source predecessor".into(),
        );
    }
    let initial_names = name_observations::expected_cache(ModelStage::Initial, 7);
    let initial_names_model = name_observations::expected_model(ModelStage::Initial);
    let initial_names_path = checked_files.path().join("names-current-semantic6.rkyv");
    nms_cache::write_cache(&initial_names, &initial_names_path)?;
    live_mcp::assert_cache_oracle(&initial_names_path, &initial_names, &initial_names_model)?;
    let empty = live_mcp::expected_empty_cache(4720, 0, 7, 8, 9, 0);
    let empty_model = live_mcp::expected_empty_model(0, 7, 8, 9, 0);
    let cold_cross = live_mcp::expected_cold_cross_cache();
    let cold_cross_model = live_mcp::expected_cold_cross_model();
    let genuine_empty_cross = live_mcp::expected_genuine_empty_cross_cache();
    let genuine_empty_cross_model = live_mcp::expected_genuine_empty_cross_model();
    for (name, cache, model) in [
        ("warm", &expected, &expected_model),
        ("empty", &empty, &empty_model),
        ("cold-cross", &cold_cross, &cold_cross_model),
        (
            "genuine-empty-cross",
            &genuine_empty_cross,
            &genuine_empty_cross_model,
        ),
    ] {
        let path = checked_files.path().join(format!("{name}.rkyv"));
        nms_cache::write_cache(cache, &path)?;
        live_mcp::assert_cache_oracle(&path, cache, model)?;
    }

    let mut wrong_currency = expected_cache(4720, 0, 7, 8, 9, None, 7);
    wrong_currency.player_state.as_mut().unwrap().units = 70;
    reject("currency-only", &wrong_currency, &expected, &expected_model)?;
    let mut missing_planet = expected_cache(4720, 0, 7, 8, 9, None, 7);
    missing_planet.systems[0].planets.clear();
    reject(
        "missing planet",
        &missing_planet,
        &expected,
        &expected_model,
    )?;
    let mut wrong_selection = expected_cache(4720, 0, 7, 8, 9, None, 7);
    wrong_selection.active_galaxy = 9;
    reject(
        "selected galaxy",
        &wrong_selection,
        &expected,
        &expected_model,
    )?;
    let mut wrong_version = expected_cache(4720, 0, 7, 8, 9, None, 7);
    wrong_version.save_version = 4721;
    reject("source version", &wrong_version, &expected, &expected_model)?;
    let mut extra = expected_cache(4720, 0, 7, 8, 9, None, 7);
    extra.systems.push(nms_cache::data::CachedSystem {
        source: DiscoverySource::Personal,
        address: GalacticAddress::new(4, 4, 4, 4, 0, 9),
        name: Some("Unexpected".into()),
        discoverer: None,
        timestamp_secs: None,
        planets: vec![],
        is_player_placeholder: false,
    });
    reject("extra named system", &extra, &expected, &expected_model)?;
    let mut wrong_marker = live_mcp::expected_cold_cross_cache();
    wrong_marker.systems[0].is_player_placeholder = false;
    reject(
        "cold-cross marker flag",
        &wrong_marker,
        &cold_cross,
        &cold_cross_model,
    )?;
    let rebuilt = nms_cache::rebuild_model(&expected);
    live_mcp::compare_cache_model(&expected, &rebuilt, &expected, &expected_model)?;
    let mut reversed = expected_cache(4720, 0, 7, 8, 9, None, 7);
    reversed.systems.reverse();
    let reversed_model = nms_cache::rebuild_model(&reversed);
    live_mcp::compare_cache_model(&reversed, &reversed_model, &expected, &expected_model)?;
    let mut wrong_name_index = live_mcp::expected_model(0, 7, 8, 9, None, 7);
    wrong_name_index.name_index.clear();
    reject_model("name index", &wrong_name_index, &expected, &expected_model)?;
    let mut wrong_biome_index = live_mcp::expected_model(0, 7, 8, 9, None, 7);
    wrong_biome_index.biome_index.clear();
    reject_model(
        "biome index",
        &wrong_biome_index,
        &expected,
        &expected_model,
    )?;
    let mut wrong_graph_weight = live_mcp::expected_model(0, 7, 8, 9, None, 7);
    let node = *wrong_graph_weight.node_map.values().next().unwrap();
    *wrong_graph_weight.graph.node_weight_mut(node).unwrap() =
        nms_graph::SystemId::from_address(&GalacticAddress::new(4, 4, 4, 4, 0, 9));
    reject_model(
        "graph node weight",
        &wrong_graph_weight,
        &expected,
        &expected_model,
    )?;
    let mut wrong_spatial = live_mcp::expected_model(0, 7, 8, 9, None, 7);
    wrong_spatial.spatial.remove(&9);
    reject_model(
        "spatial-only key",
        &wrong_spatial,
        &expected,
        &expected_model,
    )?;
    let mut unexpected_empty_spatial = live_mcp::expected_model(0, 7, 8, 9, None, 7);
    unexpected_empty_spatial
        .spatial
        .insert(7, Default::default());
    reject_model(
        "unexpected empty spatial galaxy",
        &unexpected_empty_spatial,
        &expected,
        &expected_model,
    )?;
    reject_model(
        "cold-cross derived placeholder provenance",
        &genuine_empty_cross_model,
        &cold_cross,
        &cold_cross_model,
    )?;

    let (name_raw_negatives, name_derived_negatives, name_golden_files) =
        check_name_cache_oracles(checked_files.path())?;
    let name_file_negatives = check_name_file_boundary_self_check(checked_files.path())?;
    live_mcp::check_paired_sse_cursor_contract().map_err(std::io::Error::other)?;

    let lifecycle_stage = live_mcp::UnknownLifecycleStage::Startup;
    let lifecycle_data = live_mcp::expected_unknown_lifecycle_cache(lifecycle_stage);
    let lifecycle_live = live_mcp::expected_unknown_lifecycle_model(lifecycle_stage, false);
    let lifecycle_rebuilt = live_mcp::expected_unknown_lifecycle_model(lifecycle_stage, true);
    let lifecycle_dir = tempfile::tempdir()?;
    let mut lifecycle_file_count = 0;
    for stage in [
        live_mcp::UnknownLifecycleStage::Startup,
        live_mcp::UnknownLifecycleStage::Promoted,
    ] {
        let expected_data = live_mcp::expected_unknown_lifecycle_cache(stage);
        let expected_rebuilt = live_mcp::expected_unknown_lifecycle_model(stage, true);
        for reversed in [false, true] {
            let mut written = live_mcp::expected_unknown_lifecycle_cache(stage);
            if reversed {
                written.systems.reverse();
            }
            let path = lifecycle_dir.path().join(format!(
                "lifecycle-{}-{}.rkyv",
                if matches!(stage, live_mcp::UnknownLifecycleStage::Startup) {
                    "startup"
                } else {
                    "promoted"
                },
                if reversed { "reversed" } else { "literal" }
            ));
            nms_cache::write_cache(&written, &path)?;
            let checked = nms_cache::read_cache(&path)?;
            let rebuilt = nms_cache::rebuild_model(&checked);
            live_mcp::compare_cache_model(&checked, &rebuilt, &expected_data, &expected_rebuilt)?;
            lifecycle_file_count += 1;
        }
    }
    live_mcp::compare_cache_model(
        &lifecycle_data,
        &lifecycle_rebuilt,
        &lifecycle_data,
        &lifecycle_rebuilt,
    )?;
    let u2 = GalacticAddress::new(0, 0, 0, 2, 0, 0);
    let mut raw_marker_false = live_mcp::expected_unknown_lifecycle_cache(lifecycle_stage);
    raw_marker_false
        .systems
        .iter_mut()
        .find(|system| system.address == u2)
        .ok_or("literal U2 cache row missing")?
        .is_player_placeholder = false;
    let raw_marker_path = lifecycle_dir.path().join("lifecycle-raw-marker-false.rkyv");
    nms_cache::write_cache(&raw_marker_false, &raw_marker_path)?;
    let raw_marker_checked = nms_cache::read_cache(&raw_marker_path)?;
    let raw_marker_rebuilt = nms_cache::rebuild_model(&raw_marker_checked);
    if live_mcp::compare_cache_model(
        &raw_marker_checked,
        &raw_marker_rebuilt,
        &lifecycle_data,
        &lifecycle_rebuilt,
    )
    .is_ok()
    {
        return Err("raw U2 marker mutation passed the consumed combined predicate".into());
    }
    let mut derived_marker_cleared =
        live_mcp::expected_unknown_lifecycle_model(lifecycle_stage, false);
    derived_marker_cleared.insert_system(nms_core::system::System::new(
        u2,
        None,
        None,
        None,
        vec![],
    ));
    let node_keys = |model: &nms_graph::GalaxyModel| {
        model
            .node_map
            .keys()
            .map(|id| format!("{id:?}"))
            .collect::<std::collections::BTreeSet<_>>()
    };
    let node_weights = |model: &nms_graph::GalaxyModel| {
        model
            .graph
            .node_weights()
            .map(|id| format!("{id:?}"))
            .collect::<std::collections::BTreeSet<_>>()
    };
    let edge_rows = |model: &nms_graph::GalaxyModel| {
        let mut rows = model
            .graph
            .edge_indices()
            .map(|edge| {
                let (left, right) = model.graph.edge_endpoints(edge).expect("edge endpoints");
                let mut left = format!("{:?}", model.graph.node_weight(left).expect("left id"));
                let mut right = format!("{:?}", model.graph.node_weight(right).expect("right id"));
                if left > right {
                    std::mem::swap(&mut left, &mut right);
                }
                (
                    left,
                    right,
                    model
                        .graph
                        .edge_weight(edge)
                        .expect("edge weight")
                        .to_bits(),
                )
            })
            .collect::<Vec<_>>();
        rows.sort();
        rows
    };
    let spatial_rows = |model: &nms_graph::GalaxyModel| {
        let mut rows = model
            .spatial
            .iter()
            .flat_map(|(galaxy, tree)| {
                tree.iter().map(|point| {
                    (
                        *galaxy,
                        point.id.reality_index(),
                        point.id.system_address(),
                        point.point.map(f64::to_bits),
                    )
                })
            })
            .collect::<Vec<_>>();
        rows.sort();
        rows
    };
    if derived_marker_cleared.systems != lifecycle_live.systems
        || derived_marker_cleared.planets != lifecycle_live.planets
        || derived_marker_cleared.bases != lifecycle_live.bases
        || derived_marker_cleared.name_index != lifecycle_live.name_index
        || derived_marker_cleared.biome_index != lifecycle_live.biome_index
        || derived_marker_cleared.address_to_id != lifecycle_live.address_to_id
        || derived_marker_cleared.player_state != lifecycle_live.player_state
        || derived_marker_cleared.active_galaxy != lifecycle_live.active_galaxy
        || node_keys(&derived_marker_cleared) != node_keys(&lifecycle_live)
        || node_weights(&derived_marker_cleared) != node_weights(&lifecycle_live)
        || !derived_marker_cleared
            .node_map
            .iter()
            .all(|(id, index)| derived_marker_cleared.graph.node_weight(*index) == Some(id))
        || !lifecycle_live
            .node_map
            .iter()
            .all(|(id, index)| lifecycle_live.graph.node_weight(*index) == Some(id))
        || derived_marker_cleared.graph.node_count() != lifecycle_live.graph.node_count()
        || derived_marker_cleared.graph.edge_count() != lifecycle_live.graph.edge_count()
        || edge_rows(&derived_marker_cleared) != edge_rows(&lifecycle_live)
        || spatial_rows(&derived_marker_cleared) != spatial_rows(&lifecycle_live)
    {
        return Err("derived marker mutation changed state beyond marker provenance".into());
    }
    reject_model(
        "derived U2 marker promotion",
        &derived_marker_cleared,
        &lifecycle_data,
        &lifecycle_live,
    )?;
    let mut missing_u2 = live_mcp::expected_unknown_lifecycle_cache(lifecycle_stage);
    missing_u2.systems.retain(|system| system.address != u2);
    let missing_u2_rebuilt = nms_cache::rebuild_model(&missing_u2);
    if live_mcp::compare_cache_model(
        &missing_u2,
        &missing_u2_rebuilt,
        &lifecycle_data,
        &lifecycle_rebuilt,
    )
    .is_ok()
    {
        return Err("missing retained U2 passed the consumed combined predicate".into());
    }
    let physical_u2 = GalacticAddress::new(0, 0, 0, 2, 5, 0);
    let literal_u2 = serde_json::json!({
        "system": "(unknown)", "planets_in_system": 0, "galaxy": "Euclid",
        "voxel_x": 0, "voxel_y": 0, "voxel_z": 0, "solar_system_index": 2,
        "portal_glyphs_hex": "500200000000",
        "portal_glyphs_emoji": nms_query::display::hex_to_emoji("500200000000"),
        "signal_booster": "07FF:007F:07FF:0002",
    });
    live_mcp::assert_location_at(
        &literal_u2,
        physical_u2,
        "(unknown)",
        0,
        "500200000000",
        "07FF:007F:07FF:0002",
    )?;
    let mut wrong_ssi_location = literal_u2;
    wrong_ssi_location["system"] = serde_json::json!("Decoy-0");
    if live_mcp::assert_location_at(
        &wrong_ssi_location,
        physical_u2,
        "(unknown)",
        0,
        "500200000000",
        "07FF:007F:07FF:0002",
    )
    .is_ok()
    {
        return Err("same-voxel wrong-SSI location passed full literal predicate".into());
    }

    println!(
        "golden=pass checked-files=4 negatives=12 (6 raw + 6 derived); predecessor=pass; names-files={name_golden_files} names-negatives={name_raw_negatives}+{name_derived_negatives}+file{name_file_negatives}; paired-uri-cursors=pass; lifecycle-files={lifecycle_file_count} lifecycle-negatives=4; reversed-order=pass"
    );
    Ok(())
}

fn check_name_file_boundary_self_check(
    directory: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let fixture = directory.join("name-file-boundary");
    let home_config = fixture.join("home/.nms-copilot");
    std::fs::create_dir_all(&home_config)?;
    std::fs::write(
        fixture.join(NAME_MARKER),
        b"Project01 Arc06 Slice23 name fixture\n",
    )?;
    let source = fixture.join("save.json");
    let cache = fixture.join("cache.rkyv");
    let config = home_config.join("config.toml");
    let source_bytes = name_observations::source_bytes(SourcePhase::Recover);
    let source_time = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    std::fs::write(&source, &source_bytes)?;
    File::options()
        .write(true)
        .open(&source)?
        .set_times(FileTimes::new().set_modified(source_time))?;
    nms_cache::write_cache(
        &name_observations::expected_cache(ModelStage::Recover, 7),
        &cache,
    )?;
    let cache_time = source_time + Duration::from_secs(100);
    File::options()
        .write(true)
        .open(&cache)?
        .set_times(FileTimes::new().set_modified(cache_time))?;
    std::fs::write(&config, names_config(&cache, true))?;
    std::fs::write(fixture.join("mode.txt"), "names-warm\nenabled\n")?;
    std::fs::write(fixture.join("cache-path.txt"), "cache.rkyv\n")?;
    let expected_stamp = cache_stamp(&cache)?.ok_or("self-check cache stamp missing")?;
    let boundary = NamesBoundary {
        source: &source_bytes,
        source_time,
        cache_stamp: &expected_stamp,
        stage: ModelStage::Recover,
        profile: "warm",
        watch_enabled: true,
    };
    check_names_file_boundary(&fixture, &boundary)?;
    let mut negatives = 0;

    std::fs::write(&source, b"source-byte-mutation")?;
    File::options()
        .write(true)
        .open(&source)?
        .set_times(FileTimes::new().set_modified(source_time))?;
    if check_names_file_boundary(&fixture, &boundary).is_ok() {
        return Err("name file boundary accepted changed source bytes".into());
    }
    negatives += 1;
    std::fs::write(&source, &source_bytes)?;
    File::options()
        .write(true)
        .open(&source)?
        .set_times(FileTimes::new().set_modified(source_time + Duration::from_secs(1)))?;
    if check_names_file_boundary(&fixture, &boundary).is_ok() {
        return Err("name file boundary accepted changed source mtime".into());
    }
    negatives += 1;
    File::options()
        .write(true)
        .open(&source)?
        .set_times(FileTimes::new().set_modified(source_time))?;

    let expected_cache_bytes = std::fs::read(&cache)?;
    nms_cache::write_cache(
        &name_observations::expected_cache(ModelStage::Rename, 7),
        &cache,
    )?;
    File::options()
        .write(true)
        .open(&cache)?
        .set_times(FileTimes::new().set_modified(cache_time))?;
    if check_names_file_boundary(&fixture, &boundary).is_ok() {
        return Err("name file boundary accepted changed cache bytes".into());
    }
    negatives += 1;
    std::fs::write(&cache, expected_cache_bytes)?;
    File::options()
        .write(true)
        .open(&cache)?
        .set_times(FileTimes::new().set_modified(cache_time))?;
    File::options()
        .write(true)
        .open(&cache)?
        .set_times(FileTimes::new().set_modified(cache_time + Duration::from_secs(1)))?;
    if check_names_file_boundary(&fixture, &boundary).is_ok() {
        return Err("name file boundary accepted changed cache mtime".into());
    }
    negatives += 1;
    File::options()
        .write(true)
        .open(&cache)?
        .set_times(FileTimes::new().set_modified(cache_time))?;

    std::fs::write(&config, "changed config\n")?;
    if check_names_file_boundary(&fixture, &boundary).is_ok() {
        return Err("name file boundary accepted changed config".into());
    }
    negatives += 1;
    std::fs::write(&config, names_config(&cache, true))?;
    std::fs::write(cache.with_extension("rkyv.tmp"), b"stale temporary")?;
    if check_names_file_boundary(&fixture, &boundary).is_ok() {
        return Err("name file boundary accepted a cache temporary".into());
    }
    negatives += 1;
    std::fs::remove_file(cache.with_extension("rkyv.tmp"))?;
    check_names_file_boundary(&fixture, &boundary)?;
    Ok(negatives)
}

fn check_name_cache_oracles(
    directory: &Path,
) -> Result<(usize, usize, usize), Box<dyn std::error::Error>> {
    use name_observations::{ModelStage, a0, a9, b0, expected_cache, expected_model};
    use nms_core::biome::Biome;
    use nms_core::system::SystemId;

    let mut files = 0;
    for stage in [
        ModelStage::Initial,
        ModelStage::Overwrite,
        ModelStage::Promote,
        ModelStage::Rename,
        ModelStage::Clear,
        ModelStage::Ignore,
        ModelStage::Recover,
    ] {
        let expected_data = expected_cache(stage, 7);
        let expected_model = expected_model(stage);
        for reversed in [false, true] {
            let mut observed = expected_cache(stage, 7);
            if reversed {
                observed.systems.reverse();
            }
            let suffix = if reversed { "reversed" } else { "literal" };
            let path = directory.join(format!("names-{stage:?}-{suffix}.rkyv"));
            nms_cache::write_cache(&observed, &path)?;
            let raw = nms_cache::read_cache(&path)?;
            let rebuilt = nms_cache::rebuild_model(&raw);
            live_mcp::compare_cache_model(&raw, &rebuilt, &expected_data, &expected_model)?;
            files += 1;
        }
    }

    let expected_data = expected_cache(ModelStage::Recover, 7);
    let golden_model = expected_model(ModelStage::Recover);
    let mut raw_negatives = 0;
    macro_rules! raw_negative {
        ($label:expr, $mutation:expr) => {{
            let mut actual = expected_cache(ModelStage::Recover, 7);
            ($mutation)(&mut actual);
            reject($label, &actual, &expected_data, &golden_model)?;
            raw_negatives += 1;
        }};
    }
    raw_negative!("wrong B0 system name", |cache: &mut CacheData| {
        cache
            .systems
            .iter_mut()
            .find(|row| row.address == b0())
            .unwrap()
            .name = Some("Wrong B0".into());
    });
    raw_negative!("wrong nested B0 planet name", |cache: &mut CacheData| {
        cache
            .systems
            .iter_mut()
            .find(|row| row.address == b0())
            .unwrap()
            .planets[0]
            .name = Some("Wrong B0 moon".into());
    });
    raw_negative!("wrong GG9 A9 name", |cache: &mut CacheData| {
        cache
            .systems
            .iter_mut()
            .find(|row| row.address == a9())
            .unwrap()
            .name = Some("Wrong galaxy name".into());
    });
    let mixed_label = " NMSCE α ";
    let mut mixed_data = expected_cache(ModelStage::Recover, 7);
    let mixed_a9 = mixed_data
        .systems
        .iter_mut()
        .find(|row| row.address == a9())
        .ok_or("literal A9 cache row missing")?;
    mixed_a9.source = nms_core::system::DiscoverySource::Community {
        source: mixed_label.to_owned(),
    };
    let mut mixed_model = expected_model(ModelStage::Recover);
    let mixed_a9_id = SystemId::from_address(&a9());
    mixed_model
        .systems
        .get_mut(&mixed_a9_id)
        .ok_or("literal A9 model row missing")?
        .source = nms_core::system::DiscoverySource::Community {
        source: mixed_label.to_owned(),
    };
    let mixed_path = directory.join("names-recover-mixed-origin.rkyv");
    nms_cache::write_cache(&mixed_data, &mixed_path)?;
    let mixed_checked = nms_cache::read_cache(&mixed_path)?;
    let mixed_rebuilt = nms_cache::rebuild_model(&mixed_checked);
    live_mcp::compare_cache_model(&mixed_checked, &mixed_rebuilt, &mixed_data, &mixed_model)?;
    files += 1;

    let mut wrong_origin = nms_cache::read_cache(&mixed_path)?;
    let mixed_cached_at = wrong_origin.cached_at;
    wrong_origin
        .systems
        .iter_mut()
        .find(|row| row.address == a9())
        .ok_or("literal A9 checked row missing")?
        .source = nms_core::system::DiscoverySource::Personal;
    let wrong_origin_model = nms_cache::rebuild_model(&wrong_origin);
    if live_mcp::compare_cache_model(
        &wrong_origin,
        &wrong_origin_model,
        &mixed_data,
        &mixed_model,
    )
    .is_ok()
    {
        return Err("A9 source-only Personal mutant passed the combined predicate".into());
    }
    wrong_origin
        .systems
        .iter_mut()
        .find(|row| row.address == a9())
        .unwrap()
        .source = mixed_data
        .systems
        .iter()
        .find(|row| row.address == a9())
        .unwrap()
        .source
        .clone();
    live_mcp::compare_cache(&wrong_origin, &mixed_data)?;
    if wrong_origin.cached_at != mixed_cached_at {
        return Err("A9 source-only Personal mutant changed cached_at".into());
    }
    raw_negatives += 1;

    let mut wrong_label = nms_cache::read_cache(&mixed_path)?;
    let mixed_cached_at = wrong_label.cached_at;
    wrong_label
        .systems
        .iter_mut()
        .find(|row| row.address == a9())
        .ok_or("literal A9 checked row missing")?
        .source = nms_core::system::DiscoverySource::Community {
        source: " XMSCE α ".to_owned(),
    };
    let wrong_label_model = nms_cache::rebuild_model(&wrong_label);
    if live_mcp::compare_cache_model(&wrong_label, &wrong_label_model, &mixed_data, &mixed_model)
        .is_ok()
    {
        return Err("A9 one-byte wrong-label mutant passed the combined predicate".into());
    }
    wrong_label
        .systems
        .iter_mut()
        .find(|row| row.address == a9())
        .unwrap()
        .source = mixed_data
        .systems
        .iter()
        .find(|row| row.address == a9())
        .unwrap()
        .source
        .clone();
    live_mcp::compare_cache(&wrong_label, &mixed_data)?;
    if wrong_label.cached_at != mixed_cached_at {
        return Err("A9 wrong-label mutant changed cached_at".into());
    }
    raw_negatives += 1;
    let initial_data = expected_cache(ModelStage::Initial, 7);
    let initial_model = expected_model(ModelStage::Initial);
    let mut initial_marker_false = expected_cache(ModelStage::Initial, 7);
    let a0_row = initial_marker_false
        .systems
        .iter_mut()
        .find(|row| row.address == a0())
        .ok_or("literal initial A0 cache row missing")?;
    if !a0_row.is_player_placeholder {
        return Err("literal initial A0 is not the marked player placeholder".into());
    }
    a0_row.is_player_placeholder = false;
    let initial_marker_path = directory.join("names-initial-a0-marker-false.rkyv");
    nms_cache::write_cache(&initial_marker_false, &initial_marker_path)?;
    let initial_marker_checked = nms_cache::read_cache(&initial_marker_path)?;
    let initial_marker_rebuilt = nms_cache::rebuild_model(&initial_marker_checked);
    if live_mcp::compare_cache_model(
        &initial_marker_checked,
        &initial_marker_rebuilt,
        &initial_data,
        &initial_model,
    )
    .is_ok()
    {
        return Err("initial raw A0 marker mutation passed the consumed combined predicate".into());
    }
    raw_negatives += 1;
    raw_negative!("dropped P2", |cache: &mut CacheData| {
        cache
            .systems
            .iter_mut()
            .find(|row| row.address == b0())
            .unwrap()
            .planets
            .clear();
    });
    raw_negative!("changed B0 biome", |cache: &mut CacheData| {
        cache
            .systems
            .iter_mut()
            .find(|row| row.address == b0())
            .unwrap()
            .planets[0]
            .biome = Some(Biome::Toxic);
    });
    raw_negative!("duplicated B0 planet", |cache: &mut CacheData| {
        let row = cache
            .systems
            .iter_mut()
            .find(|row| row.address == b0())
            .unwrap();
        row.planets.push(row.planets[0].clone());
    });
    raw_negative!("wrong player units", |cache: &mut CacheData| {
        cache.player_state.as_mut().unwrap().units = 70;
    });
    raw_negative!("wrong save version", |cache: &mut CacheData| {
        cache.save_version += 1;
    });
    raw_negative!("wrong selected galaxy", |cache: &mut CacheData| {
        cache.active_galaxy = 0;
    });
    raw_negative!("wrong base identity", |cache: &mut CacheData| {
        cache.bases[0].name = "Wrong Anchor".into();
    });

    let mut derived_negatives = 0;
    macro_rules! model_negative {
        ($label:expr, $mutation:expr) => {{
            let mut actual = expected_model(ModelStage::Recover);
            ($mutation)(&mut actual);
            reject_name_model($label, &actual, &expected_data, &golden_model)?;
            derived_negatives += 1;
        }};
    }
    let a0_id = SystemId::from_address(&a0());
    let b0_id = SystemId::from_address(&b0());
    model_negative!(
        "wrong flat planet name",
        |model: &mut nms_graph::GalaxyModel| {
            model.planets.get_mut(&(b0_id, 2)).unwrap().name = Some("Wrong flat moon".into());
        }
    );
    model_negative!(
        "stale old normalized alias",
        |model: &mut nms_graph::GalaxyModel| {
            model.name_index.insert((0, "current".into()), a0_id);
        }
    );
    model_negative!(
        "missing current alias",
        |model: &mut nms_graph::GalaxyModel| {
            model.name_index.remove(&(0, "current recovered".into()));
        }
    );
    model_negative!(
        "wrong galaxy alias",
        |model: &mut nms_graph::GalaxyModel| {
            model
                .name_index
                .insert((9, "current recovered".into()), a0_id);
        }
    );
    model_negative!(
        "duplicated biome membership",
        |model: &mut nms_graph::GalaxyModel| {
            let key = *model
                .biome_index
                .get(&Biome::Lush)
                .unwrap()
                .first()
                .unwrap();
            model.biome_index.get_mut(&Biome::Lush).unwrap().push(key);
        }
    );
    model_negative!(
        "dropped biome membership",
        |model: &mut nms_graph::GalaxyModel| {
            model.biome_index.get_mut(&Biome::Lush).unwrap().pop();
        }
    );
    model_negative!(
        "wrong edge distance",
        |model: &mut nms_graph::GalaxyModel| {
            let edge = model.graph.edge_indices().next().unwrap();
            *model.graph.edge_weight_mut(edge).unwrap() += 1.0;
        }
    );
    model_negative!(
        "wrong live system count",
        |model: &mut nms_graph::GalaxyModel| {
            model.systems.remove(&b0_id);
        }
    );
    model_negative!("wrong live player", |model: &mut nms_graph::GalaxyModel| {
        model.player_state.as_mut().unwrap().units += 1;
    });
    model_negative!(
        "wrong base identity",
        |model: &mut nms_graph::GalaxyModel| {
            model.bases.get_mut("anchor").unwrap().name = "Wrong Anchor".into();
        }
    );
    model_negative!(
        "wrong active galaxy",
        |model: &mut nms_graph::GalaxyModel| {
            model.active_galaxy = 0;
        }
    );
    let initial_expected_data = expected_cache(ModelStage::Initial, 7);
    let initial_expected_model = expected_model(ModelStage::Initial);
    let initial_a0_id = SystemId::from_address(&a0());
    let mut initial_marker_cleared = expected_model(ModelStage::Initial);
    if !initial_marker_cleared.is_player_placeholder(&initial_a0_id) {
        return Err("literal initial A0 model is not marked as a placeholder".into());
    }
    initial_marker_cleared.insert_system(nms_core::system::System::new(
        a0(),
        None,
        None,
        None,
        vec![],
    ));
    if initial_marker_cleared.is_player_placeholder(&initial_a0_id) {
        return Err("ordinary identical A0 insertion did not clear placeholder provenance".into());
    }
    reject_name_model(
        "derived A0 marker promotion",
        &initial_marker_cleared,
        &initial_expected_data,
        &initial_expected_model,
    )?;
    derived_negatives += 1;
    Ok((raw_negatives, derived_negatives, files))
}

fn reject_name_model(
    label: &str,
    actual: &nms_graph::GalaxyModel,
    expected_data: &CacheData,
    expected_model: &nms_graph::GalaxyModel,
) -> Result<(), Box<dyn std::error::Error>> {
    if live_mcp::compare_cache_model(expected_data, actual, expected_data, expected_model).is_ok() {
        return Err(format!("same raw+rebuilt validator accepted {label} derived mismatch").into());
    }
    Ok(())
}

fn verify_unknown_fixture(fixture: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source = live_mcp::unknown_lifecycle_source(
        4720,
        2,
        7,
        8,
        9,
        Some(GalacticAddress::new(2, 3, 4, 1, 1, 9)),
        false,
    );
    let expected_source_time = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    let cache = fixture_cache_path(fixture)?;
    if fs::read_to_string(fixture.join("mode.txt"))? != "unknown\nenabled\n"
        || fs::read_to_string(fixture.join("cache-path.txt"))? != "cache.rkyv"
        || fs::read(fixture.join("save.json"))? != source
        || fs::metadata(fixture.join("save.json"))?.modified()? != expected_source_time
        || !nms_cache::is_cache_fresh(&cache, &fixture.join("save.json"))
        || cache.with_extension("rkyv.tmp").exists()
    {
        return Err("unknown fixture source/mode/freshness/temp state differs".into());
    }
    let actual = nms_cache::read_cache(&cache)?;
    let expected = live_mcp::unknown_lifecycle_seed_cache();
    live_mcp::compare_cache(&actual, &expected).map_err(|error| error.to_string())?;
    let config = fs::read_to_string(fixture.join("home/.nms-copilot/config.toml"))?;
    let expected_config = format!(
        "[display]\nshow_banner = false\nshow_system_banner = false\ncolor = false\n\n[watch]\nenabled = true\ndebounce_ms = 137\n\n[cache]\nenabled = true\npath = {:?}\n",
        cache.to_string_lossy()
    );
    if config != expected_config {
        return Err("unknown fixture cache/watcher config differs".into());
    }
    Ok(())
}

fn run_unknown_monitor(
    fixture: &Path,
    address: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    verify_unknown_fixture_after_startup(fixture)?;
    let source = fixture.join("save.json");
    let cache = fixture_cache_path(fixture)?;
    let control = fixture.join("monitor.commands");
    let mut expected_source = live_mcp::unknown_lifecycle_source(
        4720,
        2,
        7,
        8,
        9,
        Some(GalacticAddress::new(2, 3, 4, 1, 1, 9)),
        false,
    );
    let mut expected_source_time = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    let mut checkpoint = cache_stamp(&cache)?;
    let mut stage = live_mcp::UnknownLifecycleStage::Startup;
    let mut next_step = 0usize;
    let (session, _) = initialize_http(address)?;
    let mut sse = subscribe_sse(address, &session)?;
    let (tool, resource) = location_rpc_pair(address, &session)?;
    live_mcp::assert_location_pair_at(
        &tool,
        &resource,
        GalacticAddress::new(0, 0, 0, 2, 5, 0),
        "(unknown)",
        0,
        "500200000000",
        "07FF:007F:07FF:0002",
    )
    .map_err(|error| format!("repaired U2 location: {error}"))?;
    check_unknown_boundary(
        fixture,
        &expected_source,
        expected_source_time,
        &checkpoint,
        stage,
    )?;
    fs::write(
        fixture.join("monitor.ready"),
        format!("address={address} session={session}\n"),
    )?;
    File::open(fixture.join("monitor.ready"))?.sync_all()?;
    let mut offset = 0;

    loop {
        let (line, next_offset) = wait_control_line(&control, offset, Duration::from_secs(60))?;
        offset = next_offset;
        let required = [
            "move",
            "malformed",
            "invalid-address",
            "promote",
            "identical",
            "pre-exit",
            "finish",
        ]
        .get(next_step)
        .copied()
        .ok_or("unknown lifecycle monitor received a command after finish")?;
        if line != required {
            return Err(format!(
                "unknown lifecycle step {next_step} requires {required:?}, received {line:?}"
            )
            .into());
        }
        check_unknown_boundary(
            fixture,
            &expected_source,
            expected_source_time,
            &checkpoint,
            stage,
        )?;
        match line.as_str() {
            "move" => {
                let cursor = sse.cursor();
                let moved = live_mcp::unknown_lifecycle_source(
                    4721,
                    3,
                    70,
                    80,
                    90,
                    Some(GalacticAddress::new(0, 0, 0, 2, 5, 0)),
                    false,
                );
                expected_source_time = atomic_replace(&source, &moved)?;
                expected_source = moved;
                sse.wait_for_resource_updated_since(
                    cursor,
                    "nms://player/location",
                    Duration::from_secs(8),
                )?;
                stage = live_mcp::UnknownLifecycleStage::Moved;
                let (tool, resource) = location_rpc_pair(address, &session)?;
                live_mcp::assert_location_pair_at(
                    &tool,
                    &resource,
                    GalacticAddress::new(0, 0, 0, 3, 5, 0),
                    "(unknown)",
                    0,
                    "500300000000",
                    "07FF:007F:07FF:0003",
                )
                .map_err(|error| format!("moved U3 location: {error}"))?;
                checkpoint = cache_stamp(&cache)?;
                check_unknown_boundary(
                    fixture,
                    &expected_source,
                    expected_source_time,
                    &checkpoint,
                    stage,
                )?;
                next_step += 1;
                println!("unknown-move=pass exact-U3+whole-cache");
                io::stdout().flush()?;
            }
            "malformed" => {
                let bad = br#"{"Version":broken"#;
                let before = location_rpc_pair(address, &session)?;
                let cursor = sse.cursor();
                expected_source_time = atomic_replace(&source, bad)?;
                expected_source = bad.to_vec();
                sse.wait_quiet_since(cursor, Duration::from_millis(650))?;
                if location_rpc_pair(address, &session)? != before {
                    return Err("malformed source changed full tool/resource location".into());
                }
                let (tool, resource) = location_rpc_pair(address, &session)?;
                live_mcp::assert_location_pair_at(
                    &tool,
                    &resource,
                    GalacticAddress::new(0, 0, 0, 3, 5, 0),
                    "(unknown)",
                    0,
                    "500300000000",
                    "07FF:007F:07FF:0003",
                )?;
                check_unknown_boundary(
                    fixture,
                    &expected_source,
                    expected_source_time,
                    &checkpoint,
                    stage,
                )?;
                next_step += 1;
                println!("unknown-malformed=pass last-good+cache-retained");
                io::stdout().flush()?;
            }
            "invalid-address" => {
                let invalid = live_mcp::unknown_lifecycle_invalid_address_source();
                let cursor = sse.cursor();
                expected_source_time = atomic_replace(&source, &invalid)?;
                expected_source = invalid;
                sse.wait_quiet_since(cursor, Duration::from_millis(650))?;
                let (tool, resource) = location_rpc_pair(address, &session)?;
                live_mcp::assert_location_pair_at(
                    &tool,
                    &resource,
                    GalacticAddress::new(0, 0, 0, 3, 5, 0),
                    "(unknown)",
                    0,
                    "500300000000",
                    "07FF:007F:07FF:0003",
                )?;
                check_unknown_boundary(
                    fixture,
                    &expected_source,
                    expected_source_time,
                    &checkpoint,
                    stage,
                )?;
                next_step += 1;
                println!("unknown-invalid-address=pass last-good+cache-retained");
                io::stdout().flush()?;
            }
            "promote" => {
                let cursor = sse.cursor();
                let promoted = live_mcp::unknown_lifecycle_source(
                    4722,
                    3,
                    700,
                    800,
                    900,
                    Some(GalacticAddress::new(0, 0, 0, 2, 5, 0)),
                    true,
                );
                expected_source_time = atomic_replace(&source, &promoted)?;
                expected_source = promoted;
                sse.wait_for_resource_updated_since(
                    cursor,
                    "nms://player/location",
                    Duration::from_secs(8),
                )?;
                stage = live_mcp::UnknownLifecycleStage::Promoted;
                let (tool, resource) = location_rpc_pair(address, &session)?;
                live_mcp::assert_location_pair_at(
                    &tool,
                    &resource,
                    GalacticAddress::new(0, 0, 0, 3, 5, 0),
                    "-",
                    1,
                    "500300000000",
                    "07FF:007F:07FF:0003",
                )?;
                checkpoint = cache_stamp(&cache)?;
                check_unknown_boundary(
                    fixture,
                    &expected_source,
                    expected_source_time,
                    &checkpoint,
                    stage,
                )?;
                next_step += 1;
                println!("unknown-promotion=pass exact-real-unnamed+whole-cache");
                io::stdout().flush()?;
            }
            "identical" => {
                if stage != live_mcp::UnknownLifecycleStage::Promoted {
                    return Err("identical rewrite requires promoted U3".into());
                }
                let fixed = UNIX_EPOCH + Duration::from_secs(1_700_000_777);
                let bytes = fs::read(&cache)?;
                File::options()
                    .write(true)
                    .open(&cache)?
                    .set_times(FileTimes::new().set_modified(fixed))?;
                checkpoint = Some(CacheStamp {
                    bytes,
                    modified: fixed,
                });
                let cursor = sse.cursor();
                expected_source_time = atomic_replace(&source, &expected_source)?;
                sse.wait_quiet_since(cursor, Duration::from_millis(650))?;
                let (tool, resource) = location_rpc_pair(address, &session)?;
                live_mcp::assert_location_pair_at(
                    &tool,
                    &resource,
                    GalacticAddress::new(0, 0, 0, 3, 5, 0),
                    "-",
                    1,
                    "500300000000",
                    "07FF:007F:07FF:0003",
                )?;
                check_unknown_boundary(
                    fixture,
                    &expected_source,
                    expected_source_time,
                    &checkpoint,
                    stage,
                )?;
                next_step += 1;
                println!("unknown-identical=pass exact-cache-bytes+mtime");
                io::stdout().flush()?;
            }
            "pre-exit" => {
                if stage != live_mcp::UnknownLifecycleStage::Promoted || next_step != 5 {
                    return Err("pre-exit requires the ordered promoted lifecycle boundary".into());
                }
                let (tool, resource) = location_rpc_pair(address, &session)?;
                live_mcp::assert_location_pair_at(
                    &tool,
                    &resource,
                    GalacticAddress::new(0, 0, 0, 3, 5, 0),
                    "-",
                    1,
                    "500300000000",
                    "07FF:007F:07FF:0003",
                )?;
                next_step += 1;
                println!("unknown-pre-exit=pass");
                io::stdout().flush()?;
            }
            "finish" => {
                if stage != live_mcp::UnknownLifecycleStage::Promoted || next_step != 6 {
                    return Err("finish requires the ordered promoted lifecycle boundary".into());
                }
                sse.wait_closed_without_resource_updated(
                    Duration::from_secs(8),
                    "nms://player/location",
                )?;
                sse.close_and_join()?;
                thread::sleep(Duration::from_millis(650));
                check_unknown_boundary(
                    fixture,
                    &expected_source,
                    expected_source_time,
                    &checkpoint,
                    stage,
                )?;
                println!("unknown-first-eof=pass sse-reader-joined+cache-stable");
                return Ok(());
            }
            other => return Err(format!("unknown lifecycle monitor command {other:?}").into()),
        }
    }
}

fn verify_unknown_fixture_after_startup(fixture: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source = live_mcp::unknown_lifecycle_source(
        4720,
        2,
        7,
        8,
        9,
        Some(GalacticAddress::new(2, 3, 4, 1, 1, 9)),
        false,
    );
    check_unknown_boundary(
        fixture,
        &source,
        UNIX_EPOCH + Duration::from_secs(1_700_000_000),
        &cache_stamp(&fixture_cache_path(fixture)?)?,
        live_mcp::UnknownLifecycleStage::Startup,
    )
}

fn check_unknown_boundary(
    fixture: &Path,
    source: &[u8],
    source_time: std::time::SystemTime,
    checkpoint: &Option<CacheStamp>,
    stage: live_mcp::UnknownLifecycleStage,
) -> Result<(), Box<dyn std::error::Error>> {
    let marker = fixture.join(MARKER);
    let cache = fixture_cache_path(fixture)?;
    let mode = fs::read_to_string(fixture.join("mode.txt"))?;
    let watch_enabled = match mode.as_str() {
        "unknown\nenabled\n" => true,
        "unknown\ndisabled\n" => false,
        _ => return Err("owned lifecycle mode/watch boundary changed".into()),
    };
    if !marker.is_file()
        || fs::read(&marker)? != b"Project01 Arc06 Slice19 fixture\n"
        || fs::read_to_string(fixture.join("cache-path.txt"))? != "cache.rkyv"
        || fs::read(fixture.join("save.json"))? != source
        || fs::metadata(fixture.join("save.json"))?.modified()? != source_time
        || cache.with_extension("rkyv.tmp").exists()
    {
        return Err("owned lifecycle source/marker/temp boundary changed".into());
    }
    let expected_config = format!(
        "[display]\nshow_banner = false\nshow_system_banner = false\ncolor = false\n\n[watch]\nenabled = {}\ndebounce_ms = 137\n\n[cache]\nenabled = true\npath = {:?}\n",
        watch_enabled,
        cache.to_string_lossy()
    );
    if fs::read_to_string(fixture.join("home/.nms-copilot/config.toml"))? != expected_config {
        return Err("complete lifecycle config differs".into());
    }
    let actual = cache_stamp(&cache)?;
    match (checkpoint, actual) {
        (Some(expected), Some(actual))
            if expected.bytes == actual.bytes && expected.modified == actual.modified => {}
        _ => return Err("lifecycle cache exact bytes/mtime checkpoint changed".into()),
    }
    live_mcp::assert_unknown_lifecycle_cache(&cache, stage)
        .map_err(|error| format!("whole raw/rebuilt cache oracle: {error}"))?;
    Ok(())
}

fn prepare_unknown_restart(fixture: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let cache = fixture_cache_path(fixture)?;
    let promoted = live_mcp::unknown_lifecycle_source(
        4722,
        3,
        700,
        800,
        900,
        Some(GalacticAddress::new(0, 0, 0, 2, 5, 0)),
        true,
    );
    let cache_state = cache_stamp(&cache)?.ok_or("promoted cache missing")?;
    let source = fs::read(fixture.join("save.json"))?;
    if source != promoted {
        return Err("restart preparation requires exact promoted source".into());
    }
    let malformed = br#"{"Version":restart-opposed"#;
    atomic_replace(&fixture.join("save.json"), malformed)?;
    let older = cache_state.modified - Duration::from_secs(1);
    File::options()
        .write(true)
        .open(fixture.join("save.json"))?
        .set_times(FileTimes::new().set_modified(older))?;
    if fs::metadata(fixture.join("save.json"))?.modified()? >= cache_state.modified
        || fs::read(&cache)? != cache_state.bytes
        || fs::metadata(&cache)?.modified()? != cache_state.modified
    {
        return Err("opposed restart source/cache freshness boundary differs".into());
    }
    fs::write(fixture.join("mode.txt"), b"unknown\ndisabled\n")?;
    let config = format!(
        "[display]\nshow_banner = false\nshow_system_banner = false\ncolor = false\n\n[watch]\nenabled = false\ndebounce_ms = 137\n\n[cache]\nenabled = true\npath = {:?}\n",
        cache.to_string_lossy()
    );
    fs::write(fixture.join("home/.nms-copilot/config.toml"), config)?;
    let source_time = fs::metadata(fixture.join("save.json"))?.modified()?;
    let config = fs::read_to_string(fixture.join("home/.nms-copilot/config.toml"))?;
    fs::write(
        fixture.join("restart-checkpoint.json"),
        serde_json::to_vec(&serde_json::json!({
            "cache_bytes_hex": hex_bytes(&cache_state.bytes),
            "cache_modified_nanos": cache_state.modified.duration_since(UNIX_EPOCH)?.as_nanos(),
            "source_hex": hex_bytes(&fs::read(fixture.join("save.json"))?),
            "source_modified_nanos": source_time.duration_since(UNIX_EPOCH)?.as_nanos(),
            "config": config,
            "mode": fs::read_to_string(fixture.join("mode.txt"))?,
            "cache_path": fs::read_to_string(fixture.join("cache-path.txt"))?,
        }))?,
    )?;
    File::open(fixture.join("restart-checkpoint.json"))?.sync_all()?;
    Ok(())
}

fn run_unknown_restart_monitor(
    fixture: &Path,
    address: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let cache = fixture_cache_path(fixture)?;
    let checkpoint: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.join("restart-checkpoint.json"))?)?;
    let cache_checkpoint = CacheStamp {
        bytes: decode_hex(
            checkpoint["cache_bytes_hex"]
                .as_str()
                .ok_or("checkpoint cache bytes")?,
        )?,
        modified: UNIX_EPOCH
            + Duration::from_nanos(
                checkpoint["cache_modified_nanos"]
                    .as_u64()
                    .ok_or("checkpoint cache time")?,
            ),
    };
    let expected_source = decode_hex(
        checkpoint["source_hex"]
            .as_str()
            .ok_or("checkpoint source")?,
    )?;
    let expected_source_time = UNIX_EPOCH
        + Duration::from_nanos(
            checkpoint["source_modified_nanos"]
                .as_u64()
                .ok_or("checkpoint source time")?,
        );
    if expected_source.as_slice() != br#"{"Version":restart-opposed"#
        || expected_source_time >= cache_checkpoint.modified
        || fs::read(&cache)? != cache_checkpoint.bytes
        || fs::metadata(&cache)?.modified()? != cache_checkpoint.modified
        || fs::read_to_string(fixture.join("home/.nms-copilot/config.toml"))?
            != checkpoint["config"].as_str().ok_or("checkpoint config")?
        || fs::read_to_string(fixture.join("mode.txt"))?
            != checkpoint["mode"].as_str().ok_or("checkpoint mode")?
        || fs::read_to_string(fixture.join("cache-path.txt"))?
            != checkpoint["cache_path"]
                .as_str()
                .ok_or("checkpoint cache path")?
    {
        return Err("prelaunch restart checkpoint differs from the literal opposed state".into());
    }
    let (session, _) = initialize_http(address)?;
    let mut sse = subscribe_sse(address, &session)?;
    let (tool, resource) = location_rpc_pair(address, &session)?;
    live_mcp::assert_location_pair_at(
        &tool,
        &resource,
        GalacticAddress::new(0, 0, 0, 3, 5, 0),
        "-",
        1,
        "500300000000",
        "07FF:007F:07FF:0003",
    )?;
    check_unknown_boundary(
        fixture,
        &expected_source,
        expected_source_time,
        &Some(cache_checkpoint.clone()),
        live_mcp::UnknownLifecycleStage::Promoted,
    )?;
    fs::write(
        fixture.join("restart.ready"),
        format!("address={address} session={session}\n"),
    )?;
    File::open(fixture.join("restart.ready"))?.sync_all()?;
    let control = fixture.join("restart.commands");
    let (line, offset) = wait_control_line(&control, 0, Duration::from_secs(60))?;
    if line != "pre-exit" {
        return Err(format!("restart monitor expected pre-exit, received {line:?}").into());
    }
    let (tool, resource) = location_rpc_pair(address, &session)?;
    live_mcp::assert_location_pair_at(
        &tool,
        &resource,
        GalacticAddress::new(0, 0, 0, 3, 5, 0),
        "-",
        1,
        "500300000000",
        "07FF:007F:07FF:0003",
    )?;
    check_unknown_boundary(
        fixture,
        &expected_source,
        expected_source_time,
        &Some(cache_checkpoint.clone()),
        live_mcp::UnknownLifecycleStage::Promoted,
    )?;
    println!("unknown-restart-pre-exit=pass");
    io::stdout().flush()?;
    let (line, _) = wait_control_line(&control, offset, Duration::from_secs(60))?;
    if line != "finish" {
        return Err(format!("restart monitor expected finish, received {line:?}").into());
    }
    sse.wait_closed_without_resource_updated(Duration::from_secs(8), "nms://player/location")?;
    sse.close_and_join()?;
    thread::sleep(Duration::from_millis(650));
    check_unknown_boundary(
        fixture,
        &expected_source,
        expected_source_time,
        &Some(cache_checkpoint),
        live_mcp::UnknownLifecycleStage::Promoted,
    )?;
    println!("unknown-restart=pass warm-source-bypassed+cache-stable+eof");
    Ok(())
}

fn run_monitor(fixture: &Path, address: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let initial = load_initial_fixture(fixture)?;
    let source = fixture.join("save.json");
    let cache = fixture_cache_path(fixture)?;
    let control = fixture.join("monitor.commands");
    let mode_lines = fs::read_to_string(fixture.join("mode.txt"))?;
    let mode = mode_lines
        .lines()
        .next()
        .ok_or("fixture mode missing")?
        .to_owned();
    let watch_enabled = mode_lines.lines().nth(1) != Some("disabled");
    let mut expected_source = initial.source;
    let mut expected_source_mtime = initial.source_modified;
    let initial_cache = initial.cache.clone();
    let mut checkpoint = initial.cache;
    let mut semantic = initial.semantic;
    let mut last_good_location = if mode == "warm" {
        LiveLocation {
            physical: 0,
            system: "Origin-0",
            planets: 1,
        }
    } else if matches!(
        mode.as_str(),
        "cold" | "nocache" | "bypass-missing" | "sentinel" | "blocker" | "predecessor"
    ) {
        LiveLocation {
            physical: 0,
            system: "(unknown)",
            planets: 0,
        }
    } else {
        LiveLocation {
            physical: 0,
            system: "-",
            planets: 0,
        }
    };

    let (session, _tools) = initialize_http(address)?;
    let mut sse = subscribe_sse(address, &session)?;
    let (tool, resource) = location_rpc_pair(address, &session)?;
    match mode.as_str() {
        "warm" => {
            assert_location_pair(&tool, &resource, 0, "Origin-0", 1)?;
        }
        "cold" | "nocache" | "bypass-missing" | "sentinel" | "blocker" | "predecessor" => {
            assert_location_pair(&tool, &resource, 0, "(unknown)", 0)?;
        }
        other => return Err(format!("unsupported monitor fixture mode {other}").into()),
    }
    if mode == "cold" || mode == "predecessor" {
        // Cold and incompatible predecessor startup must produce a literal current empty cache.
        if mode == "predecessor" {
            let old = initial_cache
                .as_ref()
                .ok_or("predecessor cache checkpoint missing")?;
            if old.bytes == fs::read(&cache)? {
                return Err("predecessor startup did not replace old cache bytes".into());
            }
        }
        assert_semantic_cache(&cache, CacheExpectation::ColdStartup)?;
        semantic = CacheExpectation::ColdStartup;
        checkpoint = cache_stamp(&cache)?;
    }
    check_fixture_files(
        fixture,
        &expected_source,
        expected_source_mtime,
        &checkpoint,
        semantic,
    )?;
    fs::write(
        fixture.join("monitor.ready"),
        format!("address={address} session={session}\n"),
    )?;
    File::open(fixture.join("monitor.ready"))?.sync_all()?;
    let mut offset = 0;

    loop {
        let (line, next_offset) = wait_control_line(&control, offset, Duration::from_secs(60))?;
        offset = next_offset;
        // Every command begins by proving the prior complete state, so no later
        // checkpoint can absorb an unobserved mutation.
        check_fixture_files(
            fixture,
            &expected_source,
            expected_source_mtime,
            &checkpoint,
            semantic,
        )?;
        let mut words = line.split_whitespace();
        match words.next().unwrap_or("") {
            "phase" => {
                let phase: u8 = words.next().ok_or("phase number required")?.parse()?;
                let state = phase_state(phase)?;
                let (version, physical, units, nanites, specials, previous, selected) = state;
                let previous_save = previous.map(|(r, x, y, z, system, planet)| {
                    (r, i64::from(x), i64::from(y), i64::from(z), system, planet)
                });
                let bytes = save_bytes(version, physical, units, nanites, specials, previous_save);
                let cursor = sse.cursor();
                expected_source_mtime = atomic_replace(&source, &bytes)?;
                expected_source = bytes;
                if watch_enabled {
                    sse.wait_for_resource_updated_since(
                        cursor,
                        "nms://player/location",
                        Duration::from_secs(8),
                    )?;
                } else {
                    sse.wait_quiet_since(cursor, Duration::from_millis(650))?;
                }
                let (tool, resource) = location_rpc_pair(address, &session)?;
                let should_apply = mode == "warm" && watch_enabled;
                let displayed = if should_apply { physical } else { 0 };
                let system = if !should_apply {
                    "Origin-0"
                } else if physical == 9 {
                    "Imported-9"
                } else {
                    "Origin-0"
                };
                assert_location_pair(&tool, &resource, displayed, system, 1)?;
                if should_apply {
                    let next_semantic = CacheExpectation::Warm(state);
                    assert_semantic_cache(&cache, next_semantic).map_err(|error| {
                        format!("phase {phase} complete raw+rebuilt cache oracle: {error}")
                    })?;
                    semantic = next_semantic;
                    last_good_location = LiveLocation {
                        physical,
                        system: if physical == 9 {
                            "Imported-9"
                        } else {
                            "Origin-0"
                        },
                        planets: 1,
                    };
                    checkpoint = cache_stamp(&cache)?;
                }
                check_fixture_files(
                    fixture,
                    &expected_source,
                    expected_source_mtime,
                    &checkpoint,
                    semantic,
                )?;
                println!(
                    "phase-{phase}=pass event=http-tool+resource+whole-cache version={version} selected={selected}"
                );
                io::stdout().flush()?;
            }
            "single-cross" => {
                let state: PhaseState = (4721, 9, 700, 800, 900, None, 9);
                let bytes = save_bytes(4721, 9, 700, 800, 900, None);
                let cursor = sse.cursor();
                expected_source_mtime = atomic_replace(&source, &bytes)?;
                expected_source = bytes;
                sse.wait_for_resource_updated_since(
                    cursor,
                    "nms://player/location",
                    Duration::from_secs(8),
                )?;
                let (tool, resource) = location_rpc_pair(address, &session)?;
                let (system, planets) = if mode == "warm" {
                    ("Imported-9", 1)
                } else {
                    ("(unknown)", 0)
                };
                assert_location_pair(&tool, &resource, 9, system, planets)?;
                let next_semantic = match mode.as_str() {
                    "warm" => CacheExpectation::Warm(state),
                    "cold" => CacheExpectation::ColdCross,
                    _ => CacheExpectation::Unpersisted,
                };
                if mode == "warm" || mode == "cold" {
                    assert_semantic_cache(&cache, next_semantic)?;
                    semantic = next_semantic;
                    checkpoint = cache_stamp(&cache)?;
                }
                last_good_location = LiveLocation {
                    physical: 9,
                    system,
                    planets,
                };
                check_fixture_files(
                    fixture,
                    &expected_source,
                    expected_source_mtime,
                    &checkpoint,
                    semantic,
                )?;
                println!("single-cross=pass tool+resource+sse version=4721 selected=9");
                io::stdout().flush()?;
            }
            "disabled-cross" => {
                if watch_enabled || mode != "warm" {
                    return Err("disabled-cross requires a warm cache and disabled watcher".into());
                }
                let bytes = save_bytes(4721, 9, 700, 800, 900, None);
                let cursor = sse.cursor();
                expected_source_mtime = atomic_replace(&source, &bytes)?;
                expected_source = bytes;
                sse.wait_quiet_since(cursor, Duration::from_millis(650))?;
                let (tool, resource) = location_rpc_pair(address, &session)?;
                assert_location_pair(&tool, &resource, 0, "Origin-0", 1)?;
                check_fixture_files(
                    fixture,
                    &expected_source,
                    expected_source_mtime,
                    &checkpoint,
                    semantic,
                )?;
                println!("disabled-cross=pass quiet+physical-location+cache-unchanged");
                io::stdout().flush()?;
            }
            "malformed" => {
                let bad = br#"{"Version":broken"#;
                check_location_rpc(address, &session, last_good_location)?;
                let (tool_before, resource_before) = location_rpc_pair(address, &session)?;
                let cursor = sse.cursor();
                expected_source_mtime = atomic_replace(&source, bad)?;
                expected_source = bad.to_vec();
                sse.wait_quiet_since(cursor, Duration::from_millis(650))?;
                let (tool_after, resource_after) = location_rpc_pair(address, &session)?;
                if tool_after != tool_before || resource_after != resource_before {
                    return Err("malformed source changed full served location values".into());
                }
                assert_location_pair(
                    &tool_after,
                    &resource_after,
                    last_good_location.physical,
                    last_good_location.system,
                    last_good_location.planets,
                )?;
                check_fixture_files(
                    fixture,
                    &expected_source,
                    expected_source_mtime,
                    &checkpoint,
                    semantic,
                )?;
                println!(
                    "malformed=pass cache+location retained; source-bytes={}",
                    bad.len()
                );
                io::stdout().flush()?;
            }
            "identical" => {
                let identical = expected_source.clone();
                let good_stamp = checkpoint
                    .as_ref()
                    .ok_or("identical requires a cache checkpoint")?;
                let fixed_cache_mtime = UNIX_EPOCH + Duration::from_secs(1_700_000_777);
                File::options()
                    .write(true)
                    .open(&cache)?
                    .set_times(FileTimes::new().set_modified(fixed_cache_mtime))?;
                checkpoint = Some(CacheStamp {
                    bytes: good_stamp.bytes.clone(),
                    modified: fixed_cache_mtime,
                });
                assert_semantic_cache(&cache, semantic)?;
                check_fixture_files(
                    fixture,
                    &expected_source,
                    expected_source_mtime,
                    &checkpoint,
                    semantic,
                )?;
                check_location_rpc(address, &session, last_good_location)?;
                let (tool_before, resource_before) = location_rpc_pair(address, &session)?;
                let cursor = sse.cursor();
                expected_source_mtime = atomic_replace(&source, &identical)?;
                expected_source = identical;
                sse.wait_quiet_since(cursor, Duration::from_millis(650))?;
                let (tool_after, resource_after) = location_rpc_pair(address, &session)?;
                if tool_after != tool_before || resource_after != resource_before {
                    return Err(
                        "identical source rewrite changed full served location values".into(),
                    );
                }
                assert_location_pair(
                    &tool_after,
                    &resource_after,
                    last_good_location.physical,
                    last_good_location.system,
                    last_good_location.planets,
                )?;
                check_fixture_files(
                    fixture,
                    &expected_source,
                    expected_source_mtime,
                    &checkpoint,
                    semantic,
                )?;
                println!(
                    "identical-rewrite=pass source/cache bytes and fixed cache mtime retained"
                );
                io::stdout().flush()?;
            }
            "quiet" => {
                let duration: u64 = words.next().unwrap_or("650").parse()?;
                let cursor = sse.cursor();
                sse.wait_quiet_since(cursor, Duration::from_millis(duration))?;
                check_location_rpc(address, &session, last_good_location)?;
                check_fixture_files(
                    fixture,
                    &expected_source,
                    expected_source_mtime,
                    &checkpoint,
                    semantic,
                )?;
                println!("quiet-window=pass ms={duration}");
                io::stdout().flush()?;
            }
            "pre-exit" => {
                check_location_rpc(address, &session, last_good_location)?;
                check_fixture_files(
                    fixture,
                    &expected_source,
                    expected_source_mtime,
                    &checkpoint,
                    semantic,
                )?;
                println!("pre-exit-boundary=pass");
                io::stdout().flush()?;
            }
            "close" => {
                sse.wait_closed(Duration::from_secs(8))?;
                sse.close_and_join()?;
                check_fixture_files(
                    fixture,
                    &expected_source,
                    expected_source_mtime,
                    &checkpoint,
                    semantic,
                )?;
                if mode == "predecessor" {
                    thread::sleep(Duration::from_millis(650));
                    check_fixture_files(
                        fixture,
                        &expected_source,
                        expected_source_mtime,
                        &checkpoint,
                        semantic,
                    )?;
                }
                let model = std::sync::Arc::new(tokio::sync::RwLock::new(live_mcp::golden_model()));
                let owner = mcp::spawn_mcp_background_with_updates(model, address, None)?;
                owner.shutdown()?;
                println!(
                    "remote-sse-close=pass joined-reader+owned-server-address-reuse=pass address={address}"
                );
                return Ok(());
            }
            command => return Err(format!("unknown monitor command {command:?}").into()),
        }
    }
}
#[derive(Clone)]
struct CacheStamp {
    bytes: Vec<u8>,
    modified: std::time::SystemTime,
}

#[derive(Clone, Copy)]
enum CacheExpectation {
    Warm(PhaseState),
    ColdStartup,
    ColdCross,
    Unpersisted,
    PredecessorSemantic3,
}

struct InitialFixture {
    source: Vec<u8>,
    source_modified: std::time::SystemTime,
    cache: Option<CacheStamp>,
    semantic: CacheExpectation,
}

#[derive(Clone, Copy)]
struct LiveLocation {
    physical: u8,
    system: &'static str,
    planets: usize,
}

fn check_location_rpc(
    address: SocketAddr,
    session: &str,
    expected: LiveLocation,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tool, resource) = location_rpc_pair(address, session)?;
    assert_location_pair(
        &tool,
        &resource,
        expected.physical,
        expected.system,
        expected.planets,
    )?;
    Ok(())
}

fn load_initial_fixture(fixture: &Path) -> Result<InitialFixture, Box<dyn std::error::Error>> {
    let initial: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.join("initial-state.json"))?)?;
    let source = decode_hex(
        initial["source_hex"]
            .as_str()
            .ok_or("initial source missing")?,
    )?;
    let expected_source = save_bytes(4720, 0, 7, 8, 9, None);
    let source_modified = UNIX_EPOCH
        + Duration::from_nanos(
            initial["source_modified_nanos"]
                .as_u64()
                .ok_or("initial source time missing")?,
        );
    if source != expected_source
        || source_modified != UNIX_EPOCH + Duration::from_secs(1_700_000_000)
    {
        return Err("prepared source bytes or fixed mtime differ from literal initial save".into());
    }
    let cache = match &initial["cache"] {
        serde_json::Value::Null => None,
        value => Some(CacheStamp {
            bytes: decode_hex(
                value["bytes_hex"]
                    .as_str()
                    .ok_or("initial cache bytes missing")?,
            )?,
            modified: UNIX_EPOCH
                + Duration::from_nanos(
                    value["modified_nanos"]
                        .as_u64()
                        .ok_or("initial cache time missing")?,
                ),
        }),
    };
    let mode = fs::read_to_string(fixture.join("mode.txt"))?
        .lines()
        .next()
        .ok_or("fixture mode missing")?
        .to_owned();
    let semantic = match mode.as_str() {
        "warm" => CacheExpectation::Warm((4720, 0, 7, 8, 9, None, 7)),
        "cold" => CacheExpectation::ColdStartup,
        "predecessor" => {
            let expected =
                include_bytes!("../../nms-cache/tests/fixtures/predecessor-schema3-semantic3.rkyv");
            if source != save_bytes(4720, 0, 7, 8, 9, None)
                || source_modified != UNIX_EPOCH + Duration::from_secs(1_700_000_000)
                || cache.as_ref().map(|stamp| stamp.bytes.as_slice()) != Some(expected.as_slice())
                || cache.as_ref().map(|stamp| stamp.modified)
                    != Some(UNIX_EPOCH + Duration::from_secs(1_700_000_100))
                || expected.len() != 752
                || &expected[..16] != b"NMSCACHE\x03\0\0\0\x03\0\0\0"
            {
                return Err("prepared predecessor source/cache bytes or fixed times differ".into());
            }
            CacheExpectation::PredecessorSemantic3
        }
        "sentinel" => {
            if cache
                .as_ref()
                .map(|stamp| (stamp.bytes.as_slice(), stamp.modified))
                != Some((
                    b"slice19-no-cache-sentinel\n".as_slice(),
                    UNIX_EPOCH + Duration::from_secs(1_700_000_077),
                ))
            {
                return Err("prepared sentinel bytes/mtime differ".into());
            }
            CacheExpectation::Unpersisted
        }
        "blocker" | "bypass-missing" | "nocache" => {
            if cache.is_some() {
                return Err("prepared unpersisted mode unexpectedly has a cache file".into());
            }
            CacheExpectation::Unpersisted
        }
        other => return Err(format!("unsupported fixture mode {other}").into()),
    };
    if matches!(semantic, CacheExpectation::Warm(_))
        && cache.as_ref().map(|stamp| stamp.modified)
            != Some(UNIX_EPOCH + Duration::from_secs(1_700_000_100))
    {
        return Err("prepared warm-cache mtime differs from literal fixed time".into());
    }
    Ok(InitialFixture {
        source,
        source_modified,
        cache,
        semantic,
    })
}

fn assert_semantic_cache(
    path: &Path,
    expected: CacheExpectation,
) -> Result<(), Box<dyn std::error::Error>> {
    match expected {
        CacheExpectation::Warm((
            version,
            physical,
            units,
            nanites,
            specials,
            previous,
            selected,
        )) => {
            let previous = previous.map(|(reality, x, y, z, system, planet)| {
                GalacticAddress::new(x, y, z, system, planet, reality)
            });
            live_mcp::assert_cache_oracle(
                path,
                &expected_cache(
                    version, physical, units, nanites, specials, previous, selected,
                ),
                &live_mcp::expected_model(physical, units, nanites, specials, previous, selected),
            )?;
        }
        CacheExpectation::ColdStartup => live_mcp::assert_cache_oracle(
            path,
            &live_mcp::expected_empty_cache(4720, 0, 7, 8, 9, 0),
            &live_mcp::expected_empty_model(0, 7, 8, 9, 0),
        )?,
        CacheExpectation::ColdCross => live_mcp::assert_cache_oracle(
            path,
            &live_mcp::expected_cold_cross_cache(),
            &live_mcp::expected_cold_cross_model(),
        )?,
        CacheExpectation::Unpersisted => {}
        CacheExpectation::PredecessorSemantic3 => {
            if !matches!(
                nms_cache::read_cache(path),
                Err(nms_cache::CacheError::Incompatible(
                    nms_cache::CacheCompatibilityError::SchemaMismatch {
                        found: 3,
                        expected: 4
                    }
                ))
            ) {
                return Err("schema3 predecessor did not fail schema compatibility first".into());
            }
        }
    }
    Ok(())
}

fn cache_stamp(path: &Path) -> Result<Option<CacheStamp>, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(CacheStamp {
        bytes: fs::read(path)?,
        modified: fs::metadata(path)?.modified()?,
    }))
}

/// Shared whole-fixture boundary checker used at readiness, every mutation,
/// negative windows, and final EOF. Semantic cache expectations are checked
/// separately through the combined raw-data/rebuilt-model oracle.
fn check_fixture_files(
    fixture: &Path,
    expected_source: &[u8],
    expected_source_mtime: std::time::SystemTime,
    expected_cache: &Option<CacheStamp>,
    semantic: CacheExpectation,
) -> Result<(), Box<dyn std::error::Error>> {
    check_fixture_files_internal(
        fixture,
        expected_source,
        expected_source_mtime,
        expected_cache,
        semantic,
        true,
    )
}

fn check_fixture_files_before_startup(
    fixture: &Path,
    expected_source: &[u8],
    expected_source_mtime: std::time::SystemTime,
    expected_cache: &Option<CacheStamp>,
    semantic: CacheExpectation,
) -> Result<(), Box<dyn std::error::Error>> {
    check_fixture_files_internal(
        fixture,
        expected_source,
        expected_source_mtime,
        expected_cache,
        semantic,
        false,
    )
}

fn check_fixture_files_internal(
    fixture: &Path,
    expected_source: &[u8],
    expected_source_mtime: std::time::SystemTime,
    expected_cache: &Option<CacheStamp>,
    semantic: CacheExpectation,
    ready: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let marker = fixture.join(MARKER);
    if !marker.is_file() || fs::read(&marker)? != b"Project01 Arc06 Slice19 fixture\n" {
        return Err("owned fixture marker differs".into());
    }
    let mode_lines = fs::read_to_string(fixture.join("mode.txt"))?;
    let mut lines = mode_lines.lines();
    let mode = lines.next().ok_or("fixture mode missing")?;
    let watch = lines.next().ok_or("fixture watch mode missing")?;
    let cache = fixture_cache_path(fixture)?;
    let manifest = fs::read_to_string(fixture.join("cache-path.txt"))?;
    if PathBuf::from(manifest.trim()).is_absolute()
        || fixture.join(manifest.trim()) != cache
        || !fs::read_to_string(fixture.join("save.json"))?
            .as_bytes()
            .eq(expected_source)
        || fs::metadata(fixture.join("save.json"))?.modified()? != expected_source_mtime
        || cache.with_extension("rkyv.tmp").exists()
    {
        return Err("complete source/cache-path/temp boundary differs".into());
    }
    let config = fs::read_to_string(fixture.join("home/.nms-copilot/config.toml"))?;
    let expected_cache_enabled = mode != "nocache";
    let expected_watch_enabled = watch != "disabled";
    let expected_config = format!(
        "[display]\nshow_banner = false\nshow_system_banner = false\ncolor = false\n\n[watch]\nenabled = {expected_watch_enabled}\ndebounce_ms = 137\n\n[cache]\nenabled = {expected_cache_enabled}\npath = {:?}\n",
        cache.to_string_lossy()
    );
    if config != expected_config {
        return Err("literal config mode/cache/watch/path differs".into());
    }
    let actual_cache = cache_stamp(&cache)?;
    match (expected_cache, actual_cache) {
        (Some(expected), Some(actual))
            if expected.bytes == actual.bytes && expected.modified == actual.modified => {}
        (None, None) => {}
        _ => return Err("complete cache bytes/mtime checkpoint differs".into()),
    }
    match mode {
        "predecessor"
            if cache.parent() != Some(fixture)
                || !fixture.is_dir()
                || fixture.join("cache-parent").exists() =>
        {
            return Err(
                "predecessor cache parent/path differs from the literal existing fixture directory"
                    .into(),
            );
        }
        "bypass-missing" if cache.parent().is_some_and(Path::exists) || cache.exists() => {
            return Err("bypass-missing parent/cache path exists".into());
        }
        "blocker" => {
            let parent = fixture.join("cache-parent");
            if !parent.is_file()
                || fs::read(&parent)? != b"slice19-cache-write-blocker\n"
                || fs::metadata(&parent)?.modified()?
                    != UNIX_EPOCH + Duration::from_secs(1_700_000_077)
                || cache.exists()
            {
                return Err("write-error blocker/parent/cache state differs".into());
            }
        }
        "sentinel" if fs::read(&cache)? != b"slice19-no-cache-sentinel\n" => {
            return Err("no-cache sentinel bytes changed".into());
        }
        "cold" if ready && !cache.is_file() => {
            return Err("cold cache file is absent".into());
        }
        "cold" if !ready && cache.parent().is_some_and(Path::exists) => {
            return Err("cold cache parent exists before startup".into());
        }
        _ => {}
    }
    if ready
        || matches!(
            semantic,
            CacheExpectation::Warm(_) | CacheExpectation::PredecessorSemantic3
        )
    {
        assert_semantic_cache(&cache, semantic)?;
    }
    Ok(())
}

fn wait_control_line(
    path: &Path,
    offset: usize,
    timeout: Duration,
) -> Result<(String, usize), Box<dyn std::error::Error>> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        let bytes = fs::read(path)?;
        if let Some(relative_end) = bytes
            .get(offset..)
            .and_then(|tail| tail.iter().position(|b| *b == b'\n'))
        {
            let end = offset + relative_end + 1;
            let line = std::str::from_utf8(&bytes[offset..end])?.trim().to_owned();
            return Ok((line, end));
        }
        if std::time::Instant::now() >= deadline {
            return Err(format!("control command deadline elapsed for {}", path.display()).into());
        }
        thread::sleep(Duration::from_millis(15));
    }
}

fn phase_state(phase: u8) -> Result<PhaseState, String> {
    match phase {
        1 => Ok((4720, 0, 70, 80, 90, None, 7)),
        2 => Ok((4720, 0, 70, 80, 90, Some((9, 3, 3, 4, 1, 1)), 7)),
        3 => Ok((4721, 0, 70, 80, 90, Some((9, 3, 3, 4, 1, 1)), 7)),
        4 => Ok((4722, 9, 700, 800, 900, Some((0, 2, 3, 4, 1, 1)), 9)),
        5 => Ok((4723, 9, 701, 801, 901, Some((0, 2, 3, 4, 1, 1)), 9)),
        6 => Ok((4724, 0, 900, 901, 902, Some((9, 0, 3, 4, 1, 1)), 0)),
        _ => Err(format!("unknown live phase {phase}")),
    }
}
fn reject(
    label: &str,
    actual: &CacheData,
    expected: &CacheData,
    expected_model: &nms_graph::GalaxyModel,
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("observed-mutant.rkyv");
    nms_cache::write_cache(actual, &path)?;
    let observed_raw = nms_cache::read_cache(&path)?;
    let observed_rebuilt = nms_cache::rebuild_model(&observed_raw);
    if live_mcp::compare_cache_model(&observed_raw, &observed_rebuilt, expected, expected_model)
        .is_ok()
    {
        return Err(format!("same raw+rebuilt validator accepted {label} mismatch").into());
    }
    Ok(())
}

fn reject_model(
    label: &str,
    actual: &nms_graph::GalaxyModel,
    expected_data: &CacheData,
    expected_model: &nms_graph::GalaxyModel,
) -> Result<(), Box<dyn std::error::Error>> {
    if live_mcp::compare_cache_model(expected_data, actual, expected_data, expected_model).is_ok() {
        return Err(format!("same raw+rebuilt validator accepted {label} mismatch").into());
    }
    Ok(())
}
