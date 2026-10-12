# Slice08 closing report — proposed completion

## Assignment and source state

Assignment: `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt.md`.

Source implementation was performed on isolated branch `codex/galaxy-scoped-routing` from the accepted Slice07 baseline `7118a486ea43a776aa2451ec0807d43384fb954f`. The planning assignment was issued from planning HEAD `b792257db624d1a1c8edc35068c30481cf69648a` (parent `8aca066a1bf90e2eec29fd12b7bf8101e6740d29`). Main remained at `a800950ef3558fcf463517696c3c140061e7353d`; identity foundation `f57a38f7d1f00d1a3f5aa500ed5f7ce7d8c26eb8` and cache foundation `decd8d6c09c1199a6cdd01fd48aa50f3adef2b1a` remained separate. The source commit is `e1347bc24fafa5049f73854bede444cfa5fe2a8b`, directly parented by the required baseline. Planning commits contain only the four prompt-allowlisted planning paths.

## Outcome by ledger row

All rows are **proposed done for CDC review; status remains `open`**. No acceptance is inferred.

| Row | Proposed disposition and evidence |
|---|---|
| SR-01 | Proposed satisfied: active wrappers reject foreign addresses before distances; four explicit-address APIs use the selected galaxy without mutation; empty galaxy 7 returns empty. See graph query tests listed in `ledger.md`. |
| SR-02 | Proposed satisfied: complete scan, total distance order, qualified numeric key tie-break, planet-index association, inclusive 399/400 radius boundaries, invalid radius and zero/`usize::MAX` limits; collision/tie controls run in both insertion orders. |
| SR-03 | Proposed satisfied: absent IDs are typed before same-ID, distance, radius or limit work; foreign route targets return `NoPath { from, to }`; builders, 2-opt, and hop constraints validate IDs and galaxy membership. |
| SR-04 | Proposed satisfied: disconnected same-galaxy fallback is exactly 800 ly; injected cross-galaxy bridge edges are ignored by A*; KNN, warp-range and incremental edge composition stay galaxy-local. |
| SR-05 | Proposed satisfied: closed route retains the first/last system and exact 2,400-ly line distance; waypoint search advances beyond 60 decoys; reachability returns all 120 systems deterministically and excludes the injected foreign bridge endpoint. |
| SR-06 | Proposed satisfied: current-position mismatch is typed; explicit/base starts scope to their own galaxy without mutation; base-first foreign target handling, validation-before-radius/limit, 220-planet dedup, no implicit 100 cap, and round-trip count are exercised. |
| SR-07 | Proposed satisfied: unchanged genuine schema2/semantic2 fixture (360 bytes, SHA-256 `291b283d06db222bf355c259269328d23fef34d0d020c89012fb922bdf4d31c7`) checked-decodes and rebuilds. A richer current archive has independently expected typed route parity. Schema and semantic revisions stay 2. The one-system-per-galaxy predecessor fixture is not claimed as multi-node route evidence. |
| SR-08 | Proposed satisfied: real `nms route` subprocesses cover nearest-neighbor and 2-opt success, Lush targets distinct from the start, one-count round-trip, and missing-target nonzero/no itinerary; typed formatter tests verify shared names remain galaxy-qualified and legs are exactly 400 ly. Public route/error docs build. |
| SR-09 | Proposed satisfied with a recorded capture limitation: the final nine gates all exit 0 and have separated raw stdout/stderr captures; source and staged planning whitespace checks pass. Some early exploratory failures were not wrapped by the capture helper and their raw bytes were not retained; they are identified without fabricated receipts in `execution.md` and `validation.json`. |

## Changed public behavior and migration notes

- `GalaxyModel` keeps the four existing active-galaxy Vec-returning spatial methods and adds explicit-address counterparts: `nearest_systems_from_address`, `systems_within_radius_from_address`, `nearest_planets_from_address`, and `planets_within_radius_from_address`.
- `RouteError` adds `ForeignGalaxy`. `GraphError` adds `NoSystemsInGalaxy(u8)` and `TargetNotFound(String)`. `shortest_path` now returns typed foreign/missing failures and ignores cross-galaxy shortcut edges. TSP uses `NoPath { from, to }` for a valid foreign target.
- `build_route_from_order`, `two_opt_improve`, and `constrain_hops` now return `Result<Route, RouteError>`. All callers in the permitted graph/query paths were migrated. Repository-wide caller search found no additional non-test callers outside the allowlist; watch and MCP consumers were inspected and left unchanged.
- Route target semantics now scan complete selected-galaxy planet/system populations before dedup and truncation. Biome routing has no implicit 100-target cap. Waypoint routes remain best-effort and may include a direct final leg longer than `max_ly` when progress is unavailable or the guard ends.

## Validation and remaining limits

The final sequence ran in the candidate with `CARGO_NET_OFFLINE=true`: formatting, lint, scoped all-feature tests, full `make test`, workspace all-feature build, graph/query/cache doctests, broad no-deps documentation, strict cache documentation, and source/planning diff checks. The scoped suite reported 663 passed, 0 failed, 0 ignored; full `make test` reported 881 passed, 0 failed. Exact argv, cwd, relevant env, exit code, separate channel lengths/hashes/base64, and earlier captured attempts are in `artifacts/validation.json`.

The broad documentation command exits 0 while reporting existing unresolved `VP[0]`/`VP[1]` links in untouched `crates/nms-graph/src/extract.rs` and unclosed `<label>` examples in untouched `crates/nms-query/src/table.rs`. Strict cache documentation passes. Cargo also reports the pre-existing `proc-macro-error2 v2.0.1` future-incompatibility advisory. No dependency or cache envelope changed.

This slice does not prove real-save galaxy extraction semantics beyond the preserved fixture and test-built models. It does not accept live REPL/MCP routing, startup policy, session switching, duplicate-base policy, save-field oracle, portal UX, or strict numeric policy. The existing cache lifecycle and current in-scope tests passed, but startup and live-interface acceptance remain parent-owned. Parent arc/project rows remain open.

## Artifact inventory

- `artifacts/execution.md` — prompt intake, baseline receipts, implementation record, validation interpretation, and limitations.
- `artifacts/validation.json` — ordered final gates and captured command receipts with raw channel metadata/base64, plus earlier captured attempts and uncaptured-diagnostic disclosures.
- `closing-report.md` — this proposed closeout.
- `ledger.md` — Evidence/Notes additions only; all nine statuses remain open.

## Bubble-up to the arc

1. **Was Slice08 delivered as assigned?** Yes, as CC proposed-done. The authorized source paths, focused integration tests, evidence artifacts, required commits and commit trailers are present. Independent CDC acceptance remains pending for all nine rows.
2. **Did anything unforeseen arise?** The candidate worktree needed an escalated Git metadata write after the sandbox denied its branch lock; a later planning index staging attempt also needed escalation. A compiler typo and one formatter assertion were corrected. The broad docs gate retained pre-existing warnings outside the allowlist. Early unwrapped diagnostic failures have no retained raw-channel bytes; that capture gap is disclosed rather than backfilled.
3. **Does the parent plan need a change, and who owns it?** No parent plan change is justified by this evidence. CDC owns independent row-by-row reproduction and acceptance, plus any later strict combined-doc cleanup or live/session/startup work. The Operator remains the approval authority for any scope or policy change. No parent artifact was edited.

The iteration correction has now been implemented and freshly validated. The source correction candidate is the existing `codex/galaxy-scoped-routing` worktree based on `e1347bc24fafa5049f73854bede444cfa5fe2a8b`. The planning issuance commit is `09a460ff6591b6b487fa90b0381d24071e4d7770`; the current planning branch included the later receipt-only commit `6f9c282ac08a572b3d207c822270b1789c3a4262`. Source and planning commit identities, direct parents, exact paths and trailers will be recorded after the named-path commits below.

### Iteration01 row dispositions

All entries below are proposed dispositions for independent CDC review. The ledger remains nine rows with every Status `open`.

| Row | Proposed disposition |
|---|---|
| SR-01 | Exact active foreign-reference exclusion and explicit 0/9/empty-7 queries, canonical borrowed planets, and no state mutation are covered by new graph tests. |
| SR-02 | Complete population ordering, SSI ambiguity and opposing numeric/name tie witnesses, all biome predicates, insertion orders, radius boundaries and zero/max limits have literal tests. |
| SR-03 | Membership-before-galaxy/path order, exact `NoPath` foreign route semantics and typed missing-ID propagation are covered across graph/query entry points. |
| SR-04 | Foreign shortcut exclusion, disconnected fallback projection and positive KNN/warp-range/incremental edge weights are covered. `edges.rs` runtime is unchanged. |
| SR-05 | Exact hop normalization/marker retention, 60 non-progress decoys, 120-member reachability and closed 2-opt endpoint/order/distance projections are covered. |
| SR-06 | Named/base/ID/biome selection order, validation before limits, radius modes, no implicit 100 cap, current-position mismatch, mutation invariants and exact selected target count are covered. |
| SR-07 | Original 360-byte schema2/semantic2 fixture is unchanged and rebuilt; independent predecessor projection plus rich current-cache route projection are checked without claiming old-fixture multi-node evidence. |
| SR-08 | Native parser/query witness and actual NN/2-opt CLI subprocesses use the single Lush target and round-trip count; missing target has nonzero/no itinerary behavior; typed formatter rows pair qualified IDs and 400-ly legs. |
| SR-09 | All nine gates were freshly run in order; each command has direct separate channel capture. Historical diagnostics and prior missing Git staging channels remain disclosed and unrecovered, so this row remains open. |

### Corrected initial report statements

The earlier report's statements that `RouteError` adds `ForeignGalaxy`, that explicit spatial methods return empty for foreign addresses, and that the old implementation's foreign behavior represents the final contract are superseded by the Iteration01 source readback in `artifacts/execution.md`. The corrected contract is: active wrappers reject foreign references; explicit methods use the address galaxy; canonical planets come from `model.planets`; valid mixed-galaxy route pairs yield `NoPath { from, to }` after complete membership checks; route normalization resets distances and preserves supplied endpoint markers; and selected targets are counted before routing as the post-filter non-start IDs. Cache schema and semantic version remain 2 because these route/index results are derived and not persisted archive fields.

### Validation and evidence limits

The fresh ordered gates all exited 0. The scoped all-feature suite reports 670 passed, 0 failed, 0 ignored; `make test` reports 888 passed, 0 failed, 0 ignored. Formatting and lint pass; workspace all-feature build, doctests, strict cache docs, and both source/planning diff checks pass. Broad docs exit 0 with existing unresolved links in untouched `crates/nms-core/src/address.rs`, `crates/nms-graph/src/extract.rs`, and unclosed `<label>` examples in untouched `crates/nms-query/src/table.rs` (4 core, 3 graph, 2 query warnings). The existing `proc-macro-error2 v2.0.1` future-incompatibility warning remains. Detailed raw captures are in `artifacts/validation.json` under `iteration01.gates`.

A bounded lookup of available task receipts found no original stdout/stderr or exact status for the four prior unwrapped diagnostics, nor the raw bytes/status of the earlier source/planning staging failures. These remain explicit unknowns; current gate receipts do not reconstruct them. Thus the historical-evidence portion of SR-09 is not self-waived and requires CDC/Operator disposition. No source, planning, prompt, reviewer, or fixture boundary was expanded.

### Supporting artifact inventory

The five supporting artifacts are `artifacts/author-baseline.json` (sealed author baseline), `artifacts/author-probe.rs` (sealed baseline probe), `artifacts/execution.md` (CC intake/readback and execution record), `artifacts/validation.json` (preserved validation plus Iteration01 captures), and `artifacts/cdc-review-receipts.json` (independent reviewer receipts). The first two and reviewer artifact are unchanged/read-only. Report, ledger, and prompts are first-class records and excluded from this five-item inventory.

### Bubble-up

1. **Assigned work and gaps:** the seven source paths and four planning paths were used; all nine rows have proposed evidence, but remain open. The exact remaining known gap is missing original raw captures/status for named historical tool and Git attempts; no live-save/session/startup acceptance is claimed.
2. **Unforeseen issues:** test fixtures and one formatter text-count expectation needed correction; final gates pass. Existing broad-doc warnings remain outside the allowlist. Historical capture recovery was unsuccessful; no new product or policy issue arose.
3. **Parent plan and owner:** no parent-plan change is warranted by this evidence. CDC owns independent row verification and the capture-gap disposition; the Operator retains approval authority for scope or acceptance changes. No parent plan was edited.

All evidence remains CC proposed-done pending independent CDC review. No slice, arc, or project acceptance is claimed.

Source correction commit: `21494b963a5ffbfc5b283c5865a06fa3da096335`; direct parent `e1347bc24fafa5049f73854bede444cfa5fe2a8b`; seven original allowlisted paths; required adjacent trailers present exactly once. Initial commit attempt exit 128 due sandbox denial of worktree `index.lock` is captured, followed by successful exact scoped retry (exit 0).

First planning return commit: `b63db3133cbd7cee77d396b112d8413003afc9e3`, parent `6f9c282ac08a572b3d207c822270b1789c3a4262`, exactly four permitted paths and both required trailers. Initial commit attempt failed exit 128 on sandbox-denied index lock; its channels and the successful exact retry are recorded in `artifacts/validation.json`. A later scoped planning receipt commit adds this identity to the durable record.

Receipt follow-up commit: `6a6f304fb7b59998d3d1ff1a1211296995ca03c7`, parent `b63db3133cbd7cee77d396b112d8413003afc9e3`, three changed paths within the four-path allowlist, both required trailers. Its failed and successful exact commit attempts are preserved in `artifacts/validation.json`.


## Iteration02 execution status — stopped for CDC; not a slice close

Current assignment: `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration02.md`. The Iteration02 focused suite found a production discrepancy in closed-route waypoint handling. Required with 400 ly range: A9/B9/C9/B9/A9, waypoint flags `[false,false,false,true,false]`, legs `[0,400,400,400,400]`, cumulative `[0,400,800,1200,1600]`, total 1600, two targets. Observed: A9/B9/C9/A9, flags all false, legs `[0,400,400,800]`, cumulative `[0,400,800,1600]`, total 1600, two targets. The exact native failure and full channel receipts are in `artifacts/validation.json`. No production change or expected-value relaxation was made. Per the assignment, execution stopped for CDC; gates 04–09 and both source/planning commits remain unrun.

The four authorized test modules contain uncommitted, reviewable test-only changes. Planning files have corresponding uncommitted intake, validation, stop and ledger evidence updates. The latest source head remains `21494b963a5ffbfc5b283c5865a06fa3da096335`; no Iteration02 source commit exists. Current planning base remains `69ff089662eb081c7a23d2621053f599a0831af9`; no Iteration02 planning return exists. Main remains read-only at `6d183bde79dd7328c32d4739dca249f5f1f9cd6c`.

SR-08 remains CDC-accepted and `done`. SR-01–SR-07 and SR-09 remain `open`; this partial attempt accepts no row and closes no slice. Four original diagnostic groups, three Iteration01 uncaptured groups (third count unknown), the historical strict-gate environment gap, and the newly disclosed pre-capture/inspection limitations remain distinct and unresolved. Current fresh strict-doc gate was not run. The five prior supporting artifacts retain their existing contents except `artifacts/execution.md` and `artifacts/validation.json`, which now include this Iteration02 intake/attempt record; no new reusable artifact was introduced.

### Bubble-up to the arc

1. **Assigned behavior delivered?** Partial test evidence and literal oracles were added, but Iteration02 stopped before the full assignment or fresh ordered gate set completed.
2. **Unforeseen scope/architecture/acceptance/evidence issue?** The closed-route query projection exceeds the 400 ly warp range on the return leg and omits the prescribed waypoint. This is a production discrepancy requiring CDC disposition. Intake reads before capture and one later API-name read also lacked separate raw capture; all were disclosed in validation.
3. **Parent change required and owner?** No parent-plan edit is proposed. CDC owns review and contract/implementation disposition; CC should resume only after direction. All other row and historical evidence gates remain open.

This is an interim handoff, not a closeout. The plain relative assignment path is `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration02.md`.


Scope/readback audit after the stop verified that only the four permitted source files changed and all source diff hunks are additions (no deletions); only the four permitted planning records changed. Ledger remains nine rows/eight columns; SR-08 is `done`, and SR-01–SR-07/SR-09 are `open`. All 228 currently recorded channel objects validate; the original 134 and all original validation values are preserved. No further software gate was run after the mandatory discrepancy stop.


## Iteration03 closeout candidate — proposed done pending CDC

Iteration03 corrected the test configuration that led to the Iteration02 false production-bug claim. The previous failure showed the unconstrained \`None\` behavior; it did not exercise the intended 400 ly constraint. The corrected tests explicitly distinguish \`None\` from \`Some(400.0)\` for both route algorithms, retain exact route projections and state snapshots, and make the foreign-radius control explicit. CDC’s independent public probe had already confirmed both literal outcomes.

All nine Iteration03 gates passed in the assigned order. The focused five-package all-features run passed 680 tests in 19 suites; full \`make test\` passed 898 tests in 23 suites. Offline locked build, selected doctests, normal docs, strict cache docs, and both diff checks passed. The normal docs retain existing nms-query unclosed-HTML warnings. Gate receipts and their complete channels are preserved in \`artifacts/validation.json\`.

The source return is \`edd66642c051c04bc6ccdf1bd3b7d20f74a0f816\` (parent \`21494b963a5ffbfc5b283c5865a06fa3da096335\`). It changes exactly four test files and has the required adjacent trailers. The four planning records are being returned separately. No parent plan, CDC-owned review file, prompt, product code, cache schema, fixture, or accepted SR-08 behavior was changed.

**Row disposition:** SR-08 remains accepted/done. SR-01–SR-07 and SR-09 remain open and proposed-open; this iteration does not close them. The slice remains pending independent CDC row-by-row review and any Operator disposition. No arc or project close is claimed.

**Complete slice artifact inventory at closeout:** \`slice-plan.md\`; \`cc-prompt.md\`; \`cc-prompt-iteration01.md\`; \`cc-prompt-iteration02.md\`; \`cc-prompt-iteration03.md\`; \`ledger.md\`; \`closing-report.md\`; \`cdc-verification.md\`; \`artifacts/author-probe.rs\`; \`artifacts/author-baseline.json\`; \`artifacts/cdc-review-receipts.json\`; \`artifacts/execution.md\`; \`artifacts/validation.json\`. Iteration03 changed only execution, validation, closing report and the SR-06/SR-09 ledger notes; the other inventory items retain their prior bytes.

### Bubble-up

1. **Assigned behavior delivered?** Yes for the Iteration03 correction and gate set: the constrained and unconstrained route projections are separately configured and checked, and the accepted SR-08 behavior remains intact.
2. **Unforeseen scope, architecture, acceptance or evidence issue?** The Iteration02 discrepancy was a wrong test setup, not a product defect. The old false claim is explicitly retracted. Historical and Iteration03 capture diagnostics remain disclosed; the Iteration03 temporary \`pwd\` receipt was removed during wrapper-output cleanup before it was copied into the durable record, and that loss is disclosed.
3. **Parent-plan change and owner?** None proposed. CDC owns the independent evidence review and row disposition. The Operator retains scope and acceptance authority. No plan-level status change is implied by this CC return.

All four assignment paths are retained as provenance: \`arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt.md\`, \`.../cc-prompt-iteration01.md\`, \`.../cc-prompt-iteration02.md\`, and \`.../cc-prompt-iteration03.md\`.


## Iteration04 final return — 2026-10-11

This dated return supersedes prior Iteration03 statements that described the planning records as uncommitted and the earlier test totals. Iteration04 was assigned by `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration04.md` (CDC issuance commit `ae3cdef467be8f02f421190c04b8cefbb1b29750`). The complete assignment sequence is `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt.md`, `cc-prompt-iteration01.md`, `cc-prompt-iteration02.md`, `cc-prompt-iteration03.md`, and `cc-prompt-iteration04.md`. The iteration04 intake/readback and historical limits are in `artifacts/execution.md`; full receipts, attempts, per-row summary and fences are in `artifacts/validation.json`.

### Source return and validation

Candidate source branch `codex/galaxy-scoped-routing` returned commit `02260b778e6f6967fac14aa44073e948b3121d66`, directly parented by `edd66642c051c04bc6ccdf1bd3b7d20f74a0f816`. It changes exactly these four test files: `crates/nms-graph/src/query.rs`, `crates/nms-graph/src/route.rs`, `crates/nms-query/src/route.rs`, and `crates/nms-copilot/tests/route_cache_integration.rs`. The commit has the required adjacent Codex and Billo AI trailers. Source status is clean. The accepted SR-04 bridge test and Iteration03 None/Some(400) route test remain byte-identical; production prefixes, CLI integration, `edges.rs`, error definitions and genuine fixture remain unchanged.

All nine gate categories passed in order, with every first failure and retry retained. `make format` passed. `make lint` first exited 2 from three cache-test compile mistakes (unbound deserialized data names and wrong `Planet` field); the correction was limited to the test and lint retry passed. Focused all-features tests passed 682 tests (19 nonzero suites, 4 zero-test summaries); workspace `make test` passed 900 (23 nonzero suites, 10 zero-test summaries). Offline locked workspace build passed. Selected graph/query/cache doctest command passed, with all three packages reporting zero doctests. Normal docs passed with existing warnings: four in core, three in graph, two in query; the `proc-macro-error2` future-incompatibility advisory was also present. Strict cache docs passed with `RUSTDOCFLAGS=-D rustdoc::broken_intra_doc_links`. Final source and planning diff checks passed. No coverage, performance, package, live-session, startup or save-repair acceptance is claimed.

### Row-by-row disposition

| Row | Iteration04 CC evidence and disposition |
|---|---|
| SR-01 | `test_explicit_spatial_apis_return_exact_canonical_galaxy_views` now covers index-1 planets in both insertion orders, explicit 0/9 calls under active 7, all active wrappers, empty 7, exact IDs/PlanetKeys/distances/radius/limits, canonical borrowed pointers and player/active snapshots. Proposed complete; remains open for CDC. |
| SR-02 | `test_spatial_ssi_ties_and_all_planet_filter_fields_are_exact` adds literal qualified-ID ordering and numeric addresses for n=1/radius/planet ties, same-voxel SSI1 own/SSI2 other distances, reversed near/far insertion, four one-field filter misses, exact far key/boundaries and canonical borrow. Proposed complete; remains open for CDC. |
| SR-03 | `test_route_entrypoints_validate_exact_membership_and_foreign_payloads` checks exact missing/foreign payloads across shortest/build/2-opt/constrain, valid empty and one-hop projections, caller marker retention, and both TSP algorithms and return modes with foreign-before-missing precedence. Query typed error checks are included. Proposed complete; remains open for CDC. |
| SR-04 | Previously accepted/done by CDC; unchanged. The accepted positive nine-galaxy bridge/foreign-shortcut projection was verified byte-identical, and `edges.rs` was not changed. |
| SR-05 | Dense reachability now proves the exact 120-ID set with galaxy collisions, an isolated system, player0 and snapshots. The waypoint witness uses A/B/C and 60 same-voxel decoys in galaxy9 while galaxy0 is active, with exact IDs, flags, legs, cumulatives and total. Existing closed 2,400-ly and normalization proofs remain. Proposed complete; remains open for CDC. |
| SR-06 | Repeated named/ID targets prove first-occurrence retention before max1, exact max1/max2 routes, start removal/dedup, all-start and valid max0 typed failures, actual unknown-name and empty-galaxy errors, and snapshots. The accepted Iteration03 None/Some(400) test is unchanged. Proposed complete; remains open for CDC. |
| SR-07 | Typed predecessor projection adds all four radius APIs with independent literal results for fresh, genuine old and current paths. Assertions read actual decoded old/current `CacheData` metadata and raw fields and check canonical planet associations. The 360-byte fixture and revision2/2 remain unchanged; rich route literals remain. Proposed complete; remains open for CDC. |
| SR-08 | Previously accepted/done by CDC; unchanged. Accepted CLI integration file is byte-identical; current ordered gates were rerun. |
| SR-09 | Intake/readback, gate receipts, exact scope/control/fixture/protected-function audit and current source commit are recorded. Historical Iteration03 diagnostics are classified from accessible receipts without rewriting their strings or fabricating missing channels. Proposed complete with explicit evidence limits; remains open for CDC/Operator disposition. |

The candidate source change is test-only and does not alter the archive schema, semantic compatibility, runtime, APIs or fixture. The predecessor fixture remains the genuine checked 360-byte schema2/semantic2 archive with SHA-256 `291b283d06db222bf355c259269328d23fef34d0d020c89012fb922bdf4d31c7`. It is not claimed as multi-node route evidence.

### Planning return and artifacts

The authorized planning set is exactly `artifacts/execution.md`, `artifacts/validation.json`, `closing-report.md`, and `ledger.md` within this slice. The ledger remains nine rows/eight columns, preserves all criteria and first six cells, and keeps SR-04/SR-08 `done`; SR-01/02/03/05/06/07/09 remain `open`. Source and planning are separate worktree commits. The planning commit and any receipt-only followup identities, parents, exact paths and trailers are recorded in `artifacts/validation.json`; earlier planning return prose is historical and superseded here.

First-class slice documents: `slice-plan.md`, the five assignment prompts (`cc-prompt.md` and Iterations01–04), `ledger.md`, `closing-report.md`, and CDC-owned `cdc-verification.md`. Supporting artifacts: `artifacts/author-baseline.json`, `artifacts/author-probe.rs`, `artifacts/execution.md`, `artifacts/validation.json`, and `artifacts/cdc-review-receipts.json`. No other slice artifact was created or edited by this iteration.

### Bubble-up and remaining authority

1. **Was the assigned work delivered?** The seven rows' assigned test/evidence corrections and all nine fresh gate categories are present as CC proposed-complete evidence. CDC has not accepted the seven rows, and SR-04/SR-08 retain their prior independent acceptance.
2. **Did an unforeseen scope, architecture, acceptance or evidence issue emerge?** No product or plan change emerged. Historical diagnostics remain mixed: captured updater failures and pre-child wrapper rejections are distinguished from uncaptured inspections and the removed `pwd` receipt. Original, Iteration01/02 gaps and the historical strict-doc environment omission remain unresolved and unwaived. The Rust anti-pattern guide was consulted selectively, not read in full; the intake records this limit.
3. **Does the parent plan need a change, and who owns remaining decisions?** No parent-plan change is proposed. CDC owns independent row review and contract disposition; the Operator retains acceptance authority. Required session/live/full startup evidence, evidence-gated save repair, and original higher-scale owner/gate requirements remain outstanding.

All seven proposed rows remain open pending independent CDC review and any required Operator disposition. The slice, arc and project remain open.


## Iteration05 final return — 2026-10-11

This dated return supersedes Iteration04's proposed-complete descriptions for SR-01, SR-02, SR-06 and SR-09. It does not change the five prior accepted rows, their status cells, or any historical owner decision. The current assignment is `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration05.md`; the full intake/readback and receipt map are preserved in `artifacts/execution.md` and `artifacts/validation.json`.

### Assignment history

| Assignment path | Issuance revision | Status |
|---|---|---|
| `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt.md` | `b792257db624d1a1c8edc35068c30481cf69648a` | executed; superseded by later corrective iteration |
| `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration01.md` | `09a460ff6591b6b487fa90b0381d24071e4d7770` | executed; superseded by later corrective iteration |
| `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration02.md` | `4c3d405f90152159b0f56232d6c348b512638dfa` | executed; superseded by later corrective iteration |
| `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration03.md` | `50eac5224f200c2d4fe1d6b535d21d3931869fbc` | executed; superseded by later corrective iteration |
| `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration04.md` | `ae3cdef467be8f02f421190c04b8cefbb1b29750` | executed; superseded by later corrective iteration |
| `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration05.md` | `9a9bf1147ee6cdf9caeb72a4d7b3a8484bf745fe` | executed; current evidence returned for CDC |

### Source and fresh validation

Source commit `0e1e8b4239881a378bc8fb928ef25741c228e171` directly parents `02260b778e6f6967fac14aa44073e948b3121d66`; it changes exactly the two assigned test files: `crates/nms-graph/src/query.rs` and `crates/nms-query/src/route.rs`. The commit has exactly once, adjacent, the required Codex and Billo AI trailers. Candidate source is clean. Production prefixes and the protected route test body remain byte-identical; the protected body is 4,976 bytes with SHA-256 `57895ddf7c65a9da8def443972bedb1f0d72d64666881ba859b2cc1ae2b53663`.

All nine final gate categories passed in order. Focused all-features tests passed 682 (19 nonzero suites / 23 result summaries); workspace `make test` passed 900 (23 nonzero suites / 33 summaries). The selected doctest command passed with three zero-test summaries. Offline workspace build, format, lint, normal docs, strict cache docs and both diff checks passed. The first lint attempt exited 2 for a misplaced test import and is retained; corrected final format/lint and the complete final ordered gate pass followed. Normal docs retain four core, three graph and two query warnings plus the proc-macro-error2 v2.0.1 future-incompatibility advisory. Strict cache docs passed with the actual child `RUSTDOCFLAGS=-D rustdoc::broken_intra_doc_links`. No strict combined-docs, coverage, performance, packaging, live-session or full-startup acceptance is claimed.

### Nine-row accounting

| Row | Iteration05 observation and disposition |
|---|---|
| SR-01 | Reversed four-system index-1 fixture now supplies `PlayerState(A0, galaxy0, counters 7/8/9)`. Explicit spatial APIs assert exact system IDs and PlanetKeys/distances, exact radius boundary/invalid-radius populations, empty galaxy7, canonical planet borrows and state. Active wrappers have exact 0/9 populations and foreign/empty controls with full snapshots. CC proposes complete; remains open for CDC. |
| SR-02 | Near/far insertion is actually reversed. Each near planet independently misses exactly biome, subtype, infestation=false, or named-only. Far planet result is exactly `((far_id, 0), 800.0)` at radius800, empty at799, with canonical borrowed pointer. CC proposes complete; remains open for CDC. |
| SR-03 | CDC accepted in Iteration04; row remains `done` and byte-identical. No source change to its proof. |
| SR-04 | Previously CDC accepted; row remains `done` and byte-identical. Accepted bridge/edge evidence remains retained. |
| SR-05 | CDC accepted in Iteration04; row remains `done` and byte-identical. |
| SR-06 | Foreign-target rejection retains exact `NoPath { from, to }` for both algorithms before max0/max1/radius0 truncation, with supplied player0 and full snapshots after each call. The unbounded fixture retains all 105 literal biome targets and 106 hops, with player0 and full snapshots. CC proposes complete; remains open for CDC. |
| SR-07 | CDC accepted in Iteration04; row remains `done` and byte-identical. Genuine 360-byte fixture remains unchanged. |
| SR-08 | Previously CDC accepted; row remains `done` and byte-identical. Accepted CLI route integration remains unchanged. |
| SR-09 | Sixteen Iteration04 gate names each uniquely match an attempt; 14 old pointers resolve to another named attempt. Iteration05 final gates use stable nonnegative links and separate full channels. Historical missing channels and intake remain unresolved/unwaived. CC proposes complete for current evidence; remains open pending CDC/Operator disposition. |

### Iteration04 corrections and fences

Dated corrections in execution and the ledger supersede the earlier weak descriptions: SR-01's old fixture lacked player state and its final active-0 reset masked selection; SR-02's near/far fixture was not reversed, planet3 had two simultaneous filter misses, and radius800 was count-only; SR-06's two assigned tests lacked supplied player state and full immediate snapshots. Iteration04's captured `s08-i04-fixture-search` launched `rg` with literal pipe tokens and exited 2, so it is a child argument error rather than a pre-child rejection. The following captured shell pipeline succeeded.

All 744 preexisting channel objects passed base64/length/SHA validation. Every Iteration05 gate attempt has separate complete stdout/stderr byte and SHA receipts; the full final validation tree also decodes and verifies. Prior validation JSON values and the existing execution/closing prefixes were preserved and checked. The main, cache, identity and query controls remain pinned at `6d183bde`, `decd8d6c`, `f57a38f7` and `7118a48`. Author baseline/probe hashes and the genuine fixture remain unchanged.

### Planning return and inventory

The authorized planning commit will contain exactly `artifacts/execution.md`, `artifacts/validation.json`, `closing-report.md`, and `ledger.md`, using named paths and separate from the source commit. Five unrelated Project02 planning paths became dirty after Iteration05 intake; they are excluded from this return. The ledger remains nine rows and eight columns; SR-03/04/05/07/08 remain the five accepted `done` rows, and SR-01/02/06/09 remain `open`.

Five first-class slice documents: `slice-plan.md`, `cc-prompt.md`, `ledger.md`, `closing-report.md`, and CDC-owned `cdc-verification.md`. Supporting artifacts: `artifacts/author-baseline.json`, `artifacts/author-probe.rs`, `artifacts/execution.md`, `artifacts/validation.json`, and `artifacts/cdc-review-receipts.json`. The five corrective assignment prompts `cc-prompt-iteration01.md` through `cc-prompt-iteration05.md` remain separately listed in the assignment history above.

### Bubble-up

1. **Was the assigned work delivered?** The assigned test-only corrections and all nine fresh gate categories are complete as CC proposed-complete evidence; four rows remain open for independent CDC review.
2. **Did scope, architecture, acceptance or evidence change?** No runtime, API, compatibility, cache or sequence change occurred. The original parent gates and historical owner decisions persist. Prior unknown/missing evidence remains disclosed and unwaived; the Iteration04 pointer map is corrected additively.
3. **Does the parent plan need a change, and who owns remaining decisions?** No parent-plan change is proposed. CDC owns independent row review; the Operator retains acceptance authority. No slice, arc or project closure or integration is claimed. After acceptance, the required session/live/full-startup composition still precedes evidence-gated save-field repair; all original higher-scale owner gates remain.

Current prompt: `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration05.md`. Current report: `arc06-fit-and-finish/slice08-galaxy-scoped-routing/closing-report.md`.

### Planning commit receipts — dated correction (2026-10-11)

The four-record Iteration05 planning return is commit `3f21788ca8edbd20507509066d07cf902940906e`, parent `40c36abbee7574122dd766c384cf921e002c5140`, with exactly `artifacts/execution.md`, `artifacts/validation.json`, `closing-report.md`, and `ledger.md`; it carries the required adjacent Codex and Billo AI trailers. Validation-only post-commit receipt commit `26aabb2f9ff9b1e2eb264a6a916a2f283d064640` parents that return and changes only `artifacts/validation.json`, with the same required trailers. The intervening parent `40c36ab` is the separate Project02 planning commit; its paths are not in either Iteration05 commit.


## Iteration06 — bounded Sofie correction, CC review pending — 2026-10-11

**Proposed current corrections delivered; independent acceptance pending.** The Operator authorized one extra bounded pass beyond the five-refinement cap and selected Sofie as implementer, with a separate CC context independently reviewing. Sofie does not independently accept this return. Current assignment `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration06.md`, issued340c7c62108f7a422d959c3eb417db352a7b307c. Pre-edit intake is execution.md Iteration06 and validation.json iteration06.intake/readback_manifest. Planning intake23079309; author issuance340c7c6; sourcef71d607e27e18d7266855a2708449f9d6b90c632 directly parents0e1e8b4239881a378bc8fb928ef25741c228e171. Planning return/receipts follow in the appended commit record.

Only crates/nms-graph/src/query.rs cfg(test) changed: existing main spatial test now spans2 actual insertion orders ×active0/9/7 ×reference0/9/7, nearest0/MAX and all six prescribed radii through explicit and active APIs. Literal fixture0/400 expected IDs/PlanetKeys distinguish wrong-key/count/galaxy/scope/selection behavior. All576 calls have immediate full player/active checks without resetting; all288 planet vectors have exact projections,80 nonempty vectors/128 references have canonical ptr checks,208 vectors are literal-empty controls. Filtered-radius800 additionally checks exact far-key800 and every canonical borrow in both actual near/far orders. Complete unchanged nearest799/tie/SSI/single-field subproof retained. No runtime/API/public-signature/cache schema or semantic revision change; genuine360-byte2/2 predecessor remains unchanged.

All nine gate categories passed:682 focused tests (19 nonzero suites/23 result summaries),900 workspace (23/33), zero failures/ignored; formatting/lint/build/doctests/ordinary scoped docs/actual flagged strict cache docs and separate source/planning diff checks. Three scoped doctest summaries contain zero tests. Old4 core/3 graph link and2 query HTML warnings and proc-macro-error2 advisory remain; strict combined docs stays parent-open. No current Rust failure/blocked gate. Preparation output-name collision and recovered inspection truncations are recorded honestly; complete child channels/metadata, gate links and temporary script sources are embedded in validation.json.

| Row | Doer disposition | Current proof / retained applicability |
|---|---|---|
| SR-01 | proposed current repair delivered; open for CC | Complete576-call literal matrix with every planet canonical reference and full state; both true insertion orders; previously missing active0/9 boundaries and explicit0/9 under7 now exercised. |
| SR-02 | proposed current repair delivered; open for CC | Bound filtered800 views exact far-key800 plus canonical pointers in both true near/far orders; all prior numeric-ID/SSI/four isolated misses/799 controls byte-preserved. |
| SR-03 | retained accepted, done | Route payload/precedence/normalization files and prior12/12 public specificity unchanged; current tests pass. No new independent verdict by doer. |
| SR-04 | retained accepted, done | Protected positive9 bridge/direct fallback/foreign shortcut and edge tests unchanged; tests pass. |
| SR-05 | retained accepted, done | Dense120 reachability and9 waypoint/60decoy/closed2400 fixtures unchanged; tests pass; best-effort unbridged limitation persists. |
| SR-06 | retained accepted, done | Query route whole file unchanged, including protected4976-byte100planetNone/Some400 and supplied-player/error/105-target controls; tests pass. |
| SR-07 | retained accepted, done | Sealed genuine2/2 fixture and cache/current-route tests unchanged; cache/query/workspace pass, no revision bump. |
| SR-08 | retained accepted, done | CLI and typed formatter source/tests unchanged; native integration/docs pass. |
| SR-09 | current corrections proposed delivered; open for CC and historical disposition | Accurate manifest/coverage/state/pointer/strict-env/stopped02/actor roles; old values/prefixes preserved, new complete captures and scope/commit fences. Unavailable history is still unwaived. |

Assignments preserved: initial arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt.md (b792257); iteration01 cc-prompt-iteration01.md (09a460ff), iteration02 cc-prompt-iteration02.md (4c3d405), iteration03 cc-prompt-iteration03.md (50eac522), iteration04 cc-prompt-iteration04.md (ae3cdef4), iteration05 cc-prompt-iteration05.md (9a9bf11), iteration06 cc-prompt-iteration06.md (340c7c6), all in that same canonical slice directory. Initial/01/03/04/05 returned and reviewed, superseded as active assignment. **02 executed partially/stopped/no CC commit**, then superseded; its false closed-route runtime-discrepancy claim is retracted because input warpNone, not intendedSome400. Current06 is returned as Sofie proposed proof for separate independent CC review, with no automatic seventh correction.

Strict-doc correction supersedes earlier final-flag prose: original05 attempt9 properly records flag/exit0; final17 flag missing from recorded overrides, inherited environment unknown. Current correctly flagged gate08 is new evidence, not original recovery. Current full body/guide intake resolves only current work; it cannot establish previous pre-edit attention or erase disclosed drafting/restoration/reapplication history. Original four groups,01 three groups (third invocation count unknown),02 stopped/intake,03 unavailable items1–6/8, historical strict-env/prior intake remain unwaived. Captured03 SyntaxErrors7/9, pre-child rejections and04 launched rg exit2 remain distinct; missing data not invented.

Artifact inventory: execution.md owns pre-edit intake, execution/corrections and commit history; validation.json embeds identities, guide/source manifest, child channels/env/exits/stable links, totals, preservation audits and incidental scripts; closing-report.md owns all-nine row accounting/assignment history/limits/bubble-up; ledger.md only updates Evidence/Notes on three open rows and preserves six accepted entire rows; cc-prompt-iteration06.md is immutable issued assignment and independent **review-only** handoff. Existing cdc-verification.md and artifacts/cdc-review-receipts.json remain untouched by Sofie during this doer pass; separate CC context owns the next dated independent record. Author-only slice/arc/project/closure status changes are separately scoped. No extra maintained validator artifact or product created.

### Bubble-up to the arc — Iteration06

1. Assigned current test/evidence corrections delivered as doer-proposed proof; all required current gates pass. Six accepted rows retained; SR01/02/09 remain open until independent CC assessment. Slice/arc/project remain open.
2. No new runtime/API/cache/criteria issue reproduced. Historical unavailable channels/env/intake still need explicit bounded disposition after independent current review; completing new captures does not waive them. The Operator cap approval authorized only this extra pass.
3. Parent scope/counts/architecture/sequence unchanged; routine authorization/actor/assignment records appended. Separate CC verifies current return and reports historical residuals to Operator/CDC; neither actor self-disposes missing historical evidence. Main/dependencies/Project02 held. Session/live/full-startup then evidence-gated save-field repair, duplicate-base/save-name/portal/numeric/original closeout and higher-scale gates remain with original owners. No integration/release/publication/closure authorized.


Iteration06 commit receipts — sourcef71d607e27e18d7266855a2708449f9d6b90c632 parent0e1e8b4; author issuance340c7c62108f7a422d959c3eb417db352a7b307c parent23079309 (seven paths, intake before source edit); doer returndb5416cfe3f683bbe28002f7af79f73fa610be63 parent340c7c6 (four evidence paths); author handoff39971ca769de6f950b7bf764cf1d9e3f966c1184 parentdb5416c (four plan/closure paths). Exact parents/pathsets/once-adjacent terminal trailers and clean candidate/planning verified; raw children and structural audit embedded in validation.json iteration06. The following receipt-only commit records these completed commits within execution/validation/closing paths; resolve its final identity from Git/the Operator handoff. No ledger/status/criteria/source mutation in that receipt followup. Six done/three open, CC independent review pending; history unwaived. Final receipt commit itself will be checked in visible tools; it cannot embed its own future SHA.


## SR-09 bounded historical-evidence decision — proposed, awaiting Operator — 2026-10-11

CC independently accepted SR-01/SR-02 and retained SR-03–08 at source f71d607e27e18d7266855a2708449f9d6b90c632. Review e6bbfe53fe84f86592c353e9b24856f6dbf78078 and receipt f8d31258fe2e235af034a010b3f329444455414e are the independent basis. Current native contracts and current required gates/evidence passed; eight rows done, SR-09 open. Sofie's current intake validated the recorded review, all884 doer channels/26 current independent-review channels,10 current/22 prior links, actual strict flag and review commit fences. This is record reconciliation, not another independently executed Rust verification by the implementing context. No Rust rerun was needed: source has not changed, CC has just reproduced the affected tests and required gates.

**Recommendation: approve a single, explicitly enumerated historical-evidence exception for SR-09 on this exact candidate.** It would permit accepting SR-09 on the independently reproduced current evidence plus the Operator-approved provenance exception. It does not prove or recreate the past. None is approved yet; the ledger/independent verdict remains untouched, slice/arc/project remain open, no implementation continuation or integration is authorized by this proposal.

| Included past observation | Missing or unproven part retained in the record | Proposed disposition |
|---|---|---|
| Initial Slice08 diagnostic/capture history | Four originally disclosed groups lack original complete command/channel receipts. | Keep those gaps unknown; exclude them from current proof; authorize the bounded historical exception only. |
| Iteration01 | Three disclosed uncaptured groups; third group invocation count unknown. | Preserve the unknown count and unavailable original bytes/exits; no inferred success. |
| Iteration02 | Disclosed stopped-pass capture/intake limits; partial execution and no CC commit. | Preserve stopped status and retracted false runtime discrepancy (test warpNone rather than Some400); no retroactive intake claim. |
| Iteration03 | Unavailable diagnostic items1–6/8 and disclosed historical strict-doc environment/intake limits. | Preserve unknown original fields; captured SyntaxErrors7/9 and pre-child rejections keep their distinct recorded classes. |
| Iteration05 final strict child17 | Recorded overrides omit required RUSTDOCFLAGS; original inherited environment unknown; earlier child9 records the flag. | Preserve17's environment as unknown. Use current correctly flagged doer/independent gate08 as new proof only. |
| Prior required reading/intake and edit sequence | Earlier incomplete/unspecified body/guide coverage and disclosed draft-before-intake, restoration/reapplication sequence cannot establish original pre-edit compliance. | Preserve those limits; current complete intake is current-only and never backdated. |
| Disclosed extra Iteration06 independent-review strict-doc invocation | CC reports one extra unflagged invocation, excluded from gates; its separate argv/channel/exit receipt is not present in the review packet. | Retain narrative-only classification and unknown unavailable raw details for this one specific invocation. It remains nonqualifying; captured flagged gate08 is the strict proof. |

The exception is limited to these listed observations and the exact reviewed source. It does not waive any current failing test, runtime disagreement, current qualifying gate/capture, future recordkeeping, another slice/project, or required remaining lifecycle/save/startup/performance/coverage/distribution/higher-scale work. Existing captured failed/blocked attempts and historical reports stay intact. Ordinary doc warnings and dependency advisory retain their existing parent owners; strict combined docs remains open. No private-log sweep, missing byte synthesis, passing-exit assumption, fake invocation count, retroactive reading attestation, or rerun-as-history is proposed. The earlier approval for a sixth correction and Slice06's unrelated exception do not approve this exception.

Residual risk: the original failure details, environment and chronology/pre-edit compliance for those past attempts cannot be independently audited from the available packet. Independent current native tests, correct flagged strict docs, full required gates and unchanged-input/source fences support the current source's behavior; they do not remove that provenance limit. Recovery of original records would be stronger historical evidence, but repeated new Rust runs cannot recover it. Re-scoping SR-09 would preserve the obligation and require an approved owner/re-entry plan; it would not count as acceptance.

**If the Operator approves:** record the exact approval, candidate, included observations and unknown fields as an explicit exception; disposition SR-09 using CC's independently verified current evidence and that Operator decision, without claiming complete historical capture; complete all-nine slice accounting, artifact inventory and bubble-up/formal Slice08 closeout, then resume the roadmap under Expedited Mode. Source stays at the reviewed pin. No automatic Arc06/Project01 close, main merge, push or release. **If approval is declined or unclear:** keep SR-09 and Slice08 open and discuss original-record recovery or explicit re-scoping; no automatic seventh implementation pass.

### Bubble-up — pending SR-09 decision

Assigned spatial/routing behavior is independently accepted; full slice closure still awaits historical-evidence disposition. No new runtime/design/cache finding; the provenance exception is the only proposed amendment and is not in force. Parent capability criteria/counts/structural sequence remain unchanged: arc26/seven done/nineteen open; project25 open. Required session/live/full-startup composition, subsequent evidence-gated save-field repair and original remaining acceptance owners/gates are preserved.


## Operator-approved SR-09 exception and final closure — 2026-10-11

**Slice08 closed: nine rows done.** The Operator answered “yes, I approve” to the precise bounded historical-evidence exception recorded at planning aa83dbee3488033ebc232604809885f56e2e9fde. Approval covers only the seven explicitly listed observations, at unchanged source f71d607e27e18d7266855a2708449f9d6b90c632. This supersedes pending/unwaived status for those specific historical proof obligations; it does not reconstruct their unavailable data. The prior proposal, failed/stopped reports and independent reviews remain exact historical prefixes.

Independent current proof remains CC review e6bbfe53fe84f86592c353e9b24856f6dbf78078 / receipt f8d31258fe2e235af034a010b3f329444455414e: both affected native tests1/1;682 focused and900 workspace, zero failures/ignored; all ten captured commands/nine gate categories including actual flagged strict cache docs;884 implementation channels,26 review channels,10 current/22 previous links, exact source/prompt/fixture/commit fences. These fresh runs prove current behavior only. Sofie records the Operator decision and final disposition; she does not relabel her implementation self-checks as independent. No new Rust run is claimed for this metadata-only closeout; unchanged current evidence retains applicability. Ordinary4 core/3 graph/2 query documentation warnings, dependency advisory and strict combined-doc parent gate persist.

The approved exception retains unknown: initial four capture groups; Iteration01 three groups/unknown third count; Iteration02 stopped/intake limits/no CC commit and retracted warpNone claim; Iteration03 unavailable1–6/8 and historical strict/intake limits; final05 child17 inherited flag; prior coverage/draft-before-intake/restoration/reapplication compliance; one extra narrative-only unflagged independent-review doc invocation, excluded from strict proof. Captured03 SyntaxErrors7/9, pre-child rejections and04 launched rg exit2 retain their recorded classifications. Original9/current correctly flagged strict proof does not recover17 or the excluded extra run. Structured approval/inventory is validation.json operator_approved_sr09_final_closure. No unavailable bytes/count/exit/environment/reading time synthesized.

Source finalf71d607 directly parents0e1e8b4; latest correction is exactly crates/nms-graph/src/query.rs cfg(test). Earlier runtime/caller/cache/CLI migration source chain and accepted row witnesses remain attached to their reviewed revisions. The closure commit changes only Slice08 ledger/slice-plan/closing/report execution/validation/review plus arc-plan/ledger and project-plan/closure records; exact named paths and commit identity follow in the receipt. No source movement, archive meaning/revision change, merge, push, release or Project02 mutation. Current assignment arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration06.md is executed/returned/independently reviewed and now closed, superseding all earlier active assignments without modifying any issued prompt. Initial plus01–06 all remain.

| Row | Final status | Independent basis / amendment |
|---|---|---|
| SR-01 | done | Iteration06 independent CC review: cdc-verification.md and artifacts/cdc-review-receipts.json iteration06; source f71d607e27e18d7266855a2708449f9d6b90c632; all affected native tests and required gates reproduced. |
| SR-02 | done | Iteration06 independent CC review: cdc-verification.md and artifacts/cdc-review-receipts.json iteration06; source f71d607e27e18d7266855a2708449f9d6b90c632; all affected native tests and required gates reproduced. |
| SR-03 | done | Iteration04 independent review: cdc-verification.md; artifacts/cdc-review-receipts.json iteration04; source02260b7. Accepted. |
| SR-04 | done | Iteration03 independent review: cdc-verification.md; artifacts/cdc-review-receipts.json iteration03; sourceedd6664. Accepted. |
| SR-05 | done | Iteration04 independent review: cdc-verification.md; artifacts/cdc-review-receipts.json iteration04; source02260b7. Accepted. |
| SR-06 | done | Iteration05 independent review: cdc-verification.md; artifacts/cdc-review-receipts.json iteration05; source0e1e8b4. Accepted. |
| SR-07 | done | Iteration04 independent review: cdc-verification.md; artifacts/cdc-review-receipts.json iteration04; source02260b7. Accepted. |
| SR-08 | done | Iteration01 independent review: cdc-verification.md; artifacts/cdc-review-receipts.json iteration01. Accepted actual CLI/typed formatter native tests; preserved source21494b9. |
| SR-09 | done | Current CC proof reproduced; explicit Operator-approved seven-observation historical-evidence amendment above; unknown past fields retained. |

### Artifact inventory and what worked

Five supporting artifacts, all under artifacts/: author-baseline.json seals initial source/probe observations; author-probe.rs is the bounded original native witness; execution.md preserves intake/attempt/assignment/correction/commit history; validation.json preserves implementation child metadata/channels/script reconstruction aids and exact Operator exception; cdc-review-receipts.json preserves independent reproduction channels/audits. No extra artifact home. First-class plans/ledger/closing/verification plus seven preserved prompts remain in slice root and are not counted as supporting artifacts. Supplied immutable360-byte2/2 cache fixture is a dependency under source, not a new slice-produced artifact.

What worked: independent literal galaxy collision/boundary fixtures, actual reversed insertions, exact canonical borrowed views and full state snapshots made the missing regression matrix testable. Separate CC review of Sofie's correction preserved independence. Actual strict child flags, stable gate links and retained failed history distinguish current proof from missing history. Named-path commits/pinned worktrees protect unrelated source and Project02.

### Bubble-up to the arc — final

1. Assigned spatial/routing composition delivered and independently accepted, nine of nine child rows closed under the specific SR09 amendment. R06-S08 closes;26 arc rows/eight done/eighteen open before opening the next child. All original/expanded parent capability rows retain their criteria and open dispositions. Project25 rows remain open.
2. No new runtime/API/cache/architecture finding changes the approved identity direction or sequencing. The only acceptance amendment is the explicitly approved bounded history exception, recorded with residual provenance risk. Existing best-effort unbridged waypoint behavior is not a strict maximum-hop guarantee.
3. Scope-as-specified versus delivered: child spatial/routing/query/CLI/raw-cache controls and current validation delivered. Historical missing proof handled only by the approved exception. Required movement/session/galaxy command/watch/REPL/MCP and full startup/cache composition remain next, then evidence-gated discovery/base/nonzero reality policy repair; original duplicate-base/save-name/portal/numeric, docs/coverage/performance/distribution and higher-scale gates persist. Arc plan updated before next slice authoring. No administrative parent closure or silent transfer.
