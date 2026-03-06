# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

NMS Copilot is a real-time galactic copilot for No Man's Sky, built in Rust. It reads NMS save files (raw binary `save.hg` or exported JSON), builds a live in-memory model of discovered systems/planets/bases, and exposes it through three interfaces: a one-shot CLI (`nms`), an interactive REPL (`nms-copilot`), and an MCP server (`nms-mcp`). It is **not** a save editor — it is a queryable atlas.

**Status:** Pre-Phase 1 — design documents and project scaffolding only. No Cargo.toml or Rust source code exists yet.

## Document Hierarchy

For Rust code quality (once code exists):

1. `assets/ai/ai-rust/skills/claude/SKILL.md` — Advanced Rust programming skill (primary reference)
2. `assets/ai/ai-rust/guides/*.md` — Comprehensive Rust guidelines referenced by the skill
3. `assets/ai/CLAUDE-CODE-COVERAGE.md` — Test coverage guide (target: 95%+)
4. This file — Project-specific conventions

**Note:** `assets/ai/ai-rust` is a symlink to `~/lab/oxur/ai-rust`. If it doesn't resolve, clone it: `git clone https://github.com/oxur/ai-rust assets/ai/ai-rust`

## Design Documents

Managed via ODM (`./bin/odm`). Key docs:

- `./bin/odm show 1` — Known resources for parsing NMS save files (Final)
- `./bin/odm show 2` — Detailed project plan with crate specs, dependency graph, and all 7 phases (Active)
- `./bin/odm show 3` — Project summary with milestone tables per phase (Active)

Doc paths: `crates/design/docs/` (finalized) and `crates/design/dev/` (working notes).

## Crate Architecture

```
nms/                    Workspace root
├─ nms-core             Types, enums, address math, glyph emoji (zero heavy deps)
├─ nms-save             Raw binary save parser (LZ4 + XXTEA + key mapping)
├─ nms-compat           Format adapters (goatfungus JSON fixer)
├─ nms-graph            petgraph spatial model, R-tree index, routing (the brain)
├─ nms-query            Shared query engine (find, route, show, stats)
├─ nms-watch            notify file watcher, delta computation, event stream
├─ nms-cache            rkyv zero-copy serialization for fast startup
├─ nms-cli              clap one-shot CLI (`nms` binary)
├─ nms-copilot          reedline interactive REPL (`nms-copilot` binary)
└─ nms-mcp              MCP server for AI integration
```

Data flow: `save file → parser → galaxy model → query engine → CLI / REPL / MCP`

**nms-graph is the core.** Everything upstream feeds into it; everything downstream queries from it. All three interfaces share `nms-query` — no duplicated logic.

## Build & Test Commands

Once the workspace exists:

```bash
make build          # Build all crates
make test           # Run all tests
make lint           # Clippy linting
make format         # rustfmt formatting
make coverage       # Code coverage (target: 95%+)
cargo test -p nms-core          # Test a single crate
cargo test -p nms-save -- test_name  # Run a single test
```

Always run `make format` after changes, then `make lint` before testing.

## Key Technical Details

### Save File Parsing Pipeline

1. Read `save.hg` — detect format (plaintext JSON vs LZ4 compressed)
2. Parse sequential LZ4 blocks (magic `0xFEEDA1E5`), decompress, concatenate
3. Deobfuscate JSON keys using MBINCompiler's `mapping.json`
4. Deserialize into typed Rust structs via serde

No encryption on modern saves (format 2002+, post-Frontiers). XXTEA only on metadata file `mf_save.hg`.

### Portal Glyph System

16 glyphs (0-F) rendered as emoji throughout all interfaces. The converter is fully multidirectional: index, name, hex, emoji, coordinates, signal booster format — all interconvertible. See README.md for the full glyph table.

### Galactic Address

`GalacticAddress` — 48-bit packed coordinate: VoxelX/Y/Z (signed), SolarSystemIndex, PlanetIndex, RealityIndex (galaxy 0-255). Distance = Euclidean voxel distance × 400 ly.

## Conventions

- Test naming: `test_<fn>_<scenario>_<expectation>`
- Load `11-anti-patterns.md` from ai-rust guides before writing Rust code
- Reference data ships in `data/` directory: `biomes.toml`, `glyphs.toml`, `galaxies.toml`, `mapping.json`
- Config location: `~/.nms-copilot/config.toml`
- Table output uses `oxur-table` crate (see `oxur-odm` for usage examples)
- Logging via `twyg`, config via `confyg`

## Workbench

`workbench/` is gitignored and contains local tools and reference implementations (e.g., NMSSaveEditor). Not part of the project source.
