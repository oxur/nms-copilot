# Slice28 iteration06 CC close report — proposed evidence

This is the additive CC return for the final bounded EOF-handshake correction in original M7.9. It records a reviewable one-path child commit and current proof; CDC owns independent verification and acceptance. All eight CI rows remain open.

## Assignment and delivered change

The exact current assignment is [cc-prompt-iteration06.md](/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/planning/project01-mvp/arc06-fit-and-finish/slice28-community-import-contract/cc-prompt-iteration06.md). This was the fifth and final actual refinement: initial plus pre-edit01 were the first implementation, and 02–05 were actual refinements one through four. The only tracked path changed in this final pass is `crates/nms-copilot/examples/repl_live_probe.rs`; the two native controller files are private v3 copies.

The four affected monitor handshakes now require a complete live-state acknowledgment before process exit. The native controller waits for that exact acknowledgment, sends the existing quit input, verifies exit0, and only then begins the existing eight-second SSE EOF check. The monitor retains its subscription and queued events until peer closure. Existing assertions, terminal actions, retry bounds, and timers remain in place.

The delivered child commit is `be4145b631405ff19c3e403457ecaa581afe805f`, parent `718f389195a41cd3eef17432e84e6bbec32637b2`, tree `c706d0ae7064171da4809037a97c75f9fc1ba09d`. It changes exactly one path and has consecutive Codex and Billo AI trailers, each once. The worktree and index were clean at CDC handoff.

## Actual current validation

Captured format, lint, Python compile, Node syntax check, all-features binary/probe build, self-check, four affected native cases, and working/baseline diff checks are listed with their complete raw attempts in [validation.json](/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/community-import-contract/workbench/slice28-return/iteration06/validation.json). The first self-check attempt failed in the sandbox at paired SSE bind (`Operation not permitted`); the identical authorized-host retry with the same argv, source and original TMPDIR passed. Both attempts remain recorded. Fresh row10 unknown/enabled, row12 warm/enabled, row13 predecessor/disabled, and row14 predecessor-source/disabled cases passed. Their 131 full launch/raw/post tool-call triplets and four actual fixture trees are retained under [evidence](/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/community-import-contract/workbench/slice28-return/iteration06/evidence/sha256-manifest.json), with a SHA-256 manifest covering 621 files and 2,576,737 bytes. This is a newly created CC inventory of this iteration’s evidence; it does not restate or reattribute the earlier CDC inventory claim.

The original three returns remain byte-exact: execution SHA-256 `6b11cb3f13c705c5601d8484832280a18c9c041ab7cb9af856c28872e3121543`, validation `65223a2f6a3fa4428773cb5e81140d709db830d377ea2767720c2738804780b9`, and closing report `9f7634ead2d73573e9a3fcfdf18cd49b55d8c26aabfd08986b1778661b9b2943`. Their 88 capture entries match the prefix of the current 110-entry raw capture snapshot exactly. The new 22 captured attempts and all four native case directories are isolated in this iteration06 namespace.

## CI proposals

These statuses remain open pending CDC independent disposition:

| Row | CC evidence proposal |
|---|---|
| CI-01 | Prior literal source-tag, serde, constructor-compatibility, and checked-extraction evidence remains in the preserved original return. |
| CI-02 | Prior CSV corpus, counts, complete atlas comparisons, replay, and promotion evidence remains in the preserved original return. |
| CI-03 | Prior query/name/update and whole-state evidence remains in the preserved original return. |
| CI-04 | Prior source-bearing cache roundtrip and genuine predecessor rebuild/warm-hit evidence remains in the preserved original return. |
| CI-05 | Prior real CLI import, output/error, and input-immutability evidence remains in the preserved original return. |
| CI-06 | Original native cases plus fresh current row10 and rows12–14; 131 current launch/raw/post triplets retained. |
| CI-07 | Fresh format/lint/build/self-check, four affected native cases, and final diff checks; CDC’s independent and originally pending groups remain outstanding. |
| CI-08 | Current readback, 110-entry capture snapshot, exact one-path child commit, and additive return; the original direct-first Node diagnostic gap remains open. |

## Bubble-up to the arc

1. **What capability was assigned, and what remains local or unintegrated?** Original M7.9 delivers community CSV import with explicit provenance into the queryable in-memory atlas and preserves that provenance through the existing cache lifecycle. The `nms import` command is a one-shot read-only session; import persistence, a REPL import command, and wider UI/distribution integration remain outside this milestone. Iteration06 changes only the native test-probe EOF handshake and does not add application capability.
2. **What changed or surprised us?** The earlier row12 close wait could start before the native controller’s quit reached the child, so the monitor’s existing eight-second timer did not bound the time until process exit. The final test correction acknowledges a live full-state boundary first, observes actual exit0, then starts the same EOF wait. This is a bounded ordering correction; the old failed run and successful unchanged replay remain distinct evidence, and their exact timing does not prove a unique cause.
3. **Was anything in the specified scope silently dropped?** No item in the issued four-monitor/two-driver-function correction was intentionally dropped. The exact live assertions, queued-event-sensitive EOF predicates, restart checks, process exits, file/cache postconditions, existing timeouts, and original controls remain represented. No timer was extended and no production/API behavior changed. All other M7.9 parent requirements and all eight CI decisions remain open for CDC review.

## Handoff

CDC owns source/index/gates from the clean child commit. CC performed only new ignored iteration06 return work after that handoff. This is proposed evidence, not self-acceptance. The original three returns and their historical values remain unchanged.
