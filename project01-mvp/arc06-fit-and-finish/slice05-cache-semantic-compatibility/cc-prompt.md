# CC assignment — Slice05 cache semantic compatibility

Execute this initial **production implementation** assignment; return evidence through the Operator to CDC/Sofie. Project01 / Arc06 / Slice05. Operator explicitly approved all three scope directions on2026-10-04; this slice implements the cache prerequisite. Two contributors and Expedited Mode apply. Preflight then proceed: no new permission gate. Source/main stays unchanged; no decoder/identity repair or release in this assignment.

## Source and planning boundaries

Pinned source: `/Users/oubiwann/lab/oxur/nms-copilot` main at `a800950ef3558fcf463517696c3c140061e7353d`, clean. Planning: `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/planning`, branch planning; parent before this preparation `082270d720d7e69669c97711328b72b9db6d8309`. Record the actual issuance commit containing this prompt and open set; do not mistake its parent for the current planning HEAD.

Inspect git worktree list and both statuses first. If source main drifts or contains relevant dirty work, stop affected work and report; do not repin silently. Create the isolated source worktree at `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/cache-semantic-compatibility`, branch `codex/cache-semantic-compatibility`, from the full pinned SHA. If that path/branch already exists, verify its registration, clean state and exact baseline before reuse; if it differs, report rather than resetting/replacing it. Git metadata approval may be required by the environment; existing Operator scope is authorization. Keep main pinned and planning separate.

The authoritative binding design is slice-plan.md §Binding cache contract; this prompt supplies its implementation spine and proof. In any contradiction, report to CDC before dependent edits. Later identity/decoder policies in parent plans are roadmap, not authorization to implement them now.

## Required reading and intake

Before affected work, actually load the required text. Use bounded contiguous reads; recover every truncation. Existence, hashes/headings, another context's read or redirected logs do not prove you received content. Record source states, actual loaded extents and gaps in artifacts/execution.md, followed by a brief source-cited contract readback covering envelope/error precedence, fallback/bypass, scope and independent review. Recheck/reload contracts after context recovery. Complete intake permits execution, not acceptance.

Paths below use P=`/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/planning/project01-mvp`, S=P/arc06-fit-and-finish/slice05-cache-semantic-compatibility, R=isolated source worktree, K=`/Users/oubiwann/.codex/skills`. These are explicit path aliases, not shell variables to overwrite HOME/CODEX_HOME.

| Order | Exact input and state | Scope / purpose |
| --- | --- | --- |
|1 | S/cc-prompt.md, slice-plan.md, ledger.md, issuance commit | Required-full; active contract before any edits |
|2 | P/project-plan.md §13; P/arc06-fit-and-finish/arc-plan.md §Approved scope and current execution; P/closure-preparation.md §Approved expansion and active assignment; project/arc ledger added repair sections | Required-section, complete named sections; approved boundaries and remaining gates |
|3 | Repository AGENTS.md/user-supplied instructions; K/collaboration-framework/SKILL.md routing/Expedited and implementation-handoff sections; K/project-management/guides/README.md | Required applicable instructions; required-full README; established workflow, not a new contributor selection |
|4 | K/engineering-methods/guides/07-implementation-prompt-authoring.md §Required reading and CC intake; K/work-verification/guides/01-ledger-discipline.md and 02-evidence-strength.md | Required-section for intake; required-full verification guides; attestation and closure |
|5 | K/rust-guidelines/SKILL.md overview, document locations/selection and writing/error/API workflows; K/rust-guidelines/guides/11-anti-patterns.md AP01–06 and AP33; K/rust-guidelines/guides/03-error-handling.md EH01–04; K/rust-guidelines/guides/05-type-design.md TD07; K/rust-guidelines/guides/13-documentation.md DC01 | Required-section with complete pattern definitions; applicable Rust guidance before coding; retain additional governing skill obligations if triggered |
|6 | K/testing/guides/01-testing-discipline.md and 03-validation-gates.md | Required-full; meaningful oracles and proportionate mandatory gates |
|7 | R/crates/nms-cache/src/{lib,data,error,serialize,freshness}.rs; R/crates/nms-cache/tests/load_integration.rs; R/crates/nms-cache/Cargo.toml | Required-full at pinned source; all changed behavior and existing tests |
|8 | R/crates/nms-copilot/src/main.rs load_model; R/crates/nms-copilot/src/watch.rs drain_watch_events; R/Makefile format/lint/test recipes; R/Cargo.toml workspace edition/MSRV/dependencies; R/Cargo.lock rkyv package | Required-section, full named bodies/definitions; actual consumers and gates. Do not claim full-file reads |
|9 | Locked rkyv 0.8.15 local source src/api/high/checked.rs from_bytes and src/util/alloc/aligned_vec.rs AlignedVec/with_capacity/extend_from_slice | Required-section; verified APIs/alignment. Resolve installed registry path from Cargo, not from a guessed version |
|10 | S/artifacts/author-baseline.json and author-probe.rs; Slice04 final review §Next gate | Reference-only author reconnaissance and historical direction proposal; current approval is in parent plans |

No private save population or historical full packet query is required. Newly encountered binding dependencies must be loaded before affected work, recorded and resolved within authority. If a required guide is missing, report exact gap; do not invent its contents.

## Source reconnaissance and applied guidance

At pinned source, serialize.rs serialize/from_bytes are headerless; read/write delegate to them. freshness.rs load_or_rebuild accepts any newer readable archive, otherwise warns and rebuilds. CacheData contains save_version/cached_at but no model revision. Existing load_integration.rs tests only miss/fresh/bypass; it is the viable integration test home. Startup load_model and watch drain_write call shared cache APIs, so updating the shared serializer covers both write paths without consumer edits. Source-wide CacheError scan finds no exhaustive downstream matches; a public error variant still has external compatibility consequences.

| Guidance actually applied by CDC | Decision / proof |
| --- | --- |
| EH01 MUST and EH04 MUST | Recoverable compatibility failures use a named matchable error, no panic or opaque string; typed negative tests. Reuse manual Display/Error style, no dependency. Existing boxed startup error is not redesigned here |
| AP02 AVOID and AP33 AVOID | Borrow &[u8]; explicitly distinguish compatibility rejection and best-effort cache fallback/write warning. Source parse failure propagates; byte preservation test |
| TD07 SHOULD | New compatibility enum is non_exhaustive for future revision failures; existing CacheError extension is documented, not treated as universally backward compatible |
| DC01 MUST | Document new public type/variants/reexport and changed byte contract, plus semantic revision process |
| Testing Proportionate Proof | Extend actual deserialize/read_cache/load_or_rebuild tests and repository gates; no standalone validator or hidden product |

Wider API37/TD03 recommendations motivate the approved separate save boundary/identity directions; they are not changes in this cache slice. Rust 2024/MSRV 1.91 and existing dependencies/features/logging remain. No asynchronous lifecycle or new concurrency owner is introduced.

## Implementation spine

Binding behavior and error shapes are in slice-plan.md §Binding cache contract. Private helper names/test organization are local discretion; the following implementation structure is recommended within that contract.

1. **error.rs / lib.rs**: add documented `CacheCompatibilityError` with exact variants from the plan and standard traits, Display and Error. Add CacheError::Incompatible, its Display and source chain; reexport the nested error beside CacheError. Preserve existing variants/messages. External exhaustive matching changes deliberately; no version bump/publication in this slice.
2. **serialize.rs serialize**: define internal magic/header-length and separate schema/semantic u32 constants (both1). Get the existing checked rkyv serialization bytes; allocate output for16+payload, append magic and both to_le_bytes values, then payload. Return existing Result<Vec<u8>,CacheError>. Never serialize compatibility metadata inside CacheData or infer it from game version.
3. **serialize.rs deserialize**: first check length, magic, schema, semantics in that order, emitting exact nested errors with found/expected values. Bounds checks precede slicing, no panic for input length. After a compatible header, copy bytes[16..] to AlignedVec<16>::with_capacity/extend_from_slice and call checked from_bytes::<CacheData,RkyvError>, mapping to existing Deserialize. Manual u32::from_le_bytes byte arrays after the length check avoid fallible-conversion unwraps. Pseudocode is a design sketch, not author-tested production code. Do not use unchecked access, raw pointer alignment tricks or codec fallback on a rejected header.
4. **write_cache/read_cache**: retain public signatures and existing parent creation/temp write/rename semantics; they now write/read the envelope through the shared functions. No concurrent write or source fingerprint redesign. Reject old bytes; do not rewrite their decoded data into a new header.
5. **freshness.rs / lib.rs / serialize.rs docs**: distinguish mtime freshness from compatibility; explain old-cache source rebuild, separate revision meanings, exact equality and future semantic bump obligations. load_or_rebuild's existing fallback can remain mechanically unchanged if all tests show correct ordering. No-cache and parse/write failure behavior must remain as specified.
6. **serialize.rs inline tests / tests/load_integration.rs**: extend meaningful tests below; reuse the existing synthetic fixture/model helpers. Preserve existing assertions and create no user-directory files. Finish validation/evidence, commit exact source and planning allowlists separately and return for review.

Checked aligned copy adds O(payload) temporary bytes to the existing owned decode path. It is explicit correctness overhead; the original cached-startup<100ms promise remains a later measured project gate. No benchmark improvement claimed. Current envelope is not a repair of current address interpretation or active galaxy restoration.

## Behavioral oracles

Use synthetic fixtures in existing tests only. Give source Version 4720 and poisoned cached save_version 9999 so the old implementation cannot pass by coincidence. Assert meaningful populated fields (system/planet addresses, bases where provided, player state/currencies) as well as counts, using source fixture expectations or the pinned unchanged cold model. This demonstrates transport/fallback, not game semantic validity.

| Rows | Setup/action | Required observation / competing bug excluded |
| --- | --- | --- |
| CS02 | Serialize populated data; assert first16 bytes against literal ASCII plus `01 00 00 00` twice; read through serialize::deserialize and file read_cache | Exact independent header bytes; all populated fields retained; game save_version 4720 differs from header revisions1 |
| CS02 | Place the valid full envelope at a deliberately misaligned starting address in a backing byte vector, then pass that subslice to deserialize | Successful same data; construct offset after checking actual pointer alignment rather than assuming a vector base address |
| CS03 | Empty and short byte slices (lengths 7, 8, 15); valid length with wrong magic; schema 0/2; semantic 0/2; include invalid payload after mismatched headers | Exact TruncatedHeader(actual), InvalidMagic, SchemaMismatch(found,expected 1), SemanticMismatch(found,expected 1). Mismatch wins over payload invalidity. If both revisions mismatch, schema error wins |
| CS03 | Header-current plus invalid/empty payload; missing file | Deserialize for bad payload; existing Io for missing file; no panic. Display revision fields and nested source are observable |
| CS03/04 | Generate a populated headerless legacy archive using locked rkyv::to_bytes directly, with version 9999; direct deserialize/read_cache, then make it newer than source | Direct typed rejection; load_or_rebuild returns was_cached=false/save_version 4720 and source model, rewrites current header, next fresh load returns true/4720 |
| CS04 | Repeat real startup sequence for current-shaped envelope with wrong schema, and separately wrong semantic revision, both containing sentinel9999 and newer than source | Both rebuild from source and produce compatible warm hit; mtime alone cannot save stale semantics |
| CS05 | Newer current header with corrupt payload | Source rebuild false/4720 followed by valid warm hit; codec corruption fallback preserved |
| CS05 | Incompatible newer cache and malformed source JSON | Startup Err, cache bytes unchanged; rejected sentinel never served and no successful recache claim |
| CS05 | Missing cache; stale compatible sentinel cache | Source model/false; current cache written and usable. Retain existing missing/fresh/bypass tests |
| CS05 | Cache write path has a parent component that is a regular file, while source JSON is valid | Cache write fails with warning; successfully parsed model/false/4720 still returns. This preserves best-effort writes through the real startup route |
| CS05 | no_cache=true with an existing incompatible sentinel cache | Source4720/false and existing cache byte-identical; also existing no-cache missing-cache test remains |
| CS06/07 | Review docs and changed consumers; format/lint/focused/full tests/build/doctest gates below | Process explicit, callers compile, faithful outcomes. Headers alone do not prove subsequent semantic migrations |

Use temp directories and deterministic mtimes, not assumptions about scheduling. The source can be set to UNIX_EPOCH+1700000000 seconds and a deliberately fresh cache to+1700000100 with std::fs::FileTimes/File::set_times, available under declared MSRV; open the test file for writing. Before claiming freshness, assert is_cache_fresh. Set rebuilt cache mtime explicitly later than source before the warm-hit step if needed; record each transition. Never change the real save/cache timestamps. Integration tests use public exports and serialize module; inline tests can inspect private revision constants but byte-order oracle remains literal.

From the isolated source worktree, run **make format**, then **make lint before tests**, then `cargo test --offline --locked -p nms-cache`, `make test`, `cargo build --offline --locked --workspace --all-features` and `cargo test --offline --locked -p nms-cache --doc`. Repository Makefile targets cover all-feature/all-target lint and all-feature workspace tests. Use CARGO_NET_OFFLINE=true for Makefile gates if dependencies are cached. Format changes outside the allowlist must be surfaced, not committed. Fix in-scope failures and rerun affected gates; no ignored/weakened tests. Preserve every failed/blocked/unrun attempt and exact process outputs, counts and warnings in validation.json. A listed command is not an executed gate.

Known broader baseline gaps: no-default-feature tests fail, workspace coverage 81.16% vs95%, strict workspace docs/package/MSRV/performance composition remain open. They are not waived or claimed fixed here; no requirement to rerun an unchanged broad failing gate merely to deliver this envelope slice. Any new feature/build regression blocks acceptance and returns to CDC. No publishing/network mutation as validation.

## Exact permitted paths and commit/return contract

Source worktree modifications and source commit **only**:

- crates/nms-cache/src/error.rs
- crates/nms-cache/src/lib.rs
- crates/nms-cache/src/serialize.rs
- crates/nms-cache/src/freshness.rs
- crates/nms-cache/tests/load_integration.rs

Planning modifications and planning commit **only**, each prefixed `project01-mvp/arc06-fit-and-finish/slice05-cache-semantic-compatibility/`:

- artifacts/execution.md (create)
- artifacts/validation.json (create)
- closing-report.md (create on completion)
- ledger.md (evidence/proposed dispositions only; criteria/statuses preserved open)

Temporary build/test files may live in /private/tmp, never in user save/cache/config locations. Do not alter CacheData/core/save/graph/query/watch/interface code, Cargo manifests/lockfile, sealed author inputs, issued prompt, slice plan, parent plans/ledgers, predecessor packets or CDC records. If a target/source assumption fails or another path/API change is needed, report the precise gap and stop affected work; complete independent authorized work. Do not silently expand scope, reduce criteria or substitute a self-roundtrip for the oracles.

Commit the exact source paths on codex/cache-semantic-compatibility and the exact planning paths on planning. Use explicit `git commit --only -- <named paths>`, preserve unrelated staged work and inspect exact commit pathsets. Each assistant-authored commit has these trailers exactly once, contiguous:

    Co-authored-by: Codex <noreply@openai.com>
    Co-authored-by: Billo AI <ai-engineering@billo.systems>

No merge, push, release or amend of historical commits. Report both complete commit SHAs, main/planning/candidate statuses and source fixture fences. Validate JSON and whitespace in each worktree. Do not claim working-tree cleanliness if unrelated work exists; identify it without touching it.

execution.md records intake/readback, design application, implementation/tests and all failures/recoveries/deviations. validation.json retains command cwd/argv/exit/stdout/stderr, gate status and test summaries; capture channels directly (no paraphrased exact-output claims). It may include the seven-row evidence mapping; no separate validator product. closing-report.md walks all seven rows as CC proposed-done/open with precise evidence and limits, scopes delivered versus requested, and bubbles up future semantic-bump/identity/active-galaxy work. Leave ledger statuses open for CDC. Any unrun or failed required gate remains open; structural checks/CC commits do not close the slice, arc or project.
