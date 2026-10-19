---
number: 2
title: "NMS Copilot — Detailed Project Plan"
author: "the watcher"
component: All
tags: [change-me]
created: 2026-03-05
updated: 2026-10-19
state: Active
supersedes: null
superseded-by: null
version: 1.42
project: project01-mvp
status: active
depends-on: []
blocks: []
related: [project02-personal-atlas]
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

## 11. Closeout recovery — added 2026-10-03

The original sections above are preserved. This addendum records the user's instruction to commit project02, enter Expedited Mode, and prepare project01 for closure without skipping steps. It adds verification planning; it does not weaken original scope, mark the project closed, or retroactively invent accepted implementation slices.

The source candidate for this preparation is main `a800950ef3558fcf463517696c3c140061e7353d` (0.3.0 development). The historical 0.2.0 tag remains separate publication evidence; a local tag does not establish registry publication or installation. The original seven phases were reorganized into six existing planning arcs. The preserved summary contains 61 milestone rows; 43 numbered documents live under the arcs, including later UI refinements. Summary numbering and detailed milestone numbering differ in places, so acceptance cites paths and sections as well as numbers.

Use [closure preparation](closure-preparation.md), the [project ledger](ledger.md), each arc's recovery plan/ledger, and the [captured baseline](artifacts/closeout-2026-10-03/baseline-report.md). Every original leaf specification must be reconciled, including distinct requirements missing from the summary tables. Historical implementation plus passing tests is evidence to inspect, not independent acceptance.

Friendly account/save UX remains project02 Arc01. Existing project01 correctness defects—including wrong save addresses and wrong planet/system associations—must be repaired or explicitly dispositioned through the closure gate; their mention in project02 is not acceptance of a deferral.

Two-Contributor Workflow applies by default. Expedited Mode is active: commit exact named paths promptly, close units once full evidence and independent gates are satisfied, and advance within the approved roadmap. No new contributor workflow is inferred. Independent project composition review and Operator acceptance remain pending.

First prepared corrective assignment: `arc06-fit-and-finish/slice01-address-encoding-evidence/cc-prompt.md`. It is a bounded evidence investigation that will establish the address contracts needed for a safe repair. It authorizes no production-code edits.

## 12. Version history

- **1.3 — 2026-10-04:** Arc06 Slice05 runtime protection/gates independently reproduce, but three original delivery obligations require iteration01. Record current handoff; approved scope/sequence and all25 project criteria unchanged.


- **1.2 — 2026-10-04:** Arc06 Slice04 surfaced separate save decoding, missing galaxy identity and cache evolution needs; Operator explicitly approved expanding Project01 to require all three. Add three project acceptance rows and sequence cache protection before identity and evidence-gated save repair. Original scope/gates preserved; only the named direction/scope hold is discharged.

- **1.1 — 2026-10-03:** Project-level closeout preparation surfaced missing formal acceptance records, unmeasured broad coverage, and functional defects. Added an explicit recovery route and open ledgers across the preserved arcs; retained original specifications and publication/version distinctions. User enabled Expedited Mode; no acceptance criteria or closure gates were waived.
- **1.0 — 2026-03-05:** Original detailed project plan, retained as historical scope authority.

## 13. Approved correctness scope expansion — 2026-10-04

Operator decision after Arc06 Slice04's accepted decision packet: “this isn't just nice to have: there's one definitive bug here (missing galaxy coord) and I'd class #1 as a bug, and #3 as a needed process. Please, expand scope to include this work!” This approves the three CDC directions in Slice04's final review and makes their delivery required for Project01 closure. Approval settles the direction/scope gate; it does not accept implementation or establish unobserved save-format semantics.

| Required outcome | Approved direction | Ownership / proof |
| --- | --- | --- |
| Save addresses decode correctly | Preserve the existing 48-bit core/portal contract; introduce a distinct field-specific, fallible save-decoding boundary. Treat discovery and base fields separately until evidence supports shared interpretation. | Arc06 coordinates the repair across parser/model/interfaces; P01-R01 requires independent component/glyph/galaxy oracles. |
| Galaxy participates in identity | Use an explicit galaxy-qualified system identity containing reality and planet-zeroed coordinates. Propagate it through planet keys, indexes, queries, routing, import and watcher additions. | P01-R02 requires identical coordinates in galaxies0/9 to coexist in either insertion order and remain correctly associated. |
| Derived caches evolve safely | Reject and rebuild incompatible caches from source; distinguish archive schema from decoded/model semantic revision. Future decoder/identity changes must assess and bump semantic compatibility explicitly. | P01-R03 requires actual startup rejection/rebuild/warm-hit evidence and a continuing revision process. |

These requirements expand the existing correctness closeout work. They do not replace the original22 project rows or move defects into Project02. The ledger now has25 open project criteria. The original69 recovered leaf rows and six arc composition rows remain; Arc06 adds explicit repair and child acceptance rows. Friendly save/account UX and richer atlas/search work retain Project02 placement.

Arc06 execution sequence after the four independently closed preparation slices: **Slice05 cache semantic compatibility first**, then galaxy-qualified identity migration, then the field-specific save repair after sufficient evidence and exact input/reality/failure policy are recorded. CDC will prepare later slices near execution; they are roadmap entries, not pre-issued production assignments. Cache protection is first because an older mtime-fresh derived model can bypass subsequent parser repairs. Every later semantic repair must also demonstrate old-cache rejection.

D02/D03 lexical/requiredness/range/failure scope and D04 per-record/nonzero reality/conflict semantics remain evidence/design work owned by CDC with Operator disposition where user-visible policy is consequential. Their unresolved state holds the affected save-decoding implementation, not independent cache protection or a separately detailed identity repair. Lost names, query association, feature/build, documentation, coverage, performance, distribution and final composition retain their existing gates. No release, push or publish is authorized.

Initial CC assignment (now returned for refinement): `arc06-fit-and-finish/slice05-cache-semantic-compatibility/cc-prompt.md`. Current assignment is `arc06-fit-and-finish/slice05-cache-semantic-compatibility/cc-prompt-iteration01.md` after the initial review described below. Two-Contributor Workflow and Expedited Mode continue; CDC designs/reviews, CC implements in an isolated source worktree, and higher-scale assembly still needs independent composition and Operator acceptance.

## 14. Slice05 review progress — 2026-10-04

Four Slice05 rows accepted and three remain open for bounded proof/docs refinement;31 cache tests and824 workspace tests independently pass. The current iteration01 implements original obligations without changing runtime semantics or architecture. This does not close the cache repair/process parent criterion or permit identity work to skip the Slice05 acceptance gate. Original coverage/features/distribution/composition requirements remain unchanged.

## Slice05 final review — 2026-10-10

[Final independent review](arc06-fit-and-finish/slice05-cache-semantic-compatibility/cdc-verification.md#iteration01-final-independent-cdc-review--2026-10-10) accepts all seven child rows at source decd8d6/planning180a9f6, closing Slice05 and R06-S05. Fresh31 cache/824 workspace tests and lint/build/docs pass. Historical active assignments above are superseded: no active CC assignment while the approved identity migration is detailed. All25 project criteria remain open; semantic-transition/live-composition proof and every original closure gate remain required. Main stays clean/pinned at a800950; no merge/release authority inferred.

Version1.4 (2026-10-10): routine Slice05 child closure progress, no scope/acceptance amendment.

## 15. Identity foundation execution — 2026-10-10

[Slice06](arc06-fit-and-finish/slice06-galaxy-qualified-identity/slice-plan.md) details the approved identity direction into a foundation pass and subsequent query/routing/interface composition, before evidence-gated save repair. Current production CC assignment: `arc06-fit-and-finish/slice06-galaxy-qualified-identity/cc-prompt.md`. Dependency source is accepted decd8d6; new isolated worktree, main remainsa800950. All25 project criteria stay open, with no amendment to required outcomes, original gates or approval authority. Child acceptance cannot replace parent composition.

Version1.5 (2026-10-10): routine bounded identity execution detail after Slice05 closure; all approved scope and acceptance preserved.

## Slice06 independent review — 2026-10-10

Seven foundation rows independently accepted at sourcef57a38f/planning3293cc2; current gates pass (359 focused/840 workspace). GI-08 remains open for exact historical failed-lint capture. Current assignment supersedes initial implementation: `arc06-fit-and-finish/slice06-galaxy-qualified-identity/cc-prompt-iteration01.md`, evidence recovery only, no source changes. Slice06, arc and all25 project criteria remain open. Subsequent identity composition, save policy and all original gates are unchanged; an unavailable historical record requires explicit disposition rather than implicit acceptance.

Version1.6 (2026-10-10): routine partial Slice06 review/current-assignment update; no scope/architecture/acceptance amendment.

## Slice06 evidence disposition pending — 2026-10-10

Evidence-only iteration01 at planninga317761 is independently reviewed; sourcef57a38f remains unchanged and seven child rows retain acceptance. GI-08 stays open for Operator disposition of the original failed lint's missing raw channels/direct exit, with an authentic combined record now verified. [CDC](arc06-fit-and-finish/slice06-galaxy-qualified-identity/cdc-verification.md#proposed-one-time-gi-08-exception--awaiting-operator-approval) proposes a one-time exception limited to that historical call; it is not yet an accepted amendment. No active CC assignment while awaiting the decision. All25 project rows and all original gates remain open/unchanged; no source integration/publication authorized.

Version1.7 (2026-10-10): routine recovery review and pending Operator decision; no acceptance amendment applied.

## Slice06 final closure — 2026-10-10

Operator approved the precise one-time historical lint evidence exception and continuation. [CDC final review](arc06-fit-and-finish/slice06-galaxy-qualified-identity/cdc-verification.md#operator-approved-exception-and-final-closure--2026-10-10) closes all eight foundation rows and R06-S06 at unchanged sourcef57a38f. Missing historical channels/direct exit remain unknown; all current gates and every other capture requirement persist. All25 project criteria remain open, including required identity composition/save repair/original closure gates. No active CC assignment until next unit is ready. Main remains pinned; no merge/release authorization.

Version1.8 (2026-10-10): Slice06 child closure after explicit evidence amendment; no other acceptance/scope/architecture change.

## Slice07 query composition — 2026-10-10

Current CC assignment: `arc06-fit-and-finish/slice07-galaxy-scoped-queries/cc-prompt.md`, production query repair from acceptedf57a38f in an isolated candidate. The approved identity composition is detailed as query correctness then required routing/spatial/session/live interfaces before save-field repair; every requirement remains in Project01. Explicit/default query scope, exact planet associations/filters/ranking, names and meaningful show distances are resolved in the child contract. Derived-index/query changes retain raw2/2 archive compatibility with genuine predecessor proof; later stored-meaning changes still require revision advances.

Arc25/six done/nineteen open; project25 rows all open. Required route/selected-galaxy/session/movement/live cache/CLI/REPL/MCP composition, save semantics and all original docs/features/coverage/packages/performance/final review/Operator gates remain. Main and accepted dependencies unchanged; no source integration/publication.

Version1.9 (2026-10-10): source-grounded query composition detail and assignment after Slice06 closure, with required follow-on owners retained; no scope/acceptance/architecture amendment.

## Slice07 initial review and Iteration01 — 2026-10-10

Initial [CDC review](arc06-fit-and-finish/slice07-galaxy-scoped-queries/cdc-verification.md) independently accepts six child rows at b2d27017, with focused636/workspace854 and prescribed gates. Three remain open for test-matrix completeness, rendered pair/error docs and complete canonical return. Current CC assignment: `arc06-fit-and-finish/slice07-galaxy-scoped-queries/cc-prompt-iteration01.md`; candidate, runtime contract and raw2/2 compatibility remain. No structural plan amendment; later routing/spatial/session/live and save-field owners stay required and held until child acceptance. Arc25/six done/nineteen open; project25 open. No release/merge/main movement authorized.

Version1.10 (2026-10-10): routine partial Slice07 acceptance and bounded correction handoff; all criteria/scope/architecture/final gates retained.

## Slice07 Iteration01 review and Iteration02 — 2026-10-10

Eight Slice07 rows accepted; QG-09 remains open for closing-report assignment identification. Current CC assignment `arc06-fit-and-finish/slice07-galaxy-scoped-queries/cc-prompt-iteration02.md` is evidence-only on unchanged7118a486. Fresh637/855 and all prescribed gates pass; no structural design/runtime/cache change. Required broader composition and save-field work remain held until child closes. Arc25/six done/nineteen open; project25 open. Main/dependencies and all parent criteria/gates remain.

Version1.11 (2026-10-10): routine eight-row acceptance and evidence-only correction handoff; no scope/architecture/acceptance amendment.

## Slice07 final closure — 2026-10-11

CDC independently accepts QG-09 after reviewing Iteration02 planningaf98f31 against issuancea609b21 and unchanged source7118a486. All nine child rows and R06-S07 are done. All186 capture channels, old prefixes/objects/eight accepted rows, assignment identification, fixture and three commit fences pass; independent637/855 and nine source gates retain applicability without rerun. No structural design/acceptance/cache change arose. Arc25/seven done/eighteen open; project25 open. Required routing/spatial/session/live composition is next, before evidence-gated save-field repair. No Slice07 CC assignment remains active; main/dependencies and all original requirements/gates remain.

## Slice08 spatial/routing composition assignment — 2026-10-11

After independent Slice07 closure8aca066, CDC details the next required identity composition into [Slice08](arc06-fit-and-finish/slice08-galaxy-scoped-routing/slice-plan.md), spatial/routing first, then required session/live-interface/full startup composition before evidence-gated field decoding. This supersedes the previously unspecified routing/spatial/session/live unit's granularity only; all original/expanded scope and acceptance persist. Author source inspection and compiled probe at7118a486 reproduce active0 measurements of reference9, cross-galaxy direct0ly, explicit9 query hops[0,9], foreign reachable IDs and101/120 dense reachability. Native type/cache/CLI proof and exact source/caller/compatibility contracts are in the issued packet. No source repair has been made or accepted.

Current CC assignment: `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt.md`, separate candidate from7118a486. Retain original active wrappers with foreign-reference exclusion; add explicit-address APIs; route input guards and three fallible builder signatures propagate known missing/foreign-ID errors. Complete same-galaxy neighborhoods and target dedup/filter-before-limit remove fixed caps. Retain raw archive2/2 because stored meaning is unchanged; prove actual predecessor/fresh/current parity. Best-effort unbridged waypoints are not accepted as strict warp-feasibility; original routing/numeric and all session/live/save/name/base/portal owners remain. This is approved internal identity composition, no new serialization architecture or broader failure-scope amendment.

Add childR06-S08 only:26 arc rows/seven done/nineteen open; project25 open. All25 prior arc criteria/statuses and25 project statuses retained. Main/query/foundation/cache remain pinned; no merge/publication authority. Project version1.13 (2026-10-11), surfaced by Arc06 Slice08: Slice08 author reconnaissance and distinct session/lifecycle packet justify bounded sequencing of the approved follow-on; no acceptance reduction or parent closure.


## Slice08 initial independent review and Iteration01 — 2026-10-11

CDC reviewed sourcee1347bc/planning721964e against preserved initial b792257 assignment. All nine current validation gates independently pass663 focused/881 workspace; exact paths/parents/trailers, sealed inputs/fixture and six clean worktree fences pass. All nine child rows remain open: canonical spatial population, consistent NoPath, route normalization/marker preservation, named membership ordering and exact Displays need correction; prescribed native cache/CLI/spatial/routing controls and historical capture/intake/inventory proof remain incomplete. See [independent review](arc06-fit-and-finish/slice08-galaxy-scoped-routing/cdc-verification.md).

Current CC implementation/evidence assignment: `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration01.md`, directly from candidatee1347bc. Historical originals must be recovered honestly; missing channels/exits are unknown and require CDC/Operator disposition, not a current rerun or inherited exception. No structural scope/architecture/cache-policy/acceptance amendment: same child, same seven/four CC boundaries, no next slice until independent acceptance. Arc26/seven done/nineteen open; project25 open. Required session/live/full-startup and save-field work plus original closeout gates persist; main/dependencies remain pinned and no merge/release/publication authority follows.

Project version1.14 (2026-10-11), surfaced by Arc06 Slice08: initial review and corrective iteration; original requirements and all25 project statuses retained.


## Slice08 Iteration01 independent review and Iteration02 — 2026-10-11

CDC reviewed source21494b9/planningfc895617 against preserved Iteration01 issuance09a460ff. Corrected F1–F5 are reproduced; all current gates pass670 focused/888 workspace, sealed inputs/fixture/source7/planning4/parents/trailers/134channels checked. SR-08 accepted; eight child rows open. Native ties use their runtime oracle; line120 passes a capped-neighborhood negative control; required dense/9 decoy/selection/cache typed literals incomplete. New Iteration01 uncaptured groups, missing historical strict env and report/ledger metadata need correction; no evidence exception adopted. See the Slice08 cdc-verification.md current dated section.

Current assignment: `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration02.md`, test/evidence only, directly from21494b9, four permitted source test paths/four CC return paths. Production held unchanged; accepted CLI/positive-edge/normalization subproof retained subject to delta review. No architecture/cache-policy/criterion/structural sequence change. Main is clean6d183bde with externally committed README/logo-only change from historicala800950; preserve observed fence, no integration authority. Other dependencies unchanged. Slice/arc/project remain open: arc26/seven done/nineteen open, project25 open. Required session/live/full startup then evidence-gated save repair and all original closeout owners/gates persist.

Project version1.15 (2026-10-11), surfaced by Arc06 Slice08: routine correction review, eight child rows remain open; all25 project statuses retained.


## Slice08 Iteration02 interim-stop disposition and Iteration03 — 2026-10-11

CC stopped with four uncommitted test files/four CC records at source21494b9/planning69ff089. CDC reproduced the closed test failure and identified actual warp_rangeNone; public NN/2opt Some400 returns the required A/B/C/B/A projection, None correctly returns A/B/C/A. The constrained typed cache regression passes. No production bug established by this witness; fix test configuration and append honest superseding evidence, then complete all existing proof/ordered gates.236 CC channels, old values/prefixes, production/accepted-row/sealed/dependency fences independently checked.

Current assignment: `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration03.md`, continuing the authorized dirty four-source-test/four-CC-record work from21494b9, without reset or production mutation. Original/Iteration01/new Iteration02 capture limits remain explicit and unwaived. SR-08 retained done/eight child rows open; arc26/seven done/nineteen open, project25 open. Routine assignment update only, no structural scope/architecture/cache/acceptance/sequence amendment. Main6d183bde and dependencies preserved; required session/live/full startup then save-field repair and original higher-scale gates remain; no integration/release/publication authority.

Project version1.16 (2026-10-11), surfaced by Arc06 Slice08: routine fixture-error disposition/third refinement; original25 project statuses retained.


## Slice08 Iteration03 independent review and Iteration04 — 2026-10-11

CDC reviewed sourceedd6664/planning1a931327 against preserved Iteration03 issuance50eac522. All fresh prescribed categories pass680 focused/898 workspace; all486 CC channels, exact source4/planning4 pathsets/parents/trailers, stopped prefixes/JSON, production and sealed fixture/input/control fences pass. None-versus-Some400 correction is sound, not a production repair. SR-04 newly accepted, SR-08 retained, seven child rows open. Remaining exact native spatial/error/decoy/selection/cache and historical/current evidence obligations are enumerated in the dated Slice08 cdc-verification.md.

Current CC assignment: `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration04.md`, fourth refinement, directly from cleanedd6664, same four test-only source/four CC return paths. Completed subproof retained; no production, cache2/2, acceptance or structural-sequence change. Historical original and iteration01/02/03 capture/intake limits require exact honest disposition; no waiver. Arc26/seven done/nineteen open; project25 open. Required session/live/full startup then evidence-gated save repair and original higher-scale owners persist. Main6d183bde/dependencies unchanged; no merge/release/publication authority.

Project version1.17 (2026-10-11): routine Slice08 review/partial acceptance/fourth refinement, original parent criteria and statuses retained.


## Slice08 Iteration04 independent review and Iteration05 — 2026-10-11

CDC reviewed source02260b7/planning24eae38 against preserved Iteration04 issuanceae3cdef4. All fresh nine categories pass682 focused/900 workspace;12/12 public missing9/zero-route checks supplement native constructor evidence.744 CC channel objects, exact source4/planning4/receipt-only commits, stopped prefixes/JSON, production/protected-body/fixture/input and six clean control fences pass. SR-03/05/07 newly accepted; SR-04/08 retained; SR-01/02/06/09 remain open. Remaining native spatial/player-state/filter coverage and two query-state fixtures,14/16 broken relative gate links and historical evidence/intake limits are recorded in Slice08 cdc-verification.md. No current runtime discrepancy or historical waiver is established.

Current CC assignment: `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration05.md`, fifth/final refinement under existing budget, directly from02260b7; only graph query.rs/query route.rs tests plus four existing CC records. Accepted graph-route/cache/CLI/edges and protectedNone/Some400 proof held unchanged. No automatic sixth pass: nonconvergence requires re-scoping or explicit Operator decision. Parent statuses unchanged: arc26/seven done/nineteen open, project25 open. Required session/live/full startup then evidence-gated save-field repair and original gates persist. Main6d183bde/dependencies held, no merge/release/publication authority.

Project version1.18 (2026-10-11): routine Slice08 review/final refinement; synchronize stale frontmatter1.14 with appended effective versions1.15–1.17 and this update; historical notes preserved, parent criteria/statuses retained.


## Slice08 Iteration05 independent review and refinement cap — 2026-10-11

CDC reviewed source0e1e8b4/planningaae22e7 against immutable issuance9a9bf114. All fresh nine gate categories pass682 focused/900 workspace;804 CC channels and stable16 predecessor/22 current mappings validate. Scope/parents/trailers, six clean intake controls, stopped prefixes/JSON and fixture/protected proof fences pass. SR-06 newly accepted; SR-03/04/05/07/08 retained; SR-01/02/09 open (six child rows done, three open). Remaining native active-galaxy boundary matrix, filtered-radius canonical reference and current strict-env/intake/report qualifications are documented in Slice08 cdc-verification.md/receipts iteration05; no new runtime discrepancy or historical waiver established.

Five executed refinements include stoppedIteration02; the current Iteration05 assignment is returned and reviewed. Budget exhausted, **no sixth prompt issued**. Operator decision pending on the concrete recommendation: one additional bounded graph-query-tests/current-evidence correction pass beyond the cap, preserving six accepted rows/all other source, or explicit re-scoping. Historical evidence exception remains outside that proposal and unapproved. No child/parent closure. Parent counts unchanged: arc26/seven done/nineteen open, project25 open. Separate Project02 commit40c36ab preserved. Required session/live/full-startup composition, then evidence-gated save-field repair and original higher-scale owners/gates persist. Main6d183bde/dependencies held; no merge/release/publication authority.

Project version1.19 (2026-10-11): routine Slice08 review/partial acceptance/refinement-cap disposition; parent criteria/statuses retained.


## Operator-authorized Slice08 Iteration06 — 2026-10-11

Supersedes the earlier pending-cap disposition: the Operator explicitly approved one additional bounded correction pass and selected Sofie as implementer, followed by a separate CC context as independent reviewer. Current assignment: `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration06.md` (canonical home project01-mvp). Source baseline0e1e8b4239881a378bc8fb928ef25741c228e171; planning intake23079309da08ed6908c7eb703021111a3ee28b71. Issuance is the commit containing this prompt; intake recorded before source edits in existing execution.md/validation.json. Source only graph query.rs cfg(test); return only four existing evidence records; six accepted entire rows retained and SR01/02/09 remain open. Sofie self-checks are proposed evidence; she will not independently accept her own changes. CC receives review-only handoff through Operator after return. Historical missing evidence remains unwaived; no automatic seventh pass, criteria/runtime/cache/API change, integration or closure. Arc26/seven done/nineteen open; project25 open. Preserve Project02 and held main/dependency pins. Required session/live/full-startup, subsequent evidence-gated save-field repair and original higher-scale closeout gates remain.


### Slice08 Iteration06 proposed return / independent CC handoff — 2026-10-11

Sofie returned sourcef71d607e27e18d7266855a2708449f9d6b90c632 (direct parent0e1e8b4, graph query.rs test-only) and planningdb5416cfe3f683bbe28002f7af79f73fa610be63 (parent340c7c62108f7a422d959c3eb417db352a7b307c issuance; four evidence paths). Current assignment `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration06.md` is executed/returned as proposed current corrections and now the independent CC review-only handoff. All nine categories pass682 focused/900 workspace; full matrix/canonical filtered radius/current evidence delivered. Six accepted rows retained, SR01/02/09 open; Sofie does not accept her own implementation. Historical missing evidence remains unwaived for independent review/explicit Operator disposition. No further correction automatically authorized. Preserve main/dependencies/Project02, existing parent counts and all subsequent lifecycle/save-field/closeout owners. Receipt-only followups to four existing evidence records may record these commits; no integration or child/parent closure.


## Slice08 Iteration06 independent result and pending SR-09 decision — 2026-10-11

Supersedes the earlier six-done/three-open handoff status: independent CC review e6bbfe53/receipt f8d31258 on source f71d607 accepted SR01/02 and retained SR03–08. Current disposition eight done, SR09 open; all required current evidence/gates independently reproduced (682 focused/900 workspace). Current assignment cc-prompt-iteration06.md has been executed/returned/reviewed; no further implementation pass authorized. Sofie reconciled the recorded review without rerunning or self-accepting code. Concrete bounded historical-evidence exception proposal is in existing Slice08 closing-report.md, with exact missing groups/unknown fields, current independent evidence/residual risk and conditional formal-close steps; proposal not approved. No ledger/reviewer verdict/acceptance contract changed. Slice/arc/project stay open; arc26/seven done/nineteen open, project25 open. Keep all issued prompts and historical reports, main/dependency pins and Project02. Original session/live/full-startup then save-field repair and remaining closeout owners/gates remain.

Version history2026-10-11: routine child-review status/evidence synchronization and explicit pending-decision pointer; slice1.8, arc1.30, project1.21. Trigger: Slice08 independent CC review resolves two current native rows, while historical SR09 evidence remains unwaived. This versioning does not approve the proposed exception or authorize closure/integration.


## Slice08 final closure after Operator decision — 2026-10-11

Operator explicitly approved the aa83dbe bounded SR09 exception (“yes, I approve”), only its seven enumerated past observations at source f71d607e27e18d7266855a2708449f9d6b90c632. Independent CC e6bbfe53/receipt f8d31258 supplies current reproduction; unavailable past fields remain unknown and the extra unflagged review doc run remains excluded. Formal final closure/row walk/artifact inventory/bubble-up is in Slice08 closing-report.md and cdc-verification.md. Nine child rows done, R06-S08 done;26 arc rows/eight done/eighteen open; project25 open. No broader/current/future evidence waiver or source integration.

Current bounded sequence: Slice08 closed; detail required session/live-interface/full-startup composition next, within the approved identity-before-save route, then field-specific discovery/base/nonzero reality policy evidence and implementation. Original scope/capabilities and all remaining owners/gates preserved. No automatic arc/project closure/release. Version history2026-10-11: arc1.31/project1.22 record the specific approved Slice08 historical-evidence amendment and formal child closure; no architecture or sequence change.


## Arc06 Slice09 bounded consumer execution — 2026-10-11

Slice08 closed in9d6e43e after specific Operator evidence approval and independent CC proof. Arc06 opens Slice09 galaxy-scoped live session state at acceptedf71d607; separate CC implements, Sofie/CDC reviews. Its seven rows remain open; arc27/eight done/nineteen open, project25 open. Existing mandatory identity consumer work is bounded into09 movement/session/location/typed drain-cache,10 REPL command/reference integration,11 actual watch/transports/full startup-cache/unknown-system and historical cached-state disposition, then evidence-gated save-field policy/repair. All original scope/correctness/coverage/docs/distribution/independent/Operator gates stay open and unchanged. No merge/main/source movement/release/Project02 mutation, old data reconstruction or extension of Slice08's historical exception.

Version1.23 (2026-10-11): Arc06 Slice08 closeout/source inspection supplies the next bounded consumer assignment and explicit remaining owners, without changing approved architecture, scope or acceptance.


## Arc06 Slice09 independent acceptance and framework refresh — 2026-10-16

Slice09 seven rows independently accepted at e9cabd5;717 focused/213 no-default/913 workspace and all required gates reproduced. Arc27/nine done/eighteen open, project25 open. Delivery is a reviewed candidate; main6d183bde remains unchanged, with authorized integration and final-baseline composition owned by CDC/Operator later. Preserve Project02 and all existing scope and gate owners. Framework2.7.2 is now applied, including useful capability increments, actual bounded prerequisite results and pre-agreed investigation purposes/budgets. Existing two-contributor workflow remains; no new role/inquiry/waiver introduced. Proceed to planned Slice10 user-facing galaxy command/reference integration; Slice11 live/transports/fullstartup/cache/unknown state, later raw-field repair and all original parent acceptance remain open.

Version1.24 (2026-10-16): Arc06 child09 acceptance, honest candidate/integration distinction and refreshed planning rules; no substantive scope or acceptance amendment.


## Arc06 Slice10 usable REPL increment — 2026-10-16

Open planned Slice10 galaxy-scoped-repl from acceptede9cabd5 after child09 closure dcd30c4. Production command wiring/help/completion/docs and reference semantics are delivered/tested together as a reviewable candidate. Seven rows open; arc28/nine done/nineteen open, project25 open. Actual bounded dependency is accepted09; no unnecessary whole-project dependency or new investigation. Main6d183bde and Project02 preserved; CDC/Operator own later authorized integration and final-baseline acceptance, while11 live/transport/startup/cache/unknown state, subsequent field repair and original whole-project gates remain. Version1.25 (2026-10-16): Arc06 details existing next owner under framework2.7.2; approved scope/architecture/acceptance unchanged.

## Slice10 independent review and bounded correction — 2026-10-16

CDC independently reproduced all ordered gates at0346b193:724 focused/215 no-default/920 workspace, plus40/40 public reference/error/state checks. RG-01/02/03/04/06 accepted; RG-05/07 open. Reproduced completion bug discards a multi-word prefix ending in space; correct only the existing galaxy-completion helper/tests, clarify current inventory/environment evidence and rerun required gates. Current assignment arc06-fit-and-finish/slice10-galaxy-scoped-repl/cc-prompt-iteration01.md, first refinement; initial prompt preserved. No separate investigation or scope/architecture/criterion/sequence amendment.

R06-S10 stays open:28 arc rows/nine done/nineteen open; all25 project rows open. Main6d183bde, dependencies and Project02 unchanged; candidate0346b193 is partially reviewed and not integrated/installed. CDC/Operator retain authorized integration/final-baseline drift/composition; Slice11 actual live/transports/fullstartup/cache/unknown-state, later raw repair and all original owners/gates remain. Version1.26 (2026-10-16): routine review/refinement status only; five child rows do not close the parent.

## Slice10 Iteration01 review / pending evidence disposition — 2026-10-16

CDC accepts completion repair RG-05 at e615f254; five prior rows retained under exact completion-only delta, all required fresh gates3/726/217/922 and public original endpoint probe5/5. Six child rows done; RG-07 open solely for unrecoverable original1–53 environment metadata (49 empty maps/four override-only). Current gate/environment capture, all144 CC channels/72 invocations, source/commit fences and preserved history verified. Iteration01 returned/reviewed; no further CC assignment issued. Narrow Operator exception proposed in Slice10 cdc-verification.md, not approved/applied; no historical value invented and no acceptance amendment adopted.

R06-S10 remains open; arc28/nine done/nineteen open, project25 open. No Slice11 opening before decision/closure. Main6d183bde/dependencies/Project02 held; candidate e615f254 not integrated/installed. CDC/Operator own the bounded decision and later authorized integration/final-baseline composition; all original Slice11 live/transport/startup/cache/unknown-state, later raw-field and parent owners/gates remain. Version1.27 (2026-10-16): child correction review exposes only historical metadata acceptance question; routine status/proposal, no scope/architecture/sequence/criterion change applied.

## Slice10 Operator approval and formal closure — 2026-10-16

Operator explicitly approved the exact original1–53 environment-metadata waiver (“I approve and give the waiver :-D”). CDC records the accepted historical amendment without reconstructing past fields; all seven child rows accepted at e615f254. R06-S10 done;28 arc rows/ten done/eighteen open,25 project rows open. Current full independent gates/behavior/source proof unchanged and retained. Original unknowns and prompt/report/evidence history preserved; no broader/current/future or parent waiver. Final row walk/inventory/silent-drop/bubble-up is in Slice10 close-set.

Advance to planned Slice11 actual watch/transport/startup-cache/unknown-state useful increment from accepted e615f254; CDC details bounded recipe before issuance, all original obligations retained. Main6d183bde/dependency/Project02 fences held; no integration/installed release. Version1.28 (2026-10-16): Slice10 approval/closure and ready next owner; the sole criterion amendment is the expressly approved historical metadata exception. Architecture/scope/original parent acceptance and later raw repair/gates unchanged.

## Slice11 reliable MCP watch increment — 2026-10-16

After Slice10 formal close5fed0e0 at acceptede615f254, issue arc06-fit-and-finish/slice11-mcp-live-watch/cc-prompt.md. Source inspection of mcp::start_watcher shows receiver-only return drops the debouncer; apply_deltas_loop holds model guard across notifier awaits. Ready useful repair: preserve owned handle throughout actual stdio/HTTP serving, release mutation lock before notifications, protect headless protocol stdout, native file→subscribe/update→tool/resource tests and user docs in one candidate. Actual prerequisite is e615f254, not whole-project closure. Separate CC implements, Sofie/CDC reviews; Expedited Mode and two contributors persist. Seven child rows open;29 arc rows/ten done/nineteen open;25 project rows open.

The earlier single planned11 live/lifecycle placeholder is expressly subdivided for coherent implementation/review headroom:11 actual MCP/watch lifetime/transports; planned12 full startup-cache C04/old-current envelopes/warm and write-through in both modes, unknown-system placeholder/promotion, historical inconsistent cached player-state diagnosis/disposition, and watcher configuration/duplicate-producer composition. All remain required before parent R06-R02/R03/M7.4/C04 acceptance and later raw-field policy/repair;12 has no issued assignment or adopted unresolved policy. This is implementation detail of existing approved obligations, no criterion waived or silent scope drop. Any inquiry/policy change still needs its own authority/Operator agreement.

Candidate .worktrees/mcp-live-watch/codex/mcp-live-watch is promised as wired/tested/documented candidate; main6d183bde/dependencies/Project02 held. CDC/Operator own later authorized integration/final-baseline drift/composition; not installed/released. Original exports/imports/format/docs/distribution/coverage/independent/Operator gates retained. Version1.29 (2026-10-16): Slice10 closure/source inspection motivates bounded ready11 and explicit12 remainder; no architecture/original parent acceptance/scope reduction.

## Slice11 reviewed stop / path correction — 2026-10-16

CC stopped atcdf1e20 after real watcher/MCP gates remained red on unchanged authorized retries. CDC existing-verifier replay isolates path spelling: original /var temp input fails; canonical /private/var input passes retained-owner lifecycle and full stdio, while HTTP reaches B then rejects unsupported cross-galaxy label equality. HTTP subscription assertion also needs success-result proof. Current assignment arc06-fit-and-finish/slice11-mcp-live-watch/cc-prompt-iteration01.md; all seven child rows remain open and initial prompt preserved.

CDC revises only implementation boundary for the already approved live-watch capability: watcher.rs canonical parent/event identity/direct regression tests, bounded existing HTTP test proof and MCP return rustdoc. No public API/architecture/criterion/new feature/parent scope amendment or evidence waiver, and no separate investigation. All normal-input native gates still required; changed-input controls are not acceptance. If bounded repair fails, new diagnosis needs its own agreement. Planned12/later raw-field and original acceptance/integration owners retained.29 arc rows/ten done/nineteen open,25 project rows open; sourcecdf1e20/main/dependency/Project02 fences held, no integration or release. Version1.30 (2026-10-16): initial stop review grounds first correction without acceptance reduction or scope drop.


## Slice11 Iteration01 review / bounded test correction — 2026-10-17

CDC independently reproduced original-TMPDIR live stdio/HTTP phases at46ca153 and all current gates:733 focused/219 no-default/929 workspace; watcher sandbox24/2 timeout preserved before unchanged host26/0 retry. Four child rows LW-01/02/04/06 accepted. LW-03/05/07 remain open for HTTP D-to-A assertion accidentally removed with invalid B equality, direct alias regression using9/X1 rather than prescribed0/X1, and appended current evidence correction. No runtime defect reproduced; source implementation remains frozen during this narrow refinement.

Current assignment arc06-fit-and-finish/slice11-mcp-live-watch/cc-prompt-iteration02.md; predecessor iteration01 returned/reviewed and preserved, second refinement of five. Two existing test bodies/four current CC evidence paths only; all required fresh gates and independent review retained. No investigation, waiver, architecture/API/criterion/parent scope or sequence change. R06-S11 stays open:29 arc rows/ten done/nineteen open; all25 project rows open. Planned12 full startup/cache/unknown-state/historical-state/watch-config/duplicate-producer composition, raw-field repair and all original parent acceptance/integration/release owners remain. Main/dependencies/Project02 held, candidate unintegrated/uninstalled. Version1.31 records child review/refinement only.


## Slice11 independent closure — 2026-10-17

All seven LW rows accepted at239313 (parent46ca153), CC returnb1d1e09/receiptbe9cdb7. Iteration02 exactly restores HTTP D-to-A and alias0/X1 tests, production frozen; CDC fresh watcher26/lifecycle2/native2/format/lint/diff and nine actual observations, original TMPDIR, preserved restricted failure plus unchanged host retry. Prior four-row independent733/219/929 broader evidence retained with explicit applicability, and complete current CC gates/82 attempts/164 channels audited. Both close-set records preserve original history, final row walk/inventory/what-worked/silent-drop and delivery state.

R06-S11 done;29 arc rows/11 done/18 open, all25 project rows open. Current11 assignment accepted/closed, no further correction. No new scope/architecture/acceptance/sequence change is required by the final child correction; status/evidence versioning only. Detail planned12 full startup-cache C04/old-current envelopes/warm/write-through in both modes, unknown-system placeholder/promotion, historical inconsistent player-state diagnosis/disposition and watcher-config/duplicate-producer composition next; no unresolved policy/inquiry delegated without required authority. Original raw-field repair/export/import/format/docs/distribution/coverage/parent independent/Operator gates remain. Main/dependencies/Project02 held; candidate unintegrated/uninstalled, CDC/Operator own later authorized integration/final-baseline composition/release. Version1.32: formal Slice11 acceptance and bubble-up, preserving every original owner and gate.


## Slice12 cache-state integrity / explicit remaining composition owner — 2026-10-17

After accepted11 source239313 and closuread552d4, open arc06-fit-and-finish/slice12-cache-player-state-integrity/cc-prompt.md. Ready useful repair: byte/file cache entrypoints reject contradictory player.current_reality versus current_address.reality_index, allowing the already wired common startup loader to rebuild from authoritative source and warm-hit. Never guess which old field is correct; do not reject independently selected galaxy/previous/freighter differences. Existing Serialize/Deserialize errors, public APIs, shape, schema2/semantic2 and valid consistent-cache meaning remain unchanged; this enforces the existing invariant rather than changing coordinate interpretation. Current consistent2/2 route fixture and all old compatibility controls remain supported. Future model/decoder meaning changes retain mandatory revision assessment/bump proofs.

Was: broad planned12 full startup/cache C04, old/current/warm/write-through in both modes, unknown-system placeholder/promotion, historical-state diagnosis/disposition and watch configuration/duplicate-producer composition. Routine source inspection now details ready cache-integrity recovery as12 and assigns **all remainder explicitly to planned13/CDC**, before raw-field repair and original parent acceptance: full startup/live-cache old/current/warm/write-through across both modes; unknown placeholders/promotion; watcher enabled/debounce and duplicate producers; historical questions beyond this precisely contradictory-field case.13 is a named required owner, not issued work or a new investigation. No original capability/criterion is deferred, removed or accepted by this sizing refinement; no new architecture/public API or empirical user-save survey. Any inquiry or consequential policy change retains CDC/Operator authority and prior agreement.

Separate CC implements, Sofie/CDC reviews; Expedited Mode. Actual prerequisite239313, candidate .worktrees/cache-player-state-integrity/codex/cache-player-state-integrity; main6d183bde/accepted11/dependencies/Project02 held. Wired/documented/tested isolated candidate, not main integration/installed release; later authorized integration/final-baseline composition stays CDC/Operator-owned. Seven child CI rows open; new R06-S12 open;30 arc rows/11 done/19 open, all25 project rows open. Original R06-R02/R03/M7.4/C04, raw-field repair/export/import/format/docs/distribution/coverage/parent independent/Operator gates remain. Version1.33 records child11 closure/source inspection and a bounded next implementation/remainder mapping, preserving scope, architecture and quality floor.


## Slice12 initial review / bounded test-and-doc correction — 2026-10-17

CDC independently reproduced all required gates at88c5980:38 cache/4 route/2 native/737 focused/219 no-default/933 workspace, format/lint/build/docs/strict-cache/diff. Initial restricted native failure and unchanged host retry retained. Runtime invariant implementation is correct against the assigned contract; CI-04/05 accepted. Five child rows remain open for specific required test oracles (no temp artifact, full original state/maps, source9 and cached view7 recovery), omitted direct rebuild boundary documentation and current42-attempt inventory clarification. No runtime defect or missing raw channel reproduced;84 CC channels audited.

Current assignment arc06-fit-and-finish/slice12-cache-player-state-integrity/cc-prompt-iteration01.md, first refinement of five; initial prompt returned/reviewed/preserved. Two existing test/doc paths only, all production frozen; complete current gate/evidence and independent review remain. R06-S12 stays open:30 arc rows/11 done/19 open, all25 project rows open. Planned13 full startup/live/cache/unknown-state/watch configuration/duplicate-producer and broader historical-state ownership remains before raw repair and original parent acceptance. Main/dependency/Project02 held; candidate unintegrated/uninstalled, later authorized integration/composition/release remains CDC/Operator-owned. Version1.34: routine child review/correction status, no scope/architecture/criterion/sequence change, inquiry or waiver.


## Slice12 Iteration01 independent closure — 2026-10-17

All seven CI rows independently accepted at a101bd44 (direct parent88c5980), current assignment cc-prompt-iteration01.md returned/reviewed/closed, immutable initial and iteration prompts preserved. Fresh CDC38 cache/4 route/format/lint/strict-cache/diff proof resolves full original-state, exact temp-effect and source9/0/view7 oracles; direct rebuild boundary docs correct. Unchanged prior88c5980 broader independent proof retained with explicit applicability; current CC14 gates and63 attempts/126 raw channels audited. No waiver or runtime correction. R06-S12 done;30 arc rows/12 done/18 open, all25 project rows open.

Final child correction reveals no structural/scope/architecture/criterion/sequence amendment. Detail planned13/CDC next: full startup/live-cache C04 old/current/warm/write-through in both modes, unknown placeholders/promotion, watcher enabled/debounce and duplicate producers, broader historical questions beyond this exact duplicate-field contradiction. Every original raw-field/export/import/format/docs/distribution/coverage/parent independent/Operator gate retained. Main6d183bde/dependencies/Project02 held, candidate isolated/unintegrated/uninstalled; later authorized integration/final-baseline composition/release remains CDC/Operator-owned. version: 1.35 records child closure and bubble-up only.


## Slice13 existing watcher settings / explicit composition remainder — 2026-10-17

Following Slice12 acceptancea101bd44/closureae9bd82, open arc06-fit-and-finish/slice13-mcp-watch-settings/cc-prompt.md. Source main honors existing watch.enabled/debounce for its REPL producer, while MCP startup constructors independently use default500ms whenever a path exists. Ready useful correction: resolve existing settings once, wire both main MCP modes and normal producer, preserve public default wrappers via additive configured entrypoints; actual disabled stdio/HTTP and enabled nondefault native checks, lifecycle preservation and user/API documentation together. No new config schema/architecture/notify/cache/model policy. Actual dependency is accepteda101bd44; candidate .worktrees/mcp-watch-settings/codex/mcp-watch-settings is isolated, not integrated/installed. CC implements; Sofie/CDC independently reviews, Expedited Mode.

Was: broad planned13 combined full startup/live-cache C04 old/current/warm/write-through both modes, unknown placeholders/promotion, settings/duplicate producers and broader historical-state questions. Routine source inspection details13 settings correctness; ALL remainder is explicitly required planned14/CDC: full startup/live-cache C04 old/current/warm/write-through both modes; unknown placeholders/promotion; duplicate producer ownership and settings composition; broader historical-state questions beyond contradictory cache physical fields. Normal mode still has two producers in13, both obey settings. No single-producer, live cache-write-through, full REPL PTY or final lifecycle composition claim.14 is not yet issued; unanswered policy/inquiry remains CDC/Operator-owned with prior agreement where required. Original raw-field repair/export/import/format/docs/distribution/coverage/R06-R02/R03/M7.4/C04/parent independent/Operator gates retained before parent acceptance. This sizing refinement removes no capability or criterion.

Seven WS rows open, R06-S13 open:31 arc rows/12 done/19 open; all25 project rows open. Main6d183bde, accepted12/11/dependencies and Project02 held; CDC/Operator retain later authorized integration/final-baseline drift/composition/release. Version1.36 records Slice12 closure/source-grounded detailing of existing settings work and complete next-owner mapping, without scope reduction or acceptance amendment.


## Slice13 independent closure — 2026-10-17

Seven WS rows accepted at16c0787 (direct parenta101bd44), CC close65dc590/receiptsb3556a7/753023a. Fresh CDC format/lint/cache38/route4/native4/focused744/no-default220/workspace940/build/docs/strict-cache/diffs pass; restricted native failure and unchanged original-TMPDIR host retry retained. Existing settings honored in both main MCP calls; default public wrappers preserved, actual enabled137/disabled child tests and docs pass. All23 CC primary attempts/46 channels and protected26/frozen3 scope/history/commit fences verified. No waiver or new structural design finding.

R06-S13 done:31 arc rows/13 done/18 open, project25 open. Detail planned14/CDC next; all startup/live-cache C04/old-current/warm/write-through both modes, unknown placeholders/promotion, duplicate producer ownership/settings composition and broader historical questions remain required before raw-field repair and original parent acceptance. Main/dependencies/Project02 held; candidate isolated/unintegrated/uninstalled, later authorized integration/final-baseline composition/release stays CDC/Operator-owned. Original export/import/format/docs/distribution/coverage/R06-R02/R03/M7.4/C04/parent independent/Operator gates preserved. Version1.37 records child closure and bubble-up only.


## Slice14 native headless cache startup / complete remaining ownership — 2026-10-17

After Slice13 accepted16c0787/closure320de77, issue arc06-fit-and-finish/slice14-headless-cache-startup/cc-prompt.md. Existing native tests always pass --no-cache; component/static loader checks do not prove real cached startup. Detail ready native headless startup part of existing C04 owner: eight literal cold/warm/stale/schema1/semantic1/corrupt/bypass-existing/bypass-absent cases per real stdio/HTTP transport. Opposed source0/cache9 identities, exact full state/file effects, real cache freshness and synthetic/literal field authority make the check discriminating. Existing runtime already defines the expected behavior; no new inquiry, production policy or source repair delegated. Two test/doc source paths only; runtime/config/API/dependencies/fixtures frozen. Candidate .worktrees/headless-cache-startup/codex/headless-cache-startup starts at accepted16c0787, isolated/wired test suite and docs, not installed/integrated. CC implements, Sofie/CDC independently reviews, Expedited Mode.

Was: broad planned14 full startup/live-cache C04 old/current/warm/write-through both modes, unknown placeholders/promotion, duplicate producers/settings and historical-state questions.14 now details native headless INITIAL startup composition; ALL remainder explicitly required planned15/CDC: normal-mode startup/cache; full both-mode live-cache C04/write-through; unknown-system placeholders/promotion; duplicate producer ownership/settings; broader historical-state questions and final-baseline semantic transition composition once raw-field repair lands. No full C04/R06-R02/R03/M7.4, normal REPL PTY, live-cache or single-producer acceptance inferred. No capability removed or moved out of Project01; remainder remains required before raw repair/original parent acceptance as applicable. Unresolved policy/inquiry still requires CDC/Operator authority and prior agreement. Original export/import/format/docs/distribution/coverage/parent independent/Operator gates unchanged.

Seven HC rows/R06-S14 open:32 arc rows/13 done/19 open, project25 open. Main6d183bde/dependencies/accepted13/Project02 held, later authorized integration/final-baseline drift/composition/release stays CDC/Operator-owned. Version1.38 records Slice13 closure and routine sizing/source-grounded native test delivery plus complete next-owner mapping, with no original scope/criterion/architecture reduction or waiver.


## Slice14 initial review / bounded fixture correction — 2026-10-17

CDC reproduces all16 nominal startup witnesses and all14 gates at88b184f:38 cache/4 route/6 native/746 focused/220 no-default/942 workspace; sandbox failure/unchanged host retry and current39 CC attempts/78 channels retained. HC-01/02/03/06 accepted; HC-04/05/07 open. Explicit missing-input prelaunch absence checks are omitted, and HTTP bind retries reuse a cache fixture potentially rewritten before the failed bind. Runtime unchanged/correct against reviewed startup contract, no production repair assigned.

Active arc06-fit-and-finish/slice14-headless-cache-startup/cc-prompt-iteration01.md, first refinement of five; initial prompt returned/reviewed/preserved. One existing test path/current evidence only; fresh per-attempt fixture ownership and one controlled first-bind-collision stale-cache regression plus absent preconditions, full gates/independent review retained. No scope/architecture/criterion/sequence change, inquiry or waiver. R06-S14 open:32 arc rows/13 done/19 open; project25 open. Planned15/full original composition/independent/Operator/integration/release owners unchanged, main/dependencies/Project02 held. Version1.39 records child review/correction status only.


## Slice14 Iteration01 independent closure — 2026-10-17

All seven HC rows accepted at130034e (direct parent88b184f), CC e373cd2/fb8f6a4/2aed2da. Required absent preconditions and fresh HTTP retry ownership corrected; actual occupied-port stale rewrite/new stale retry plus16 nominal cases independently pass. All14 fresh CDC gates38/4/7/747/220/943, restricted failure/unchanged host retry,59 CC attempts/118 channels and exact history/source/commit fences checked. Runtime/README unchanged, no waiver. R06-S14 done:32 arc rows/14 done/18 open; project25 open.

Planned15/CDC retains normal startup/full both-mode live-cache C04/write-through, unknown placeholders/promotion, duplicate producer/settings ownership, broader history and final-baseline semantic transitions after raw repair. Source inspection shows normal REPL and MCP independently produce/apply deltas, only REPL drain persists cache, and current watch snapshot/delta omits complete player state and save version. A complete write-through design therefore requires ownership/state-authority decisions before issuing affected implementation; next15 remains planned/unissued pending CDC/Operator design resolution, without commissioning an inquiry. Main/dependencies/Project02 held, candidate unintegrated/uninstalled; later authorized integration/final-baseline composition/release and every original raw-field/export/import/format/docs/distribution/coverage/parent independent/Operator gate retained. Version1.40 records child closure and explicit next design gate, no architecture adopted.


## Planned15 ownership decision proposal — 2026-10-17

Slice14 is independently closed at130034e/5c1c88e; arc32/14 done/18 open, project25 open. Arc06 arc-plan.md “Proposed Slice15 ownership design — awaiting Operator” records CDC's concrete recommendation: one app-owned save watcher/update coordinator and serialized live cache writer shared by REPL/MCP, carrying complete supported PlayerState and save_version from each accepted parse. Current two producers/REPL-only writer and position-only snapshot/delta cannot establish that full composition by adding a write call. Existing APIs/settings/selection/manual anchors/atlas data/error recovery/valid-cache behavior and native gates remain preservation obligations.

The recommendation is unapproved; Operator decides that ownership/state-authority direction before dependent CC implementation. No source or future prompt/terminal/dependency/cache-layout choice adopted, no investigation commissioned. All normal startup/both-mode live C04/write-through/unknown promotion/duplicate settings/history and original raw-field/export/import/format/docs/distribution/coverage/parent independent/Operator/integration/release owners remain. Main/dependency/Project02 held. Version1.41 records proposed next design and pending authority only, without silently changing scope/architecture/acceptance.


## Arc06 Autonomous authorization — 2026-10-19

The Operator adopted the single app-owned watcher/update/cache-writer direction and explicitly selected one-arc Autonomous Mode with OpenAI/Codex CDC `gpt-6.1-sol` High and CC `gpt-6-luna` High. The governing record is [Arc06 authorization and current preflight](arc06-fit-and-finish/arc-plan.md#autonomous-arc06-authorization-and-current-preflight--2026-10-19), version1.51. Scope ends at Arc06; no next arc or Project02 advancement, fallback model, release or external action is authorized. CDC owns planning/index and independent CC review; CC receives a dedicated source worktree and a preserved author-ready assignment. Direct in-scope subagent transport replaces human relay only.

Original scope, all25 project rows, arc32/14 done/18 open, full normal/live/cache/unknown/history/raw-field/format/export/import/docs/distribution/coverage and higher-scale independent/Operator gates remain. The prior pending direction wording is historical and superseded only for the adopted direction; unimplemented details and consequential policy choices remain gated. Version1.42 records this narrower arc-level execution selection and links its authority without propagating it to other arcs or projects.
