# Research and library decisions

## Current consumers — 2026-10-27 reconciliation

The [current six-arc plan](../project-plan.md), [current architecture](../design.md) and [repository grouping](../../backlog.md) supersede the old arc ownership below. Prior research content remains verbatim, dated source examination; its version/API observations are not current verification. No database/clipboard/geometry trial, dependency adoption or benchmark was performed by this reconciliation. Refresh primary documentation at actual adoption.

| Existing gate | Current consumer and boundary |
|---|---|
| G01 — Identity | Project02 Arc01; distinguish four saves/two accounts/replicas and provisional links. No native claim certainty manufactured. |
| G02 — Inventory responsiveness | Project02 Arc01's real inventory/chooser boundary. The older 30/300-file workloads and thresholds below are proposals to agree before measurement, not observed performance or an execution budget. |
| G03 — Addresses | Project02 Arc03 for newly supported field families and unknown/malformed states. Consume accepted Project01 independent address/galaxy evidence; no repeat of its investigation programme. |
| G04 — Catalog build | Project02 Arc04, agreed bounded adoption trial; preserve existing target/MSRV commitments unless explicitly changed. |
| G05 — Atlas interaction | Project02 Arc04 current ingestion/history/name/location/filter/aggregate workloads. The old note/attachment short-write and long-geometry consumers move to their introducing projects; their numbers are not Project02 gates. Agree current fixture distributions, resource/time limits and decision targets before execution. |
| G06 — Process/recovery | Project02 Arc04 actual selected-engine CLI/REPL/MCP contention, restart, migration and storage limits. A DuckDB endpoint/owner design is conditional, not a mandatory framework. |
| G07 — Historical truth | Project02 Arc03/04/05 for observed evidence, missing/partial states and counting; notebook annotation survival moves to its project. Imported atlas provenance remains current. |
| G08 — Journal interaction | Adventure-notebook project; frozen target/author, multiline/draft/cancel semantics. Not required for Project02 closure. |
| G09 — Managed media | Adventure-notebook project; native/file/display/backup lifecycle. Not a current engine trial prerequisite. |
| G10 — Traveller/claim identity | Project02 Arc01/03 owns supported save/replica and native/unresolved/user-attested evidence. Culture affiliations and journal narrative composition move to their projects. Inability to prove save-specific native ownership must remain explicit, not trigger unlimited identity archaeology. |
| G11 — Population/precision | Unnumbered spatial-evidence research, with agreed question/consumer/budget/stop before commissioning; accepted result feeds exploration tools. No undiscovered-star gate on Project02. |
| G12 — Geometry | Spatial-evidence research selects and evaluates methods; exploration-tools project implements the accepted contract. No mathematical library is selected here. |
| G13 — Frontier composition | Exploration-tools project after the applicable research/culture inputs. Existing routing is reused; new visualization/stronger B05 route guarantees are not silently added. |
| G14 — Regions/names | Spatial-evidence research and traveller-culture presentation; observed/derived/user-labelled claims stay separate. Not a Project02 completion gate. |

Decision-register scope update: D01–D08 retain friendly-selection/conditional-engine/optional-FTS direction; D06 startup preference is still provisional until Duncan answers. D09's separate active/read scope applies now, while journal authorship belongs to the notebook. D10's honest discovery evidence applies now, culture semantics later. D11/D12/D14 move with spatial research/cultures; D13 moves to the notebook. Research trials need a separately agreed useful question, consumer, concrete budget and stop/fallback condition. This routing notice does not commission any of them.

The old Gates/Decisions/Arc labels below are historical, not a second current roadmap. Field/API claims remain qualified by their dated sources. Future experiments retain necessary outcomes and errors using normal project records, without a new mandatory capture framework.

Initial research date: 2026-10-03; augmentation: 2026-10-11. Original version observations remain dated and require refresh at adoption. Documentation/source examination only: no dependency was added, no prototype was compiled, and no benchmark was run. Version observations below are research inputs, not approved dependency pins.

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
| D09 | Separate active play, read-query scope and journal authorship | Confirmed requirement; explicit cross-save queries preserve active selection and distinct narratives. |
| D10 | Discovery is claim; culture affiliation belongs to one traveller | Confirmed domain requirement; no manual claimed/cherished flag. Account-to-save attribution requires G10 evidence. |
| D11 | Derive culture structure from geometry/ownership, not a user radius | Confirmed requirement; no selected formula, geometry crate or combined score. G11/G12 are adoption gates. |
| D12 | Preserve historical culture identity when territories adjoin | Confirmed scenario; proposed stable-ID/connection-history mechanism. No automatic narrative merge. |
| D13 | Make journals and managed images explicit Arc06 capabilities | User-required workflow; frozen context, file import, explicit clipboard action and storage lifecycle are proposed mechanisms, tested by G08/G09. |
| D14 | Research candidate populations and native regions early | Required investigation, not evidence that undiscovered systems or their ownership can currently be enumerated. G11/G14. |

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
| G05 — Interactive catalog | Sanitized real snapshot; generated 100k/1m observations with declared distributions; repeated imports, name/location lookups, filtered joins, grouped history, frequent short note/attachment metadata writes during ingestion and long analytics | Target warm ordinary queries p95 under 200 ms; 1m-row summaries under 2 s on the recorded host. Record memory, temporary disk, DB growth, cancellation, UI responsiveness and short-write commit latency; ordinary journal commits target p95 under 200 ms excluding human editing and media encoding. No lost/duplicated writes or starvation. If misses occur, diagnose query/layout before switching engines. |
| G06 — Multiple interfaces and recovery | REPL owner + concurrent CLI and MCP clients; simultaneous ownership start; owner death; interrupted import; cancellation; migration failure; storage exhaustion | No independent writer race, partial visible generation, silent data loss, or permanently stuck ownership. If user-visible coordination is brittle, compare SQLite on the same scenarios before adoption. |
| G07 — Historical truth | Discovery present → absent → present; partial/malformed snapshot; duplicate sightings; main/expedition conflicts; annotations after re-extraction | Correct source/date labels and counting semantics. Missing/failed parsing does not become false deletion; prior facts and annotations remain reconstructable. |

## Rust-guidelines reading applied to this design

The user explicitly requested [rust-guidelines](</Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/rust-guidelines/SKILL.md>). The following selected portions shaped decisions; this is a targeted planning pass, not a claim to have read every guide or to have audited all Rust code.

| Material | Applied design consequence |
|---|---|
| [API design](</Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/rust-guidelines/guides/02-api-design.md>), API-23/37/39/40/46 | Concrete errors; different ID/address types; validated boundaries; concrete implementations first; hide database types from interface contracts. |
| [Type design](</Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/rust-guidelines/guides/05-type-design.md>), TD-03/06/07/13 | Newtypes and explicit states, forward-compatible unknowns, human-oriented display distinct from serialization. |
| [Anti-patterns](</Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/rust-guidelines/guides/11-anti-patterns.md>), AP-06/09/23/25/29/33/40/41/44/50/59/67 | No recoverable filesystem panic, silently ignored scan errors, primitive address confusion, ambiguous nested option states, leaked DB types, or blocking native work on async executors. Measure before optimizing. |
| [Concurrency](</Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/rust-guidelines/guides/07-concurrency-async.md>), selected ownership/cancellation/channel guidance | Worker ownership, bounded queues, explicit shutdown and cancellation, generation-based session publication. Tokio's primary docs qualify what cancellation can actually stop. |
| [CLI output/UX](</Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/rust-guidelines/guides/14-cli-tools/04-output-and-ux.md>), CLI-21–27/58 | Friendly interactive labels; machine IDs in structured output; terminal-aware prompts/color; diagnostics on stderr; ordinary cancellation. |
| [CLI configuration](</Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/rust-guidelines/guides/14-cli-tools/05-configuration.md>), CLI-29–33 | Explicit precedence, validation and migration. Preserve the established project path instead of introducing a distracting config-location migration. |
| [Project structure](</Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/rust-guidelines/guides/12-project-structure.md>), PS-02/08/11/14/24 | Thin binaries, focused shared libraries, tested MSRV, optional heavyweight features, explicit native-build responsibility. |
| [Documentation](</Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/rust-guidelines/guides/13-documentation.md>), DC-05/29/31 | Describe errors, trait/selection contracts, and user-facing limitations alongside APIs. |

For implementation, reread the particular guides for the actual code being changed and refresh candidate dependency versions. Keep authoritative SDK docs ahead of illustrative snippets if they differ.

## 2026-10-11 research extension and library leads

Historical local skill links have been repaired to the current skill installation; the initial reading claims remain dated. No dependency pins changed. Current API pages were consulted for clipboard/terminal/geometry leads; no native integration or geometric prototype was executed. The Rust type-design guidance (TD-03/TD-06) was reread for separate traveller/culture/attachment IDs and explicit unavailable/conflicting/insufficient-data states; CLI-58 was reread for terminal-aware interaction and ordinary cancellation. Keep existing logging/configuration conventions and thin shared interfaces. Scope and data contracts are specified before concrete Rust API signatures.

| Area | Lead / design consequence | Adoption proof |
|---|---|---|
| Terminal text paste | [crossterm Event](https://docs.rs/crossterm/latest/crossterm/event/enum.Event.html) exposes `Paste(String)`; do not promise arbitrary binary paste through Reedline | G08 multiline submit/cancel and target freezing |
| Clipboard images | [arboard Clipboard::get_image](https://docs.rs/arboard/latest/arboard/struct.Clipboard.html#method.get_image) is an explicit native image-read candidate; may return decoded pixels rather than original encoded bytes | G09 desktop/unsupported/headless host matrix, format/size limits, ownership of captured bytes |
| Image presentation | [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) is a display reference; external viewer/metadata fallback needed | G09 supported terminal witness; no universal terminal claim |
| Spatial adjacency | [CGAL 3D triangulations](https://doc.cgal.org/Manual/latest/doc_html/cgal_manual/Triangulation_3/Chapter_main.html) describes Delaunay geometry | G12 3D/degenerate/coarse-coordinate and missing-population proof; reference does not commit us to CGAL/FFI |
| Multiscale boundaries | [Original alpha-shape paper](https://arxiv.org/abs/math/9410208) and [CGAL manual](https://doc.cgal.org/latest/Alpha_shapes_3/index.html) describe families of shapes | G12 justify scale selection, boundary behavior and holes; not parameter-free by assertion |
| Graph analysis | Existing petgraph's [algorithm module](https://docs.rs/petgraph/latest/petgraph/algo/) offers connectivity and paths | G13 shared domain queries; frontier semantics remain application work |
| DuckDB geometry | [Spatial functions](https://duckdb.org/docs/current/core_extensions/spatial/functions) offer spatial operations | Verify dimensionality, units and behavior before use for 3D galaxy distances; relational population queries do not require this extension |
| Game-region names | Independent [nms_namegen](https://github.com/stuart/nms_namegen) implementation is a derivation lead | G14 reproduce names/galaxy/boundaries in game; not Hello Games authority or a population/ownership source |

The extended source references support the stated leads, not the complete feature design. Full Rust 3D library selection, exact region mapping, candidate-star acquisition and owner-to-traveller attribution remain assigned research. [Journals/media](journals-and-media.md) and [cultures/exploration](cultures-and-exploration.md) carry the detailed constraints.

## Additional focused gates

G08–G14 augment G01–G07; none has been run. Record fixture/source identity, commands, host/tool/library versions, all relevant outcomes and limits when executed. Geometry evaluation should separate tuning examples from held-out acceptance cases, and preserve negative results.

| Gate | Workload / evidence | Decision rule / owner |
|---|---|---|
| G08 — Journal interaction | Explicit/implicit targets, ambiguous names, multiline/text paste, stale/unknown location, Escape, save switch and watcher movement during composition, failed write/recovery, grouped/chronological reads and cross-save scopes | Frozen target/author, no silent retarget or partial submit, exact expected listing/attribution, durable retry. Arc06 with Arc01 scope contract. |
| G09 — Managed media | Real file and supported native clipboard capture, nonimage clipboard, SSH/headless, terminal/viewer fallback; identical bytes with distinct captions/authors; corrupt/oversized files, interrupted writes, disk full, unlink, dedup usage, clean-directory backup/restore | Integrity and associations survive; physical usage counts unique blobs; unsupported surfaces are explicit. Adopt clipboard/display/codec dependencies only after recorded target support and resource limits. Arc03/06. |
| G10 — Traveller/claim identity | Four saves/two accounts, shared native owner fields, copied versus synced replicas, two travellers with bases/notes on one world, foreign discoveries present in a save, conflicts and missing identity evidence | No automatic account-to-all-saves ownership or narrative merge. Record proven/unknown/user-attested attribution separately; preserve histories across re-extraction. Arc01/02/08. |
| G11 — Candidate population and coordinate precision | Inspect actual source fields and supported sources; distinguish known discoveries from unvisited-system enumeration; check same-voxel system indices and precise position availability; independent in-game existence/location/status witnesses where possible | Declare supported populations, coverage and resolution. Never infer stars or globally undiscovered status from absence. If enumeration fails, keep explicit capability gap for Operator disposition/re-entry, not a silent pass. Begin alongside Arc02/03; Arc09 owns delivery decision. |
| G12 — Culture geometry | All fixtures in cultures/exploration: singleton, pair, intervening systems, full 100-system ball, fixed-boundary holes, chain, bridge, neighbouring owners, missing data, outliers, tied/degenerate coordinates; held-out shapes and invariance checks | Select explicit boundary/scale/measure rules satisfying the examples; no user radius or warp range defines culture. Unknown coverage/precision cannot inflate completeness. No automatic named-culture merge; record limits. Arc09 with Arc08 identity. |
| G13 — Frontier query composition | Inventory/interior-gap/corridor/outward objectives; frontier versus current-origin versus route distances, ties, warp limits, foreign owner and cross-galaxy controls; CLI/REPL/MCP equivalence; analysis during note writes | Exact results against independent synthetic oracles; observed game witnesses for actionable real candidates; return approximate/insufficient-data where warranted; keep active save and author unchanged. Arc05/08/09. |
| G14 — Regions and names | Research game subdivision semantics; inspect name/ID sources and candidate derivation implementation; compare across galaxy/region boundaries and renamed/unknown cases; compare terminology with astronomical subdivisions | Region identity distinct from culture and name; separate observed/derived/user-labelled evidence and explain semantic differences. Unsupported derivation remains explicit. Arc02/08. |

## Change record

2026-10-11: expanded G05 to include journal-write latency/contention, added D09–D14 and G08–G14, and recorded primary-source leads for native images and geometric/region research. This is a design extension prompted by Operator gameplay examples, not implementation acceptance. Original G01–G07 obligations and conditional DuckDB/optional FTS decisions remain.
