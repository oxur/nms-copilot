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

## Features

- **Native save file parsing** -- reads `save.hg` directly (LZ4 block decompression + JSON key deobfuscation), no export step needed
- **Planet search** -- find planets by biome, distance, discoverer, infested status, or any combination
- **Route planning** -- nearest-neighbor and 2-opt TSP solvers with warp-range hop constraints
- **Portal glyph converter** -- fully multidirectional: hex, emoji, coordinates, signal booster, galactic address
- **Interactive galaxy map** -- full-screen TUI with galaxy, region, and local zoom levels
- **Live file watching** -- detects auto-saves while you play and updates the model in real time
- **rkyv cache** -- zero-copy serialization for near-instant startup after the first load
- **Multi-save support** -- switch between save slots (up to 15)
- **Export & import** -- JSON/CSV export of filtered data; CSV import of community coordinates
- **MCP server** -- stdio and HTTP transports for AI copilot integration (Claude Desktop, etc.)
- **Configurable color themes** -- ANSI terminal themes via `~/.nms-copilot/config.toml`
- **Shell completions** -- bash, zsh, fish, powershell, elvish
- **Multi-galaxy routing** -- per-galaxy spatial indexes across all 256 NMS galaxies

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

All commands below work with both the CLI (`nms`) and the REPL (`nms-copilot`), unless noted otherwise. The CLI accepts `--save` and `--slot` flags; the REPL uses its pre-loaded model.

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
nms route --target "Base A" --target "Base B"    # explicit waypoints
nms route --round-trip                           # return to start
```

### Info & Details

```bash
nms info                              # save overview, player location, discovery counts
nms show system 369                   # system details + all planets
nms show base "Acadia National Park"  # base details with portal glyphs
nms stats --biomes                    # biome distribution table
nms stats --discoveries               # discovery counts by type
nms saves                             # list all save slots
```

### Coordinate Conversion

```bash
nms convert --glyphs 01717D8A4EA2                  # hex glyphs to all formats
nms convert --glyphs "🌅🕊️🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑"  # emoji glyphs to all formats
nms convert --coords 0EA2:007D:08A4:0171           # signal booster format
nms convert --ga 0x40050003AB8C07                   # galactic address
nms convert --voxel 100,50,-200 --ssi 42           # voxel coordinates
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

The REPL (`nms-copilot`) supports all the commands above plus session management and an interactive galaxy map:

```bash
nms-copilot

[Euclid │ 644 planets │ 293 systems] 🚀 set position "Acadia National Park"
📍 Position set to Acadia National Park (Lush, Gugestor Colony)

[Euclid │ 644 planets │ 293 systems] 🚀 find --biome Lava --nearest 3
  #  Planet       Biome  Distance   Portal Glyphs
  1  (unnamed)    Lava     127K ly  🌅🦕🌀🕊️🐜🌳🌜🕋🌅🌀🕋🦕
  2  (unnamed)    Lava     204K ly  🌅🌜🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑
  3  (unnamed)    Lava     318K ly  🌅😑🐜🕊️🐜🌅🌜🕋🌅🔺🕋🐜

[Euclid │ 644 planets │ 293 systems] 🚀 list bases --limit 5
  #  Base                    System            Planet          Biome
  1  Acadia National Park    Gugestor Colony   Metok-Kalpa     Lush
  2  Sealab 2038             Esurad            Sushimi         Lush
  ...

[Euclid │ 644 planets │ 293 systems] 🚀 map
  (opens full-screen interactive galaxy map with zoom levels)
```

REPL-only commands:

| Command | Description |
|---------|-------------|
| `set position <base>` | Set reference position for distance calculations |
| `set biome <biome>` | Set default biome filter for find/route |
| `set warp-range <ly>` | Set default warp range for route planning |
| `reset [position\|biome\|warp-range\|all]` | Reset session state |
| `status` | Show current session state |
| `list bases\|systems\|galaxies\|biomes\|glyphs\|terrain-types` | Browse reference data and discoveries |
| `map` | Interactive galaxy map (galaxy/region/local zoom) |

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
