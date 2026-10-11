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
