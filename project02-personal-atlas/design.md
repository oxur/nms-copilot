# Architecture — observations, identity, and projections

Status: proposed architecture, updated 2026-10-11 (initial: 2026-10-03). See the [roadmap](project-plan.md) for sequencing and the [research](artifacts/research-and-library-decisions.md) for library evidence.

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

The managed-media store is authoritative for captured attachment bytes; caption/association/attribution records live in the catalog. The archive is authoritative for captured save bytes; the catalog for normalized observations and user knowledge; the profile registry for local registrations, aliases, and selection. Derived graph/cache/search indexes are rebuildable. Backups must include all authoritative components, not just the database. Registry revisions copied into catalog provenance are historical observations, not a competing editable profile store.

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
| Travellers and world experiences | App-local traveller identity linked to a logical save lineage; save/context-specific observations and narratives concerning shared worlds. Account/native owner identity is evidence, not an automatic traveller key. |
| Journals and media | Authored entries with save/context/traveller scope, event and capture times, multiple typed subject links, edit provenance; managed blobs by digest with stable attachment IDs, captions and separate links. |
| Cultures | Stable identity, one originating traveller, custom name/noun, founding/anchor/history references; evidence-bearing discovery affiliations. Distinct historical cultures may later connect. |
| Culture analyses | Rebuildable membership/geometry/metrics at a catalog generation, source-population coverage, galaxy, algorithm/version, coordinate precision and ownership resolution. Separate from named culture authority. |
| Personal knowledge and interchange | Tags/collections, explicit user assertions, imports and exports with provenance. Re-extraction never overwrites user content. |

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

Start with prepared SQL and exact/prefix/substring name matching; do not expose unrestricted SQL as an MCP tool. Advanced local analytical exports can be a later interface decision. Counts must distinguish unique world entities, distinct discovery claims, per-save observations/visits, bases, and journal entries. A discovery seen in two saves is not two discoveries. Cross-save query scope does not merge traveller narratives.

Reserve stable entity IDs and generation metadata for an optional Tantivy projection. If introduced, commit an indexing outbox with catalog changes, apply it idempotently, track the indexed generation, and allow a full rebuild. Index lag must be visible. FTS relevance never determines entity identity or replaces structured filters.

## Play context, query scope, and authorship

Keep three independent values: selected play context (logical save, active context, latest accepted location/snapshot), read-query scope (one/many saves, account, culture, history), and journal authorship (traveller/save/context). An explicit all-save query changes only its read population. Mutations always resolve a concrete author and target; a broad read scope must not create an ambiguous or multi-save write.

Freeze resolved author/subjects/snapshot at composition start. Watcher changes and save switching cannot retarget an open note/image capture. Show the selected name, galaxy and source freshness. Unavailable/ambiguous locations require an explicit target or a recoverable unresolved draft, not an inferred current planet. Retain drafts on failed commits; define cancellation and draft cleanup in Arc06.

The proposed `TravellerId`, `CultureId`, `JournalEntryId`, `AttachmentId` and `AnalysisId` are semantic IDs, not aliases for paths, player names or DB keys exposed to users. Unknown/conflicting ownership and insufficient geometry are explicit states. These distinctions extend the existing Rust type guidance; exact APIs/DDL wait for slice design.

## Discovery ownership and culture identity

The Operator's domain rule is one discoverer for a system, and discovery is the claim. Multiple imported assertions about that claim are evidence that can conflict, not multiple ownership slots. A foreign traveller's visit/base/note is an experience link, never a transfer of discovery affiliation. Planet discovery evidence remains separate: do not automatically attribute every planet to its system's discoverer.

Separate native account/player identity from the traveller represented by a save lineage. Two saves under one account can share native owner fields. Validate save-specific attribution in G10; merely storing a record in a save or matching the account name is insufficient. Preserve unresolved claims and explicit user attribution with its source, rather than manufacturing certainty. Linked replicas preserve one traveller; divergent copies require an explicit identity decision without losing either narrative.

A culture has stable user-authored identity/history and derived spatial structure. It concerns one traveller's discoveries, partitionable into historically distinct cultures for that traveller. Different travellers' adjacent/interleaved cultures stay separate. Geometry may report that two named cultures have joined spatially without renaming or merging their journals. The user's chosen collective noun is display vocabulary, not an algorithm selector.

Base presence/counts describe attachment without a new manual cherished flag. General bookmark collections remain possible, but adding a place to one cannot change its discovery affiliation. Prospective exploration regions are separate from currently claimed systems. Detailed semantics and geometric acceptance cases: [cultures and exploration](artifacts/cultures-and-exploration.md).

## Journal and media durability

Arc06 adds short authoritative writes independent of ingestion: an authored entry and its links commit together, carry revision information, and survive re-extraction. A single entry can refer to a system, planet, creature and culture without duplicate prose. Cross-save views preserve author labels; two travellers' independently authored entries never collapse because text or subjects match.

Attachments are managed files outside the database with digest, size, media type, capture/import provenance and associations inside it. Publish bytes before committing references, reconcile unreferenced staged blobs after failures, and report missing/corrupt referenced media. Deduplicate exact stored bytes without merging attachment captions/authorship. Do not let snapshot pruning delete authored notes or media. Define unlink/delete/garbage-collection separately; only verified unreferenced content is eligible for collection. Backup/export includes authoritative media plus catalog/registry/archives and verifies restore into a clean directory. See [journal/media behavior](artifacts/journals-and-media.md).

## Culture analysis as a reproducible projection

DuckDB handles scoped populations/joins/aggregates; geometry and `rstar` handle 3D spatial questions; `petgraph` handles adjacency/connectivity/routes. Neither engine supplies absent stars or live discovery ownership. Reuse independently accepted Project01 galaxy-qualified routing contracts. Geometry must stay within one galaxy; all-galaxy queries aggregate per-galaxy results rather than drawing cross-galaxy spatial edges.

Record input generation/population, sources and coverage, ownership attribution, coordinate precision, algorithm/version and internal scale decisions for each analysis. Identity coordinates may be too coarse for exact star geometry; multiple system indices at the same voxel must remain distinct, without fabricated positional jitter. Require precision/error evidence before claiming exact nearest systems in LY. Keep measured completeness, cohesion and scale separate from evidence coverage; no combined score or formula is adopted here.

The candidate-population/coordinate gate G11 and geometry gate G12 precede actionable frontier claims. A known-population analysis is still useful, but must not claim global completeness or enumerate unobserved systems. A blocked full-enumeration capability remains tracked for Operator disposition. Derived membership/metrics can be rebuilt; authored culture identity and connection history cannot be silently discarded with caches.

## Change record

2026-10-11: expands the original personal-knowledge and cross-save architecture with independent play/query/authorship scopes, traveller and claim evidence, journals/media authority, stable culture history and reproducible geometric analyses. Supersedes generic collections as the sole model for cultures and any conversational radius/manual-claim proposals. Database choice remains conditional; no new crate, schema or production API is approved by this document.
