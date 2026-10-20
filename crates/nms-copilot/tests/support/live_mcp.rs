use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use nms_cache::CacheData;
use nms_cache::data::CachedSystem;
use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::player::{BaseType, PlayerBase, PlayerState};
use nms_core::system::{Planet, System};
use nms_graph::GalaxyModel;
use serde_json::Value;

pub const HTTP_TIMEOUT: Duration = Duration::from_secs(5);

pub fn golden_model() -> GalaxyModel {
    let mut model = GalaxyModel::new();
    for (reality, name, biome, planet_name) in [
        (0, "Origin-0", Biome::Lush, "Leaf-0"),
        (9, "Imported-9", Biome::Toxic, "Leaf-9"),
    ] {
        let address = GalacticAddress::new(0, 0, 0, 1, 0, reality);
        model.insert_system(System::new(
            address,
            Some(name.to_owned()),
            None,
            None,
            vec![Planet::new(
                0,
                Some(biome),
                None,
                false,
                Some(planet_name.to_owned()),
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

pub fn expected_cache(
    version: u32,
    physical_reality: u8,
    units: u64,
    nanites: u64,
    specials: u64,
    previous: Option<GalacticAddress>,
    selected: u8,
) -> CacheData {
    let systems = vec![
        CachedSystem {
            address: GalacticAddress::new(0, 0, 0, 1, 0, 0),
            name: Some("Origin-0".into()),
            discoverer: None,
            timestamp_secs: None,
            planets: vec![Planet::new(
                0,
                Some(Biome::Lush),
                None,
                false,
                Some("Leaf-0".into()),
                None,
            )],
        },
        CachedSystem {
            address: GalacticAddress::new(0, 0, 0, 1, 0, 9),
            name: Some("Imported-9".into()),
            discoverer: None,
            timestamp_secs: None,
            planets: vec![Planet::new(
                0,
                Some(Biome::Toxic),
                None,
                false,
                Some("Leaf-9".into()),
                None,
            )],
        },
    ];
    let bases = [0, 9]
        .into_iter()
        .map(|reality| {
            PlayerBase::new(
                format!("Base-{reality}"),
                BaseType::HomePlanetBase,
                GalacticAddress::new(0, 0, 0, 1, 0, reality),
                [0.0; 3],
                Some("fixture-owner".into()),
            )
        })
        .collect();
    CacheData {
        systems,
        bases,
        player_state: Some(PlayerState::new(
            GalacticAddress::new(0, 0, 0, 1, 0, physical_reality),
            physical_reality,
            previous,
            None,
            units,
            nanites,
            specials,
        )),
        active_galaxy: selected,
        save_version: version,
        cached_at: 0,
    }
}

pub fn expected_model(
    physical_reality: u8,
    units: u64,
    nanites: u64,
    specials: u64,
    previous: Option<GalacticAddress>,
    selected: u8,
) -> GalaxyModel {
    let mut model = GalaxyModel::new();
    for (reality, name, biome, planet_name) in [
        (0, "Origin-0", Biome::Lush, "Leaf-0"),
        (9, "Imported-9", Biome::Toxic, "Leaf-9"),
    ] {
        let address = GalacticAddress::new(0, 0, 0, 1, 0, reality);
        model.insert_system(System::new(
            address,
            Some(name.to_owned()),
            None,
            None,
            vec![Planet::new(
                0,
                Some(biome),
                None,
                false,
                Some(planet_name.to_owned()),
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
        GalacticAddress::new(0, 0, 0, 1, 0, physical_reality),
        physical_reality,
        previous,
        None,
        units,
        nanites,
        specials,
    ));
    model.set_active_galaxy(selected);
    model
}

pub fn expected_empty_cache(
    version: u32,
    physical_reality: u8,
    units: u64,
    nanites: u64,
    specials: u64,
    selected: u8,
) -> CacheData {
    CacheData {
        systems: vec![],
        bases: vec![],
        player_state: Some(PlayerState::new(
            GalacticAddress::new(0, 0, 0, 1, 0, physical_reality),
            physical_reality,
            None,
            None,
            units,
            nanites,
            specials,
        )),
        active_galaxy: selected,
        save_version: version,
        cached_at: 0,
    }
}

pub fn expected_empty_model(
    physical_reality: u8,
    units: u64,
    nanites: u64,
    specials: u64,
    selected: u8,
) -> GalaxyModel {
    let mut model = GalaxyModel::new();
    model.player_state = Some(PlayerState::new(
        GalacticAddress::new(0, 0, 0, 1, 0, physical_reality),
        physical_reality,
        None,
        None,
        units,
        nanites,
        specials,
    ));
    model.set_active_galaxy(selected);
    model
}

pub fn expected_cold_cross_cache() -> CacheData {
    CacheData {
        systems: vec![CachedSystem {
            address: GalacticAddress::new(0, 0, 0, 1, 0, 0),
            name: None,
            discoverer: None,
            timestamp_secs: None,
            planets: vec![],
        }],
        bases: vec![],
        player_state: Some(PlayerState::new(
            GalacticAddress::new(0, 0, 0, 1, 0, 9),
            9,
            None,
            None,
            700,
            800,
            900,
        )),
        active_galaxy: 9,
        save_version: 4721,
        cached_at: 0,
    }
}

pub fn expected_cold_cross_model() -> GalaxyModel {
    let mut model = GalaxyModel::new();
    model.insert_system(System::new(
        GalacticAddress::new(0, 0, 0, 1, 0, 0),
        None,
        None,
        None,
        vec![],
    ));
    model.player_state = Some(PlayerState::new(
        GalacticAddress::new(0, 0, 0, 1, 0, 9),
        9,
        None,
        None,
        700,
        800,
        900,
    ));
    model.set_active_galaxy(9);
    model
}

/// Compare all persisted semantic data, excluding only the wall-clock timestamp.
pub fn assert_cache_oracle(
    path: &std::path::Path,
    expected_data: &CacheData,
    expected_model: &GalaxyModel,
) -> Result<(), String> {
    let actual = nms_cache::read_cache(path).map_err(|error| error.to_string())?;
    let rebuilt = nms_cache::rebuild_model(&actual);
    compare_cache_model(&actual, &rebuilt, expected_data, expected_model)
}

pub fn compare_cache_model(
    actual_data: &CacheData,
    actual_model: &GalaxyModel,
    expected_data: &CacheData,
    expected_model: &GalaxyModel,
) -> Result<(), String> {
    compare_cache(actual_data, expected_data)?;
    assert_model_matches(actual_model, expected_model)
}

pub fn compare_cache(actual: &CacheData, expected: &CacheData) -> Result<(), String> {
    let mut actual_systems = system_rows(&actual.systems);
    let mut expected_systems = system_rows(&expected.systems);
    actual_systems.sort();
    expected_systems.sort();
    if actual_systems != expected_systems {
        return Err("complete cached systems/planet vectors differ".into());
    }
    let mut actual_bases = base_rows(&actual.bases);
    let mut expected_bases = base_rows(&expected.bases);
    actual_bases.sort();
    expected_bases.sort();
    if actual_bases != expected_bases {
        return Err("complete cached bases differ".into());
    }
    if actual.player_state != expected.player_state {
        return Err("complete cached player state differs".into());
    }
    if actual.active_galaxy != expected.active_galaxy {
        return Err("selected galaxy differs".into());
    }
    if actual.save_version != expected.save_version {
        return Err("source version differs".into());
    }
    Ok(())
}

fn system_rows(systems: &[CachedSystem]) -> Vec<String> {
    systems
        .iter()
        .map(|system| {
            format!(
                "{:?}|{:?}|{:?}|{:?}|{:?}",
                system.address,
                system.name,
                system.discoverer,
                system.timestamp_secs,
                system.planets
            )
        })
        .collect()
}
fn base_rows(bases: &[PlayerBase]) -> Vec<String> {
    bases
        .iter()
        .map(|base| {
            format!(
                "{:?}|{:?}|{:?}|{:?}|{:?}",
                base.name, base.base_type, base.address, base.position, base.owner_uid
            )
        })
        .collect()
}

pub fn assert_model_matches(actual: &GalaxyModel, expected: &GalaxyModel) -> Result<(), String> {
    if actual.systems != expected.systems {
        return Err("whole systems map differs".into());
    }
    if actual.planets != expected.planets {
        return Err("whole flattened planets map differs".into());
    }
    if actual.bases != expected.bases {
        return Err("whole bases map differs".into());
    }
    if actual.name_index != expected.name_index {
        return Err("whole name index differs".into());
    }
    if actual.biome_index != expected.biome_index {
        return Err("whole biome index differs".into());
    }
    if actual.address_to_id != expected.address_to_id {
        return Err("whole address index differs".into());
    }
    let graph_ids = |model: &GalaxyModel| {
        model
            .node_map
            .keys()
            .map(|id| format!("{id:?}"))
            .collect::<BTreeSet<_>>()
    };
    let graph_weights = |model: &GalaxyModel| {
        let mut rows: Vec<_> = model
            .graph
            .node_weights()
            .map(|weight| format!("{weight:?}"))
            .collect();
        rows.sort();
        rows
    };
    let node_map_is_valid = |model: &GalaxyModel| {
        model
            .node_map
            .iter()
            .all(|(id, index)| model.graph.node_weight(*index) == Some(id))
    };
    if graph_ids(actual) != graph_ids(expected)
        || !node_map_is_valid(actual)
        || !node_map_is_valid(expected)
        || graph_weights(actual) != graph_weights(expected)
        || actual.graph.node_count() != expected.graph.node_count()
        || actual.graph.edge_count() != expected.graph.edge_count()
    {
        return Err("complete graph node IDs/weights or counts differ".into());
    }
    if actual.active_galaxy != expected.active_galaxy {
        return Err("selected galaxy differs".into());
    }
    if actual.player_state != expected.player_state {
        return Err("whole player state differs".into());
    }
    let actual_galaxies: BTreeSet<_> = actual.spatial.keys().copied().collect();
    let expected_galaxies: BTreeSet<_> = expected.spatial.keys().copied().collect();
    if actual_galaxies != expected_galaxies {
        return Err("exact spatial galaxy key set differs".into());
    }
    for galaxy in actual_galaxies {
        let mut found: Vec<_> = actual
            .spatial_for(galaxy)
            .into_iter()
            .flat_map(|tree| tree.iter())
            .map(|point| format!("{:?}|{:?}", point.id, point.point))
            .collect();
        let mut wanted: Vec<_> = expected
            .spatial_for(galaxy)
            .into_iter()
            .flat_map(|tree| tree.iter())
            .map(|point| format!("{:?}|{:?}", point.id, point.point))
            .collect();
        found.sort();
        wanted.sort();
        if found != wanted {
            return Err(format!("spatial results differ in galaxy {galaxy}"));
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}
impl HttpResponse {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
    }
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

pub fn http_request(
    addr: SocketAddr,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> Result<HttpResponse, String> {
    let mut stream =
        TcpStream::connect_timeout(&addr, HTTP_TIMEOUT).map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(HTTP_TIMEOUT))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(HTTP_TIMEOUT))
        .map_err(|error| error.to_string())?;
    let body = body.unwrap_or("");
    let mut request = format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n");
    for (name, value) in headers {
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    if !body.is_empty() {
        request.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    request.push_str("\r\n");
    request.push_str(body);
    stream
        .write_all(request.as_bytes())
        .map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    stream
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    parse_response(&bytes)
}

fn parse_response(bytes: &[u8]) -> Result<HttpResponse, String> {
    let split = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or("missing HTTP header boundary")?;
    let head = std::str::from_utf8(&bytes[..split]).map_err(|error| error.to_string())?;
    let mut lines = head.lines();
    let status = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or("missing status")?
        .parse()
        .map_err(|error| format!("bad status: {error}"))?;
    let mut headers = BTreeMap::<String, String>::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.to_ascii_lowercase(), value.trim().into());
        }
    }
    let body = if headers
        .get("transfer-encoding")
        .is_some_and(|value| value.eq_ignore_ascii_case("chunked"))
    {
        decode_chunked(&bytes[split + 4..])?
    } else {
        bytes[split + 4..].to_vec()
    };
    Ok(HttpResponse {
        status,
        headers,
        body,
    })
}

fn decode_chunked(mut body: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoded = Vec::new();
    loop {
        let line_end = body
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or("chunk size missing")?;
        let size = usize::from_str_radix(
            std::str::from_utf8(&body[..line_end])
                .map_err(|error| error.to_string())?
                .trim(),
            16,
        )
        .map_err(|error| error.to_string())?;
        body = &body[line_end + 2..];
        if size == 0 {
            return Ok(decoded);
        }
        if body.len() < size + 2 || &body[size..size + 2] != b"\r\n" {
            return Err("invalid HTTP chunk".into());
        }
        decoded.extend_from_slice(&body[..size]);
        body = &body[size + 2..];
    }
}

pub fn json_messages(body: &str) -> Result<Vec<Value>, String> {
    let mut values = Vec::new();
    for event in body.split("\n\n") {
        let data = event
            .lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .map(str::trim)
            .collect::<String>();
        if data.starts_with('{') {
            values.push(serde_json::from_str(&data).map_err(|error| error.to_string())?);
        }
    }
    Ok(values)
}

pub fn initialize_http(addr: SocketAddr) -> Result<(String, HttpResponse), String> {
    let init = http_request(
        addr,
        "POST",
        "/mcp",
        &[
            ("Content-Type", "application/json"),
            ("Accept", "application/json, text/event-stream"),
        ],
        Some(
            &serde_json::json!({
                "jsonrpc":"2.0", "id":1, "method":"initialize",
                "params":{"protocolVersion":"2025-03-26", "capabilities":{"resources":{"subscribe":true}},
                "clientInfo":{"name":"slice17-probe","version":"1"}}
            })
            .to_string(),
        ),
    )?;
    if init.status != 200 {
        return Err(format!(
            "initialize returned {}: {}",
            init.status,
            init.text()
        ));
    }
    let session = init
        .header("mcp-session-id")
        .ok_or("initialize omitted Mcp-Session-Id")?
        .to_owned();
    let initialized = rpc_http(addr, &session, "notifications/initialized", None, None)?;
    if initialized.status != 202 {
        return Err(format!(
            "initialized notification returned {}",
            initialized.status
        ));
    }
    let tools = rpc_http(
        addr,
        &session,
        "tools/list",
        Some(2),
        Some(serde_json::json!({})),
    )?;
    if tools.status != 200
        || !json_messages(&tools.text())?.iter().any(|message| {
            message["result"]["tools"]
                .as_array()
                .is_some_and(|tools| tools.iter().any(|tool| tool["name"] == "where_am_i"))
        })
    {
        return Err(format!(
            "tools/list did not expose where_am_i: {}",
            tools.text()
        ));
    }
    Ok((session, tools))
}

pub fn rpc_http(
    addr: SocketAddr,
    session: &str,
    method: &str,
    id: Option<u64>,
    params: Option<Value>,
) -> Result<HttpResponse, String> {
    let mut message = serde_json::json!({"jsonrpc":"2.0", "method":method});
    if let Some(id) = id {
        message["id"] = serde_json::json!(id);
    }
    if let Some(params) = params {
        message["params"] = params;
    }
    http_request(
        addr,
        "POST",
        "/mcp",
        &[
            ("Content-Type", "application/json"),
            ("Accept", "application/json, text/event-stream"),
            ("Mcp-Session-Id", session),
        ],
        Some(&message.to_string()),
    )
}

pub fn location_rpc(addr: SocketAddr, session: &str) -> Result<Value, String> {
    let (tool_value, resource_value) = location_rpc_pair(addr, session)?;
    if tool_value != resource_value {
        return Err("where_am_i tool and location resource differ".into());
    }
    Ok(tool_value)
}

pub fn location_rpc_pair(addr: SocketAddr, session: &str) -> Result<(Value, Value), String> {
    let tool = rpc_http(
        addr,
        session,
        "tools/call",
        Some(10),
        Some(serde_json::json!({"name":"where_am_i", "arguments":{}})),
    )?;
    if tool.status != 200 {
        return Err(format!(
            "tools/call status {}: {}",
            tool.status,
            tool.text()
        ));
    }
    let message = json_messages(&tool.text())?
        .into_iter()
        .find(|message| message["id"] == 10)
        .ok_or("tools/call response ID 10 missing")?;
    let text = message["result"]["content"][0]["text"]
        .as_str()
        .ok_or("where_am_i text content missing")?;
    let tool_value: Value = serde_json::from_str(text).map_err(|error| error.to_string())?;
    let resource = rpc_http(
        addr,
        session,
        "resources/read",
        Some(11),
        Some(serde_json::json!({"uri":"nms://player/location"})),
    )?;
    if resource.status != 200 {
        return Err(format!(
            "resources/read status {}: {}",
            resource.status,
            resource.text()
        ));
    }
    let message = json_messages(&resource.text())?
        .into_iter()
        .find(|message| message["id"] == 11)
        .ok_or("resources/read response ID 11 missing")?;
    let text = message["result"]["contents"][0]["text"]
        .as_str()
        .ok_or("location resource text missing")?;
    let resource_value: Value = serde_json::from_str(text).map_err(|error| error.to_string())?;
    Ok((tool_value, resource_value))
}

pub fn assert_location(
    actual: &Value,
    physical: u8,
    system: &str,
    planets: usize,
) -> Result<(), String> {
    let address = GalacticAddress::new(0, 0, 0, 1, 0, physical);
    // Fixture oracle is deliberately independent of address packing output.
    let hex = "000100000000";
    let expected = serde_json::json!({
        "system": system,
        "planets_in_system": planets,
        "galaxy": if physical == 9 { "Eissentam" } else { "Euclid" },
        "voxel_x": 0,
        "voxel_y": 0,
        "voxel_z": 0,
        "solar_system_index": 1,
        "portal_glyphs_hex": hex,
        "portal_glyphs_emoji": nms_query::display::hex_to_emoji(hex),
        "signal_booster": address.to_signal_booster(),
    });
    if actual != &expected {
        return Err(format!(
            "full location mismatch; expected {expected}, observed {actual}"
        ));
    }
    Ok(())
}

pub fn assert_location_pair(
    tool: &Value,
    resource: &Value,
    physical: u8,
    system: &str,
    planets: usize,
) -> Result<(), String> {
    if tool != resource {
        return Err("where_am_i tool and location resource differ".into());
    }
    assert_location(tool, physical, system, planets)?;
    assert_location(resource, physical, system, planets)
}

pub fn subscribe_sse(addr: SocketAddr, session: &str) -> Result<SseStream, String> {
    let response = rpc_http(
        addr,
        session,
        "resources/subscribe",
        Some(3),
        Some(serde_json::json!({"uri":"nms://player/location"})),
    )?;
    if response.status != 200 {
        return Err(format!(
            "subscribe status {}: {}",
            response.status,
            response.text()
        ));
    }
    Ok(SseStream::connect(addr, session))
}

enum SseRead {
    Data(Vec<u8>),
    PeerClosed(String),
    Failed(String),
}

pub struct SseStream {
    close_socket: TcpStream,
    events: mpsc::Receiver<SseRead>,
    reader: Option<JoinHandle<()>>,
    buffer: Vec<u8>,
    consumed: usize,
}

impl SseStream {
    fn connect(addr: SocketAddr, session: &str) -> Self {
        let mut stream =
            TcpStream::connect_timeout(&addr, HTTP_TIMEOUT).expect("connect SSE stream");
        stream
            .set_read_timeout(Some(HTTP_TIMEOUT))
            .expect("set SSE read timeout");
        stream
            .set_write_timeout(Some(HTTP_TIMEOUT))
            .expect("set SSE write timeout");
        write!(stream, "GET /mcp HTTP/1.1\r\nHost: {addr}\r\nConnection: keep-alive\r\nAccept: text/event-stream\r\nMcp-Session-Id: {session}\r\n\r\n").expect("write SSE request");
        let mut headers = Vec::new();
        let mut byte = [0u8; 1];
        while !headers.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte).expect("read SSE headers");
            headers.push(byte[0]);
        }
        stream
            .set_read_timeout(None)
            .expect("clear SSE body timeout after bounded handshake");
        let head = String::from_utf8_lossy(&headers);
        assert!(head.contains("200"), "SSE response headers: {head}");
        let chunked = head
            .to_ascii_lowercase()
            .contains("transfer-encoding: chunked");
        let close_socket = stream.try_clone().expect("clone SSE close socket");
        let (sender, events) = mpsc::channel();
        let reader = thread::spawn(move || {
            let mut stream = BufReader::new(stream);
            loop {
                let next = if chunked {
                    read_sse_chunk(&mut stream)
                } else {
                    let mut bytes = [0u8; 4096];
                    match stream.read(&mut bytes) {
                        Ok(0) => Ok(None),
                        Ok(count) => Ok(Some(bytes[..count].to_vec())),
                        Err(error) => Err(error),
                    }
                };
                match next {
                    Ok(Some(bytes)) => {
                        if sender.send(SseRead::Data(bytes)).is_err() {
                            return;
                        }
                    }
                    Ok(None) => {
                        let _ = sender.send(SseRead::PeerClosed("EOF".into()));
                        return;
                    }
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::UnexpectedEof
                                | std::io::ErrorKind::ConnectionReset
                                | std::io::ErrorKind::ConnectionAborted
                        ) =>
                    {
                        let _ = sender.send(SseRead::PeerClosed(error.to_string()));
                        return;
                    }
                    Err(error) => {
                        let _ = sender.send(SseRead::Failed(error.to_string()));
                        return;
                    }
                }
            }
        });
        Self {
            close_socket,
            events,
            reader: Some(reader),
            buffer: Vec::new(),
            consumed: 0,
        }
    }

    /// Mark the last fully consumed SSE event before causing an update.
    pub fn cursor(&self) -> usize {
        self.consumed
    }

    pub fn wait_for_resource_updated_since(
        &mut self,
        cursor: usize,
        uri: &str,
        timeout: Duration,
    ) -> Result<usize, String> {
        let deadline = std::time::Instant::now() + timeout;
        let mut scan = cursor;
        loop {
            while let Some((end, frame)) = next_sse_frame(&self.buffer, scan)? {
                scan = end;
                self.consumed = end;
                if frame.as_ref().is_some_and(|message| {
                    message["method"] == "notifications/resources/updated"
                        && message["params"]["uri"] == uri
                }) {
                    return Ok(end);
                }
            }
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                return Err(format!(
                    "SSE resource-updated notification timeout for {uri}: {}",
                    String::from_utf8_lossy(&self.buffer[cursor..])
                ));
            }
            match self.events.recv_timeout(remaining) {
                Ok(SseRead::Data(bytes)) => self.buffer.extend(bytes),
                Ok(SseRead::PeerClosed(reason)) => {
                    return Err(format!("SSE peer closed before expected event: {reason}"));
                }
                Ok(SseRead::Failed(error)) => {
                    return Err(format!("SSE reader failed before expected event: {error}"));
                }
                Err(error) => return Err(format!("SSE receiver timeout/disconnect: {error}")),
            }
        }
    }

    pub fn wait_quiet_since(&mut self, cursor: usize, duration: Duration) -> Result<(), String> {
        let deadline = std::time::Instant::now() + duration;
        let mut scan = cursor;
        loop {
            while let Some((end, frame)) = next_sse_frame(&self.buffer, scan)? {
                scan = end;
                if frame.as_ref().is_some_and(|message| {
                    message["method"] == "notifications/resources/updated"
                        && message["params"]["uri"] == "nms://player/location"
                }) {
                    self.consumed = end;
                    return Err(
                        "unexpected player-location notification during negative window".into(),
                    );
                }
                self.consumed = end;
            }
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                self.consumed = scan;
                return Ok(());
            }
            match self.events.recv_timeout(remaining) {
                Err(mpsc::RecvTimeoutError::Timeout) => return Ok(()),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err("SSE reader disconnected during negative window".into());
                }
                Ok(SseRead::Data(bytes)) => self.buffer.extend(bytes),
                Ok(SseRead::PeerClosed(reason)) => {
                    return Err(format!("SSE peer closed during negative window: {reason}"));
                }
                Ok(SseRead::Failed(error)) => {
                    return Err(format!("SSE failed during negative window: {error}"));
                }
            }
        }
    }

    pub fn wait_closed(&mut self, timeout: Duration) -> Result<(), String> {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                return Err("SSE did not close before deadline".into());
            }
            match self.events.recv_timeout(remaining) {
                Ok(SseRead::Data(bytes)) => self.buffer.extend(bytes),
                Ok(SseRead::PeerClosed(_reason)) => return Ok(()),
                Ok(SseRead::Failed(error)) => {
                    return Err(format!("SSE reader failed before close: {error}"));
                }
                Err(error) => return Err(format!("SSE did not close: {error}")),
            }
        }
    }

    pub fn close_and_join(&mut self) -> Result<(), String> {
        let shutdown = self.close_socket.shutdown(std::net::Shutdown::Both);
        let join = self
            .reader
            .take()
            .map(|reader| {
                reader
                    .join()
                    .map_err(|_| "SSE reader thread panicked".to_owned())
            })
            .unwrap_or(Ok(()));
        join?;
        match shutdown {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotConnected => Ok(()),
            Err(error) => Err(format!("close SSE socket: {error}")),
        }
    }
}

impl Drop for SseStream {
    fn drop(&mut self) {
        let _ = self.close_and_join();
    }
}

fn read_sse_chunk(stream: &mut BufReader<TcpStream>) -> std::io::Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    stream.read_until(b'\n', &mut line)?;
    if line.is_empty() {
        return Ok(None);
    }
    let size = usize::from_str_radix(
        std::str::from_utf8(&line)
            .map_err(std::io::Error::other)?
            .trim()
            .trim_end_matches('\r'),
        16,
    )
    .map_err(std::io::Error::other)?;
    if size == 0 {
        return Ok(None);
    }
    let mut bytes = vec![0; size];
    stream.read_exact(&mut bytes)?;
    let mut crlf = [0; 2];
    stream.read_exact(&mut crlf)?;
    if &crlf != b"\r\n" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid chunk terminator",
        ));
    }
    Ok(Some(bytes))
}

fn next_sse_frame(buffer: &[u8], cursor: usize) -> Result<Option<(usize, Option<Value>)>, String> {
    let tail = &buffer[cursor..];
    let lf = tail.windows(2).position(|window| window == b"\n\n");
    let crlf = tail.windows(4).position(|window| window == b"\r\n\r\n");
    let (offset, delimiter) = match (lf, crlf) {
        (None, None) => return Ok(None),
        (Some(offset), None) => (offset, 2),
        (None, Some(offset)) => (offset, 4),
        (Some(lf), Some(crlf)) if lf <= crlf => (lf, 2),
        (Some(_), Some(crlf)) => (crlf, 4),
    };
    let end = cursor + offset + delimiter;
    let text = String::from_utf8_lossy(&tail[..offset]);
    let data = text
        .lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(|line| line.strip_prefix(' ').unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");
    let message =
        if data.is_empty() {
            None
        } else {
            Some(serde_json::from_str(&data).map_err(|error| {
                format!("invalid JSON in SSE data frame: {error}; frame={text}")
            })?)
        };
    Ok(Some((end, message)))
}

#[cfg(test)]
mod sse_cursor_tests {
    use super::next_sse_frame;

    #[test]
    fn test_sse_cursor_ignores_historical_events_and_observes_new_bytes() {
        let old = br#"data: {"jsonrpc":"2.0","method":"notifications/resources/updated","params":{"uri":"nms://player/location"}}"#;
        let old = [old.as_slice(), b"\n\n"].concat();
        let end = next_sse_frame(&old, 0).unwrap().unwrap().0;
        assert!(next_sse_frame(&old, end).unwrap().is_none());

        let mut updated = old.to_vec();
        updated.extend_from_slice(old.as_slice());
        let (_, frame) = next_sse_frame(&updated, end).unwrap().unwrap();
        assert!(frame.is_some_and(|message| {
            message["method"] == "notifications/resources/updated"
                && message["params"]["uri"] == "nms://player/location"
        }));
    }
}
