# 🚀 NMS Copilot

[![][build-badge]][build]
[![][crate-badge]][crate]
[![][tag-badge]][tag]
[![][docs-badge]][docs]
[![License](https://img.shields.io/crates/l/treadle.svg)](LICENSE-MIT)

**A real-time galactic copilot for [No Man's Sky](https://www.nomanssky.com/), built in Rust.**

[![][logo]][logo-large]

Search planets by biome. Plan warp routes through the stars. Convert portal glyphs with emoji. Watch your save file live as you play — and let an AI explore the galaxy *with* you.

```
[Euclid │ 644 planets │ 293 systems] 🚀 find --biome Lush --nearest 5

  #  Planet            Biome   System             Distance   Portal Glyphs
  1  Metok-Kalpa       Lush    Gugestor Colony       0 ly    🌅🕊️🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑
  2  Sushimi           Lush    Esurad               18K ly   🌅🕊️🐜🦕🌜🎈⛵🐜🦋🌀🕋🐋
  3  (unnamed)         Lush    Ogsjov XV            42K ly   🌅😑🐜🕊️🐜🌳🌜🕋🌅🔺🕋🦕
  4  (unnamed)         Lush    Rastarc-Zukk         67K ly   🌅🦕🐜🕊️🐜🌅🦋🕋🌜🔺🕋🐜
  5  Dipadri Grosso    Lush    Ipswic               91K ly   🌅🌜🐜🕊️🐜🌳🌜🕋🌅🌀🕋😑
```

---

## What is this?

NMS Copilot reads your No Man's Sky save files — either the raw binary format (`save.hg`) directly or exported JSON — and builds a live, in-memory model of every system, planet, and base you've discovered. It's not a save editor. It's a **queryable atlas** of your personal galaxy.

Three ways to use it:

| Interface | What it does |
|-----------|-------------|
| **CLI** (`nms`) | One-shot commands for quick lookups and scripted pipelines |
| **REPL** (`nms-copilot`) | Interactive session with persistent state — run it alongside the game |
| **MCP Server** (`nms-mcp`) | Exposes your galaxy to an AI assistant for real-time co-exploration |

The copilot watches your save directory for changes. When you warp to a new system, scan a planet, or build a base, it detects the auto-save and updates the model automatically. If you're running the MCP server, your AI copilot knows where you are *right now*.

---

## Portal Glyphs

NMS Copilot renders portal addresses as emoji throughout all interfaces:

Portal Glyphs

| Index | Name  |      Hex | Emoji | Unicode|
|-------|-------|----------|-------|--------|
|  0 |   Sunset  |     0 |  🌅  |   U+1F305
|  1 |   Bird    |     1 |  🕊️  |   U+1F54A U+FE0F
|  2 |   Face    |     2 |  😑  |   U+1F611
|  3 |   Diplo   |     3 |  🦕  |   U+1F995
|  4 |   Eclipse |     4 |  🌜  |   U+1F31C
|  5 |   Balloon |     5 |  🎈  |   U+1F388
|  6 |   Boat    |     6 |  ⛵  |   U+26F5
|  7 |   Bug     |     7 |  🐜  |   U+1F41C
|  8 |   Dragonfly|    8 |  🦋  |   U+1F98B
|  9 |   Galaxy   |    9 |  🌀  |   U+1F300
| 10 |   Voxel    |    A |  🕋  |   U+1F54B
| 11 |   Whale    |    B |  🐋  |   U+1F40B
| 12 |   Tent     |    C |  ⛺  |   U+26FA
| 13 |   Rocket   |    D |  🚀  |   U+1F680
| 14 |   Tree     |    E |  🌳  |   U+1F333
| 15 |   Atlas    |    F |  🔺  |   U+1F53A

Convert freely between formats:

```bash
# Emoji → coordinates
nms convert --glyphs "🌅🕊️🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑"

# Hex glyphs → coordinates
nms convert --glyphs 01717D8A4EA2

# Signal booster → emoji glyphs
nms convert --coords 0EA2:007D:08A4:0171

# Galactic address → everything
nms convert --ga 0x40050003AB8C07
```

---

## Commands

### Search

Find planets matching any combination of criteria, sorted by distance:

```bash
nms find --biome Lush                          # all lush planets
nms find --biome Scorched --infested           # infested scorched only
nms find --biome Barren --within 100000        # within 100K ly
nms find --biome Lava --nearest 5              # 5 closest lava planets
nms find --biome Swamp --from "Sealab 2038"   # distance from a named base
nms find --named --discoverer oubiwann         # your named discoveries
```

### Route Planning

Plan optimal routes through the galaxy with warp range constraints:

```bash
nms route --biome Scorched                       # visit all scorched, nearest-neighbor
nms route --biome Scorched --within 500000       # only within radius
nms route --biome Lush,Swamp --warp-range 2500   # S-class hyperdrive hops
nms route --biome Frozen --algo 2opt             # improved TSP
nms route --targets "Base A" "Base B" "Base C"   # explicit waypoints
```

### Info & Details

```bash
nms info                            # save overview, player location, discovery counts
nms show system 369                 # system details + all planets
nms show base "Acadia National Park"  # base details with portal glyphs
nms stats --biomes                  # biome distribution table
nms saves                           # list all save slots
```

### Export & Import

```bash
nms export --format json                          # export all planets as JSON
nms export --biome Lush --format csv              # export filtered planets as CSV
nms import community_data.csv --source "NMSCE"    # import community coordinates
```

### Shell Completions

```bash
nms completions bash > ~/.bash_completion.d/nms   # bash completions
nms completions zsh > ~/.zfunc/_nms               # zsh completions
nms completions fish > ~/.config/fish/completions/nms.fish  # fish completions
```

### Multi-Save Support

```bash
nms info --slot 3                   # use save slot 3 instead of most recent
nms find --slot 5 --biome Lush      # search slot 5's discoveries
```

### Interactive REPL

```bash
nms-copilot

[Euclid │ 644 planets] 🚀 set position "Acadia National Park"
📍 Position set to Acadia National Park (Lush, Gugestor Colony)

[Euclid │ 644 planets] 🚀 find --biome Lava --nearest 3
  #  Planet       Biome  Distance   Portal Glyphs
  1  (unnamed)    Lava     127K ly  🌅🦕🌀🕊️🐜🌳🌜🕋🌅🌀🕋🦕
  2  (unnamed)    Lava     204K ly  🌅🌜🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑
  3  (unnamed)    Lava     318K ly  🌅😑🐜🕊️🐜🌅🌜🕋🌅🔺🕋🐜

[Euclid │ 644 planets] 🚀 route --targets 1 2 3 --warp-range 2500
  Planning route from current position through 3 waypoints...
  Hop  System              Distance    Cumulative   Glyphs
   1   Ikusam-Rista          127K ly      127K ly   🌅🦕🌀🕊️🐜🌳🌜🕋🌅🌀🕋🦕
   2   Ovfast XI              77K ly      204K ly   🌅🌜🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑
   3   Yatsinbur-Epp         114K ly      318K ly   🌅😑🐜🕊️🐜🌅🌜🕋🌅🔺🕋🐜
  Total: 318K ly, 3 hops (128 warp jumps at 2500 ly range)
```

---

## Architecture

NMS Copilot is a Rust workspace of focused crates:

```
nms/
├─ nms-core       Types, enums, address math, glyph emoji
├─ nms-save       Raw binary save parser (LZ4 + XXTEA + key mapping)
├─ nms-compat     Format adapters (NomNom save format detection)
├─ nms-graph      petgraph spatial model, R-tree index, routing
├─ nms-query      Shared query engine (find, route, show, stats)
├─ nms-watch      File watcher, delta computation, live updates
├─ nms-cache      rkyv zero-copy serialization for fast startup
├─ nms-cli        clap one-shot CLI (the `nms` binary)
├─ nms-copilot    reedline interactive REPL (the `nms-copilot` binary)
└─ nms-mcp        MCP server for AI integration
```

The data flows in one direction:

```
save file → parser → galaxy model → query engine → CLI / REPL / MCP
                          ↑
               file watcher (live updates)
```

The galaxy model is the core: a petgraph of systems with an R-tree spatial index, incrementally updated as the game auto-saves. All three interfaces share the same query engine — no duplicated logic.

### How Save Parsing Works

NMS saves are **LZ4 block-compressed JSON** (not a proprietary binary format). The pipeline:

1. Read sequential 16-byte block headers (magic `0xFEEDA1E5`) + LZ4 payloads
2. Decompress and concatenate all blocks
3. Deobfuscate JSON keys using MBINCompiler's `mapping.json`
4. Deserialize into typed Rust structs via serde

No encryption on modern saves (format 2002+, post-Frontiers). The only crypto is XXTEA on the small metadata file (`mf_save.hg`), used for integrity verification.

### MCP Server

Run alongside Claude or another AI assistant for real-time co-exploration:

```bash
nms-mcp                           # stdio transport (for Claude Desktop)
nms-mcp --http 127.0.0.1:3000    # HTTP transport (for remote clients)
```

The MCP server exposes all query capabilities as tools — your AI copilot can search planets, plan routes, convert coordinates, and track your position as you play.

---

## Status

All 7 phases complete.

| Phase | Status | Description |
|-------|--------|-------------|
| 1. Foundation | ✅ | Core types, parsers, coordinate converter, CLI |
| 2. Search & Display | ✅ | In-memory model, find/show/stats commands |
| 3. REPL & Cache | ✅ | Interactive copilot, session state, rkyv cache, config file |
| 4. Routing | ✅ | Pathfinding, TSP, warp-range planning |
| 5. Live Watch | ✅ | Real-time save file monitoring, REPL integration, cache write-through |
| 6. MCP Server | ✅ | AI copilot integration via stdio and HTTP transports |
| 7. Polish | ✅ | Export, import, themes, completions, integration tests, CI |

<details>
<summary>Phase 1 — Foundation</summary>

| Milestone | Description |
|-----------|-------------|
| 1.0 Workspace scaffold | Cargo workspace with all 11 crates |
| 1.1 Design documents | ODM-managed project plan and resources |
| 1.2 Core types | `GalacticAddress`, `PortalAddress`, `Glyph`, `Biome`, `Galaxy`, `System`, `Planet`, `PlayerState`, `Discovery` |
| 1.3 Portal glyph converter | Full multidirectional conversion: hex, emoji, name, coordinates, signal booster |
| 1.4 Distance calculator | Euclidean voxel distance x 400 ly, special system detection |
| 1.5 LZ4 decompressor | Block-level LZ4 decompression with magic `0xFEEDA1E5` header parsing |
| 1.6 Metadata verifier | XXTEA decrypt/encrypt, key derivation, SHA-256 verification for `mf_save.hg` |
| 1.7 Key deobfuscation | `mapping.json` key remapping with bundled MBINCompiler + legacy + savewizard maps |
| 1.8 Serde deserialization | Typed structs for save JSON with conversion to core domain types |
| 1.9 `nms info` command | Save file summary: play time, location, discoveries, bases, currencies |
| 1.10 `nms convert` command | Coordinate converter: portal glyphs, signal booster, galactic address, voxels |
| — Save file discovery | Platform-specific save directory resolution, account/slot/type parsing |

</details>

<details>
<summary>Phase 2 — Search & Display</summary>

| Milestone | Description |
|-----------|-------------|
| 2.1 Galaxy model | `GalaxyModel` with petgraph, R-tree, HashMap indexes; system/planet extraction from save |
| 2.2 Spatial indexing | Nearest-neighbor and radius queries; `BiomeFilter` composable filtering |
| 2.3 Graph construction | `EdgeStrategy` (KNN/WarpRange); incremental edge building |
| 2.4 Query engine | `FindQuery`, `ShowQuery`, `StatsQuery` — pure stateless functions over `&GalaxyModel` |
| 2.5 Display layer | Table formatters, portal hex-to-emoji, distance K/M suffixes |
| 2.6 `nms find` command | Search by biome, distance, discoverer with emoji glyph output |
| 2.7 `nms show` command | Detail views for systems (by name or hex) and bases |
| 2.8 `nms stats` command | Aggregate statistics with biome distribution table |

</details>

<details>
<summary>Phase 3 — REPL & Cache</summary>

| Milestone | Description |
|-----------|-------------|
| 3.1 REPL scaffold | reedline REPL loop with command parsing and dispatch |
| 3.2 History & key bindings | File-backed history at `~/.nms-copilot/history.txt` |
| 3.3 Tab completion | Context-aware completion for commands, flags, bases, systems, biomes |
| 3.4 Session context | Persistent position, biome filter, warp range; `set`/`reset`/`status` commands |
| 3.5 Context-aware prompt | Dynamic prompt showing galaxy, biome filter, and planet count |
| 3.6 rkyv serialization | Zero-copy cache format with `CacheData`; serialize/deserialize roundtrip |
| 3.7 Cache management | mtime-based freshness, `load_or_rebuild` startup path, `--no-cache` flag |
| 3.8 Config file | TOML config at `~/.nms-copilot/config.toml` — save path, display, defaults, cache |

</details>

<details>
<summary>Phase 4 — Routing</summary>

| Milestone | Description |
|-----------|-------------|
| 4.1 Shortest path | Dijkstra via `petgraph::algo::astar` with zero heuristic |
| 4.2 TSP nearest-neighbor | Greedy nearest-neighbor tour using Euclidean distances |
| 4.3 TSP 2-opt | Local search improvement over NN initial tour |
| 4.4 Hop-constrained routing | Waypoint insertion via R-tree when legs exceed warp range |
| 4.5 Route query | `RouteQuery` with biome/named/system-id targets, `execute_route()` pipeline |
| 4.6 Route display | Itinerary table with waypoint markers, warp jump counts, algorithm labels |
| 4.7 `nms route` command | CLI and REPL integration with session-aware defaults |

</details>

<details>
<summary>Phase 5 — Live Watch</summary>

| Milestone | Description |
|-----------|-------------|
| 5.1 Delta types | `SaveDelta`, `PlayerMoved` in `nms-core` for cycle-free sharing |
| 5.2 Snapshot diffing | `SaveSnapshot` lightweight extraction; `compute_delta()` comparison |
| 5.3 File watcher | `notify` debounced watcher on save directory; background thread |
| 5.4 Incremental model update | `GalaxyModel::apply_delta()` — systems, planets, bases, player position |
| 5.5 REPL integration | `drain_watch_events()` between prompts; human-readable notifications |
| 5.6 Cache write-through | `LoadResult` struct; cache updated after delta application |
| 5.7 Robustness | File stability checks, consecutive failure counting, graceful recovery |

</details>

<details>
<summary>Phase 6 — MCP Server</summary>

| Milestone | Description |
|-----------|-------------|
| 6.1 Server scaffold | fabryk-mcp server with stdio transport |
| 6.2 Info tool | `nms_info` MCP tool — save summary |
| 6.3 Find tool | `nms_find` MCP tool — planet search with filters |
| 6.4 Show tool | `nms_show` MCP tool — system/base details |
| 6.5 Stats tool | `nms_stats` MCP tool — aggregate statistics |
| 6.6 Route tool | `nms_route` MCP tool — route planning |
| 6.7 Convert tool | `nms_convert` MCP tool — coordinate conversion |
| 6.8 Live updates | Watcher integration for real-time model updates |
| 6.9 HTTP transport | Optional streaming HTTP server |

</details>

<details>
<summary>Phase 7 — Polish</summary>

| Milestone | Description |
|-----------|-------------|
| 7.1 Export command | JSON and CSV export of filtered planet data |
| 7.2 Shell completions | bash, zsh, fish, powershell, elvish |
| 7.3 Multi-save support | `--slot N` flag, `nms saves` listing |
| 7.4 Multi-galaxy indexes | Per-galaxy R-tree spatial indexes |
| 7.5 NomNom format | Save format detection for NomNom compatibility |
| 7.6 Community import | CSV import with duplicate detection |
| 7.7 Integration tests | End-to-end CLI and pipeline tests with fixtures |
| 7.8 CI workflows | GitHub Actions for check, test, and publish |
| 7.9 Publishing prep | crates.io metadata verification |
| 7.10 Color themes | Configurable ANSI color themes for terminal output |

</details>

---

## Installation

```bash
cargo install nms-copilot    # interactive REPL
cargo install nms-cli        # one-shot CLI (the `nms` binary)
cargo install nms-mcp        # MCP server for AI integration
```

Or build from source:

```bash
git clone https://github.com/oxur/nms-copilot
cd nms-copilot
make build
```

---

## Requirements

- **Rust** 1.85+ (2024 edition)
- **No Man's Sky** save files (Steam, GOG, or Mac)
- A terminal with emoji support (most modern terminals)

---

## Acknowledgements

NMS Copilot builds on a decade of community reverse engineering. Special thanks to:

- **[libNOM.io](https://github.com/zencq/libNOM.io)** / **[NomNom](https://github.com/zencq/NomNom)** by zencq — the most complete save format implementation
- **[MBINCompiler](https://github.com/monkeyman192/MBINCompiler)** by monkeyman192 — game data decompilation and key mapping
- **[Chase-san](https://gist.github.com/Chase-san/704284e4acd841471d9836e6bc296f2f)** — the cleanest minimal save decoder
- **[MetaIdea/nms-savetool](https://github.com/MetaIdea/nms-savetool)** — definitive format 2001 encryption documentation
- **[NMSCD](https://github.com/NMSCD)** — community developer tools and coordinate converters
- The **NMS Modding Discord** community — collective format knowledge
- **Hello Games** — for building a universe worth exploring 🌌

---

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

---

*"The universe is a pretty big place. It's good to have a copilot."* 🚀🦀

[//]: ---Named-Links---

[logo]: assets/images/logo/v1-x250.png
[logo-large]: assets/images/logo/v1.png
[build]: https://github.com/oxur/nms-copilot/actions/workflows/ci.yml
[build-badge]: https://github.com/oxur/nms-copilot/actions/workflows/ci.yml/badge.svg
[crate]: https://crates.io/crates/nms-copilot
[crate-badge]: https://img.shields.io/crates/v/nms-copilot.svg
[docs]: https://docs.rs/nms-copilot/
[docs-badge]: https://img.shields.io/badge/rust-documentation-blue.svg
[tag-badge]: https://img.shields.io/github/tag/oxur/nms-copilot.svg
[tag]: https://github.com/oxur/nms-copilot/tags
