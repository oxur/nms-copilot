# Slice05 initial independent CDC review — 2026-10-04

Reviewed initial `cc-prompt.md`, source `8d2cf00fba41689f817cad35ec835315ba9aa415`, planning candidate `4f6ca66acd39bba522c3684c4de9e498c9cbf186`, and issuance `8ae5a1a129f382626c71d6e6db46d26cb7883ec8`. Source main stays clean at `a800950ef3558fcf463517696c3c140061e7353d`. CDC/Sofie did not implement or modify the production repair. Two-Contributor Workflow; Expedited Mode.

**Runtime repair independently reproduces; the slice remains open for three bounded delivery gaps. CS-01, CS-02, CS-05 and CS-07 are accepted. CS-03, CS-04 and CS-06 remain open.** These are missing durable proof/documentation requirements, not a reproduced runtime failure. No architecture or scope amendment is needed. [Iteration01](cc-prompt-iteration01.md) is the current CC refinement assignment.

## Reproduced method and evidence

[Fresh receipts](artifacts/cdc-review-receipts.json) preserve six exact gate subprocess captures: read-only format check, make lint, cache package tests, make test, all-feature workspace build and cache doc tests. All exit 0. Cache tests: 18 unit + 13 integration, no failures or ignored tests. Workspace: 824 passing tests across 30 test/doc summaries, no failures or ignored tests. Cache doctest target has 0 tests; its success does not prove document content. Lint/build retain the existing proc-macro-error2 future-compatibility warning. Format check is a non-mutating equivalent for reviewer verification; make lint also checks formatting. No source changes were made.

The complete changed source/test bodies and diffs, active prompt/plan/ledger and CC execution/closing reports were inspected. Structured inspection covers all 30 CC run objects: strict base64 decoding, channel byte lengths and independently reparsed test summaries match. Final six gate entries are in prescribed order and pass; five earlier failed attempts (three lint, two focused-test) are retained. The metadata/offline preflight limitations are explicitly separate from product gates. Stream identity checks establish representation fidelity, not proof of historical source state or independent runtime acceptance; the fresh gates supply current execution proof.

A separately compiled [public-API probe](artifacts/cdc-probe.rs), using a temporary copy of the pinned synthetic portal-control fixture, adds empty-header and actual-freshness observations. Empty input returns exactly Incompatible(TruncatedHeader{actual:0}). Both schema 2/semantic 1 and schema 1/semantic 2 caches with save_version 9999 are explicitly observed mtime-fresh; startup rebuilds source 4720, returns was_cached=false and exact cold-model system/planet/base/player fields, writes the literal current header, then a verified-fresh compatible warm load returns true/4720. These temporary checks independently confirm production behavior, but do not replace the durable regression assertions omitted below. The probe build/run captures, fixture digest and source hashes are in receipts; compile the source against the Cargo-emitted nms_cache rlib with dependency search path, on synthetic temporary files only.

The full source/planning commit pathsets are exactly five/four allowed paths, both trailers occur once, and their parents are the pinned baseline/issuance commits. Both worktrees and main are clean. All seven criteria/status cells were preserved by CC; prompt and author inputs are sealed. No private save/cache/config, new save decoder, galaxy identity, source hash feature, live watcher/protocol, broad coverage/no-default/MSRV/package/performance check or release was exercised.

## R01 — Empty header regression is omitted (CS-03)

Initial prompt Behavioral oracles explicitly calls for “Empty and short byte slices (lengths 7,8,15)”. `serialize.rs::test_deserialize_compatibility_error_precedence` at the reviewed candidate tests only 7/8/15. A current 16-byte header with empty payload is a different state; its Deserialize error does not cover an empty envelope.

Fresh CDC empty-input probe passes, so no decoder bug is inferred. Complete the requested durable regression by adding length 0 to the existing loop and asserting the same exact TruncatedHeader(actual) branch, retaining all other cases and precedence assertions. No new test framework or production changes.

## R02 — Freshness preconditions are not asserted in lifecycle probes (CS-04)

The initial prompt says: “Before claiming freshness, assert is_cache_fresh.” Candidate `tests/load_integration.rs::set_fresh` at lines83–86 sets deterministic timestamps but does not assert the predicate. Its standalone freshness test does assert it; the legacy/schema/semantic/corrupt/no-cache/source-error probes themselves do not. Thus the requested precondition is not guarded where source fallback could otherwise occur because of timestamps.

Fresh CDC independently confirms the actual preconditions and correct rebuild/warm behavior. Preserve that successful runtime evidence and add the assertion after the two set_mtime calls in the existing shared set_fresh helper. All its existing callers then check freshness before their subsequent action, including the warm stage in assert_rebuilt_and_warm. Retain the stale test's false assertion. This closes the durable-proof gap without changing timestamp values or production lifecycle.

## R03 — Promised compatibility/process documentation is incomplete (CS-06)

The plan's Binding cache contract explicitly requires documenting the public CacheError extension's effect on exhaustive external matches. The prompt Implementation spine step5 also requires explaining legacy rebuild, exact revision equality and the continuing semantic-bump obligation. Candidate source docs describe a16-byte envelope, separation from game version and startup fallback; private comments say when to bump. They do not document the external exhaustive-match consequence, legacy headerless rejection/exact equality or initial semantic 1's bounded meaning for future maintainers.

Complete the public crate/module docs in lib.rs/serialize.rs: describe the exact header, rejection and source-rebuild route; distinguish schema/layout/codec changes from decoded/model semantics; require semantic bumps for later decoder/identity changes even with unchanged archive layout; state initial revision1 retains current known interpretation and is not the address/identity fix; explain the CacheError::Incompatible addition may require external exhaustive matches to change. Keep all revisions, public signatures, error variants and runtime bodies unchanged. Do not claim a release/version migration has been accepted. A crate-only rustdoc build can check new links in addition to the existing doc-test gate.

## Seven-row disposition

| Row | CDC status | Evidence / remaining work |
| --- | --- | --- |
| CS-01 | done | Issuance/source/commit fences and intake/readback independently checked. Read receipts remain CC attestation and do not prove attention or comprehension. |
| CS-02 | done | Literal header, populated field transport and misaligned input tests freshly pass; checked aligned decode and API exports verified. |
| CS-03 | open | Runtime errors/precedence/source chain reproduce, including empty input in CDC probe; R01 requires the requested durable empty-envelope regression. |
| CS-04 | open | Legacy/schema/semantic rebuild and warm hit reproduce, plus fresh CDC predicate checks; R02 requires assertions inside the durable lifecycle probes. |
| CS-05 | done | Fresh tests reproduce corrupt/stale/missing/bypass, malformed-source byte preservation and best-effort write failure. Game correctness and full live-cache composition remain outside this claim. |
| CS-06 | open | Existing accurate docs retained; R03 requires the specified compatibility/process details. |
| CS-07 | done | Six relevant fresh gates,824/31 denominators, all 30 CC run representations, retained failures and exact commits/trailers/scope independently checked. Future candidate gates must still be refreshed/assessed. |

No defer/no-op or criterion reduction. The guarded try_into().unwrap conversions differ from the recommended manual arrays but cannot fail after the verified16-byte length guard; this is not a runtime finding and no gratuitous rewrite is assigned.

## Bubble-up, inventory and retention

The slice delivers the runtime protection assigned by Arc06, but specified regression/documentation obligations remain incomplete. Scope-as-specified versus delivered: no hidden production decoder/identity adoption or extra paths; R01–R03 name the remaining work exactly. All broader original closeout requirements remain. No structural sequencing/architecture/acceptance amendment is indicated; arc-plan routine status/version is updated before continuing within this slice. R06-S05, R06-R03, P01-R03, Arc06 and Project01 remain open; no Slice06 prompt is issued before Slice05 closure.

CC artifacts: execution.md and validation.json plus first-class closing-report/ledger. CDC evidence: cdc-review-receipts.json and cdc-probe.rs. Existing author inputs: author-baseline.json and author-probe.rs. First-class prompt/plan/ledger/closing-report/CDC verification and iteration01 remain in the slice root. All initial CC evidence/report and issued prompt are preserved unchanged.

Accepted rows have current reproduced evidence at candidate 8d2cf00; correction review may retain unchanged runtime claims under work-verification's retention rule. Iteration01 changes only docs/tests, not runtime or dependencies, and must freshly exercise affected regression/doc/gate checks. Retention applicability is assessed on its actual delta, not inferred from file names.

What worked: complete raw stream capture preserves five failed attempts; literal header and poisoned-version oracles distinguish old cache behavior; deterministic mtimes plus direct freshness observation verify the bypass precondition. Reviewer batched reads truncated in several places; relevant prompt/source/report/guide extents were recovered with smaller reads before verdict. A preliminary run-list display abbreviated summaries; the full thirty-object byte/summary checks and final gate denominators were subsequently loaded. No review process or runtime check failed; the three findings are source/contract comparison gaps.

## Handoff packaging checks

Ten exact reviewer/handoff paths;71 resolvable local links; all six sealed input/CC files and five candidate source files unchanged; all seven slice criteria retained (four done, three open); all25 project and23 arc criteria/statuses retained, with original capabilities still open. Source main/candidate are clean/pinned and whitespace/JSON checks pass. These structural checks support this refinement handoff, not slice closure. Commit trailers, exact pathset and post-commit planning cleanliness are verified after committing.


## Iteration01 final independent CDC review — 2026-10-10

**Accepted: CS-03/04/06 now done; prior CS-01/02/05/07 retained. All seven rows are done and Slice05 is closed.** Reviewed `cc-prompt-iteration01.md` against source `decd8d6c09c1199a6cdd01fd48aa50f3adef2b1a` and planning packet `180a9f64479ce385752e1a6bada9c98f36d510f6`. This CDC context made no production changes. Initial failed review above remains historical; this section supersedes its current verdict, not its observations.

### Fresh reproduction and correction assessment

[Iteration01 review receipts](artifacts/cdc-iteration01-review-receipts.json) preserve seven raw stdout/stderr subprocess captures, argv, cwd, environment and exits. All pass: non-mutating `cargo fmt --all -- --check`, `make lint`, cache tests, `make test`, all-feature workspace build, cache doctests, and strict crate rustdoc links. Cache results are18 unit +13 integration (31), workspace824 across30 summaries, all0 failed/ignored. Cache doctests have0 tests; documentation content is established by source inspection, link validity by rustdoc. Existing proc-macro-error2 warning remains non-failing. No gate was blocked or failed during this independent pass.

Direct source delta is exactly the assigned docs and two test changes. The error loop now distinguishes empty/short envelopes0/7/8/15 using the exact `TruncatedHeader { actual }` predicate; the unchanged current-header-empty-payload case returns Deserialize. The complete integration helper/caller set was inspected: freshness is asserted after both deterministic mtime writes and before every dependent action, including the rebuilt warm stage. Legacy/schema/semantic sentinel9999 rebuilds to source4720 with false then true, retaining populated semantic comparisons. Public crate/module/function docs now carry exact header offsets and revisions, exact equality, legacy rejection, direct errors versus startup rebuilding, separate schema/semantic evolution, predecessor rejection/rebuild/warm obligations, initial1/1 limits, exhaustive external CacheError matching consequences, and checked owned/aligned decoding without speed promises. R01–R03 are resolved without runtime changes.

### Retention, provenance and seven-row walk

The initial independent record at8d2cf00 and its accessible receipts/probe identify runtime claims, methods, inputs and outcomes. Actual delta cannot affect envelope/error/fallback implementations, model data, dependencies, constants, signatures or archive shape: only docs and test assertions changed. Criteria and synthetic inputs are unchanged; no new finding undermines prior methods. Therefore CS-01/02/05/07 retain their original independent evidence under the correction-retention rule. They are not relabelled fresh; current gates supplement that evidence, and newly affected regression/docs claims are freshly checked.

| Row | Final CDC status | Basis |
| --- | --- | --- |
| CS-01 | done, retained | Historical independently checked intake/fences; current iteration intake, commits, relocated skill-path disclosure and preservation independently inspected. |
| CS-02 | done, retained | Unchanged literal1/1 envelope, checked alignment and data transport; fresh package/workspace tests also pass. |
| CS-03 | done, reproduced | Exact empty-envelope regression added and executed; complete unchanged negative/error precedence cases inspected and passed. |
| CS-04 | done, reproduced | Shared freshness assertion covers all callers; real legacy/schema/semantic rejection/rebuild/warm tests pass with existing poisoned-version oracle. |
| CS-05 | done, retained | Runtime lifecycle unchanged; fresh source-error/no-overwrite, bypass, corruption, stale/missing and write-failure tests pass. |
| CS-06 | done, reproduced | All required public compatibility/process content inspected; strict crate rustdoc links pass. No release/game-format/performance acceptance inferred. |
| CS-07 | done, retained and refreshed | Seven fresh gates; exact3/4 commit pathsets, direct parents, trailers once each, clean fences; all nine new CC streams decode to exact recorded byte counts and their test summaries/counts agree with raw output. |

Complete preservation checks: all pre-existing validation JSON values and30 initial run objects are unchanged; the prior execution/closing reports remain exact byte prefixes; criteria/statuses unchanged by CC, four accepted rows byte-identical; all issued prompts and sealed author/CDC inputs retained. The nine iteration runs include the failed sandbox lint attempt and successful retry, six other gates plus whitespace check. That failure remains doer-recorded history, not a fresh CDC failure. Source/planning parents are8d2cf00/688486a respectively. Required trailers and exact paths match. Both candidate worktrees and main are clean at intake; main staysa800950.

### Bubble-up, artifact inventory and scope diff

Slice05 now delivers its assigned cache protection and continuing revision process. No deferrals, no-ops, missing rows or silent drops; denominator remains seven. No architecture/scope/acceptance change is revealed by this correction. Routine parent status/version updates record closure before the approved next galaxy-identity work is detailed. R06-S05 closes; R06-R03/P01-R03 remain open for later semantic transitions and final live/composition proof. Arc06 and Project01 remain open, with all original gates unchanged. Main integration/publication is still unauthorized.

Supporting inventory (seven files, all under artifacts/): author-baseline.json, author-probe.rs, execution.md, validation.json, cdc-review-receipts.json, cdc-probe.rs, cdc-iteration01-review-receipts.json. First-class slice-plan, ledger, both preserved issued prompts, CC closing report and this reviewer record are excluded from that inventory. No user save/cache/config was touched.

What worked: exact small assertion repairs guard meaningful boundaries and branch preconditions; byte-preserved failures and separate reviewer receipts make retention auditable. Earlier history remains visible. A combined reviewer read was truncated; required current correction/source/closure extents were recovered with bounded reads before acceptance. Current filesystem skill relocation was transparently recorded by CC and does not rewrite the immutable issued prompt.

Packaging check recovery: first row-count helper matched only M/R/P/CS IDs and omitted the five U-prefixed original arc rows, stopping with an assertion. The corrected whole-table population check includes every data row:25 project/23 arc/7 slice, with0/5/7 done. This was a reviewer helper error, not a product failure. Project frontmatter version/date were synchronized with the new history entry.
