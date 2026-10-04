# Research and library decisions

Research date: 2026-10-03. Documentation/source examination only: no dependency was added, no prototype was compiled, and no benchmark was run. Version observations below are research inputs, not approved dependency pins.

## Decision register

| ID | Recommendation | Status / reason |
|---|---|---|
| D01 | Deliver friendly selection as project02 Arc01 | Proposed project placement; user-required ordering. Does not depend on DuckDB. |
| D02 | Distinguish local accounts, recorded identities, logical saves, replicas, snapshots, contexts | Proposed; supported by actual cross-platform save evidence. |
| D03 | Retain raw evidence plus typed relational observations and derived spatial graphs | Proposed; supports re-extraction, history, relationships, and existing routing without forcing one representation to do everything. |
| D04 | Prefer DuckDB, behind a feasibility gate | Conditional; analytic appeal must be weighed against native packaging and process ownership. |
| D05 | Keep existing CLI/REPL libraries; add metadata and shared selection logic | Proposed; a full-screen TUI rewrite is unnecessary for readable names. |
| D06 | Remember explicit selection, make switching visible and easy | Provisional preference; user answer pending. |
| D07 | Defer Tantivy and DuckDB FTS | Confirmed low priority; reserve projection identity/generation contracts, not an implementation dependency. |
| D08 | Preserve local config location and logging conventions | Proposed scope control; no unrelated XDG/tracing migration. |

## Database calculus

DuckDB remains the preferred candidate for historical scans, grouped summaries, joins, and analytical interchange. Its Rust binding provides prepared statements, transactions, and an appender. The published docs examined show `duckdb` **1.10506.0** and its manifest declares Rust **1.85.1**; the project declares **1.91**. That is only nominal compatibility until the complete dependency/native build succeeds. The moving repository README already illustrates a different version line, so select a published release deliberately and record its matching engine version. [Published manifest](https://docs.rs/crate/duckdb/1.10506.0/source/Cargo.toml), [binding repository](https://github.com/duckdb/duckdb-rs).

The ordinary embedded read/write database has one owning process. Multiple writer threads within that process are supported; independent read-only processes require the absence of a writer. New remote alternatives described upstream do not remove the need to design our local application's ownership. This makes a coordinator/handoff trial a prerequisite for concurrent CLI, REPL, and MCP use. [DuckDB concurrency](https://duckdb.org/docs/current/connect/concurrency).

The Rust connection is `Send` but not `Sync`. Proposed implementation: one dedicated worker owns it, with bounded requests and owned domain results. Do not hold a database lock across UI interaction or an async `.await`. Add parallel read connections only when measurement justifies them. [Connection API](https://docs.rs/duckdb/1.10506.0/duckdb/struct.Connection.html).

Bundling compiles the native C++ dependency; system linking instead requires compatible installed libraries. Prefer evaluating the ordinary bundled route first for distributable binaries, with native toolchain, clean build time, binary size, and target availability recorded. Avoid assuming the experimental CMake route works from published crates. [Binding build documentation](https://github.com/duckdb/duckdb-rs).

SQLite remains a strong fallback for frequent small transactions and independently launched local clients: WAL allows readers in other processes while one writer commits. It still needs contention handling and a local filesystem. Choose it if DuckDB's coordination/build cost outweighs the measured analytical benefit. Do not maintain two authoritative catalogs merely to postpone the choice. [SQLite WAL](https://www.sqlite.org/wal.html).

## Rust library shortlist

| Area | Candidate and recommendation | Conditions / source |
|---|---|---|
| Chooser | Retain `dialoguer` 0.12; evaluate its `fuzzy-select` feature | Already used for setup. Cancellation is a normal result; only invoke with a usable terminal. Fuzzy selection is a UI aid, not script resolution. [FuzzySelect](https://docs.rs/dialoguer/0.12.0/dialoguer/struct.FuzzySelect.html). |
| Shell and output | Retain `reedline`, clap, existing table/theme layer | Keep command completion and stable JSON output; use existing project versions. A ratatui redesign adds no required identity capability. Local manifest is the authority. |
| Config editing | Add `toml_edit` only where modifying config | Preserves comments and much formatting, with documented exceptions such as dotted-key order. Parse/validate before edits; don't promise byte-identical formatting. [toml_edit](https://docs.rs/toml_edit/latest/toml_edit/). |
| Local IDs | Evaluate `uuid` 1 with `v4` and `serde` | App-assigned IDs avoid tying identity to names/paths. They are distinct from the game's native save identifier. [uuid features](https://docs.rs/uuid/latest/uuid/). |
| File locks | Prefer `std::fs::File::try_lock` over adding a locking crate | Stable since Rust 1.89, below our MSRV. Use a dedicated persistent lock file; test platform behavior and busy handling. Locks do not make game-file reads atomic. [File API](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock). |
| Atomic local replacement | Reuse `tempfile` with explicit durability steps | Same-filesystem `persist` replaces atomically but does not itself sync file/directory contents. Test interrupted writes and platform behavior. [NamedTempFile::persist](https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html#method.persist). |
| Content fingerprints | Reuse existing `sha2` 0.10 | Exact source-byte hashes for dedup/integrity; no need to introduce another hash dependency without measurement. Hash equality is not player identity. |
| JSON preservation | Keep serde/serde_json; consider `RawValue` for selected payloads | The `raw_value` feature can retain JSON fragments without eagerly typing them. It does not replace archiving original compressed bytes, and mapping transforms are separately versioned. [RawValue](https://docs.rs/serde_json/latest/serde_json/value/struct.RawValue.html). |
| Parse diagnostics | Evaluate `serde_path_to_error` | Adds field-path context to deserialization errors; scrub sensitive values in default diagnostics. [API](https://docs.rs/serde_path_to_error/latest/serde_path_to_error/). |
| Watch/reconcile | Retain `notify` and current debounce support | Events can be missed; replacement writes and some filesystems behave differently. Reconcile contents and retain a polling fallback. [Known problems](https://docs.rs/notify/latest/notify/#known-problems). |
| Runtime work | Retain Tokio; dedicated DB thread, bounded channels | Long-lived blocking workers belong on dedicated threads. Started `spawn_blocking` work cannot simply be aborted; query interruption/shutdown needs explicit design. [Tokio guidance](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html). |
| Spatial relationships | Retain `petgraph` and `rstar` | Existing code already uses them. Rebuild projections per save/context/generation; relational joins cover nonspatial relationships initially. |
| Verification | Retain `assert_cmd`, `predicates`, `tempfile`, ordinary Rust tests | Add property-based testing only if it improves address/identity boundary coverage beyond explicit independent fixtures. No new snapshot framework required for its own sake. |
| Future text index | Tantivy, optional and rebuildable | Rust-native search library worth revisiting when structured search becomes insufficient. Separate catalog and index commits require replay/outbox handling in our design. [Tantivy](https://github.com/quickwit-oss/tantivy). |

DuckDB's FTS index does not automatically track table changes. That is not a reason to choose the whole database around FTS while it is low priority. Exact/prefix/substring search is sufficient for the first name-based workflows. [DuckDB FTS limitation](https://duckdb.org/docs/current/core_extensions/full_text_search).

## Focused trials before adoption

These are proposed trial contracts, not results. Store commands, fixture sizes, versions, target/toolchain details, and raw successful/failed runs when executed. Trial thresholds are initial engineering targets for discussion, not current performance claims.

| Gate | Workload / evidence | Decision rule |
|---|---|---|
| G01 — Save metadata and identity | Sanitized multiple-account fixture; names absent/renamed/duplicated; copied saves; slot replacement; matching and conflicting native IDs; private real-save spot-check | Establish which fields are stable enough for automatic association. Any unresolved identity remains provisional; no hidden cross-root merge. |
| G02 — Chooser responsiveness | Measure actual inventory, then 30 and 300 candidate files on local storage; cold parse and warm metadata lookup separately; one corrupt/large file | Target warm list under 200 ms and first useful cold output under 1 s on the recorded Mac. If full scan is slower, progressive output must stay responsive and show pending entries. Record p50/p95 and input bytes, not just averages. |
| G03 — Address correctness | Independent golden fixture for Chili; nonzero galaxy byte; signed voxel extremes; system/planet boundaries; expanded coordinates vs packed forms | Exact agreement with known addresses. Round trips alone cannot prove encoding correctness. Audit each field family rather than shifting every integer alike. |
| G04 — DuckDB build/distribution | Published candidate with ordinary bundled build on Rust 1.91 and supported release targets; clean/offline-after-fetch builds; license/dependency inventory | Adopt only with a reproducible packaging path and explicit target results. Unsupported/unrun targets stay unclaimed. Measure build time and binary size before accepting their cost. |
| G05 — Interactive catalog | Sanitized real snapshot; generated 100k/1m observations with declared distributions; repeated imports, name/location lookups, filtered joins, grouped history | Target warm ordinary queries p95 under 200 ms; 1m-row summaries under 2 s on the recorded host. Record memory, temporary disk, DB growth, cancellation, and UI responsiveness. If misses occur, diagnose query/layout before switching engines. |
| G06 — Multiple interfaces and recovery | REPL owner + concurrent CLI and MCP clients; simultaneous ownership start; owner death; interrupted import; cancellation; migration failure; storage exhaustion | No independent writer race, partial visible generation, silent data loss, or permanently stuck ownership. If user-visible coordination is brittle, compare SQLite on the same scenarios before adoption. |
| G07 — Historical truth | Discovery present → absent → present; partial/malformed snapshot; duplicate sightings; main/expedition conflicts; annotations after re-extraction | Correct source/date labels and counting semantics. Missing/failed parsing does not become false deletion; prior facts and annotations remain reconstructable. |

## Rust-guidelines reading applied to this design

The user explicitly requested [rust-guidelines](/Users/oubiwann/.codex/skills/rust-guidelines/SKILL.md). The following selected portions shaped decisions; this is a targeted planning pass, not a claim to have read every guide or to have audited all Rust code.

| Material | Applied design consequence |
|---|---|
| [API design](/Users/oubiwann/.codex/skills/rust-guidelines/guides/02-api-design.md), API-23/37/39/40/46 | Concrete errors; different ID/address types; validated boundaries; concrete implementations first; hide database types from interface contracts. |
| [Type design](/Users/oubiwann/.codex/skills/rust-guidelines/guides/05-type-design.md), TD-03/06/07/13 | Newtypes and explicit states, forward-compatible unknowns, human-oriented display distinct from serialization. |
| [Anti-patterns](/Users/oubiwann/.codex/skills/rust-guidelines/guides/11-anti-patterns.md), AP-06/09/23/25/29/33/40/41/44/50/59/67 | No recoverable filesystem panic, silently ignored scan errors, primitive address confusion, ambiguous nested option states, leaked DB types, or blocking native work on async executors. Measure before optimizing. |
| [Concurrency](/Users/oubiwann/.codex/skills/rust-guidelines/guides/07-concurrency-async.md), selected ownership/cancellation/channel guidance | Worker ownership, bounded queues, explicit shutdown and cancellation, generation-based session publication. Tokio's primary docs qualify what cancellation can actually stop. |
| [CLI output/UX](/Users/oubiwann/.codex/skills/rust-guidelines/guides/14-cli-tools/04-output-and-ux.md), CLI-21–27/58 | Friendly interactive labels; machine IDs in structured output; terminal-aware prompts/color; diagnostics on stderr; ordinary cancellation. |
| [CLI configuration](/Users/oubiwann/.codex/skills/rust-guidelines/guides/14-cli-tools/05-configuration.md), CLI-29–33 | Explicit precedence, validation and migration. Preserve the established project path instead of introducing a distracting config-location migration. |
| [Project structure](/Users/oubiwann/.codex/skills/rust-guidelines/guides/12-project-structure.md), PS-02/08/11/14/24 | Thin binaries, focused shared libraries, tested MSRV, optional heavyweight features, explicit native-build responsibility. |
| [Documentation](/Users/oubiwann/.codex/skills/rust-guidelines/guides/13-documentation.md), DC-05/29/31 | Describe errors, trait/selection contracts, and user-facing limitations alongside APIs. |

For implementation, reread the particular guides for the actual code being changed and refresh candidate dependency versions. Keep authoritative SDK docs ahead of illustrative snippets if they differ.
