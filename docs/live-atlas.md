# Live updates and cache behavior

The REPL and headless MCP server maintain a living atlas from one selected save. Accepted changes update the shared model while you play; a separate disk cache supports later starts. The game save remains read-only.

## From a save to the atlas

The application decompresses native save data, maps obfuscated JSON keys, and validates the modeled addresses before constructing the atlas. Discovery and base fields use a save-specific packed representation that includes galaxy identity; portal addresses use a separate 48-bit representation. Keeping those boundaries separate prevents coordinates in different galaxies from collapsing into one system.

Invalid modeled addresses reject the complete parse with the offending JSON path. A failed live parse leaves the last accepted model and cache in place. This validates the supported player, base, and discovery address fields; it is not a validation or archival copy of every field in the game save. Supported plaintext JSON uses the same checked boundary after any required key mapping.

For Rust integrations, use the checked [`SaveParser` / `NmsSaveParser`](../crates/nms-save/src/parser.rs) or `parse_validated_save_file` and construct the model with `GalaxyModel::from_validated_save`. Legacy raw parsing/construction helpers remain available with their documented weaker address guarantees.

## Updates during play

One application-owned coordinator watches the selected file, applies each accepted observation, ensures the physical player-system entry, and attempts a cache write when caching is enabled. It then publishes notifications for the REPL and MCP observers. The supported player snapshot and source version come from the same parse as the atlas update.

The atlas retains additive discoveries and imported contributions; this is not a full replacement of every atlas entity on each save. Explicit discovery-name observations update names on existing entities. Names come from the save's `DM.CN` string for the first matching system or planet discovery row at the exact galaxy-qualified key. A string replaces the visible label, an empty string clears it, and an absent or unusable name leaves the accepted label alone. System names do not supply missing planet names.

Debouncing may combine intermediate writes. Failed parses produce no accepted update; the watcher warns after five consecutive failures and resumes from its last good baseline when parsing succeeds. A cache-write failure also warns, but does not roll back the accepted model or prevent notifications. File-notification backend errors are not automatically reconnected in this release.

Session-derived fields refresh at command/prompt boundaries. The map and `last` response are deliberate snapshots; see [REPL exploration](exploration.md). The map releases the model lock before interaction, so it does not stop live ingestion.

## Physical location and unknown systems

A player's current system may not yet have a genuine discovery record. The application keeps an explicitly marked location entry in that case, without inventing a name or planet. Later genuine discovery or imported data at that exact key promotes it in place.

The `where_am_i` tool and `nms://player/location` resource use exact physical identity, independently of the selected galaxy view:

| Physical system record | Location label and planet count |
|------------------------|---------------------------------|
| Absent or explicitly marked unknown entry | `(unknown)`, zero planets |
| Genuine unnamed discovery | `-`, its actual planet count |
| Genuine named discovery | Its name and actual planet count |

A nearby system in the same voxel cannot supply a name for an unknown physical system. Model system totals include marked location entries, so they are not identical to a count of scanned discoveries. Marked entries survive compatible cache restarts; their later promotion does not add another system.

## Cache loading and rebuilding

The application checks cache modification time first: the cache must be newer than the source save. A fresh file must also pass format and model checks. A stale, incompatible, or corrupt cache is rebuilt from the selected save and, when possible, replaced with a current cache. A failed source parse does not replace an existing cache.

Cache files have a versioned envelope with separate archive-layout and model-meaning revisions. The current schema is **4** and semantic revision is **6**. These revisions are independent of the game save version. Old formats are rejected and rebuilt; their missing identity or provenance is not guessed from names or empty records.

The cache retains supported player state, selected galaxy, explicit unknown-location markers, discovery names, and Personal/Community source. Contradictory persisted player-galaxy fields or malformed placeholder markers are rejected by checked cache readers. Loading deserializes data and reconstructs model indexes; it does not provide zero-copy queries over a memory-mapped atlas.

A compatible cache missing the current player-system entry is repaired in memory, with a best-effort cache rewrite. Existing valid marked or genuine entries need no repair. Read-only storage can therefore prevent persistence while the application continues from valid in-memory data.

A rebuild can only recover information present in the selected source save. Community records or labels retained only in the old cache may need re-importing; keep their original catalog. Cache files are derived data, not a durable journal or a substitute for backing up the source.

To bypass cache I/O:

```bash
nms-copilot --no-cache --save /path/to/save.hg
nms-copilot --headless --no-cache --save /path/to/save.hg
```

The in-memory location and validation rules still apply. See [configuration](configuration.md) for cache paths, disabling watching, and debounce settings. One-shot `nms` commands parse their selected save directly and do not share this running cache/model lifecycle.

## Observer lifetime for integrations

The application owns the coordinator and its MCP observer. Normal REPL shutdown stops and joins the server before dropping the coordinator. Application observer queues are unbounded: consumers must drain them, and shutdown may discard queued notifications. Parsing and filesystem work have no hard shutdown deadline. Library callers should release model guards before stopping these owners.

The [`nms-watch` APIs](../crates/nms-watch/src/watcher.rs) expose legacy delta, complete-update, and paired discovery-name observation streams. The application uses the paired stream; name-only changes can be quiet on the other two. Direct library callers own their handle and model/cache application policy. For wire-level subscriptions, see [HTTP MCP](mcp-http.md).
