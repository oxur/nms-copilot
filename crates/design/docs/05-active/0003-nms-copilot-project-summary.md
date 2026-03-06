---
number: 3
title: "NMS Copilot — Project Summary"
author: "the CLI"
component: All
tags: [change-me]
created: 2026-03-05
updated: 2026-03-05
state: Active
supersedes: null
superseded-by: null
version: 1.0
---

# NMS Copilot — Project Summary

> 🚀 A real-time galactic copilot for No Man's Sky, built in Rust.
> Search planets. Plan routes. Watch saves live. Let an AI explore with you.

---

## Overview

NMS Copilot is a Rust workspace that reads No Man's Sky save files — either raw binary (`save.hg`) or exported JSON — and builds a live, queryable, in-memory model of your galaxy. It supports three interfaces: a one-shot **CLI** for quick lookups, an interactive **REPL** for persistent exploration sessions alongside the game, and an **MCP server** that lets an AI assistant act as your real-time navigator and co-explorer.

The system watches your save files for changes as you play. When you warp to a new system, scan a planet, or build a base, the copilot detects the update and incorporates it into the model automatically. Portal glyphs render in multiple possible (configurable) formats.

### Crate Map

```
nms/                    Workspace root (umbrella crate)
├─ nms-core             Types, enums, address math, glyph emoji
├─ nms-save             Raw binary save parser (LZ4, XXTEA, key mapping)
├─ nms-compat           Format adapters (goatfungus JSON fixer)
├─ nms-graph            petgraph spatial model, R-tree index, routing
├─ nms-watch            notify file watcher, delta computation
├─ nms-cache            rkyv zero-copy serialization
├─ nms-query            Shared query engine (find, route, show, stats)
├─ nms-cli              clap one-shot commands
├─ nms-copilot          reedline interactive REPL (the main binary)
└─ nms-mcp              MCP server for AI integration
```

## Phase 0 - Prerequisites

Review game data research file `./crates/design/docs/06-final/0001-parsing-no-mans-sky-save-files-known-resources.md` and identify best quality C# project to use as reference implementation.

Clone identified project(s) to `./workbench`, analyse code, and create reference guide (Markdown, saved to ./workbench; Duncan will evaluate, add to docs, and provide final, canonical path for doc). Primary user of this doc will be AI (Claude Desktop/Claude Code).

Review open questions at end of Detailed Project Plan document, discuss with Duncan, perform necessary research, and followup. Integrate any resolved questions into project plans.

---

## Phase 1 — Foundation

**Build the bones: types, parsers, and the coordinate converter.**

This phase establishes the core type system (addresses, biomes, discoveries, planets, systems), implements both save file parsers (raw binary via LZ4 decompression and goatfungus JSON via state-machine fixer), and delivers the first working command: `nms convert`, which translates between portal glyphs (hex and emoji), signal booster coordinates, galactic addresses, and voxel positions — bidirectionally.

At the end of Phase 1, you can point the tool at any NMS save file and get a formatted summary of your discoveries, bases, and current position — with portal addresses rendered in emoji.

### Phase 1 Milestones

| # | Milestone | Crate(s) | Description | Deliverable |
|---|-----------|----------|-------------|-------------|
| 1.1 | Workspace scaffold | all | Cargo workspace with all crate stubs, CI pipeline, `data/` directory with `biomes.toml`, `glyphs.toml`, `galaxies.toml` | `cargo build` succeeds |
| 1.2 | Core types | nms-core | `GalacticAddress`, `Biome`, `BiomeSubType`, `Discovery`, `System`, `Planet`, `PlayerBase`, `PlayerState`, galaxy table | Unit tests for address encoding/decoding |
| 1.3 | Portal glyph converter | nms-core | Bidirectional: hex ↔ emoji ↔ coordinates ↔ signal booster. Parsing accepts hex digits, emoji, or glyph names | `nms convert --glyphs "🌅🕊️🐜🕊️"` works |
| 1.4 | Distance calculator | nms-core | Euclidean voxel distance × 400 ly, with helper methods (`within`, `same_region`, `same_system`) | Unit tests with known coordinate pairs |
| 1.5 | LZ4 block decompressor | nms-save | Read `save.hg`, detect format, parse block headers (magic `0xFEEDA1E5`), decompress with `lz4_flex`, concatenate | Decompresses a real save file to JSON |
| 1.6 | Metadata verifier | nms-save | Read `mf_save.hg`, XXTEA decrypt, verify SHA-256 and SpookyHash V2 | Passes verification on test save |
| 1.7 | Key deobfuscation | nms-save | Load `mapping.json`, recursive key replacement on parsed JSON | Obfuscated JSON → readable JSON |
| 1.8 | Serde deserialization | nms-save | Typed structs for save file top-level structure, `DiscoveryManagerData`, `PlayerStateData` (key fields) | Parse a full save into typed structs |
| 1.9 | Goatfungus JSON fixer | nms-compat | State-machine walker: fix `\xNN` → `\u00NN`, invalid escapes → double-backslash, inside strings only | Fixes the 22MB goatfungus export |
| 1.10 | `nms info` command | nms-cli | Save overview: version, platform, playtime, player location (with emoji glyphs), discovery counts, base count, biome summary | Formatted terminal output |
| 1.11 | `nms convert` command | nms-cli | Full coordinate converter CLI: `--glyphs`, `--coords`, `--ga`, `--voxel` flags, emoji output | Works standalone (no save file needed) |

---

## Phase 2 — Search & Display

**Load the galaxy into memory and make it searchable.**

This phase builds the in-memory galactic model: a petgraph of systems with an R-tree spatial index for fast geometric queries, plus hash-based lookup tables for filtering by biome, name, and discoverer. The query engine provides a shared API consumed by the CLI commands `find`, `show`, and `stats`. Output is formatted as pretty terminal tables with emoji glyphs and ANSI colors.

At the end of Phase 2, you can search for planets by biome ("find all Lush planets within 100K ly"), inspect any system or base in detail, and see aggregate statistics about your discoveries.

### Phase 2 Milestones

| # | Milestone | Crate(s) | Description | Deliverable |
|---|-----------|----------|-------------|-------------|
| 2.1 | Galaxy model | nms-graph | `GalaxyModel` struct: petgraph, R-tree spatial index, lookup HashMaps. Constructor from parsed `SaveData` | Model builds from test save |
| 2.2 | Spatial indexing | nms-graph | R-tree (`rstar`) insertion of all systems by voxel position. Nearest-N and within-radius queries | Benchmark: <10ms for nearest-10 on 300 systems |
| 2.3 | Graph construction | nms-graph | k-nearest-neighbor edge generation, warp-range-constrained edges, configurable strategy | Graph with edges for test data |
| 2.4 | Query engine | nms-query | `FindQuery`, `ShowQuery`, `StatsQuery` types. Pure functions over `&GalaxyModel` | Unit tests for each query type |
| 2.5 | Display layer | nms-query | Pretty table formatter with emoji glyphs, distance formatting (K/M ly), biome color coding, truncation | Readable terminal output |
| 2.6 | `nms find` command | nms-cli | Search by biome, infested flag, distance (within/nearest), name pattern, discoverer. Sorted results with glyphs | Full search working |
| 2.7 | `nms show` command | nms-cli | Detail view: system (all planets), planet (biome, seed, name), base (location, glyphs, type) | Detailed formatted output |
| 2.8 | `nms stats` command | nms-cli | Biome distribution table, distance histogram, discovery counts by type, named vs unnamed breakdown | Stats tables |

---

## Phase 3 — REPL & Caching

**Make it interactive and fast to start.**

This phase delivers the primary user experience: `nms-copilot`, an interactive REPL built on reedline with persistent history, tab completion of commands/biomes/base names, and a context system that maintains state across commands (current position, active filters, warp range). It also adds rkyv-based caching so subsequent startups are near-instant.

At the end of Phase 3, you can launch `nms-copilot`, set your position to a known base, filter to Lush planets, and interactively explore your galaxy data — all with sub-second startup from cache.

### Phase 3 Milestones

| # | Milestone | Crate(s) | Description | Deliverable |
|---|-----------|----------|-------------|-------------|
| 3.1 | REPL skeleton | nms-copilot | reedline loop, command parsing (reuse clap subcommands), exit/quit/help | Interactive prompt works |
| 3.2 | Persistent history | nms-copilot | History file at `~/.nms-copilot/history.txt`, up/down navigation, search | History persists across sessions |
| 3.3 | Tab completion | nms-copilot | Complete command names, biome names, base names, system names from loaded model | Context-aware completions |
| 3.4 | Session context | nms-copilot | `SessionState`: current position, active biome filter, warp range, last results. Commands: `set`, `reset`, `status` | `set position "Acadia National Park"` works |
| 3.5 | Context-aware prompt | nms-copilot | Prompt displays galaxy, active filters, model size: `[Euclid │ Lush │ 644 planets] 🚀` | Dynamic prompt |
| 3.6 | rkyv serialization | nms-cache | Serialize `GalaxyModel` (discovery data + metadata) to rkyv archive. Rebuild indices on load | Cache round-trips correctly |
| 3.7 | Cache management | nms-cache | Freshness check (save file mtime vs cache mtime), auto-rebuild on stale, `--no-cache` flag | Startup time: <100ms from cache |
| 3.8 | Config file | nms-copilot | `~/.nms-copilot/config.toml` for save path, defaults, display prefs. `toml` crate parsing | Config loaded on startup |

---

## Phase 4 — Graph Routing

**Plan routes through the stars.**

This phase adds pathfinding and route optimization to the graph model: Dijkstra shortest path between any two systems, TSP traversal for visiting a set of targets (e.g., "all Scorched planets within 500K ly"), and hop-constrained routing that respects actual ship warp ranges. The `route` command exposes all of this through the CLI and REPL.

At the end of Phase 4, you can say "plan a route visiting every Lava planet using my S-class hyperdrive" and get a step-by-step itinerary with portal glyphs and distances for each hop.

### Phase 4 Milestones

| # | Milestone | Crate(s) | Description | Deliverable |
|---|-----------|----------|-------------|-------------|
| 4.1 | Dijkstra shortest path | nms-graph | Shortest path between two systems using petgraph's Dijkstra. Warp-range-limited edges | Path with total distance and hop count |
| 4.2 | Nearest-neighbor TSP | nms-graph | Greedy nearest-neighbor traversal of a target set. Returns ordered visit list with total distance | Route through all targets |
| 4.3 | 2-opt improvement | nms-graph | 2-opt local search on TSP solution. Iterates until no improving swap found | Measurably shorter routes than greedy |
| 4.4 | Hop-constrained routing | nms-graph | Route that respects max warp range per hop. Inserts intermediate waypoints if direct hop exceeds range | Realistic routes for given ship class |
| 4.5 | Route query type | nms-query | `RouteQuery` with target selection (biome filter, explicit list, within radius), algorithm choice, warp range | Unified routing API |
| 4.6 | Route display | nms-query | Step-by-step itinerary table: hop #, system name, portal glyphs (emoji), distance, cumulative distance | Pretty route output |
| 4.7 | `nms route` command | nms-cli, nms-copilot | `route --biome Scorched --warp-range 2500 --algo 2opt --from "Acadia National Park"` | Full routing CLI + REPL |
| 4.8 | Reachability analysis | nms-graph | Connected components within a given warp range. "Which systems can I reach from here?" | Component membership + visualization |

---

## Phase 5 — Live Watch

**Stream the game into the copilot.**

This phase makes the copilot reactive: a background file watcher monitors the NMS save directory, detects changes when the game auto-saves, re-parses the save, computes a delta against the current model, and applies incremental updates. The REPL displays real-time notifications when new discoveries are detected or the player moves.

At the end of Phase 5, you can play NMS with the copilot running alongside it. Every time you warp, scan, or build, the copilot knows — instantly.

### Phase 5 Milestones

| # | Milestone | Crate(s) | Description | Deliverable |
|---|-----------|----------|-------------|-------------|
| 5.1 | File watcher | nms-watch | `notify` crate monitoring save directory, debounced (500ms) change detection | Detects save file writes |
| 5.2 | Delta computation | nms-watch | Diff current model against re-parsed save: new discoveries, player position change, new bases | `SaveDelta` type with typed variants |
| 5.3 | Incremental model update | nms-graph | `apply_delta()`: insert new systems/planets into graph + spatial index, update player position | Model stays current |
| 5.4 | Event channel | nms-watch | `tokio::sync::broadcast` or `crossbeam` channel distributing deltas to consumers | Multiple consumers receive updates |
| 5.5 | REPL integration | nms-copilot | Background listener prints notifications: `🌅 New scan: "Metok-Kalpa" (Lush) at 🌅🕊️🐜🕊️...` | Live REPL notifications |
| 5.6 | Cache invalidation | nms-cache | Watcher triggers cache update after applying deltas | Cache stays fresh during play |
| 5.7 | Graceful handling | nms-watch | Handle mid-write detection (partial files), permission errors, save directory not found | Robust in real-world conditions |

---

## Phase 6 — MCP Server & AI Copilot

**Let an AI play with you.**

This phase exposes the full galactic model as an MCP (Model Context Protocol) server, enabling Claude or another AI assistant to query your galaxy in real time during gameplay. The server receives the same live update stream as the REPL, so the AI always has current data. Tool calls cover search, routing, system details, coordinate conversion, and situational awareness ("what's near me right now?").

At the end of Phase 6, you can have a conversation with Claude while playing NMS, and Claude can answer questions like "Where's the nearest Lush planet?", "Plan a route through all my undiscovered biomes", or "What system am I in?" — using live data from your save file.

### Phase 6 Milestones

| # | Milestone | Crate(s) | Description | Deliverable |
|---|-----------|----------|-------------|-------------|
| 6.1 | MCP protocol layer | nms-mcp | JSON-RPC over stdio transport, tool registration, request/response handling | Server responds to tool calls |
| 6.2 | `search_planets` tool | nms-mcp | Search by biome, distance, name. Returns structured results with emoji glyphs | AI can search planets |
| 6.3 | `plan_route` tool | nms-mcp | Route planning with biome targets, warp range, algorithm selection | AI can plan routes |
| 6.4 | `where_am_i` tool | nms-mcp | Current player location from live model: system, planet, coordinates, emoji glyphs | AI knows your position |
| 6.5 | `whats_nearby` tool | nms-mcp | Nearby systems/planets sorted by distance, with biome and discovery info | AI provides situational awareness |
| 6.6 | `show_system` / `show_base` | nms-mcp | Detail views for systems and bases | AI can inspect specific locations |
| 6.7 | `convert_coordinates` tool | nms-mcp | Bidirectional glyph/coord conversion | AI can translate addresses |
| 6.8 | Live update integration | nms-mcp | Server subscribes to watcher delta channel, model stays current | AI has real-time data |
| 6.9 | SSE transport (optional) | nms-mcp | Server-sent events transport for web-based AI clients | Works with Claude web |

---

## Phase 7 — Polish & Ecosystem

**Finish, document, and ship.**

This phase covers everything needed for a public release: export commands (JSON, CSV), shell completions, multi-save support, NomNom format compatibility, comprehensive documentation, and crates.io publishing. It also includes quality-of-life improvements discovered during real-world use in Phases 1–6.

### Phase 7 Milestones

| # | Milestone | Crate(s) | Description | Deliverable |
|---|-----------|----------|-------------|-------------|
| 7.1 | `nms export` command | nms-cli | Export filtered results as JSON, CSV. `--biome`, `--within`, `--format` flags | Scriptable data extraction |
| 7.2 | Shell completions | nms-cli | `clap_complete` for bash, zsh, fish | Install instructions in README |
| 7.3 | Multi-save support | nms-save | Detect and list all save slots. `--slot` flag or interactive selector | Choose between save slots |
| 7.4 | Multi-galaxy support | nms-graph | Separate spatial index per galaxy, cross-galaxy queries, galaxy switching in REPL | Full 256-galaxy support |
| 7.5 | NomNom format | nms-compat | Parser for NomNom export format (if distinct from standard deobfuscated JSON) | Additional format support |
| 7.6 | Documentation | all | README with screenshots/examples, rustdoc for all public APIs, CONTRIBUTING guide | Publish-ready docs |
| 7.7 | Integration tests | all | End-to-end tests with fixture save files covering all commands | CI green with full coverage |
| 7.8 | crates.io publishing | all | Publish `nms`, `nms-core`, `nms-save`, `nms-graph`, `nms-copilot` to crates.io | `cargo install nms-copilot` works |
| 7.9 | Community data import | nms-graph | Import external coordinate lists (NMSCE, community CSV) as "external" discoveries | Enriched galaxy model |
| 7.10 | Color themes | nms-copilot | Configurable color schemes for terminal output, biome color mapping | Pretty by default, customizable |

---

## Quick Reference

| What | Where |
|------|-------|
| Repo | `github.com/oxur/nms` |
| Main binary | `nms-copilot` |
| Config | `~/.nms-copilot/config.toml` |
| Cache | `~/.nms-copilot/galaxy.rkyv` |
| History | `~/.nms-copilot/history.txt` |
| Bundled data | `data/biomes.toml`, `glyphs.toml`, `galaxies.toml` |
| Key mapping | `data/mapping.json` (from MBINCompiler) |

| Interface | Purpose | Startup |
|-----------|---------|---------|
| `nms <command>` | One-shot CLI | Parse or cache |
| `nms-copilot` | Interactive REPL | Cache + file watcher |
| `nms-mcp` | AI integration | Cache + file watcher + MCP protocol |

| Portal Glyph Emoji | | | |
|---|---|---|---|
| 0 🌅 Sunset | 4 🌜 Eclipse | 8 🦋 Dragonfly | C 🚀 Rocket |
| 1 🕊️ Bird | 5 🎈 Balloon | 9 🌀 Galaxy | D 🌳 Tree |
| 2 😑 Face | 6 ⛵ Boat | A 🕋 Voxel | E 🔺 Atlas |
| 3 🦕 Diplo | 7 🐜 Bug | B 🐋 Whale | F ⚫ BlackHole |

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
