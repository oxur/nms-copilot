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
