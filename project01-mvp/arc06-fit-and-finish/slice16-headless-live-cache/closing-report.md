# Slice16 CC closing report

This is a proposed implementation close for the initial Slice16 assignment, issued from planning revision `04003b978d8c0ee2d5bc0fb7ecaa80775f0943a7`. Source work is confined to `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/headless-live-cache`, branch `codex/headless-live-cache`, based on `11cf7e6a8e2a7549a223880820c0bf4543381459`. Source commit: `4404f37f70df62177689d6108e68b3685ea422a1` (parent `11cf7e6a8e2a7549a223880820c0bf4543381459`); exactly the seven allowlisted source paths are committed, both required trailers parse exactly once, and the tracked tree and index are clean. CDC owns planning/index and independently reviews this return. All ledger rows remain open until CDC acceptance.

The source adds a single app-owned full-update watcher/cache coordinator and wires it into headless stdio/HTTP as an observer-only consumer. Each accepted event formats existing notes before applying once, extracts version-matched cache state only when enabled, releases the model lock before serialized cache I/O, and publishes ordered full updates despite write errors. Legacy public wrappers, watcher formatting, and normal-mode ownership remain preserved. The README documents the behavior and the Slice17 boundary. No dependency, parser, model schema, graph/cache implementation, raw save semantics, placeholder policy, or planning files changed.

Final candidate-specific checks passed in prescribed order: `make format`, `make lint`, focused stdio and HTTP complete live-update matrices (1/1 each), full `make test` (all populations zero failures, including 229 nms-copilot library and 13 MCP live integration tests), and `git diff --check`. Earlier required build/docs/strict docs/cache/route/focused gates passed on their recorded earlier diff identities; their exact attempt population, environment and separate raw stdout/stderr channels are retained in `execution.md`, `validation.json`, and `raw-gates/`. Native restricted-run failures are retained alongside unchanged authorized-host retries. A test-oracle address typo and a writer-fixture compile typo also remain as failed attempts with their successful corrections. No historical output is reconstructed.

| Ledger row | CC proposal | Evidence / limit |
|---|---|---|
| LC-01 | Proposed done | One watcher/worker/apply owner, two ordered observers, slow/disconnected observer controls, preserved legacy APIs; CDC source review pending. |
| LC-02 | Proposed done | Literal complete `PlayerState`, version, imported system/planet/base maps and selected-galaxy checks; full checked cache plus rebuilt primary maps. |
| LC-03 | Proposed done | Barrier proves model access during blocked writer and serialized order; no-cache bypass, write-error continuation, final recovery cache. |
| LC-04 | Proposed done | Six live phases and five malformed writes/recovery on actual stdio and HTTP; state-only/version-only notifications and cache invariants. |
| LC-05 | Proposed done | Cold, bypass existing/missing, disabled current, and write-error controls on both transports; bytes/mtime/absence/error behavior. |
| LC-06 | Proposed done | Bad startup, idle/in-flight Drop, receiver lifecycle, stdio EOF; no arbitrary I/O deadline claimed. |
| LC-07 | Proposed done | Headless API/README/docs and strict rustdoc checks; normal REPL/shared MCP integration stays Slice17. |
| LC-08 | Proposed done | Per-attempt capture ledger, source commit/trailers/index proof, clean stable handback; CDC independent rerun/review is still required. |

The three transport/case dimensions are: full warm imported-cache update phases (currency-only 4720; previous-only 4720; version-only 4721; cross-reality 4722; same-reality 4723; five malformed stable writes with no events/cache mutation; recovered 4724; identical rewrite unchanged), plus Cold, BypassExisting, BypassMissing, DisabledCurrent, and WriteError on both stdio and HTTP. Full typed cache state and preserved maps are asserted at every warm phase. The component test `test_blocked_writer_releases_model_and_preserves_update_order` separately proves outside-lock serialized I/O.

## Bubble-up to Arc06

1. **What design change was necessary for the accepted live-save behavior?** The full parsed update must have one application-owned consumer that owns model apply, optional full-state cache persistence, and publication; observer-only MCP wiring prevents a second producer or stale startup-version cache writer.
2. **What remains outside this slice even when these tests pass?** Normal REPL/shared MCP composition, terminal lifetime, idle-session anchor refresh, broader raw-field/history transitions, and Operator-gated unknown placeholder semantics remain open; this does not complete both-mode or parent acceptance.
3. **What evidence should the next review preserve?** Separate native transport matrices, literal full-state/map/cache checks, failed restricted and corrected attempts with exact diff identities/channels, and CDC independent source/gate review must remain distinct; a fresh rerun does not rewrite historical evidence.

`workbench/slice16-return/execution.md` contains the full readback, source inventory, intake deviations, captured attempt table and applicability. `workbench/slice16-return/validation.json` indexes each command, environment, diff identity, channels and raw receipts, including the post-commit baseline-range diff-check receipt. These three records are filesystem-only in ignored `workbench/slice16-return/`; they are not source commit paths. Source ownership is handed back to CDC after the exact commit and final clean tracked/index proof. No push, merge, installation, release, or independent acceptance occurred.


## Refinement01 intake

The initial CC files match the canonical CDC transfer hashes exactly: closing-report 5,692 bytes / `0e7774446b16d47c449e07d207e7b36c26b973946045663ccc91a152dbde1fb0`; execution 18,545 bytes / `a1333a4d2ffa8c2765af8ef54c5203aac527ce5cc3df249ea2051c202af7c3e0`; validation 101,849 bytes / `b779ff7856378b7e83639f4f0f95f19ea75ebde57134f413c09e9738b1d4a008`. This section is appended; all three previous byte prefixes are unchanged. Source intake began at clean `4404f37f70df62177689d6108e68b3685ea422a1`, parent `11cf7e6a8e2a7549a223880820c0bf4543381459`.

The overwritten attempt-0 compile-only stderr is exactly recoverable from the existing `gate-attempts.json` base64 record: 12,317 bytes, SHA-256 `cc8520fa47c9a0ffcb50d6f1d3463c337e7ab68c0fde61abca2e6e36bb845892`, matching the original receipt. The shared raw filename currently has attempt 1's 1,061-byte channel (`cac0cb280bc96cde6f37053d03316757b149b93f18afb2629f6287681428e187`). No bytes were reconstructed or rerun as history. The validation appendix embeds all 108 original stdout/stderr channels with length, SHA-256, base64 and raw-path-match disposition.


## Refinement01 final execution addendum

This addendum records the corrected final candidate after the earlier preliminary statuses above. It supersedes their candidate-specific gate totals where the test-only refinement changed the result. The immutable initial return and earlier refinement history remain byte-preserved; this addendum adds the current evidence without replacing it.

The final correction changes only native integration-test expectations and timing: stdio cold cache is inspected after the MCP initialization handshake; forced HTTP bind-collision handling waits for the natural child exit and asserts the source-grounded successful headless status 0, the exact address-in-use diagnostic, and absence of readiness. The cold cache oracle checks the independently specified persisted `CacheData` source snapshot before startup physical-system insertion, then the distinct live cache snapshot after startup and event; it no longer calls production `rebuild_model` to generate expected assertions.

Final candidate checks pass: format, lint, focused all-feature libraries (520 total: 230 copilot, 87 core, 171 graph, 32 watcher), no-default copilot library (229), `make test` (all groups zero failures, including copilot lib 230 and MCP live 13), workspace all-features build, workspace doctests (one non-empty doctest pass), standard six-package docs, strict cache/watch docs, focused bind collision (1/1), full native HTTP/stdio matrix (13/13), and working diff check. Previously completed cache tests (two 19-test harnesses) and route integration (4) are retained because the final correction is limited to native integration tests and does not affect cache/graph/route production code. Every current and retained attempt retains its exact working-diff identity, environment, argv, exit, populations, and separate channels in `validation.json`.

One native full run failed 12/13 because the new bind-collision reaper expected a nonzero exit; the captured child emitted `Address already in use` and naturally returned success(0), as the scoped headless caller logs the server error then returns. The test now asserts that exact status instead of loosening to arbitrary exit codes. The affected collision case then passed 1/1 and the full matrix passed 13/13. Earlier stdio cold timing failure was corrected by moving only the source-cache read until after initialization; that test passes in the final 13/13 matrix.

The capture index records two wrapper-level mistakes separately: a single-token `make format` argv could not launch and did not reach make/SUT; its empty reserved channels and available wrapper exception are identified as metadata only. The correct `make format` retry passed. A mistaken `cargo make test` invocation failed before any test; the distinct required `make test` command passed. No failed attempt was dropped or relabeled.

| Row | CC proposal | Current witness | Disposition |
|---|---|---|---|
| LC-01 | Proposed done | One coordinator owner, exact-once apply and ordered observers, including native events | CDC review pending |
| LC-02 | Proposed done | Full typed state/version, imported maps, source/live checked cache states and selection | CDC review pending |
| LC-03 | Proposed done | Full-state blocked-writer lock/order, write-error progress/recovery, no-cache | CDC review pending |
| LC-04 | Proposed done | Six phases, malformed source/cache invariants and recovery over both transports | CDC review pending |
| LC-05 | Proposed done | Five cache controls on both transports plus cold bind collision/retry | CDC review pending |
| LC-06 | Proposed done | Typed startup error, observer/lifecycle and safe barrier teardown coverage | CDC review pending |
| LC-07 | Proposed done | Fallible runtime initialization boundary; legacy and normal-mode source preserved | CDC review pending |
| LC-08 | Proposed done | Original and current attempt populations/channels retained; exact commit proof to follow | CDC review pending |

All eight rows remain OPEN; green CC gates are evidence for independent review, not acceptance. `validation.json` retains all prior top-level values, verifies their stored canonical hashes, and appends the complete refinement attempt/channel manifest. The exact-path source commit and stable handback proof will be appended after commit.


## Exact-path commit and stable handback

Committed `db77e07612601ffa1bf275b4df1acd87a1787975` with parent `4404f37f70df62177689d6108e68b3685ea422a1`. `git diff-tree` reports exactly: `crates/nms-copilot/src/coordinator.rs`, `crates/nms-copilot/src/mcp/mod.rs`, and `crates/nms-copilot/tests/mcp_live_integration.rs`. The parsed final trailer block contains each required co-author line exactly once. The captured post-commit `git diff 11cf7e6a8e2a7549a223880820c0bf4543381459..HEAD --check` passed. Source status and index are clean; no return/workbench file entered Git. Stable source/index ownership is handed back to CDC for independent review. All eight LC rows remain open pending that disposition.


## Refinement02 closeout

The bounded test-only refinement is committed as `856775df80e7585aa5cedc03219fe56976acf679` on `codex/headless-live-cache`, directly above `db77e07612601ffa1bf275b4df1acd87a1787975`. It changes only the coordinator and native integration test files. The complete golden atlas/cache oracle now checks all imported system, planet, base, name, biome, address, node, spatial, selected-galaxy, player-state, version, and checked-cache results. The two blocked-writer tests release safely on assertion failure. New transport cases drain captured output through EOF, test trailing stdout rejection, retry a warm HTTP startup only after an observed bind collision using a fresh fixture, and recheck file/cache state after process teardown.

The final gates pass on the final candidate: `make format`, `make lint`, and the full authorized-host native integration binary (15/15). The coordinator-focused host suite passed 9/9 and no-default copilot library passed 229/229 on the same production/library bytes; preceding sandbox attempts timed out on native filesystem notifications and remain recorded. An earlier native 15/15 run preceded one added post-teardown assertion; the final 15/15 run includes that assertion. Final pre-commit and post-commit diff checks passed. The final commit has exactly the two authorized paths and both required parsed co-author trailers exactly once. Source/index are clean and handed back to CDC.

All eight LC rows remain `OPEN`; this is a CC proposal for independent CDC review, not acceptance. The proposed row statuses remain: LC-01 through LC-08 proposed done, with evidence applicability and failed attempts documented in `execution.md` and `validation.json`.

| Row | CC proposal | Ledger status |
|---|---|---|
| LC-01 | Proposed done | OPEN |
| LC-02 | Proposed done | OPEN |
| LC-03 | Proposed done | OPEN |
| LC-04 | Proposed done | OPEN |
| LC-05 | Proposed done | OPEN |
| LC-06 | Proposed done | OPEN |
| LC-07 | Proposed done | OPEN |
| LC-08 | Proposed done | OPEN |

The three filesystem return records are `workbench/slice16-return/closing-report.md`, `workbench/slice16-return/execution.md`, and `workbench/slice16-return/validation.json`. Capture support remains in the ignored `gate-attempts-iteration02.json` index and its UUID-named raw channels; no workbench artifact was committed.

## Bubble-up to Arc06

**BU-16-01 — Validate complete transport streams through EOF.** A reader that checks only currently queued messages cannot detect output emitted after the last expected response. The new tee preserves the protocol reader while an independent observer validates every stdio line through EOF and confirms HTTP stdout stays empty through process exit.
