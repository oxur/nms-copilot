# Slice15 CDC independent verification

## Initial review — 2026-10-19

Reviewer: actual CDC context `/root/cdc_arc06`, OpenAI/Codex requested/applied launch `gpt-6.1-sol` High. Implementer: separate CC `/root/cdc_arc06/cc_slice15`, selected `gpt-6-luna` High. Two contributors, Arc06-only Autonomous Mode. Reviewed initial immutable `cc-prompt.md` issued aec768f against committed source `e254466ffc0e2ab7c2a6ed23f355c3cf6d5bc8ee`, direct parent130034e. CDC did not implement or repair Rust. Current verdict: **return bounded correction; slice remains open**. Historical capture gate is independently held and no waiver is approved.

### Source, transport and scope

Source commit contains exactly the six authorized paths and both trailers once; source parse/cache/main/MCP/REPL/fixtures/dependencies remain unchanged. Runtime inspection confirms same SaveRoot parse supplies snapshot/state/version, legacy delta-only semantics, capacity1 enqueue/retry/cancellation, failed-parse baseline retention, owned debouncer/worker join and complete PlayerState-driven graph selection. No runtime defect was reproduced against those assigned rules. This does not prove the unimplemented application coordinator/cache or parent composition.

CC's three original return byte digests verified on canonical transfer: closing-report266f49e2e59b0d02705cda7cb60f5871409c66abdb2d294af79b46b0ecc1786c; executionf03a623ebe8542f56305159da631e206bb10bfbed7316cb381b92856435b79b8; validation50258fb09774bdbfbe99b169b7adf7d22087311367e6af26c0cc1c171f4a55ec. CC had force-staged those three ignored workbench records despite its clean-worktree handback claim. CDC copied them unchanged, then un-staged only those known CC-owned paths with `git restore --staged -- ...`, preserving all files and the source commit. Source tracked/index state is now clean. That was transport/index cleanup, not a code repair.

### Independently reproduced gates

All13 required source gates plus committed-diff whitespace gate pass at e254466; exact commands, original inherited TMPDIR, selected env/overrides, UTC times, separate raw channels/byte counts/SHA256/exit and complete harness populations are in `artifacts/cdc-review-receipts.json`. Authorized host used after native sandbox restrictions were established by the current PTY preflight and retained CC failures; no canonical TMPDIR substitution. Format/lint precede tests; format produced no source change.

- Focused core87 + graph171 + watch32 =290, three doc harnesses0.
- Cache38 =19 unit +19 distinct load_integration tests; CC's prose calling the second19 a repeated library count is wrong.
- Route-cache4; native MCP7 (including accepted startup16-case matrix/controlled bind retry and live subscriptions/recovery); no-default copilot lib220.
- Whole workspace951 passed across24 nonempty harnesses (plus one empty root harness), all zero-failure/ignored; nine doc harnesses0.
- Workspace build/doctests/docs, strict cache broken-link docs, working and committed diff checks pass. Existing proc-macro-error2 future warning and existing query/graph/config/core documentation warnings remain at untouched source paths; no new warning was found. These original parent docs/distribution/coverage/MSRV obligations remain open.

### Findings and exact correction

F1 [correctness-grade test gap; CU-02/CU-04]: watcher.rs436+ native stream test checks currencies/version but never compares the complete PlayerState. Its previous-only oracle only checks previous.voxel_x, and movement retains the same currencies/version as the prior event while not checking incoming current_address/current_reality. A hybrid old-position or wrong-reality PlayerState could pass. Malformed recovery at618+ checks only movement/current units/version and never applies the real full event through the public GalaxyModel API. Required complete source-state and file→update→graph checks are not delivered by these partial assertions. Add complete literal states for every phase, opposed state/version movement, actual full-stream atomic replacement, failure no-mutation and public graph application, retaining existing positives.

F2 [correctness-grade test gap; CU-03]: graph model.rs1080+ seeds a planet directly into the primary map without production insert_system indexes and checks only the imported system name plus one planet/base. The assigned complete map-preservation oracle is not present. Seed a coherent imported system with its planet through insert_system; snapshot complete systems/planets/bases and assert exact preservation alongside complete states/selection through the cases. Runtime method itself matches the contract.

F3 [API quality; CU-01/CU-06]: new public WatchUpdateHandle at watcher.rs41 lacks Debug required by Rust API-06 (MUST). Add a nonempty manual Debug without exposing private debouncer internals. Complete the public full-stream docs with no-initial/unchanged behavior, pending-unsent shutdown discard and queued-read/disconnect semantics; add a usable public API example. Existing legacy public handle need not be retrofitted beyond this scoped change.

F4 [evidence/report fidelity; CU-07]: current full waves pass but known historical attempts lack complete fields, and current prose miscounts the cache denominator. Captured JSON contains48 command occurrences but42 unique original observations:39 commands in three full13 waves plus3 initial commands; each initial record is copied into all three nested waves. That is96 raw channel occurrences/84 unique channel observations, valid base64 throughout. Do not count copied records as new attempts. The separate unavoidable historical gap below is not resolved by correction.

| Known uncaptured family | Known observation | Unavailable fields / bounds |
|---|---|---|
| Earlier lint/compiler failures | CC reported2 (nested-if; undeclared serde_json fixture); no new dependency retained | Exact command/source/cwd/time/env/exit/separate channels unknown in complete capture |
| Standalone watcher diagnostic | Reported `cargo test -p nms-watch -- --nocapture`,32 pass | Per-attempt time/env/separate channels unavailable; source identity not established |
| Isolated single-test diagnostic | Reported pass | Exact selector/source/time/env/exit/separate channels unavailable |
| Later sandbox aggregate failures | At least2 reported; complete population unknown | Per-attempt command/cwd/source/time/env/exit/separate channels unavailable |

Original complete captured aggregate failure exit101/six watcher timeouts remains in prior_attempts[2]. New passing runs cannot restore unknown historical channels or source identity. Initial reading receipt also does not itemize complete required-full anti-pattern/source extents reliably (for example graph extract is described as relevant flow inspection). The next intake must state actual current full/section extents and qualify original limitations honestly; no claim of earlier reading is inferred from a current read. The required prompt defines the capture obligation; framework shared invariants prohibit silently weakening it for closure. Operator decision is reserved until runtime/test/doc corrections and independent review finish.

### Row disposition and bubble-up

CU-05 accepted/reproduced: implementation owns/debouncer-stops/signals/joins worker, full capacity1 helper ordering/cancellation and native owner disconnect pass. API-19's nonblocking-destructor advice is contextual here: the binding approved lifecycle chooses a controlled synchronous join rather than detachment, with no general arbitrary-I/O shutdown deadline promised; nonpanicking best-effort join warning is implemented. This is an explicit author choice within the adopted ownership direction, not a new guarantee.

CU-01/02/03/04/06 remain open for F1–F3; CU-07 remains open for F4/reporting and historical evidence authority. Seven rows/one done/six open. No implementation acceptance or parent closure. Initial assignment returned/reviewed; correction gets preserved cc-prompt-iteration01.md, first refinement of five. All prior captures/reports remain immutable prefixes/values.

The slice is still the complete library event increment; planned16 single app coordinator/full normal/headless cache and every placeholder/history/raw-field/original closeout gate stay required. No architecture, original scope/criterion, cache semantics, investigation or approval waiver is adopted. No Slice16 dispatch through incomplete15.
