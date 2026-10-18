# CC assignment — Slice13 MCP watcher settings

Execute `project01-mvp/arc06-fit-and-finish/slice13-mcp-watch-settings/cc-prompt.md`. Implement the existing watcher-setting correction and return proposed-done to Sofie/CDC through Operator relay. Two-Contributor Workflow and Expedited Mode continue; no CRC. Seven WS rows start open. This is implementation, tests, user/API docs and evidence together, not a new investigation.

## State, authority and exact boundaries

Accepted dependency `a101bd44b277f06c75ed7260b470b31cd5510130` (Slice12), direct parent88c5980; Slice12 formal closure `ae9bd828bb614f6348a0cba5fba237130fa275e4`. Resolve this prompt's actual introducing commit and actual planning intake HEAD separately. Create/reuse `.worktrees/mcp-watch-settings`, branch `codex/mcp-watch-settings`, exactly from a101bd44; inspect git worktree list first, do not reset an existing mismatched/dirty checkout. Source commit directly parentsa101bd44. Main6d183bde, accepted12/11 and all other dependency worktrees/Project02 are held. Exact pins/digests are in immutable artifacts/author-baseline.json; verify all of them at intake and exit. No source/save/config writes outside the candidate and disposable test fixtures. No integrate/install/push/release.

Exactly four source paths may change:

1. `crates/nms-copilot/src/main.rs`: private config resolver, two MCP construction calls, equivalent configured REPL start and focused inline tests/imports. Common save/cache loading, CLI/config schema/precedence, branch semantics, REPL drain/dispatch/session/cache behavior otherwise frozen.
2. `crates/nms-copilot/src/mcp/mod.rs`: additive configured entrypoints, retained default wrappers, private watcher config forwarding and their affected lifecycle tests/rustdoc. Preserve run_mcp_server, DeltaTaskGuard and apply_deltas_loop runtime byte-identically; no notification/locking/lifetime/transport/bridge redesign.
3. `crates/nms-copilot/tests/mcp_live_integration.rs`: disposable Unix child configuration, additive disabled-watch native stdio/HTTP tests and helper plumbing; existing phase/recovery/session/subscription/system-label/stdout/EOF/kill controls retained. No timeout increase or ignored tests.
4. `README.md`: existing watcher-settings/MCP user explanation/example and truthful remaining limits only.

Everything else frozen: config.rs/paths.rs/watch.rs, cache serializer/load/revisions2/2/layout/types/error APIs and fixtures, model/delta/query/reference/notify implementation, manifests/dependencies/lock, existing route tests and other source/docs. No generic service/config/test framework, dependency, new config option/CLI flag, API removal, path canonicalization workaround, polling policy or zero-debounce clamp. Public legacy MCP entrypoint signatures/default semantics remain. The only additive public APIs are the two configured entrypoints below. This details already planned existing-settings behavior, without replacing the architecture or selecting a new duplicate-producer/cache policy. If that boundary cannot meet the contract, stop affected work for CDC; do not widen scope or weaken tests. Any new inquiry needs prior Operator agreement on usefulness/applicability/budget.

CC planning writes only four return paths under this slice: `artifacts/execution.md`, `artifacts/validation.json`, `closing-report.md`, and `ledger.md` Evidence/Notes cells. Record intake before source edits; create real reports only when work exists. Keep all seven statuses open and all criteria/verification/significance/origin cells unchanged; CDC accepts independently. Plans/prompts/author baseline/CDC records/parents/earlier slices are immutable to CC. Preserve every prior failed/retried attempt and raw channels, append dated corrections rather than rewrite history.

## Binding behavior and concrete recipe

### 1. Resolve once in main, use it in both modes

Add private `configured_watcher(config: &Config, save_path: Option<&Path>) -> Option<nms_watch::WatchConfig>` (local naming discretion). Return None if watch_enabled is false or the already resolved save path is absent. Otherwise return existing WatchConfig with the **exact supplied path** copied and `debounce: config.watch_debounce()`. Defaults stay enabled/500ms; configured137ms and0ms are forwarded literally. No filesystem access, second auto-detection, existence probe, config reload or fallback path selection in this resolver. Save path resolution/load already precedes it.

Compute the optional owned value once after successful initial load, before the headless/REPL split. Headless passes it to the configured MCP entrypoint. REPL starts its existing producer from a clone using `watch_config.clone().map(start_watching).transpose()` (Result<Option<WatchHandle>,WatchError>); preserve enabled-success announcement, disabled blank line and warn-and-continue failure handling. Replace/remove the now redundant private main::start_watcher and unused WatchHandle import; do not change save resolution or move cache loading. Background MCP receives the remaining owned configuration. The two normal-mode producers remain; both must obey the same enabled/debounce/path values. Never retain the unconfigured MCP call in either main branch.

Recommended uncompiled sketch, binding data flow:

```rust
let watch_config = configured_watcher(&config, save_path.as_deref());
// headless branch:
mcp::run_headless_with_watch_config(model, transport, watch_config);
// normal branch: start the existing REPL watcher from watch_config.clone(),
// retaining warn-and-continue/announcements; then:
mcp::spawn_mcp_background_with_watch_config(model, transport, watch_config);
```

Existing Arc wrapping, cache paths, source/save_version and session defaults stay at their current sites. Settings are startup-only; enabled=false disables subsequent file ingestion, not initial save/cache loading or the MCP server itself. No global runtime settings or implicit change to no_cache.

### 2. Preserve legacy APIs; add explicit configured entrypoints

Keep public `run_headless(model, transport, save_path: Option<PathBuf>)` and `spawn_mcp_background(model, transport, save_path: Option<PathBuf>)` signatures and behavior for external callers. Each is a thin wrapper: Some path maps to existing WatchConfig with default500ms; None maps to disabled. Delegate to additive functions with exact names/signatures:

```rust
pub fn run_headless_with_watch_config(
    model: Arc<RwLock<GalaxyModel>>,
    transport: Transport,
    watch_config: Option<nms_watch::WatchConfig>,
)
pub fn spawn_mcp_background_with_watch_config(
    model: Arc<RwLock<GalaxyModel>>,
    transport: Transport,
    watch_config: Option<nms_watch::WatchConfig>,
)
```

Move the old runtime/thread/readiness bodies into those configured functions with only their watcher-construction input changed. Optional private default-config helper may factor the two wrappers and provide a direct typed default test; no new public type or trait. The configured background function moves its owned Option<WatchConfig> into the existing `thread::spawn(move || ...)`; no borrowed config outlives main. Configured headless uses the current thread/runtime. Keep service states/readiness/transport/failure diagnostics unchanged.

Change the private MCP start_watcher to accept `Option<nms_watch::WatchConfig>` by value. None returns None immediately. Some forwards the original configuration unchanged to nms_watch::start_watching; use a pre-move path borrow/clone for existing logs as necessary. Do not reinsert Default::default, overwrite debounce, derive a second path or read user config inside MCP. Missing/malformed/notify failures still warn and return None so serving continues. Preserve the owned WatchHandle through run_mcp_server and existing teardown guards/lock boundaries exactly; this slice is constructor wiring only. Adapt existing lifecycle tests to construct explicit config while retaining all prior failure/owner-drop assertions.

### 3. Exact typed controls and actual native regressions

Main inline resolver tests exercise:

- Config::default with `save_path=None` → None; enabled=false with a present arbitrary nonexistent path → None (no I/O needed).
- Enabled/default with path → same exact path and500ms.
- Enabled137ms and enabled0ms → same exact path and137ms/0ms, respectively. Never compare a value to its own getter as the sole expected result.

MCP inline tests retain None/missing/malformed and retained-owner/later-write/drop behavior. A pure legacy-default adapter test checks None and exact path/default500ms. Source inspection of both configured bodies/private helper verifies that this typed value is the actual value consumed by start_watching. Do not add timing assertions that claim the OS delivers at precisely137ms; typed propagation plus real enabled/disabled integration is the chosen proof.

In native child helpers, isolate **Unix child-only** HOME in a unique TempDir retained until child termination, containing `.nms-copilot/config.toml`. No process-global environment changes or user's actual home/config writes. Local dirs-sys0.5.0 home_dir checks HOME on Unix; baseline source reference recorded. Child args retain `--headless --no-cache --save <fixture>` (and actual --http address for HTTP), so source is explicit and cache composition remains unclaimed. Use existing fixtures/save_bytes, launch/stdio readers, HTTP/session/SSE/readiness, EOF/kill cleanup and bounded RPC/phase timeouts. On non-Unix preserve the old test launch behavior; new HOME-isolation/disabled checks may be cfg(unix), HTTP additionally feature=http. No unsupported Windows execution claim.

Extend StdioProcess/HttpChild helper construction with a private explicit config choice (enabled bool and debounce_ms in a small test-only parameter/helper is acceptable). Their ordinary spawn convenience delegates to enabled=true/debounce137 on Unix, so both existing enabled tests become nondefault-positive controls, independent of the Operator's config. Keep their entire behavioral sequence and all old assertions: stdio A→B, five actual parse failures/retained B, C→D; HTTP A/B/C/D, D system equals initial A, real resource subscription/session/list/smoke/stdout controls. Existing650ms invalid-write spacing is still safely above137ms+100ms stability; update explanatory comment only, no relaxed barrier or longer deadline. Existing9 actual observations remain with same shapes and additional tests may add observations; derive actual totals.

Add one disabled stdio child test and one disabled HTTP child test:

1. Explicit isolated `[watch]\nenabled = false\ndebounce_ms = 137` and actual valid phase A fixture; launch server and complete real initialize/initialized/readiness/session as applicable. Tools and resources must work. Read `where_am_i` and `nms://player/location`, assert full PHASE_A fields and initial system label using existing helpers.
2. Atomically replace the fixture with valid phase C bytes (reality9/X1), verified by existing save_bytes production parser check. Observe for **at least1500ms** after replacement, repeatedly reading both actual tool/resource values and asserting they remain full PHASE_A and initial system label. Use e.g.100ms between samples, existing bounded RPC deadlines for each pair, and require samples both before and after the1500ms endpoint (avoid calling a helper with an already expired endpoint). No sole final snapshot, cached client result, unobserved sleep or fake notification as oracle. No notification absence proof required; selected contract is that disabled input cannot mutate the served model. Do not call wait_location_update when deliberately disabled.
3. Assert existing stdout discipline (stdio JSON only, HTTP empty) and terminate/reap via current cleanup. For HTTP retain actual bind-collision-only retries, health and session initialization. Share small existing setup helpers if useful; no generic harness rewrite. Tests cover run_headless's configured runtime; static actual-main and shared-start-helper composition covers background configuration, not a full REPL PTY claim.

The old implementation's MCP defaults would ingest C and fail these disabled checks. Nondefault typed tests reject default500 replacement; enabled native tests reject disabling every watcher. Configfalse/None does not suppress MCP startup or initial source loading. A successful user config parse and actual production launch are necessary, not only tests of a constructed Rust Config.

### 4. Documentation and preservation

README MCP/watch section explains `.nms-copilot/config.toml` `[watch] enabled = true; debounce_ms = 500` with actual TOML lines, false example/meaning and startup-only application to headless stdio/HTTP and normal-mode producers. Initial save still loads; disabled watcher serves that in-memory model. Keep transport commands, tools/resource subscription, stderr/stdout guidance, and useful existing descriptions. Update the old statement that watch configuration is outside described behavior. Full startup-cache lifecycle, unknown-system resolution and single-producer/write-through composition remain outstanding; do not claim they are fixed. Rustdoc for both new entrypoints explains Some/None/exact config and lifetime; old wrappers explicitly document default500 behavior and link to configured variants. No new API example claiming a tested live server startup.

Guideline application: CA-02 MUST (07-concurrency-async.md) transfers owned config into existing thread; OB-17 SHOULD (04-ownership-borrowing.md) preserves retained handle/DeltaTaskGuard scope; EH-01 MUST / AP-09 AVOID keeps recoverable watcher failures in existing warn-and-serve path with no new panic; DC-01 MUST documents additive public APIs. General recommendations to hide Arc/RwLock or add builders do not justify breaking existing signatures or adding types here. Existing detached outer service/bridge lifecycle remains current behavior, not newly endorsed/changed; duplicate-producer design remains14. Testing Behavior/Contracts and Proportionate Proof select current real child regression plus typed/static forwarding, no new reusable verifier or timing lab.

## Required reading and preflight

Installed skill root `/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills` or verified equivalent. Read full required text in complete bounded outputs, recover truncation and record extents. Structured queries are separate coverage, not full semantic text reading. Before source edits, record identity/clean fences/digests/reading/data results and a coherent source-cited contract readback in allowed execution/validation files. Explain None versus initial load, exact config forwarding versus defaults, preserved legacy wrappers, background ownership, actual disabled/enabled test oracles and all held composition owners. Complete coherent preflight authorizes execution without another permission round.

1. **Required-full:** applicable AGENTS.md, this prompt, current slice-plan.md and ledger.md. **Required-section:** latest Slice12 final closure in cdc-verification.md; current parent plans' Slice12 closure and Slice13/14 sections and current parent ledger R06-S12/S13/R02/R03. Earlier source/evidence history reference-only. The current prompt governs its bounded source mutation, never silently overrides parent acceptance.
2. **Required-full:** project-management/guides/README.md; work-verification/guides/03-row-closure.md; testing/guides/01-testing-discipline.md. **Required-section:** collaboration-framework SKILL Notes for Codex/When to use; engineering-methods/guides/01-engineering-methodology.md roles/shared invariants/Two-Contributor; guides/07-implementation-prompt-authoring.md Required reading and CC intake; project-management/guides/03-planning-top-down.md useful increments/investigation agreement; work-verification/guides/05-independent-verification.md assigned reviewer/retention; testing/guides/03-validation-gates.md gate selection/proportionate proof/minimum/gate evidence. Conditional future iteration: PM layout preservation and iteration handoff guides before any iteration; no editing issued prompt.
3. **Required-full before Rust edits:** rust-guidelines/SKILL.md and guides/11-anti-patterns.md per repository. **Required-section:** actual CA-02 owned move (07-concurrency-async.md), OB-17 RAII guards (04-ownership-borrowing.md), EH-01 (03-error-handling.md), DC-01/02 (13-documentation.md). Keep repository twyg/log/dependencies/edition rather than skill defaults.
4. **Required-full currenta101bd44:** main.rs, mcp/mod.rs, native mcp_live_integration.rs, config.rs, paths.rs and copilot Cargo.toml. **Required-section:** watcher.rs WatchConfig/default/start_watching and owned handle; README intro/watch/MCP sections; watch.rs drain caller contract only; local installed dirs-sys0.5.0 target_unix_not_redox::home_dir for child HOME. Source bodies and tests needed to interpret existing phase/recovery/cleanup are included in full native read. Remaining graph/cache source reference-only/frozen unless needed to explain failure. Author sketches not compiled or exercised; CC must implement and run actual gates.
5. **Required-data:** immutable author-baseline.json source/parent/closure/pins, all allowed4 and protected digests plus all three frozen_mcp_regions marker-delimited bodies, compare actual entire populations at intake; query actual worktree heads/clean and source parent/branch. Source commit exit: compare exact four-path boundary, protected hashes, all three frozen MCP runtime region hashes and all held clean pins; planning exact allowed return paths/trailers/parents and seven unchanged criteria/statuses. No need to reread all prior Slice12 raw receipts; they are accepted dependency provenance, not current execution evidence.
6. **Conditional:** any real sandbox notify/loopback denial retains original failed attempt then unchanged authorized-host retry with original TMPDIR. Missing required source/guide or conflicting pins blocks dependent work; preserve diagnostics, ask CDC via Operator. A new runtime-policy defect outside these constructors stops affected work for CDC; no unilateral inquiry.

## Validation, commits and return

Run from candidate, in order; gates13/14 are separate cwd checks. Preserve each attempt before launch with exact argv/cwd, timestamp, inherited RUSTDOCFLAGS/CARGO_TARGET_DIR/CARGO_NET_OFFLINE/RUSTUP_TOOLCHAIN/TMPDIR (null means observed unset), explicit overrides/effective strict flag/offline config, full separate stdout/stderr base64/byte lengths/SHA256/exits and harness selected/passed/failed/ignored/filtered counts. Preserve failures and successful retries, never substitute an empty environment map for unknown. No prescribed test total; compare actual outputs with contract. Native child-only HOME overrides are part of test setup; do not alter command TMPDIR or global HOME.

1. `make format`
2. `make lint`
3. `cargo test -p nms-cache`
4. `cargo test -p nms-copilot --all-features --test route_cache_integration`
5. `cargo test -p nms-copilot --all-features --test mcp_live_integration -- --test-threads=1 --nocapture`
6. `cargo test --all-features -p nms-graph -p nms-watch -p nms-cache -p nms-query -p nms-copilot -p nms-cli`
7. `cargo test -p nms-copilot --no-default-features --lib`
8. `make test`
9. `cargo build --workspace --all-features`
10. `cargo test --workspace --all-features --doc`
11. `cargo doc --no-deps --all-features -p nms-core -p nms-graph -p nms-query -p nms-cache -p nms-watch -p nms-copilot`
12. Actual `RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links' cargo doc --no-deps --all-features -p nms-cache`
13. Candidate `git diff --check`
14. Planning `git diff --check`

Current dependency observations: cache38/route4/native2/focused737/no-default219/workspace933 at Slice12, ordinary doc warnings/proc-macro-error2 future advisory; these are historical counts, not target totals. Strict cache docs retain actual flag; new MCP intra-doc links must also resolve under ordinary docs (inspect warnings, do not introduce broken links). Preserve full current native source fixtures and original TMPDIR; failed restricted attempts do not justify a no-watch test substitution. Host retry unchanged argv/cwd/env/input, no longer timeouts or canonical path overrides. No ignored/weakened required test, no missing-current-evidence waiver.

Commit source with `git commit --only` exactly four named changed paths, directly parenta101bd44. Separate planning commit only the four current return files; stage only new named files if necessary. Every assistant commit includes exactly once:

Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>

Capture actual source/planning parents/pathsets/trailers and held clean fences; current prompt introducing commit differs from later receipt/intake. Preserve command failures and inventory corrections without repeated guessed totals: derive actual attempt/channel totals from arrays. A final small metadata receipt may record its preceding close commit without inventing its own future hash.

Return closing-report.md identifies exact assignment/source/issuance/intake/commits, actual before/after behavior, all seven WS rows proposed/open, gates/platform limits, artifact inventory and bubble-up against assigned increment. Useful outcome: existing watcher enabled/debounce settings applied by actual main MCP calls and both-mode producers, compatible default wrappers, real disabled native and enabled nondefault proof. Candidate remains isolated/unintegrated/uninstalled. Planned14/CDC retains full startup/live-cache C04/old-current/warm/write-through both modes, unknown placeholders/promotion, duplicate producers and their settings composition, broader historical questions; original raw repair/export/import/format/docs/distribution/coverage/parent independent/Operator/integration/release owners unchanged. Supply concise handoff and project-relative closing report path. CDC independently reviews; all seven rows remain open until then.
