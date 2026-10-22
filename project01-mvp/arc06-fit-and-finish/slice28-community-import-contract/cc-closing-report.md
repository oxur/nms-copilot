# Slice28 CC close report — proposed evidence

This is the CC delivery for original M7.9, community CSV import and provenance. It records a reviewable candidate and its actual evidence; CDC owns independent disposition and acceptance. All eight CI rows remain proposed/open at this handoff.

## Assignment and delivered behavior

The exact assignment was [cc-prompt.md](/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/planning/project01-mvp/arc06-fit-and-finish/slice28-community-import-contract/cc-prompt.md), with initial/pre-edit01 as the first implementation and five actual bounded refinements. The delivered source is commit `718f389195a41cd3eef17432e84e6bbec32637b2` (tree `0f80cc680b185c079ef4c7054140fdbe9290ec3f`), parent `063c8c658fb07ae04acca7ae2d7a7971c4f85688`, subject “Persist community discovery origin through cache lifecycle.” Its diff contains the exact 20 allowlisted paths and has the required consecutive Codex and Billo AI trailers. The index and worktree were clean at handoff.

The change adds public `DiscoverySource::{Personal, Community { source }}` and `System.source`, keeps the five-argument `System::new` compatible by assigning `Personal`, and propagates source through the existing cache as schema 4 / semantic 6. CSV import preserves the exact source label and flexible row behavior, attributes new genuine discoveries and marked-placeholder promotion to Community, and preserves the first genuine system/planet unchanged on duplicates. Existing query/model/cache behavior remains in place. The one-shot CLI import is verified as session-only and read-only with respect to save/HOME inputs. No import persistence UX or REPL import command was added.

The cache predecessor fixture is the exact 632-byte accepted Slice23 schema3/semantic5 archive, SHA-256 `7e308a95dc28874e9c164c425a09c84e38fd72d1e819c5cabcaa57fee8a2d99d`; its origin is unavailable in those historical bytes and is not inferred. Incompatibility rebuilds from the current save with Personal source; a subsequent compatible cache preserves that source even when the source file becomes older/malformed. Documentation describes the re-import consequence for discarded provenance. The pre-existing 12-field export contract and generic protocol were not extended with a provenance field.

## Eight CI proposals

These are evidence proposals only, not CDC acceptance:

| Row | CC-proposed status | Evidence retained |
|---|---|---|
| CI-01 | Proposed done | Literal enum/system serde, constructor compatibility and checked extraction tests; core/workspace gates. |
| CI-02 | Proposed done | Flexible CSV corpus, added/duplicate/skipped counts, full atlas and metadata comparisons, replay and placeholder promotion controls. |
| CI-03 | Proposed done | Actual query/name/update observations, origin/destination literals, full-state checks and duplicate ownership controls. |
| CI-04 | Proposed done | Source-bearing cache roundtrip, source-only negative mutants, exact schema/semantic precedence and authentic 3/5 rebuild/warm-hit file controls. |
| CI-05 | Proposed done | Real CLI import children, exact statistics and stderr/stdout behavior, invalid input cases, save/HOME immutability. |
| CI-06 | Proposed done | Original native rows 1–13 plus row14; CDC checked all 14 result records and 238 launch/raw/post triples. Fresh Ctrl-D and Ctrl-C row2/row3 runs supplement earlier duplicate `quit` observations. |
| CI-07 | Proposed done | Captured format/lint, focused and ordered package/workspace/build/docs/diff gates, retained failures and repairs. |
| CI-08 | Proposed done | Attributable readbacks, raw capture history, exact-path commit/trailers, frozen return inventory and handback. |

## Validation and artifact inventory

The complete captured attempt log is copied without changing historical attempt values into [validation.json](/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/community-import-contract/workbench/slice28-return/validation.json). Its original capture source remains `/private/tmp/nms-slice28-cc-attempts.json` (88 entries, SHA-256 `bf66e872fcde5eabe5d492c48085542e0ecbea2111f1905037fffeb34e9595df`). The additive `slice28_final` object records current commit, actual gates, native row/fixture mapping, CI proposals and limits.

The final captured `make test` was UUID `5d73dbf3bbac4776a4182f053df6a69d`, exit 0, 48 result groups with 1,128 passed and zero failed; this includes 31 doctest cases and 1,097 runtime/integration cases. The all-features workspace build (`dd9732f14d6346edb5768ead73f458e6`) and workspace doctests (`cb76135627184e66bc3a7c76d55d208e`, 31 doctest cases) passed. Strict docs passed with `RUSTDOCFLAGS=-D warnings` (`ccf5d0828e1a4d26a2985029ddea8f8`). Working and baseline diff checks passed (`c57b1a9ce30445f999d0bf064d2a0af3`, `cdd3818154114d79b3791e7a5f5f39c6`); the postcommit baseline diff check passed (`194d4cb1485a49709c8a8aff6f846463`). The full attempt-by-attempt channels, argv, environment, statuses and source identities remain in the capture log.

Native artifacts and actual fixture paths are inventoried in [execution.md](/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/community-import-contract/workbench/slice28-return/execution.md). The fourteen designated case IDs and fixture roots are listed there; the receipt inventory comprises launch/raw/post records and related request/case/result files. CDC's bounded inventory distinguished 621 native receipt files from 155 available fixture files. These counts describe different artifact sets. The successful native controller used `/private/tmp/nms-slice28-cc-native-v2.py` and `/private/tmp/nms-slice28-cc-native-driver-v2.js`; their hashes, captured syntax checks, executable/probe digests, per-row receipts, and failures are retained in the execution record. The two corrected fresh controls are row2 Ctrl-D `8cd29253e0614fb0bb12bce9c46e2493` and row3 Ctrl-C `e800847f64bb4549b3fa73be92c56ea6`. The earlier row2/row3 `quit` results remain explicitly noncounting duplicate observations.

## Compliance, structural change, and deviations

I followed the exact 20-path manifest, used the dedicated source worktree, preserved the accepted prior fixture bytes and historical failed attempts, ran raw-first captured gates, and returned the clean exact-path commit. A mixed-root patch incident did write one line to main `crates/nms-core/src/lib.rs`; I recovered its exact tool provenance, restored that one file to verified main HEAD bytes, and retained the incident and restoration evidence in execution.md. The protected main native-test blob and main index were verified unchanged. No source, index, Git, or product gate work occurred after CDC took ownership.

The actual structural change is limited to the provenance tag on systems and its preservation through existing cache encoding/rebuild, plus the corresponding import behavior, tests, and documentation. It does not change save parsing policy, query/export schema, persistence workflow, or public constructor arity. The genuine schema3/semantic5 archive lacks provenance, so old source origin remains unknown; the tested rebuild can only establish `Personal` from current save extraction.

Operational irregularities are retained without gate credit: one direct Node syntax check before the captured check; an initial row12 wrapper type error after a unique reservation but before fixture/SUT launch; an initial Git index-lock permission denial followed by the authorized exact-path stage; and earlier row2/row3 `quit` runs followed by proper control-character runs. These are documented as support/operational history, not silently replaced or counted as successful product evidence.

## Bubble-up to the arc

1. **What application capability was assigned, and what remains local or unintegrated?** The assigned capability is importing community coordinate CSV data into a queryable in-memory atlas with explicit Community origin and faithful cache handling. Library callers can query the enriched model; the `nms import` command is a read-only one-shot session. Import persistence, a REPL import command, and wider UI/distribution integration remain outside M7.9 and are not claimed here.
2. **What changed or surprised us during implementation?** The existing CSV reader's default width behavior discarded ragged optional/trailing columns, so the implementation uses the already-pinned flexible reader setting while preserving header aliases and parsing rules. Provenance had to be carried through the existing archive format without changing `System::new`; the cache therefore advances to schema4/semantic6. Old schema3/semantic5 data cannot reconstruct its former origin. The author/native review also corrected row2 and row3 control inputs to actual Ctrl-D/Ctrl-C; earlier `quit` runs remain in the record.
3. **Was anything in the specified scope silently dropped?** No known I1–I8 requirement was intentionally omitted. The complete literal CSV, source/metadata/full-map checks, genuine-duplicate ownership, marker promotion, old-schema rebuild/warm-cache stages, CLI child boundaries, original native rows, added predecessor-source row, build/docs and diff gates are represented in the attached evidence. The retained limitations are explicit: historical cache provenance is unknowable after rejection/rebuild; import is not persisted; broader M7.5 input, M7.6–M7.8, M7.10, game-backed validation, UI, distribution, repair, coverage/performance and higher-level integration remain separate/open work.

## Handoff

CDC owns source/index/gates and independent verification from commit `718f389195a41cd3eef17432e84e6bbec32637b2`. This report and the two evidence files are CC-authored return artifacts; no acceptance is asserted. The [CDC execution record](/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/planning/project01-mvp/arc06-fit-and-finish/slice28-community-import-contract/execution.md) and the preserved [predecessor provenance record](/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/planning/project01-mvp/arc06-fit-and-finish/slice28-community-import-contract/artifacts/predecessor-provenance.json) remain the authoritative planning evidence and byte-origin mapping.
