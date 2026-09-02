---
number: 2
title: "NMS Copilot — Detailed Project Plan"
author: "the watcher"
component: All
tags: [change-me]
created: 2026-03-05
updated: 2026-03-05
state: Active
supersedes: null
superseded-by: null
version: 1.0
---

# NMS Copilot — Detailed Project Plan

> **A real-time, in-memory galactic copilot for No Man's Sky**
> Built in Rust. Queryable via CLI, REPL, and MCP server.
> Streams live save file changes so an AI can explore the galaxy with you.

---

## 1. Project Identity

| Field | Value |
|-------|-------|
| **Project name** | NMS Copilot |
| **Repo** | `oxur/nms-copilot (GitHub) |
| **Workspace root crate** | `nms` (umbrella / crates.io namespace) |
| **Primary binary** | `nms-copilot` (REPL + CLI) |
| **License** | dual MIT or Apache-2.0 |
| **Maintainer** | oubiwann |

---

## 2. Vision

NMS Copilot is not a save editor. It is a **living, queryable model of your galaxy** — a persistent in-memory graph that loads from your save file, watches for changes as you play, and exposes spatial queries, routing, and discovery data through three interfaces:

1. **CLI** — One-shot commands for quick lookups and scripted pipelines.
2. **REPL** — Interactive session with persistent state, tab completion, history; designed to run alongside the game.
3. **MCP Server** — Exposes the galactic model as tool calls, enabling an AI assistant to navigate, search, and plan routes in real time during gameplay.

The architecture is designed so that all three interfaces share the same query engine and the same live-updating data model. A save file change detected by the watcher updates the in-memory graph, which is immediately visible to the REPL session, the next CLI invocation, and the MCP server — simultaneously.

---

## 3. Crate Architecture

```
nms/                              # Workspace root (crate: nms)
├── Cargo.toml
├── crates/
│   ├── nms-core/                 # Types, enums, address math, glyph display
│   ├── nms-save/                 # Raw binary save parser (LZ4, XXTEA, mapping)
│   ├── nms-compat/               # Format adapters (goatfungus JSON fixer, etc.)
│   ├── nms-graph/                # In-memory galactic model, spatial index, routing
│   ├── nms-watch/                # File watcher, delta computation, event stream
│   ├── nms-cache/                # rkyv zero-copy serialization, mmap fast-load
│   ├── nms-query/                # Shared query engine (used by CLI, REPL, MCP)
│   ├── nms-cli/                  # clap-based one-shot CLI commands
│   ├── nms-copilot/              # Interactive REPL (reedline)
│   └── nms-mcp/                  # MCP server for AI integration
├── data/                         # Reference data (biomes.toml, glyphs.toml, etc.)
└── tests/
    └── fixtures/                 # Test save snippets, sample data
```

### Dependency flow

```
nms-core          ← foundation: zero external deps beyond std + serde
    ↑
nms-save          ← depends on nms-core (parse raw saves → core types)
nms-compat        ← depends on nms-core (parse goatfungus JSON → core types)
    ↑
nms-graph         ← depends on nms-core (build graph from core types)
nms-cache         ← depends on nms-core, nms-graph (serialize/restore model)
    ↑
nms-watch         ← depends on nms-save, nms-graph (detect changes, apply deltas)
nms-query         ← depends on nms-graph (shared query logic)
    ↑
nms-cli           ← depends on nms-query, nms-save, nms-compat, nms-cache
nms-copilot       ← depends on nms-query, nms-watch, nms-cache
nms-mcp           ← depends on nms-query, nms-watch
```

The key principle: **nms-graph is the brain**. Everything upstream feeds data into it; everything downstream queries out of it. The `nms-query` crate provides a clean, stateless query API that all three interfaces share — no duplicated logic.

---

## 4. Crate Details

### 4.1 `nms-core` — Foundation Types

The bedrock. Every other crate depends on this. Zero heavy dependencies — just `serde` for serialization and standard library types.

**Universe addressing:**

- `GalacticAddress` — the 48-bit packed coordinate: VoxelX/Y/Z (signed), SolarSystemIndex, PlanetIndex, plus RealityIndex (galaxy number 0–255).
- Bidirectional conversions: packed u64 ↔ struct fields ↔ portal glyph string ↔ signal booster string ↔ emoji glyph string.
- Distance calculation: Euclidean voxel distance × 400 ly, with helper methods for "same region", "same system", "within N ly".

**Portal glyphs with emoji:**

```
Index  Name        Hex  Emoji  Unicode
─────  ──────────  ───  ─────  ───────
  0    Sunset       0   🌅     U+1F305
  1    Bird         1   🕊️     U+1F54A U+FE0F
  2    Face         2   😑     U+1F611
  3    Diplo        3   🦕     U+1F995
  4    Eclipse      4   🌜     U+1F31C
  5    Balloon      5   🎈     U+1F388
  6    Boat         6   ⛵     U+26F5
  7    Bug          7   🐜     U+1F41C
  8    Dragonfly    8   🦋     U+1F98B
  9    Galaxy       9   🌀     U+1F300
 10    Voxel        A   🕋     U+1F54B
 11    Whale        B   🐋     U+1F40B
 12    Tent         C   ⛺     U+26FA
 13    Rocket       D   🚀     U+1F680
 14    Tree         E   🌳     U+1F333
 15    Atlas        F   🔺     U+1F53A
```

Display format example — portal address `01717D8A4EA2`:

```
Hex:    0  1  7  1  7  D  8  A  4  E  A  2
Glyph:  🌅 🕊️ 🐜 🕊️ 🐜 🌳 🦋 🕋 🌜 🔺 🕋 😑
```

The converter is fully multidirectional: index ⟺ name ⟺ emoji string ⟺ hex ⟺ coordinates. Parsing accepts index, name, hex digits, emoji, or glyph names (case-insensitive).

**Biome types:**

- `Biome` enum matching GcBiomeType: Lush, Toxic, Scorched, Radioactive, Frozen, Barren, Dead, Weird, Red, Green, Blue, Swamp, Lava, Waterworld, GasGiant.
- `BiomeSubType` enum matching GcBiomeSubType (31 variants).
- `infested: bool` flag (bit 16 of VP[1]).

**Discovery types:**

- `Discovery` enum: Planet, SolarSystem, Sector, Animal, Flora, Mineral.
- `DiscoveryRecord` struct with timestamp, universe address, discovery data, optional custom name, owner info.

**System and planet models:**

- `System` — address, name, discoverer, timestamp, collection of planets.
- `Planet` — index (0–15), biome, infested flag, seed hash, optional custom name.
- `PlayerBase` — name, type (HomePlanet/Freighter/Ship), address, position.
- `PlayerState` — current location, previous location, freighter location, visited systems.

**Galaxy metadata:**

- `Galaxy` enum or lookup table: 256 galaxies with name, type (Norm/Lush/Harsh/Empty), index.
- Region name generation (if we want to replicate the procedural naming).

### 4.2 `nms-save` — Raw Binary Save Parser

Reads NMS save files directly from disk — no goatfungus required.

**Format 2002+ pipeline (current, post-Frontiers):**

1. Read raw bytes from `save.hg` (or `save2.hg`, etc.).
2. Detect format: if first bytes are `0x7B 0x22` (`{"`), it's plaintext JSON — skip to step 5.
3. Parse sequential LZ4 blocks: read 16-byte header (magic `0xFEEDA1E5`, compressed size, uncompressed size, padding), then decompress each block using LZ4 block decompression. Concatenate all decompressed chunks.
4. Optionally verify integrity: read companion `mf_save.hg`, decrypt with XXTEA (8 rounds, key derived from save slot index + constant `0x1422CB8C` mixed into base key `NAESEVADNAYRTNRG`), verify SHA-256 of raw storage bytes, verify SpookyHash V2 of decompressed JSON (seeds `0x155AF93AC304200` / `0x8AC7230489E7FFFF`).
5. Apply key deobfuscation using `mapping.json` (MBINCompiler release artifact) — transform obfuscated JSON keys to readable names.
6. Deserialize JSON into typed structs using serde.

**Format 2001 support (legacy):**

- Storage file is XXTEA-encrypted before LZ4 compression.
- Decrypt with slot-derived key before decompression.
- Reference: MetaIdea/nms-savetool `Storage.cs`.

We will NOT be supporting this format initially, unless it becomes necessary.

**Key deobfuscation strategy:**

- Ship a bundled `mapping.json` for the latest known game version.
- Support loading an external mapping file (for new game updates before we ship a new release).
- Auto-detect whether JSON is already deobfuscated (check for known readable keys like `PlayerStateData` vs their obfuscated equivalents).
- The mapping is a flat `HashMap<String, String>` — obfuscated key → readable key. Applied recursively to all JSON object keys during or after parsing.

**Platform support (stretch goal):**

- Steam/GOG (format 2002): primary target.
- Mac: same format as Steam, different file path.
- Xbox Game Pass: WGS container format (containers.index + blob files), documented in goatfungus issue #306.
- PlayStation: not planned (PSARC container, requires physical access to save files).

**Extensibility:**

- `SaveParser` trait: `fn parse(path: &Path) -> Result<SaveData>`.
- Each format (raw binary, goatfungus JSON, NomNom export) implements the trait.
- Format auto-detection from file extension, magic bytes, or JSON structure.

### 4.3 `nms-compat` — Format Adapters

Handles non-standard save formats, starting with the goatfungus JSON export.

**Goatfungus JSON fixer:**

- State-machine character walker that tracks `in_string` state.
- Inside strings only: `\xNN` → `\u00NN`, invalid escapes like `\v` → `\\v`.
- Outputs valid JSON that serde_json can parse.

We will NOT be supporting this format initially, unless it becomes necessary.

**Future adapters:**

- NomNom export format (if it differs from standard deobfuscated JSON).
- CSV/TSV import for coordinate lists from external tools.
- NMSCE coordinate exchange format.

### 4.4 `nms-graph` — In-Memory Galactic Model

The heart of the system. Builds and maintains a spatial graph of all known systems, queryable for pathfinding, nearest-neighbor, and region analysis.

**Data structures:**

Three parallel data structures, kept in sync:

1. **petgraph::Graph<SystemId, f64>** — topology layer. Nodes are systems, edge weights are distances in light-years. Used for pathfinding (Dijkstra), TSP routing, reachability analysis (BFS within warp range), and connected component detection.

2. **Spatial index (k-d tree or R-tree)** — geometric layer. Indexes system positions in 3D voxel space for O(log n) nearest-neighbor and radius queries. Candidate crates: `kiddo` (k-d tree, very fast), `rstar` (R-tree, supports dynamic insert). Given that we want incremental updates from the file watcher, `rstar` might be the better fit — k-d trees are typically static. Both should be evaluated.

3. **HashMap-based lookup tables** — associative layer. SystemId → System, planet name → PlanetId, base name → BaseId, biome → Vec<PlanetId>. Fast O(1) lookups for search-by-name, filter-by-biome, etc.

**Graph construction:**

- Parse all discovery records (Available + Store) from save data.
- Group planets by system (same VoxelX/Y/Z/SSI).
- Create a node per system, storing the system's planets, metadata, and address.
- Edge generation strategy (configurable):
  - **k-nearest neighbors** (default, k=10–20): connect each system to its k closest neighbors. Good balance of density and performance.
  - **Warp-range constrained**: only create edges between systems within N light-years (configurable per ship class). Models actual reachability.
  - **Full mesh**: connect everything. Fine for ~300 systems, impractical for thousands.
- Distance is always available on-demand (computed from voxel coordinates); edges just pre-compute it for graph algorithms.

**Query capabilities:**

- Nearest N systems/planets to a reference point (current position, named base, arbitrary coordinates).
- All systems/planets within radius R of a reference point.
- All planets matching a biome filter (with optional infested flag, distance constraint, discoverer filter, named-only filter).
- Shortest path between two systems (Dijkstra with warp-range-limited edges).
- TSP route visiting a set of targets (nearest-neighbor heuristic + 2-opt improvement).
- Hop-constrained routing: "visit all Lush planets within 500K ly using an S-class hyperdrive (2500 ly range)".
- Connected components within a given warp range: "which clusters of systems can I reach without upgrading my drive?"
- Neighborhood/region analysis: all systems sharing the same voxel coordinates (same region).

**Incremental updates:**

- `fn apply_delta(&mut self, delta: SaveDelta)` — add new systems/planets, update player position, add new bases.
- New nodes get inserted into both the petgraph and the spatial index.
- Edges for new nodes are generated on insert (connect to k-nearest existing neighbors).
- Deletion is not needed (NMS doesn't delete discoveries).

### 4.5 `nms-watch` — File Watcher & Delta Stream

Monitors save files for changes and produces a typed event stream.

**Mechanism:**

- `notify` crate watches the NMS save directory for file modifications.
- On change: wait for write to complete (debounce ~500ms), read and parse the new save, diff against current in-memory model.
- Produce a `SaveDelta` describing what changed:
  - `NewDiscoveries(Vec<DiscoveryRecord>)` — newly scanned planets, systems, fauna, flora.
  - `PlayerMoved { from: GalacticAddress, to: GalacticAddress }` — player warped or teleported.
  - `NewBase { base: PlayerBase }` — base was placed.
  - `BaseModified { name: String, changes: BaseChanges }` — base was edited.
  - `InventoryChanged` — (low priority) inventory shifts.
  - `MissionProgress` — (low priority) quest state changes.

**Event distribution:**

- The watcher runs on a background thread (or tokio task).
- Deltas are sent via a channel (`tokio::sync::broadcast` or `crossbeam::channel`).
- Consumers: the REPL's graph model, the MCP server's graph model, the cache invalidation logic.

**Diffing strategy:**

- Full re-parse of the save is acceptable (~22MB decompresses to ~50MB JSON, parseable in <1s on modern hardware).
- Diff is computed by comparing discovery record sets (keyed by Universe Address + Discovery Type), player position, and base list.
- Optimization: if only the player position changed (most common case during gameplay), skip the full discovery diff.

### 4.6 `nms-cache` — Zero-Copy Persistence

Eliminates cold-start cost by serializing the in-memory galactic model to disk using rkyv.

**Strategy:**

- After initial parse + graph build, serialize the entire `GalaxyModel` (graph + spatial index + lookup tables) to an rkyv archive file (e.g., `~/.nms-copilot/galaxy.rkyv`).
- On subsequent startup: check if cache exists and is newer than the save file (compare mtime or store a hash). If valid, mmap the archive and access the model with zero deserialization cost.
- Cache invalidation: if the save file is newer, re-parse and rebuild. The watcher also triggers cache updates after applying deltas.

**rkyv considerations:**

- All core types need `#[derive(Archive, Serialize, Deserialize)]` from rkyv.
- petgraph's `Graph` type may not directly support rkyv — we may need to serialize the node/edge data separately and rebuild the petgraph on load. The spatial index (rstar) similarly may need custom serialization. This is a known complexity point; the fallback is to serialize just the raw discovery data (fast) and rebuild the indices (also fast, <100ms for ~300 systems).

### 4.7 `nms-query` — Shared Query Engine

A pure, stateless query layer that all three interfaces share.

**Design:**

- Takes an immutable reference to the `GalaxyModel`.
- Each query is a function: `fn find_planets(&model, &FindParams) -> Vec<PlanetResult>`.
- Query parameters are strongly typed structs (not string parsing — that's the CLI/REPL's job).
- Results are also typed structs, formatted into tables/JSON/emoji by the display layer.

**Query types:**

- `FindQuery` — search planets/systems by biome, name, discoverer, distance, infested flag.
- `RouteQuery` — plan a traversal: targets (biome filter or explicit list), algorithm (nearest-neighbor, 2-opt), warp range constraint, starting point.
- `ShowQuery` — detail view of a system, planet, base, or address.
- `ConvertQuery` — coordinate/glyph conversion (no model needed, pure math).
- `StatsQuery` — aggregate statistics: biome distribution, distance histograms, discovery counts.
- `NearbyQuery` — what's around me right now (uses player's current position from save data).

### 4.8 `nms-cli` — One-Shot CLI

Clap-based CLI for scripting, quick lookups, and pipelines.

**Commands:**

```
nms info <save>                              # Save overview
nms find --biome Lush --within 100000        # Search planets
nms find --biome Scorched --infested --nearest 5
nms find --named --discoverer oubiwann
nms route --biome Scorched --warp-range 2500 # Plan route
nms route --targets "Base A" "Base B" "Base C" --algo 2opt
nms convert --glyphs "🌅🕊️🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑"
nms convert --glyphs 01717D8A4EA2            # hex also accepted
nms convert --coords 0EA2:007D:08A4:0171     # signal booster format
nms show system 369
nms show base "Acadia National Park"
nms stats --biomes
nms export --format json --biome Lush
```

**Output modes:**

- Pretty tables (via `oxur-table`l see the  `oxur-odm` crate for example usage) with emoji glyphs and ANSI colors — default for terminal.
- Plain text (for piping).
- JSON (for scripting / downstream tools).

**Save file resolution:**

- `--save <path>` explicit path.
- Auto-detect: scan default NMS save directory for the most recent save file.
- Use rkyv cache if available and fresh.

### 4.9 `nms-copilot` — Interactive REPL

The flagship interface. A persistent interactive session with reedline, designed to run alongside the game.

**REPL features:**

- All CLI commands available as REPL commands (same parsing via clap subcommands).
- Persistent context: current position, active filters, last query results.
- Context commands: `set position <base-name|glyphs|coords>`, `set warp-range 2500`, `set filter biome=Lush`.
- Live updates: when the file watcher detects a save change, the REPL's model updates in the background and prints a notification: `🌅 New discovery! Scanned planet "Metok-Kalpa" (Lush) in system 0x242700FE0A56A3`.
- Tab completion: command names, biome names, base names, system names (from loaded data).
- History: persistent across sessions (`~/.nms-copilot/history.txt`).
- Prompt shows current context: `[Euclid | Lush filter | 644 planets] nms>`.

**Session lifecycle:**

1. On startup: load rkyv cache (fast) or parse save file (slower, first run).
2. Start file watcher in background.
3. Enter REPL loop.
4. On exit: save updated rkyv cache.

### 4.10 `nms-mcp` — MCP Server

Exposes the galactic model as MCP tools for AI integration.

**Tool definitions:**

- `search_planets` — find planets by biome, distance, name. Returns structured results with coordinates and emoji glyphs.
- `plan_route` — compute a route between systems or visiting a set of biome targets.
- `show_system` — get details for a system by address, glyphs, or name.
- `show_base` — get details for a player base by name.
- `where_am_i` — return the player's current location from save data (live-updated).
- `whats_nearby` — systems and planets near the player's current position, sorted by distance.
- `convert_coordinates` — bidirectional glyph/coord/signal-booster conversion.
- `galaxy_stats` — biome distribution, discovery counts, distance spread.

**Architecture:**

- Runs as a separate process (or background thread in the copilot).
- Shares the live-updating model via the same watcher channel.
- Stateless tool calls over the MCP protocol (likely JSON-RPC over stdio or SSE).
- The AI sees structured data, not raw save JSON — the model is pre-indexed and pre-computed.

**The dream scenario:**
You're playing NMS. You warp to a new system. The game auto-saves. The watcher detects the change. The model updates. You ask your AI copilot: "I'm looking for a Lush planet within 50,000 ly — what's closest?" The MCP server queries the spatial index, returns the top 5 results with portal glyphs in emoji, and the AI tells you: "There's a Lush planet called 'Metok-Kalpa' about 18,000 ly away — here are the portal glyphs: 🌅🕊️🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑. Want me to plan a route?"

---

## 5. Key Dependencies

| Crate | Purpose | Used by |
|-------|---------|---------|
| `serde` + `serde_json` | JSON serialization | nms-core, nms-save, nms-compat |
| `petgraph` | Graph data structure + algorithms | nms-graph |
| `rstar` or `kiddo` | Spatial index (R-tree / k-d tree) | nms-graph |
| `lz4_flex` | LZ4 block decompression (pure Rust) | nms-save |
| `sha2` | SHA-256 integrity verification | nms-save |
| `rkyv` | Zero-copy archive serialization | nms-cache |
| `notify` | File system watcher | nms-watch |
| `clap` (derive) | CLI argument parsing | nms-cli, nms-copilot |
| `reedline` | Interactive REPL readline | nms-copilot |
| `oxur-table` | Pretty terminal tables | nms-query (display) |
| `owo-colors` or `colored` | Terminal colors | nms-query (display) |
| `chrono` | Timestamp handling | nms-core |
| `tokio` | Async runtime (watcher, MCP server) | nms-watch, nms-mcp |
| `toml` | Config file parsing | nms-copilot |
| `confyg` | Config file management | multiple |
| `twyg` | Structured logging | multiple |

**Hand-implemented (no crate needed):**

- XXTEA: ~50 lines of Rust, well-specified algorithm with NMS-specific constants.
- SpookyHash V2: ~200 lines — or use `spooky-hash` crate if one exists and is correct.
- Key deobfuscation: simple HashMap-based recursive key replacement.

---

## 6. Reference Data

Shipped in `data/` directory and compiled into the binary:

- `biomes.toml` — biome enum values, names, descriptions, color codes.
- `glyphs.toml` — glyph index, name, hex value, emoji character, Unicode codepoint(s).
- `galaxies.toml` — all 256 galaxies with name, type, index.
- `mapping.json` — key deobfuscation map from MBINCompiler (bundled, overridable).

---

## 7. Configuration

`~/.nms-copilot/config.toml`:

```toml
[save]
path = "/path/to/NMS/saves/"        # auto-detected if omitted
format = "auto"                      # auto | raw | goatfungus

[display]
emoji_glyphs = true                  # use emoji for portal glyphs
color = true                         # ANSI color output
table_style = "rounded"              # table border style

[defaults]
galaxy = 0                           # Euclid
warp_range = 2500                    # default warp range (ly) for routing
tsp_algorithm = "2opt"               # nearest-neighbor | 2opt

[cache]
enabled = true
path = "~/.nms-copilot/galaxy.rkyv"

[watch]
enabled = true
debounce_ms = 500

[mcp]
enabled = false                      # opt-in
transport = "stdio"                  # stdio | sse
```

---

## 8. Implementation Phases

### Phase 1: Foundation

**Goal:** Core types, raw binary parser, goatfungus compat parser, coordinate converter with emoji glyphs, and the `nms info` command. At the end of this phase, you can point the tool at a save file and see a summary with emoji portal addresses.

Crates: `nms-core`, `nms-save`, `nms-compat`, skeleton of `nms-cli`.

### Phase 2: Search & Display

**Goal:** In-memory model with spatial index, `find`/`show`/`stats` commands, pretty table output. At the end of this phase, you can search for planets by biome, see what's near a base, and get biome distribution stats.

Crates: `nms-graph`, `nms-query`, complete `nms-cli`.

### Phase 3: REPL & Caching

**Goal:** Interactive REPL with reedline, persistent history, context state, tab completion. rkyv cache for fast startup. At the end of this phase, you have a fully interactive galactic explorer you can run alongside the game.

Crates: `nms-copilot`, `nms-cache`.

### Phase 4: Graph Routing

**Goal:** petgraph-based pathfinding — Dijkstra shortest path, TSP traversal (nearest-neighbor + 2-opt), warp-range-constrained routing. The `route` command. At the end of this phase, you can plan multi-stop routes between biome targets.

Enhancement to: `nms-graph`, `nms-query`.

### Phase 5: Live Watch

**Goal:** File watcher with delta computation, real-time model updates, REPL notifications. At the end of this phase, the REPL model updates live as you play.

Crates: `nms-watch`. Integration with `nms-copilot`.

### Phase 6: MCP Server & AI Copilot

**Goal:** MCP server exposing all query tools, designed for Claude integration. At the end of this phase, an AI can query your galactic data in real time during gameplay.

Crates: `nms-mcp`.

### Phase 7: Polish & Ecosystem

**Goal:** Export commands (JSON, CSV), shell completions, config file, multi-save support, NomNom format support, documentation, crates.io publishing.

Enhancements across all crates.

---

## 9. Answered Questions

1. **rkyv vs bincode for cache** — rkyv gives zero-copy mmap access (fastest possible startup), but adds derive complexity. bincode is simpler but requires full deserialization. For ~300 systems the difference is negligible; for future support of larger datasets (community-aggregated data?) rkyv's zero-copy could matter.

    Answer: rkyv

2. **MCP transport** — stdio (simplest, works with Claude desktop) vs SSE (works with Claude web). Support both? Start with stdio.

    Answer: Claude SSE is deprecated. Use `fabryk*` crates, which build upon `rmcp` and we get stdio + HTTP streaming for free, plus service discoverability.

## 10. Open Design Questions

1. **Spatial index choice** — `rstar` (R-tree, dynamic insert) vs `kiddo` (k-d tree, faster queries but rebuild on insert). Given live updates, `rstar` is likely the right call. Worth benchmarking.

2. **Key mapping updates** — when NMS updates and adds new obfuscated keys, how do we update? Options: (a) bundle mapping.json and ship new release, (b) auto-download latest from MBINCompiler GitHub releases, (c) support external mapping.json override in config. Probably all three, with (c) as the immediate escape hatch.

3. **Multi-galaxy support** — the save file contains discoveries across multiple galaxies (RealityIndex). The graph model needs to handle this — probably one spatial index per galaxy, with cross-galaxy queries being a union operation.

4. **Community data integration** — could we import coordinate lists from NMSCE or other community databases to enrich the local model? This would massively expand the queryable dataset beyond personal discoveries. Design the model to accept "external" discoveries tagged with their source.
