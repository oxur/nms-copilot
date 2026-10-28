# Architecture — the searchable personal atlas

Version: 2.0 — 2026-10-27. Proposed production architecture for the [current six-arc plan](project-plan.md); no new crate, catalog, schema or dependency is implemented/approved by this paper. The [shared backlog](../backlog.md) owns later notebook/culture/research/exploration destinations. The 2026-10-11 full-vision architecture remains in Git history; its later-project contracts remain in the linked design papers.

## Authority and data flow

```text
read-only save files / supported imported evidence
                    |
       stable capture + source observation
                    |
        immutable exact-byte snapshot archive
                    |
       versioned decode / map / normalize
                    |
       catalog: observations / entities / links
                    |
     shared typed queries + graph/cache projections
                    |
                CLI / REPL / MCP

profile registry ---- stable local IDs / aliases / selection
```

The registry is authoritative for registrations, aliases, replica links and remembered selection. The archive holds captured save bytes. The catalog holds normalized evidence, source observations and supported imported records. Registry revisions recorded in provenance are historical facts, not a competing editable profile store. Graph/cache/search projections are rebuildable; deleting the catalog, registry or archive is not a harmless cache reset. Project02 backup/restore covers all authoritative state it introduces.

Journal entries, attachment bytes/captions and culture histories introduce additional authority in later projects. They are not tables, storage services or restore fixtures that Project02 must build now. Reserve clear domain identity/scope boundaries without implementing a general future-proof repository framework.

## Reuse and component responsibilities

| Boundary | Current responsibility | Limit |
|---|---|---|
| `nms-core` | Validated domain IDs and galaxy-qualified address values | No database, UI or watcher dependency. |
| `nms-save` | Existing checked SaveParser boundary; byte/key decoding, metadata/raw record access and parse diagnostics | No selection policy; new field families use validated conversions rather than applying one shift to every integer. |
| Shared profile service (module/crate proposed) | Inventory, evidence-qualified identity, aliases, resolution, registry/migration and switch preparation | First useful increment without a database. Separate crate only if the actual ownership/dependency boundary warrants it. |
| Atlas storage service (module/crate proposed) | Stable capture/archive, normalized ingestion, migrations, queries and committed generations | Added for current Arc04 after its engine decision; no media/geometry/ORM platform. |
| `nms-watch` + application coordinator | Change hints, debounce/stable-read scheduling, bounded backend reconnection and one intake/apply owner per active source | Reuse the accepted LiveSaveCoordinator design. REPL/MCP consumers must not add duplicate ingestion loops. |
| `nms-graph` / `nms-cache` | Existing spatial/routing model and disposable projections | Include source/save/context and interpretation/generation identity in cache applicability; preserve accepted field-specific contracts. |
| `nms-query` | Shared typed requests/results and incremental orchestration over current model or catalog | Do not leak DuckDB/Arrow rows, connections or lifetimes into domain results. |
| `nms-cli` / `nms-copilot` / `nms-mcp` | Arguments, interaction, protocols and rendering | Separate MCP crate and shared coordinator already exist. No repeated extraction or independent selection/query policies. |

Use separate semantic SourceId, SaveId, SnapshotId and ContextId values; names/paths/slot numbers are selectors/evidence, not interchangeable keys. Reserve the distinction between recorded account/player and save traveller without inventing native certainty. Errors distinguish unavailable, ambiguous, malformed, unsupported, stale and busy states. Preserve relevant absent/null/empty/malformed/present distinctions explicitly rather than nested options or blanket defaults. Concrete internals and narrow substitution boundaries take precedence over a trait for every struct.

## Bounded logical model

| Family | Keys and links |
|---|---|
| Sources/replicas | Registry identity, physical path/platform at observation time, availability and explicit replica-link evidence. |
| Snapshots/import runs | Exact-byte digest, source sighting, captured time, file mtime, format/mapping/extractor identity, parse coverage/status. Deduplicated bytes do not erase repeated sightings. |
| Save contexts | Logical save, main/expedition evidence and unknown context identifiers; active context does not erase retained contexts. |
| Recorded identities | Namespaced native tokens and observed names; explicit local links distinguish user assertions from verified native evidence. |
| Systems/planets | Galaxy plus validated location components and applicable universe/generation evidence. Names and seeds alone are not universal keys. |
| Discovery evidence | Game record type/value/name/owner/time and resolved or unresolved association, with raw locator and source. |
| Companion observations | Save/context-scoped instance evidence, nickname/species/seed evidence and supported origin. Slot replacement cannot imply the same pet. |
| Bases/teleport destinations | Supported identity/name/owner/location evidence and per-save/context availability. |
| Relationships | Typed located-in, observed-in, discovery-owner and companion-origin links with resolution status and provenance. |

Each normalized observation/relationship identifies its source snapshot/import, raw locator, mapping/extractor version and supported scope. Typed tables cover the current supported families; original bytes preserve unmapped material. Avoid a universal untyped entity-attribute-value schema. Unresolved associations remain visible/searchable. Imported claims retain their own provenance and must not masquerade as native facts.

## Identity, selection and time

Retain [the friendly-save contract](artifacts/save-account-ux.md): physical `--save` remains exact; friendly selectors resolve explicit names/aliases/IDs, duplicate names produce candidates and no noninteractive fuzzy auto-selection is allowed. A missing remembered source cannot fall through to someone else's newest file. Keep legacy configured paths readable while migrating; preserve invalid input and unknown config keys rather than rewriting them as defaults.

Resolve a complete candidate session before publishing a switch. Selection, model/reference scope, completions, cache and watcher generation move together; failed switches preserve a usable prior state and late events are generation-checked. Explicit cross-save reads do not mutate active play selection. Script/protocol calls return structured ambiguity instead of prompting.

Account owner, native player token and a traveller represented by a logical save are not interchangeable. Same-account saves remain distinct; synced replicas require evidence; divergent copies remain unresolved until explicitly linked/separated. Discovery attribution is not inferred merely from visit/base presence or record inclusion. Save-specific attribution may remain unknown. Later notebook/culture projects consume these distinctions.

Store file mtime, Copilot capture time, game timestamps and original owner/discovery timestamps separately with basis/validation. Sync time is not visit time. Current queries use the selected context's latest accepted snapshot; historical/last-known results label their older source. Failed or partial extraction cannot prove absence. A successful snapshot's absence is scoped to the coverage actually established.

## Ingestion, durability and restore

1. Treat watcher events as hints. Bound debounce/pending work and reconcile sources so a missed hint cannot permanently lose updates. Backend re-registration is bounded, cancellable and distinct from ordinary file replacement recovery; avoid duplicate active registrations or duplicate application ingestion.
2. Bound input/decompression work, compare file metadata around reads, retry changing files, decode and validate. Metadata checks reduce torn reads but do not promise an atomic game-file snapshot. A failed candidate never replaces the last accepted generation.
3. Hash exact source bytes with the existing SHA-256 facility. Stage/sync as supported and publish the raw blob by digest within the archive filesystem; record sightings even if the content already exists.
4. Decode/normalize outside the catalog transaction with explicit format/mapping/extractor versions and raw locators. Preserve unmapped bytes; collisions/unsupported fields are diagnostics, not silent overwrite.
5. Commit the accepted snapshot/run, observations, relationships and generation coherently. Idempotency includes scope and extractor version, allowing re-extraction without multiplying entity/claim counts.
6. Publish the complete in-memory generation after commit. Repair commit/publication interruption by rebuilding the current projection at restart.

Archive publication and database commit are not atomic together. A failed commit may leave an unreferenced blob; reconcile only verified unreferenced content under a specified policy. A missing referenced blob is an integrity error. Number/check migrations; back up before destructive migration; refuse newer unsupported schemas and preserve recovery inputs after failure. An archive limit preserves existing history and visibly pauses capture; no unagreed automatic pruning. Snapshot retention does not silently discard supported imported evidence.

Back up the registry, catalog and referenced raw archive with their identities and consistency basis. Restore into a clean app directory without relying on original absolute paths, verify sources/facts/links and rebuild disposable projections. Protect original game files throughout. The exact snapshot/quiescence mechanism depends on the chosen engine and belongs to Arc04's implementation design.

## Database decision and concurrent interfaces

Conditional preference remains DuckDB; SQLite is the fallback if actual native packaging, interaction or process coordination costs fail the agreed trial. Before trial execution, agree its question, workload/fixtures, resource limit, target matrix, thresholds, expected decision and stop/fallback rule. The research paper's older numbers are proposed targets, not measurements or accepted budgets. Refresh versions/primary documentation at adoption; this reconciliation did not browse or run a database experiment.

If DuckDB passes, a single writable owner and dedicated worker with bounded typed requests is the proposed starting design. Concurrent CLI/REPL/MCP clients may require a versioned local endpoint and OS-backed ownership lock; startup races, stale owners, access control and handoff must be proven before this mechanism becomes production policy. Do not prebuild a service layer solely to justify the preferred engine. If SQLite wins, define its actual contention/transaction boundaries instead; no parallel authoritative engine.

Keep long blocking/native work off async executor threads. Cancellation/shutdown must interrupt or bound actual work, not merely drop an awaiting future. Bound pages, memory, temporary disk, execution and ingestion batches; measure current lookups/history/ingestion responsiveness. The future notebook's short writes and geometry workloads are evaluated by their introducing projects, not imposed as current trial gates. Existing twyg/confyg conventions remain; no unrelated observability/config-location migration.

## Shared query and output contract

Use owned typed results with explicit save/context/source/freshness and uncertainty, stable ordering and generation-aware pagination. Exact/prefix/substring names precede optional FTS. Supported biome and infestation evidence remain independent. Date filters name the basis. Count unique worlds, discovery claims, source observations and per-save availability separately; a record seen twice is not two discoveries.

Arc02 wires existing settings/predicates/details/statistics and human/plain/JSON consumers before richer catalog querying. Preserve explicit override precedence and real ANSI-free/plain piping; diagnostics stay off structured stdout. Existing import/export/completions get REPL parity without implying notebook/media interchange. Applicable CLI/REPL/MCP requests produce equivalent domain results, not necessarily identical transport framing.

Do not expose unrestricted SQL as an MCP tool. A future Tantivy index, if justified by an actual search limitation, is a disposable projection with stable entity/generation references and an explicit replay/lag contract designed at that time. Do not implement its outbox/indexing machinery now. Reuse accepted same-galaxy routing; B04 fixes jump arithmetic without silently promising B05's stronger every-leg guarantee.

## Later-project contracts and reading applied

The [notebook paper](artifacts/journals-and-media.md) retains frozen author/target composition, drafts, notes and managed-media durability. The [culture/exploration paper](artifacts/cultures-and-exploration.md) retains stable culture history, discovery-based affiliation, qualified geometry/population/region and frontier semantics. Their dated Project02 arc ownership is superseded by the root backlog; the requirements remain. No TravellerId/CultureId/journal/media/AnalysisId production subsystem is required here merely to anticipate those projects.

This planning reconciliation used targeted rust-guidelines guidance: distinct semantic types (AP-29), explicit result/state variants (AP-40/41), private implementation types (AP-44), bounded off-executor native work (AP-50), and configuration precedence/validation (CLI-31/32). Retain the project's established configuration/logging choices. This is scoped design guidance, not a Rust source audit or complete anti-pattern review; actual implementation reloads the relevant domain guides.

## Change record

- **2026-10-03 / 2026-10-11:** Initial architecture and full-vision augmentation; retained in Git history and later-project design papers.
- **2.0 — 2026-10-27:** Reconcile current atlas authority/model/intake/query/engine boundaries, consume accepted Project01 parser/coordinator/MCP and remove notebook/media/culture/geometry production work from current prerequisites. Preserve semantic seams and conditional library decisions without implementing speculative frameworks. No source or dependency changed.
