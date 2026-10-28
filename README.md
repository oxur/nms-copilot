# 🚀 NMS Copilot

[![][build-badge]][build]
[![][crate-badge]][crate]
[![][tag-badge]][tag]
[![][docs-badge]][docs]
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**A real-time galactic copilot for [No Man's Sky](https://www.nomanssky.com/), built in Rust.**

[![][logo]][logo-large]

Search planets by biome. Plan warp routes through the stars. Convert portal glyphs with emoji. Watch your save file live as you play — and let an AI explore the galaxy *with* you.

An illustrative REPL session:

```text
[Euclid | 644 planets] 🚀 find --biome Lush --nearest 5

  #  Planet            Biome   System             Distance   Portal Glyphs
  1  Metok-Kalpa       Lush    Gugestor Colony       0 ly    🌅🕊️🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑
  2  Sushimi           Lush    Esurad               18K ly   🌅🕊️🐜🦕🌜🎈⛵🐜🦋🌀🕋🐋
  3  (unnamed)         Lush    Ogsjov XV            42K ly   🌅😑🐜🕊️🐜🌳🌜🕋🌅🔺🕋🦕
  4  (unnamed)         Lush    Rastarc-Zukk         67K ly   🌅🦕🐜🕊️🐜🌅🦋🕋🌜🔺🕋🐜
  5  Dipadri Grosso    Lush    Ipswic               91K ly   🌅🌜🐜🕊️🐜🌳🌜🕋🌅🌀🕋😑
```

---

## What is this?

NMS Copilot reads your No Man's Sky save files — native `save.hg` files or supported JSON — and builds a live, in-memory atlas of the systems, planets, and bases recorded there. Search the places you've discovered, plan a journey from one of your bases, or explore another galaxy's discoveries. Your save files stay untouched.

Three interfaces, one queryable atlas:

| Interface | What it does |
|-----------|-------------|
| **CLI** (`nms`) | One-shot commands for quick lookups and scripted pipelines |
| **REPL** (`nms-copilot`) | Interactive exploration with command history, tab completion, session settings, and a galaxy map |
| **MCP server** | Let an AI assistant query the atlas alongside the REPL, or run it on its own with `--headless` |

Keep the REPL or headless server running alongside the game. As the selected save changes, the atlas follows your location, discoveries, names, and bases. The REPL and its MCP server share that model, so you and your AI copilot explore the same data. Each one-shot CLI command reads the selected save afresh.

---

## Features

- **Native save parsing** — read `save.hg` directly, with no export step
- **Planet search** — combine biome, distance, discoverer, named-planet, and infested filters
- **Route planning** — order visits to discovered systems with nearest-neighbor or 2-opt heuristics and optional warp-range guidance
- **Galaxy-aware exploration** — keep identical coordinates in different galaxies distinct; select a galaxy view in the REPL
- **Portal glyph conversion** — convert between hex, emoji, voxel coordinates, signal booster coordinates, and packed portal addresses
- **Interactive map** — explore galaxy, region, and local views in a full-screen terminal map
- **Live sessions** — follow accepted save updates in the REPL and MCP server, with a disk cache for subsequent starts
- **Save slots and interchange** — choose among 15 save slots, export filtered planets as JSON/CSV, and read community coordinate CSVs
- **AI co-exploration** — query planets, routes, bases, and current location through stdio or HTTP MCP
- **Terminal workflow** — command history, tab completion, session defaults, last-result recall, and shell completion scripts

---

## Portal Glyphs

NMS Copilot renders portal addresses as emoji throughout all interfaces:

| Index | Name | Hex | Emoji | Unicode |
|-------|------|-----|-------|---------|
| 0 | Sunset | 0 | 🌅 | U+1F305 |
| 1 | Bird | 1 | 🕊️ | U+1F54A U+FE0F |
| 2 | Face | 2 | 😑 | U+1F611 |
| 3 | Diplo | 3 | 🦕 | U+1F995 |
| 4 | Eclipse | 4 | 🌜 | U+1F31C |
| 5 | Balloon | 5 | 🎈 | U+1F388 |
| 6 | Boat | 6 | ⛵ | U+26F5 |
| 7 | Bug | 7 | 🐜 | U+1F41C |
| 8 | Dragonfly | 8 | 🦋 | U+1F98B |
| 9 | Galaxy | 9 | 🌀 | U+1F300 |
| 10 | Voxel | A | 🕋 | U+1F54B |
| 11 | Whale | B | 🐋 | U+1F40B |
| 12 | Tent | C | ⛺ | U+26FA |
| 13 | Rocket | D | 🚀 | U+1F680 |
| 14 | Tree | E | 🌳 | U+1F333 |
| 15 | Atlas | F | 🔺 | U+1F53A |

Convert freely between formats:

```bash
# Emoji → coordinates
nms convert --glyphs "🌅🕊️🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑"

# Hex glyphs → coordinates
nms convert --glyphs 01717D8A4EA2

# Signal booster → emoji glyphs
nms convert --coords 0EA2:007D:08A4:0171

# Packed portal address → everything
nms convert --ga 0x01717D8A4EA2
```

---

## Commands

The examples below use the one-shot CLI (`nms`). In the REPL, omit `nms` for the shared `find`, `show`, `stats`, `route`, `convert`, and `list` commands. The CLI also provides `saves`, `export`, `import`, and `completions`.

Save-backed CLI commands accept `--save /path/to/save.hg` or `--slot N`; otherwise they use the most recently modified discovered save. The REPL works with the save chosen at startup. Use `nms <command> --help` for all options. See [save selection and configuration](docs/configuration.md) for setup and precedence.

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

Plan a tour of discovered systems, starting from your current location or a named base:

```bash
nms route --biome Scorched --algo nn             # nearest-neighbor tour
nms route --biome Scorched --within 500000       # only within radius
nms route --biome Lush --warp-range 2500         # plan with a 2500 ly warp range
nms route --biome Frozen --algo 2opt             # improved TSP
nms route --target "Base A" --target "Base B"    # explicit waypoints
nms route --biome Lush --round-trip              # return after visiting lush targets
```

Routes stay within one galaxy. Warp-range planning is best-effort; inspect the itinerary for legs beyond your ship's range.

### Info & Details

```bash
nms info                              # save overview, player location, discovery counts
nms show system 369                   # system details + all planets
nms show base "Acadia National Park"  # base details with portal glyphs
nms stats --biomes                    # biome distribution table
nms saves                             # list all save slots
```

### Coordinate Conversion

```bash
nms convert --glyphs 01717D8A4EA2                  # hex glyphs to all formats
nms convert --glyphs "🌅🕊️🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑"  # emoji glyphs to all formats
nms convert --coords 0EA2:007D:08A4:0171           # signal booster format
nms convert --ga 0x01717D8A4EA2                   # packed portal address
nms convert --voxel 100,50,-200 --ssi 42           # voxel coordinates
```

### Export & Import

Export filtered planet records for a spreadsheet or another tool:

```bash
nms export --format json                          # export matching planets as JSON
nms export --biome Lush --format csv              # export filtered planets as CSV
nms import community_data.csv --source "NMSCE"    # import community coordinates
```

The import command reads the CSV into a temporary model and reports added, duplicate, and skipped rows. It does not persist an atlas for later commands. Library callers can retain imported systems and their community source labels in a model and compatible cache. See [data interchange](docs/data-interchange.md) for formats and source handling.

### Listing Data

Browse reference data and model collections:

```bash
nms list galaxies                      # all 256 galaxies
nms list galaxies --type Lush          # filter by galaxy type
nms list biomes                        # biome types and variants
nms list glyphs                        # portal glyph table
nms list terrain-types                 # terrain generation types
nms list bases                         # all player bases
nms list systems --limit 10            # first 10 systems in the atlas
nms list systems --all                 # all systems in the atlas
```

### Shell Completions

Generate a script for Bash, Zsh, Fish, PowerShell, or Elvish. For Bash:

```bash
mkdir -p "$HOME/.bash_completion.d"
nms completions bash > "$HOME/.bash_completion.d/nms"
source "$HOME/.bash_completion.d/nms"
```

Add the `source` line to `~/.bashrc` to load it in future sessions. For other shells, use `nms completions <shell>`; the [shell completion guide](docs/shell-completions.md) includes setup for Zsh and Fish.

### Multi-Save Support

`nms saves` lists discovered accounts and their manual/auto save files. `--slot` chooses the newest file in that slot from the first name-sorted account; an explicit `--save` path takes precedence.

```bash
nms info --slot 3                   # use save slot 3 instead of most recent
nms find --slot 5 --biome Lush      # search slot 5's discoveries
```

### Interactive REPL

Start with a save file, then explore without repeating its path. The transcript below is illustrative:

```text
$ nms-copilot --save /path/to/save.hg

[Euclid | 644 planets] 🚀 set position "Acadia National Park"
Position set to Acadia National Park

[Euclid | 644 planets] 🚀 find --biome Lava --nearest 3
  #  Planet       Biome  Distance   Portal Glyphs
  1  (unnamed)    Lava     127K ly  🌅🦕🌀🕊️🐜🌳🌜🕋🌅🌀🕋🦕
  2  (unnamed)    Lava     204K ly  🌅🌜🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑
  3  (unnamed)    Lava     318K ly  🌅😑🐜🕊️🐜🌅🌜🕋🌅🔺🕋🐜

[Euclid | 644 planets] 🚀 last
  (recalls the previous query response)

[Euclid | 644 planets] 🚀 galaxy Eissentam
  (selects the Eissentam view)

[Eissentam | 644 planets] 🚀 map
  (opens the full-screen galaxy map)
```

Selecting a galaxy changes the view; your physical location still comes from the save. To search from a base in another galaxy, use `--from "Base Name"` or `set position "Base Name"`. Reset position to follow the player again.

| REPL command | What it does |
|--------------|--------------|
| `galaxy [index-or-name]` | Show discovered galaxies or select a galaxy view |
| `set position <base>` | Use a named base as the distance reference |
| `set biome <biome>` | Set the default biome for searches and routes |
| `set warp-range <ly>` | Set the default route warp range |
| `reset [position\|biome\|warp-range\|all]` | Reset session settings |
| `status` | Show the reference position and session settings |
| `last` | Recall the last successful `find`, `show`, `stats`, or `route` response in this session |
| `map` | Open a map snapshot; background save updates continue while it is open |
| `help` / `exit` / `quit` | Show help or end the session |

`last` keeps the response as it appeared when you ran the query. Run the query again for current data. `info` reports the loaded model and physical player location; `status` reports your session settings. See [REPL exploration](docs/exploration.md) for galaxy views, reference positions, and session behavior.

---

## Architecture

NMS Copilot is a Rust workspace of focused crates:

```
nms/
├─ nms-core       Types, enums, address math, glyph emoji
├─ nms-save       Raw binary save parser (LZ4 + XXTEA + key mapping)
├─ nms-compat     Experimental JSON compatibility helpers
├─ nms-graph      petgraph spatial model, R-tree index, routing
├─ nms-query      Shared query engine (find, route, show, stats)
├─ nms-watch      File watcher, delta computation, live updates
├─ nms-cache      Versioned disk cache and model reconstruction
├─ nms-cli        clap one-shot CLI (the `nms` binary)
├─ nms-mcp        Reusable MCP server, tools, and resources
├─ nms-copilot    Interactive REPL, map, and live-session coordination
└─ nms            Workspace documentation and package landing page
```

All three interfaces use the same query engine:

```
save file → parser → galaxy model → query engine → CLI / REPL / MCP
                          ↑
               file watcher (live updates)
```

The galaxy model combines a system graph, per-galaxy spatial indexes, and lookups for planets, names, and bases. One application coordinator applies accepted save updates and maintains the cache for the REPL or headless server. The map uses its own snapshot so exploration does not hold up live updates.

### How Save Parsing Works

Native `save.hg` data contains **LZ4 block-compressed JSON**. The parser also accepts supported plaintext JSON, skipping decompression. The native pipeline:

1. Read sequential 16-byte block headers (magic `0xFEEDA1E5`) + LZ4 payloads
2. Decompress and concatenate all blocks
3. Deobfuscate JSON keys using MBINCompiler's `mapping.json`
4. Validate modeled addresses, then build typed save data and the atlas

Modern save data (format 2002+, post-Frontiers) is not encrypted. Separate metadata files use XXTEA. If a live save fails validation, the copilot keeps its last accepted state and waits for a valid update.

The checked parser, graph, cache, watcher, query, and MCP APIs are available as Rust libraries. See the [API documentation][docs] and [contributor guide](CONTRIBUTING.md) for the workspace details.

### MCP Server

The standard REPL build also starts an HTTP MCP server at `http://127.0.0.1:5099/mcp`. Connect an AI assistant that supports MCP to search planets, plan routes, look up bases, and follow your current location through the shared atlas.

For a server without the interactive REPL:

```bash
nms-copilot --headless --save /path/to/save.hg                        # stdio
nms-copilot --headless --http 127.0.0.1:5099 --save /path/to/save.hg   # HTTP
```

Stdio reserves stdout for the MCP protocol. Both modes follow the selected save when watching is enabled. Clients can read current location with `where_am_i` or subscribe to `nms://player/location` updates.

Watch and cache settings live in `~/.nms-copilot/config.toml`:

```toml
[watch]
enabled = true
debounce_ms = 500

[cache]
enabled = true
```

The cache is separate from your save and is rebuilt when stale or incompatible. Use `--no-cache` to bypass it. If you retain community data only in a library-managed cache, keep its source CSV: a rebuild from the game save cannot restore those imported records. The guides cover [live updates and caching](docs/live-atlas.md) and [HTTP MCP connections](docs/mcp-http.md).

---

## Installation

```bash
cargo install nms-copilot    # interactive REPL + MCP server
cargo install nms-cli        # one-shot CLI (the `nms` binary)
```

Or build from source:

```bash
git clone https://github.com/oxur/nms-copilot
cd nms-copilot
make build
```

From the checkout, try a synthetic save without accessing your game data:

```bash
cargo run -p nms-cli --bin nms -- find --save data/test/save56_multi_system.json --biome Lush
```

For your own save, pass `--save /path/to/save.hg`. Without an explicit path, the REPL uses configured or discovered saves and offers interactive setup if none is found. Run `nms-copilot --setup` to open the questionnaire yourself.

---

## Guides

- [Configuration and save selection](docs/configuration.md) — choose a save and configure watching, caching, and the server
- [REPL exploration](docs/exploration.md) — galaxy views, reference positions, session defaults, and the map
- [Data interchange](docs/data-interchange.md) — export planet records and read community coordinate catalogs
- [Live updates and caching](docs/live-atlas.md) — how the atlas follows your save and handles cache rebuilds
- [Shell completions](docs/shell-completions.md) — set up completion for your shell
- [HTTP MCP](docs/mcp-http.md) — connect a client, subscribe to resources, and check the server

For development, see [CONTRIBUTING.md](CONTRIBUTING.md). [CLI captures](docs/captures/README.md) provides recorded example output.

---

## Requirements

- **Rust** 1.91+ (2024 edition)
- **No Man's Sky** save files (Steam, GOG, or Mac)
- A terminal with emoji support (most modern terminals)

---

## Acknowledgements

NMS Copilot builds on a decade of community reverse engineering. Special thanks to:

- **[libNOM.io](https://github.com/zencq/libNOM.io)** / **[NomNom](https://github.com/zencq/NomNom)** by zencq — community save-format research
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

[logo]: assets/images/logo/v2-x250.jpg
[logo-large]: assets/images/logo/v2.jpg
[build]: https://github.com/oxur/nms-copilot/actions/workflows/ci.yml
[build-badge]: https://github.com/oxur/nms-copilot/actions/workflows/ci.yml/badge.svg
[crate]: https://crates.io/crates/nms-copilot
[crate-badge]: https://img.shields.io/crates/v/nms-copilot.svg
[docs]: https://docs.rs/nms-copilot/
[docs-badge]: https://img.shields.io/badge/rust-documentation-blue.svg
[tag-badge]: https://img.shields.io/github/tag/oxur/nms-copilot.svg
[tag]: https://github.com/oxur/nms-copilot/tags
