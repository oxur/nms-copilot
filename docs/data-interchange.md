# Exporting planets and importing community coordinates

Use `nms export` to send filtered planet records to another tool. Use `nms import` to read a community coordinate CSV into a temporary model and inspect the import counts. Neither command edits the game save.

## Exporting query results

```bash
nms export --save /path/to/save.hg --format json > planets.json
nms export --save /path/to/save.hg --biome Lush --format csv > lush-planets.csv
nms export --save /path/to/save.hg --from "Home" --within 100000 --nearest 10
```

Export accepts `--biome`, `--infested`, `--named`, `--discoverer`, `--within`, `--nearest`, and `--from`. Filters combine before the nearest-result limit. The default reference is the player in the active galaxy; a named base reference selects that base's galaxy. `--named` tests the planet's name, not its system's name.

JSON is the default format and contains a pretty-printed array followed by a newline. Empty JSON is `[]` plus a newline. CSV writes its header with the first result; an empty CSV result is zero bytes. Data goes to stdout and command errors go to stderr.

Both formats use these flat fields, in this order:

| Field | Meaning |
|-------|---------|
| `planet_name` | Planet's explicit name, or an empty string |
| `biome` | Biome label, or an empty string |
| `system_name` | System name, or an empty string |
| `distance_ly` | Distance from the query reference in light-years |
| `portal_glyphs` | Twelve uppercase hex digits from the stored system address |
| `portal_emoji` | Emoji representation of that address |
| `coords_x`, `coords_y`, `coords_z` | Voxel coordinates from the stored address |
| `galaxy` | Galaxy display name |
| `discoverer` | System discoverer name, or an empty string |
| `infested` | Whether the matched planet is infested |

Address-derived fields use the stored system address, including its stored planet-index bits; they are not recomputed from the matched planet's index. Contribution source is not an export column. These records are query results, not a backup of the complete atlas or original save.

## Reading a community CSV

Create a CSV with a system name and twelve portal glyphs as hex digits. Galaxy and biome are optional:

```csv
system_name,portal_glyphs,galaxy,biome
Example System,01717D8A4EA2,Euclid,Lush
```

```bash
nms import community_data.csv --save /path/to/save.hg --source "NMSCE"
```

The importer also recognizes common header aliases such as `System Name`, `Portal Glyphs`, `Galaxy`, and `Biome`. A missing galaxy defaults to Euclid. An unrecognized galaxy name warns and also falls back to Euclid, so check catalog galaxy spellings before import. Galaxy indices 0–255 are accepted. Malformed rows or invalid glyphs are counted as skipped.

The command reports added, duplicate, and skipped counts on stderr. Its model is discarded when the command finishes: it does not write an atlas file or cache for subsequent commands. Persistent imports and a REPL import command are not available through this interface yet.

## Contribution source and duplicate policy

Rust callers can import into a model they retain using [`nms_graph::import::import_csv`](../crates/nms-graph/src/import.rs). Accepted systems carry `DiscoverySource::Community { source }`, with the supplied source label retained literally. Systems extracted from the player's save carry `Personal`.

Contribution source identifies where a catalog entry came from. It is separate from the in-game discoverer and the displayed system or planet name.

An existing genuine system wins over a later duplicate contribution. An explicitly marked unknown player-location entry can be promoted to genuine imported data without adding a second system. Imported entries retain the galaxy supplied by their row.

A compatible [model cache](live-atlas.md) preserves contribution source. If that cache must rebuild from the game save, community records found only in the cache cannot be recovered from that source. Keep the CSV so a library-managed atlas can import it again.
