# Slice36 first actual refinement — config/session documentation

2026-10-25 UTC. First ACTUAL refinement1/5, four remain. Initial prompt/plan/literals/controls preserved. Same CDC gpt-6.1-sol High and sole existing CC gpt-6-luna High, standing Autonomous Arc06 only. This immutable assignment supersedes initial only at the demonstrated documentation findings below. No product policy/scenario/API/testing expansion.

## Frozen pin and ownership

Dedicated application-state-api-documentation baseline48b6, initial held diff831ed311756ec178f6c48f1a133d9aa66751fe3eb0fdf130fa71d05a67729c42 /identityda20f488. Actual format40a93c68 andlint35c6b3f2 complete0/fullraw/end/equalbeforeafter/status independently checked; no runtime credit. CDC full config.rs/paths.rs/session.rs production/docblocks and every E1–E3 assertion walk closed. Initial source patch preserved as DATA artifacts/initial-held.diff. All three-source non-doc bytes/tests/five payload layouts frozen. This correction permits ONLY Rustdoc/adjacent documentation blanks in config.rs and session.rs. paths.rs and every other path frozen byte-exact to held candidate. No source/index/gate work until coherent readback; then sole CC owns these writes with resolved dedicated-prefix/exactallowlist checks for EVERY target BEFOREsubmission/write and explicitcwd.

## Closed correction

F1 config truthful contract. Module says all path getters do not check returned paths: restrict this to cache-path construction/explicitfile fast path and explicitly distinguish effective_save_file/save_path filesystem checks and directory searches. SaveConfig::path first summary must be a complete single source-line sentence, then exact precedence explanation: explicit file, legacy file, explicit dir search, legacy dir search. CacheConfig::path first summary also complete on one line. Config::cache_path_for needs Panics: if override absent and home cannot be determined; override avoids home lookup. Preserve actual field/default/getter literals and all valid docs.

F2 promised load/env teaching and lookup/error boundary. Add one meaningful module `no_run` fallible example with actual imports Config/Path demonstrating Config::load_from(Path::new("config.toml"))? and Config::load()? plus Config::default().apply_env_overrides via a mutable value; no environment mutation or actual runtime config/save access. Clearly labelled compiled/not-run/read-only configuration calls. No asserted host-derived values. Key load/load_from/apply_env docs link to this real module example; Examples section if used comes BEFORE Errors/Panics. Explain load_from parsing's direct deserialize then Value fallback and original document error on both failures, without promising other frontend semantics. effective_save_file says supplied account directory/newest current save/nonrecursive/no automatic default account discovery, preserves ignored directory errors and literal explicitfile behavior. No new host fixture/scenario/test.

F3 whole session oracle labels. Current view's Base slot is name and Player/Address slots None, so it does not actually observe PositionContext::label as issued. Replace ONLY example-local position projection with `Option<(&'static str, String, (u64,u8))>`: derive kind via match, call actual p.label(), borrow p.address() for packed/reality. Example shape:
```rust
let position = s.position.as_ref().map(|p| {
    let kind = match p {
        PositionContext::Base { .. } => "base",
        PositionContext::PlayerPosition(_) => "player",
        PositionContext::Address(_) => "address",
    };
    let address = p.address();
    (kind, p.label(), (address.packed(), address.reality_index))
});
```
This is private doctest code, no new public helper/derive. Keep all eight whole SessionState fields, existing independent address/GG/galaxy/count/filter/range literals and transition order. Independently expected position labels at EVERY existing whole view assertion: player="player position"; Base="Home" even after model base replaced HOME/GG9; manual="0x300100000003"; absent=None. Existing expected packed/GG tuples unchanged (GG9P5 0x500100000003, BaseGG0P2 0x200100000003, manualGG7P3 0x300100000003, followingGG0P1 0x100100000004). Use readable multiline layout, not assertion weakening or expected projection built from observed/setup state. All original opposed untrimmed base failure, full status string/trailingLF and reset-all nonvacuity/None-follows controls retained. No new transition/scenario.

F4 session prose. Complete sync_from_model first summary on one line and remove the duplicated explanatory paragraph while preserving all original anchor semantics. from_model/system_count docs explicitly counts whole model across galaxies including marked systems, not only selected galaxy; no need new marked fixture. set_position_base error code span must show actual `Base not found: "{name}"` with quote characters and NO literal backslashes (Rustdoc prose is not a Rust string). Preserve lowercase/no-trim/snapshot/error-state behavior. No production fix.

## Pair, hold and original release route

Read back full F1–F4 and unchanged initial contract. Apply this closed two-file docs correction, every replacement count in memory before write; retain real failed editing observations proportionately. Captured offline make format→make lint then IMMEDIATE source/index/gate HOLD, no post-pair edits/runtime. Same /private/tmp/nms-slice36-cc-capture.py/raw append log, complete raw before fragile postprocess, originalTMPDIR. No direct duplicate compiler/validator diagnostics.

CDC targeted F1–F4 review then original eight groups only when released. Actual post-edit scoped Cargo JSON must zero-owned/exact24 residual diagnostics plus captured checker, not synthetic-control credit/fullcrate strict. Same original commit/trailer/normalhooks/frozen exact return/independent acceptance/retro/fullM7.6+required37 ownership. No merge/push/install/network/publish/release/new arc/Project02. Four actual refinements remain, no reset/extra authority.
