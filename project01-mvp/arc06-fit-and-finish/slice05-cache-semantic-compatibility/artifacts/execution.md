# Slice05 CC Execution Intake and Readback

Date: 2026-10-04
Role: CC, proposed-done only; all ledger statuses remain open pending independent CDC.

## Assignment and worktree fences

- Assignment: `arc06-fit-and-finish/slice05-cache-semantic-compatibility/cc-prompt.md` and sibling `slice-plan.md` in the planning worktree.
- Planning root: `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/planning`; branch `planning`; HEAD/issuance commit `8ae5a1a129f382626c71d6e6db46d26cb7883ec8`; clean at intake.
- Source root: `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/cache-semantic-compatibility`; branch `codex/cache-semantic-compatibility`; HEAD `a800950ef3558fcf463517696c3c140061e7353d`; clean at intake.
- Main remains pinned at `a800950ef3558fcf463517696c3c140061e7353d`. No source edits preceded this record.
- Source commit allowlist: `crates/nms-cache/src/error.rs`, `src/lib.rs`, `src/serialize.rs`, `src/freshness.rs`, `tests/load_integration.rs`.
- Planning commit allowlist: this intake, `artifacts/validation.json`, `closing-report.md`, and `ledger.md` under this slice directory. No prompt, plan, parent ledger, sealed evidence, or source consumer changes.
- No merge, push, release, amend, or main-branch movement. Source and planning commits remain separate and use the repository-required two co-author trailers exactly once each.

## Required inputs read

Text-loaded and treated as binding: this CC prompt; slice plan; slice ledger; Project01 plan §13; Arc06 plan approved scope/current execution; closure-preparation approved expansion/active assignment; Project01 and Arc06 ledger repair requirements; repository `AGENTS.md`; project-management guides README including Expedited Mode; collaboration-framework execution/CC sections; engineering-methods implementation-prompt authoring CC intake; work-verification ledger/evidence guides; testing discipline/validation gates; Rust-guidelines overview and selected authoring/error/API/doc workflow; Rust anti-patterns AP-01–06 and AP-33; Rust error handling EH-01–04, TD-07, and DC-01. Relevant normative ranges whose first batched output was truncated were re-read in narrower contiguous ranges.

Slice04 final review §Next gate, author-baseline JSON, and author-probe are marked Reference-only by the prompt and were not used as authority. Existing CC requirements and current assignment prompt control this work.

## Source baseline and readback

Read the complete existing cache implementation and tests at the isolated source HEAD: `nms-cache/src/lib.rs` (14 lines), `data.rs` (44), `error.rs` (32), `serialize.rs` (215), `freshness.rs` (139), and `tests/load_integration.rs` (79). Also read startup `main.rs::load_model`, `watch.rs::drain_watch_events`, Makefile format/lint/test recipes, workspace/package manifests, and the locked rkyv dependency stanza. `rg CacheError crates` found uses only inside `nms-cache`; there is no downstream exhaustive match to update. The startup and watcher both use the shared cache serialization path, so no consumer edits are required.

Read the relevant public model fields and constructors in `nms-core/src/player.rs` (`PlayerBase`, `PlayerState`) and `system.rs` (`System`, `Planet`) plus the complete current integration fixture. The fixture currently has one system and planet, zero currencies, and no base; tests will use meaningful populated values and compare semantic fields.

The baseline cache writes raw locked-rkyv 0.8.15 bytes, decodes with `rkyv::from_bytes`, reports only Serialize/Deserialize/Io, and freshness is solely mtime-based. `load_or_rebuild` already warns and falls back to parsing the save, rebuilds from source, and best-effort writes through the shared serializer. Preserve its signature and fallback/write behavior.

## Bounded contract readback

Implement a 16-byte header: ASCII `NMSCACHE`, schema revision `u32 LE = 1`, semantic revision `u32 LE = 1`, followed by the unchanged rkyv `CacheData` payload. Revisions live internally in `serialize.rs`; they are separate from game SaveRoot version, package version, and `cached_at`. Archive/layout or codec changes require schema assessment; changes in cache-model meaning or identity require semantic revision assessment/bump.

Deserialize checks in strict order: (1) fewer than 16 bytes -> `TruncatedHeader { actual }`; (2) magic -> `InvalidMagic`; (3) schema -> `SchemaMismatch { found, expected: 1 }`; (4) semantic revision -> `SemanticMismatch { found, expected: 1 }`; only then checked rkyv decoding. Compatibility mismatch precedence beats payload corruption; schema mismatch beats semantic mismatch. Current header with invalid payload remains `CacheError::Deserialize`, without panic. Copy payload into `rkyv::util::AlignedVec<16>` and use checked `rkyv::from_bytes`.

Expose public non-exhaustive `CacheCompatibilityError` with Debug/Clone/Copy/PartialEq/Eq, Display, Error, and the specified variants; add `CacheError::Incompatible`, preserve old variants/messages, identify actual and expected revisions in display, and expose the nested compatibility error through `Error::source`. Re-export it. Keep public read/write signatures and atomic temporary-file rename lifecycle.

Preserve mtime-only `is_cache_fresh`. A rejected fresh cache must warn, parse source, return `was_cached=false`, and rewrite a compatible current envelope; the next warm read returns `true`. Parse failure propagates and leaves rejected cache bytes intact. `no_cache` skips read and write. Write failure warns while returning the parsed source model. Do not edit CacheData/core/save/graph/query/watch/CLI/repl/main/manifests/lock or save/private-cache/config files. Do not add hashes, multi-writer redesign, performance claims, or reusable validation product.

Test the exact envelope bytes/round trip, misaligned borrowed input, each truncation/magic/revision case and precedence, checked decode error, Io/source-chain observability, headerless historical archive rejection and rewrite, schema/semantic rejection/rebuild/warm hit, corrupt-payload rebuild, malformed-save byte preservation, stale/missing/no-cache behavior, deterministic FileTimes freshness, and cache-write failure fallback. Preserve all existing assertions. Use only tempdirs.

## Validation and evidence obligations

Required order from prompt: `make format`, `make lint` before tests; `cargo test --offline --locked -p nms-cache`; `make test`; `cargo build --offline --locked --workspace --all-features`; `cargo test --offline --locked -p nms-cache --doc`, setting `CARGO_NET_OFFLINE=true` for make gates. Capture each command's exact cwd/argv/exit/stdout/stderr, summaries, and blocked/unrun status in `artifacts/validation.json`; raw streams must be direct subprocess bytes. Format changes outside the source allowlist must be identified and excluded from the source commit. Prompt records known unrelated gaps (no-default-feature tests, workspace coverage 81.16% vs 95%, strict docs/package/MSRV/performance); they remain open and are not accepted by these gates.

## Preflight exceptions and recovery

- Initial `cargo metadata --offline --locked --format-version=1` from source exited 101 because `android_system_properties v0.1.5` was not cached. This was a dependency-metadata preflight limitation, not a product validation result. Recovery: host-filtered `cargo metadata --offline --locked --filter-platform aarch64-apple-darwin --format-version=1` succeeded and identified the installed rkyv 0.8.15 source manifest.
- `cargo info rkyv@0.8.15 --offline --verbose` emitted a warning that Cargo could not update its read-only global last-use SQLite database; package metadata was still printed. The successful host-filtered metadata command is the source-path authority.
- A few large batched reads returned tool truncation notices; each affected normative range was recovered with smaller contiguous reads before this intake was written.
- An initial direct lookup of the prompt beneath the source root missed because the file is in the planning worktree; `rg --files` located the authoritative path. No repository file was changed by these attempts.

## CC disposition

Proceed with the approved production implementation and bounded tests. Treat all acceptance/ledger rows as open until independent CDC review; this record is a CC execution/readback, not independent acceptance.

## Execution and validation outcome

Implemented the 16-byte compatibility envelope, public typed compatibility error and nested source chain, checked aligned payload decoding, public API re-exports, and the `load_or_rebuild` compatibility/freshness documentation. The shared cache read/write path and its existing atomic rename behavior remain intact. No startup/watcher consumers, manifest, lock, CacheData, save, graph, query, CLI, REPL, private cache, or config files were changed.

Expanded the integration fixture to include a system, two populated planets, one named base with address/position, player address, and nonzero units/nanites/quicksilver. Added direct current-header, misaligned slice, error-precedence/error-chain/IO, headerless archive, schema/semantic mismatch, corruption rebuild/warm-hit, malformed-source byte preservation, stale/no-cache/write-failure, deterministic mtime, and save-version-independence coverage. Existing lifecycle tests remain and now compare populated semantic fields.

Final source paths changed: `crates/nms-cache/src/error.rs`, `src/lib.rs`, `src/serialize.rs`, `src/freshness.rs`, and `tests/load_integration.rs` only. `make format` and `make lint` passed on the final source state before tests. Then all gates passed: `cargo test --offline --locked -p nms-cache` (18 unit + 13 integration; doc test target 0), `make test` (all workspace test binaries and doc targets; all 30 summaries passed, including the 204-test and 135-test suites), `cargo build --offline --locked --workspace --all-features`, and `cargo test --offline --locked -p nms-cache --doc` (0 doc tests, successful). Exact argv, cwd, exit status, each complete stdout/stderr stream encoded losslessly from subprocess bytes, and parsed test summaries for all 30 gate/iteration attempts are in `artifacts/validation.json`.

The first lint runs found and corrected test import/type issues and one Clippy alignment idiom. The first two focused-test runs exposed and corrected a fixture assertion that assumed planet-index ordering/values. These attempts are preserved in validation.json; criteria and scope were not changed. The successful final-state gates were rerun after those fixes and again after the final documentation edits. Cargo emitted an unrelated `proc-macro-error2 v2.0.1` future-incompatibility warning; successful commands exited 0. Preflight metadata failure/recovery and bounded read-truncation recovery are recorded above.

Known repository-wide no-default-feature, 95% coverage, strict docs/package/MSRV/performance gaps stated by the assignment remain open and were not asserted as passing. No evidence justifies a parent plan, arc scope, or policy change. No CDC acceptance or closure is claimed.

## Source citation register and source commit

Relevant implementation sources were read at the source checkout: `crates/nms-cache/src/lib.rs` (all 14 lines), `data.rs` (all 44), `error.rs` (all 32), `serialize.rs` (all 215 baseline lines), `freshness.rs` (all 139 baseline lines), and `tests/load_integration.rs` (all 79 baseline lines). Consumer boundaries read: `crates/nms-copilot/src/main.rs::load_model` (lines 352–360) and `crates/nms-copilot/src/watch.rs::drain_watch_events` (lines 19–43), both using the shared cache module; Makefile recipes (lines 180–200); root and cache Cargo manifests and locked rkyv stanza. Relevant model definitions read: `nms-core/src/player.rs` (`PlayerBase`, lines 64 onward; `PlayerState`, lines 104 onward) and `nms-core/src/system.rs` (`Planet`, lines 67 onward). The installed locked dependency implementation was read at Cargo's metadata-resolved rkyv 0.8.15 `src/api/high/checked.rs` lines 195–282 and `src/util/alloc/aligned_vec.rs` lines 1–145 and 550–582; checked `from_bytes` and `AlignedVec` support the chosen safe decode path.

The exact source commit is `8d2cf00fba41689f817cad35ec835315ba9aa415`. Post-commit inspection confirmed only the five allowlisted source/test paths and exactly one occurrence of each required co-author trailer. The first default-sandbox `git commit --only` attempt could not create `.git/worktrees/cache-semantic-compatibility/index.lock`; the same already-authorized exact-path commit succeeded when rerun through the reviewed escalation path. No source worktree changes remain.
