# Architecture — observations, identity, and projections

Status: proposed, 2026-10-03. See the [roadmap](project-plan.md) for sequencing and the [research](artifacts/research-and-library-decisions.md) for library evidence.

## Responsibilities and authority

```text
Read-only game files / imported evidence
              |
     stable capture + source metadata
              |
     immutable raw snapshot archive
              |
 versioned decode / map / normalize
              |
 catalog: observations, entities, relationships, annotations
              |
 shared typed queries + spatial/routing projections
              |
          CLI / REPL / MCP

Profile registry ---- stable local IDs / aliases / selection policy
```

The archive is authoritative for captured bytes; the catalog for normalized observations and user knowledge; the profile registry for local registrations, aliases, and selection. Derived graph/cache/search indexes are rebuildable. Backups must include all authoritative components, not just the database. Registry revisions copied into catalog provenance are historical observations, not a competing editable profile store.

The relational catalog expands the current architecture: `nms-graph` remains the spatial/routing engine, while richer nonspatial queries use catalog relationships. A graph database is unnecessary for the first set of joins. Add arbitrary traversal machinery only when concrete queries justify it.

## Proposed Rust boundaries

| Component | Ownership |
|---|---|
| `nms-core` | Validated addresses and domain IDs; lean shared value types. No DuckDB, filesystem watcher, or UI dependency. |
| `nms-save` | Stable byte decoding, format mapping, typed metadata and raw record access; path-aware parse diagnostics. No user selection policy. |
| Proposed `nms-profile` | Shared inventory, identity evidence, aliases, resolution, registry, and compatibility migration. Starts in Arc01. |
| Proposed `nms-atlas` | Snapshot/archive coordination, normalized ingestion, migrations, catalog queries, and projection generations. Added only when needed in Arc03. |
| `nms-graph` / `nms-cache` | Scoped spatial model and disposable cached projections. Cache key includes source/save/context, snapshot/extractor/schema identity. |
| `nms-watch` | Change hints, stable-read scheduling, reconciliation. One ingestion coordinator per active source; no separate REPL and MCP import loops. |
| `nms-query` | Domain request/results and shared query orchestration; evolve existing model-only functions incrementally. |
| CLI / REPL / MCP | Argument/prompt/protocol translation and rendering. No duplicate identity or query rules. |

Crate names are proposals. Prefer focused modules first if a separate crate would add dependencies without a useful boundary. Do not introduce a generic repository/ORM framework or a trait for every struct. Keep concrete internal types and narrow traits only at actual substitution/test boundaries. Public results must not expose DuckDB rows, Arrow arrays, or database lifetimes.

Use separate `SourceId`, `SaveId`, `SnapshotId`, and `ContextId` types; separate `DiscoveryAddress`, `PortalAddress`, and expanded coordinates. Parsing uses validated constructors/`TryFrom`, with unknown values retained as evidence rather than silently coerced. Errors distinguish unavailable, ambiguous, malformed, unsupported, stale, and busy. Preserve absent/null/empty/malformed/present states when their difference matters; use explicit enums rather than nested `Option` or blanket defaults.

## Logical data model

This is a relationship model, not final DDL. Every normalized observation includes a snapshot/import ID, raw record locator, source format, mapping hash/version, extractor version, and scope where available.

| Record family | Important keys and relationships |
|---|---|
| Source/replica observations | Reference registry IDs; preserve physical path and platform evidence at observation time. |
| Snapshots/import runs | Content digest, captured time, file mtime, parse status, format/extractor versions, source; content dedup does not discard repeated source sightings. |
| Save contexts | Logical save + context identifier + expedition/season evidence; preserve unknown context kinds. |
| Recorded identities | Platform namespace + source identifiers; name observations and explicit user links are separate. |
| Systems and planets | Galaxy + decoded coordinates + system/planet components, with universe/generation provenance where known. Never merge across galaxies or assume procedural seeds are universally stable. |
| Discoveries | Typed game record identity and value/seed evidence, object association, owner record, source timestamp; allow unresolved associations. |
| Companion observations | Save/context-scoped pet instance evidence, nickname, creature type, species/seed evidence, origin address. Slot index alone cannot identify a pet across replacement. |
| Bases/teleport destinations | Record identity, owner/name observations, location, per-save/context availability. |
| Relationships | Typed links such as located-in, discovered-by, companion-origin, observed-in; each link has provenance and resolution status. |
| Personal knowledge | Notes/tags/collections/attachment references, explicit user assertions, imports and their provenance. Re-extraction never overwrites user content. |

Avoid a universal untyped entity-attribute-value schema. Typed tables cover established domains; raw payloads preserve unmapped data. Similar names are search matches, not join keys. Species names and pet nicknames can change; species linkage remains provisional unless its game key semantics are validated. Keep unresolved records searchable.

## Time and conflicting evidence

Keep file modification time, Copilot observation time, game timestamps, and original discovery-owner timestamps separately with a time basis and validation status. A file synced today may describe an old play session. “First seen by Copilot in the past week” is answerable after historical capture begins; “I visited it last week” requires actual personal visit evidence and may remain unknown.

Current-state queries use the selected save/context's latest accepted snapshot. History queries search retained observations. Last-known values can be offered explicitly with their older source date; they must not silently fill missing current data. Removing a discovery from a later snapshot records absence within that snapshot's known coverage. Failed/partial parsing cannot prove absence.

Shared worlds have source-specific names and facts. Prefer explicit local aliases for display, then the selected context's supported evidence; retain disagreements rather than overwriting them. Results explain why an entity matched and identify the supporting observation.

## Ingestion and recovery

1. Treat watcher events as hints. Debounce, bound pending work, and periodically reconcile registered locations; a missed event must not permanently lose an update.
2. Read into a bounded buffer, compare before/after file metadata, and retry changing files. This reduces torn reads but is not an atomic snapshot guarantee. Successful decoding and consistency checks are required; keep failed attempts visible without replacing the last accepted state.
3. Hash exact source bytes with the existing SHA-256 dependency. Stage the blob in the archive filesystem, sync as supported, and atomically publish it under its digest. Preserve source sightings even when the bytes already exist.
4. Decode and normalize outside the catalog transaction. Preserve original bytes independently from mapped JSON. Unknown fields survive through retained source data; mapping collisions/unsupported versions produce diagnostics, not silent overwrite.
5. Commit snapshot metadata, observations, relationships, and the projection generation in one database transaction. Idempotency includes snapshot, scope, and extractor version, so re-extraction is possible without duplicate discoveries.
6. Publish a new complete in-memory generation after commit. A crash between database commit and projection publication is repaired by rebuilding on restart.

Archive publication and database commit are not one atomic operation. Publishing the blob first may leave an orphan if the transaction fails; startup reconciliation reports/reclaims only verified unreferenced blobs according to retention policy. A missing referenced blob is an integrity error, not a successful replay. Never silently overwrite annotations during catalog rebuild; back up or separately export them before any destructive migration/reconstruction.

Schema migrations have numbered versions, checksums, preflight compatibility, backup, and recovery instructions. Refuse writes to a newer unsupported schema. A failed migration preserves a usable backup. Deleting derived caches is harmless; deleting the catalog or registry is not. An archive size limit pauses capture visibly and preserves existing history until the user chooses a retention action.

## Database ownership and interactive work

If DuckDB is adopted, one process owns the writable catalog. A dedicated worker thread owns its connection and accepts bounded typed requests. Short commands may take ownership while no long-lived owner exists; when the REPL/server owns it, other interfaces request work through a versioned local endpoint. No implicit background-service installation is required.

The Arc03 trial must settle the endpoint transport, access control, startup race, stale-owner detection, and handoff. Use a stable OS lock file as the ownership authority rather than trusting a PID file. Concurrent readers must not independently open the live database while a different process writes. Immutable exports may support external analysis, with their snapshot time displayed.

Cancellation requires cooperative query interruption and clear transaction boundaries; dropping an async future does not necessarily stop native work. Bound result pages, execution time, memory, temporary disk, and ingestion batches. Keep long analytics from starving save updates or the chooser; measure before adding connection pools. Existing logging/configuration conventions stay in place.

If reliable cross-process operation makes the everyday experience too complicated, prefer SQLite for the catalog and consider DuckDB only for explicit analytical exports later. That is a decision gate, not a commitment to ship both engines.

## Queries and future FTS

Use typed filters for save/account/context, source, biome, infestation, discovery type, date basis/range, location, names, and availability. Default to the selected scope; cross-save/all-history operations are explicit. Joins expose companions' origins, worlds containing bases, and shared discoveries. All interfaces return equivalent domain results and freshness metadata. Paginated results use a stable ordering and snapshot/generation token.

Start with prepared SQL and exact/prefix/substring name matching; do not expose unrestricted SQL as an MCP tool. Advanced local analytical exports can be a later interface decision. Counts must distinguish unique entities, per-save discoveries, and observations.

Reserve stable entity IDs and generation metadata for an optional Tantivy projection. If introduced, commit an indexing outbox with catalog changes, apply it idempotently, track the indexed generation, and allow a full rebuild. Index lag must be visible. FTS relevance never determines entity identity or replaces structured filters.
