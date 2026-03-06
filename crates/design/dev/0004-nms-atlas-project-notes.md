# NMS Atlas — Project Notes

## Name & Repo

- **Project**: nms-atlas (CLI tool name: `nms`)
- **Repo**: oxur/nms (<https://github.com/oxur/nms>)
- **Working dir**: ~/lab/oxur/nms/

## Crate Structure (Rust workspace)

```
nms/
├── Cargo.toml                 # Workspace root
├── crates/
│   ├── nms-core/              # Core types, enums, address math
│   ├── nms-parse/             # Save file parsers (extensible, trait-based)
│   ├── nms-graph/             # petgraph-based spatial model & routing
│   └── nms-cli/               # clap CLI: find, route, convert, show, stats, export
├── data/                      # Reference data (biomes.toml etc.)
└── tests/fixtures/            # Test save snippets
```

## CLI Commands (planned)

- `nms info <save>` — save overview
- `nms find` — search planets by biome, distance, name, discoverer
- `nms route` — TSP traversal with warp-range constraints
- `nms convert` — bidirectional glyph/coord/GA converter (with emoji)
- `nms show` — system/planet/base details
- `nms stats` — biome distribution, distance histograms
- `nms export` — filtered JSON/CSV

## Key Dependencies

- `clap` (derive), `serde`/`serde_json`, `petgraph`, `chrono`
- Table display: `comfy-table` or `tabled`
- Colors: `colored` or `owo-colors`

## Implementation Phases

1. **Foundation** — workspace, core types, JSON fixer, parser, address codec, `nms convert`
2. **Search** — in-memory index, `nms find`, `nms show`, `nms stats`
3. **Graph & Routing** — petgraph, distance edges, TSP, `nms route`
4. **Polish** — export, multiple formats, shell completions, config

## Key Technical Facts

- Save JSON (from goatfungus) has `\xNN` escapes → needs state-machine fixer
- Biome = VP[1] lower 16 bits; 0x10000 flag = Infested
- Custom names in DM.CN (~13% of planets)
- Portal glyphs = 12 hex digits from UA: P SSS YY ZZZ XXX
- Distance ≈ voxel Euclidean × 400 ly
- ~644 planets, ~293 systems in a typical save
