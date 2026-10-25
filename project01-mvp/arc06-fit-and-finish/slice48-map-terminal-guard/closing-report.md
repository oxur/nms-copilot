# Slice48 CC return — proposed disposition

Date: 2026-10-25 UTC. Work was performed on the dedicated `codex/workspace-coverage-contract` branch. This is the CC's proposed completion and handback, not independent acceptance. CDC's final reproduction, row dispositions, bottom-up review, and Autonomous Mode retrospective remain pending.

## Delivered source and commit

Commit `fb2ecebb9b1238c2c4b3d4c790f935f02fb2a6cc` has parent `3e8dd6b426d024adb9318497db49a35281b419cb` and changes exactly `crates/nms-copilot/src/map/mod.rs`. It adds the private static Crossterm adapter, owned best-effort guard and explicit cleanup orchestration; retains cleanup-first error precedence while attempting all applicable operations; and adds the four approved injected tests. The public `run_map` failure/unwind documentation now reflects setup-error precedence, attempted cleanup and its limits. The public signature, imports/exports, original `no_run` example, and entire `run_map_loop` remain frozen. No other tracked path changed.

The local normal-hook commit used the prepared message at `/private/tmp/nms-slice48-cc-source-message.txt`; the raw commit body equals those prepared bytes, with the two required consecutive trailers exactly once. Post-commit the source checkout has clean status and empty index. The main checkout remains at its expected pinned HEAD and retains only the known protected `mcp_live_integration.rs` status and expected native OID; that file was not inspected or modified.

## TG proposals

| TG | CC proposal | Evidence and limit |
| --- | --- | --- |
| TG-01 — acquisition flags and partial setup | Proposed done; CDC disposition pending. | L2 covers all three setup errors in clean and all-three-cleanup-failing configurations, with exact traces, original `PermissionDenied` operation names and owner release. Focused tests passed in both all-features and no-default-features runs. This is injected control-flow evidence. |
| TG-02 — cleanup order and error precedence | Proposed done; CDC disposition pending. | L1 and all 16 literal L3 combinations passed with exact eight-entry traces, all cleanup attempts and cleanup-first precedence; both `Ok` and body `Interrupted` outcomes are covered. This does not show physical terminal recovery. |
| TG-03 — owned Drop and unwind | Proposed done; CDC disposition pending. | Both L4 unwind outcomes passed; the known panic payload survived, all restoration methods were attempted, and the mock owner was dropped. This does not cover abort, process kill, forgotten guard, secondary panic or failing operating-system behavior. |
| TG-04 — one-file integration, truthful docs and frozen regions | Proposed done; CDC disposition pending. | CDC reviewed the full initial source and accepted the only correction in iteration01. Strict rustdoc passed; generated `run_map` HTML was read; the source diff/commit contain no changes to the signature/imports/example/loop or another tracked file. CDC must replay against the committed source. |
| TG-05 — attributable gates, coverage and pins | Proposed done for the Slice48 evidence with the 95% result explicitly red; wider coverage remains open. | All nine original groups were executed and captured. Groups 1–6 and 8–9 passed. Group 7 exited 1 at the configured 95% line threshold; report is 24,213/25,836 = 93.7181%, with 14 files below 90%. `map/mod.rs` is 208/257 (80.9339%); `run_map`, `run_map_loop`, real adapter methods and their production guard/orchestration specializations are unexecuted. Positive injected-guard witnesses are reported in `execution.md` and `validation.json`. |
| TG-06 — independent verification, retrospective, bubble-up and Slice49 ownership | Open; CDC-owned. | CC does not close its own independent-review row. CDC must reproduce the nine groups against the final committed source, disposition all rows, complete the Autonomous Mode retrospective and bottom-up handback, and retain required Slice49 native-PTY/D0042 and real-adapter/module-floor work. |

## Arc bubble-up and scope reconciliation

The assigned integration is present: public `run_map` uses the private guard with a statically dispatched Crossterm control, and the injected tests exercise the same orchestration through a mock. The real adapter, public entry, event/frame loop and operating-system restoration are still unexecuted here. Slice49 remains responsible for isolated native-PTY D0042 navigation/zoom/help/resize/exit/restoration evidence and real-adapter/entry/loop/module-floor reconciliation. The 95% workspace target, hosted CI and full parent M7.7/D0042 requirements remain open. No arc-plan change is proposed; those open owners and limits remain explicit.

Scope-as-delivered matches the authorized source and test scope. The only measured red is the existing observed workspace coverage gap; it was not hidden, waived, or “fixed” with extra tests. The source contract, error precedence, tests, docs and protected-region boundaries were preserved. No unrelated path, integration behavior, user terminal, dependency, public API, project/arc scope or release path was changed.

## Evidence inventory and handback

The full raw-first attempt ledger is `/private/tmp/nms-slice48-cc-attempts.json` and contains 14 records: two supplied capture-helper controls and twelve actual format/lint/test/docs/coverage/diff attempts, including the initial pair and the one actual refinement pair. The helper is `/private/tmp/nms-slice48-cc-capture.py`. Raw workspace coverage JSON and its lossless gzip are `/private/tmp/nms-slice48-cc-workspace-coverage.json` and `/private/tmp/nms-slice48-cc-workspace-coverage.json.gz`; both hashes and byte counts are recorded in `execution.md` and `validation.json`. The initial and iteration01 prompts, current slice plan/ledger, original D0042, and pinned dependency source remain their governing authorities; these return records supplement rather than replace them. The two support capture controls are capture-only evidence, not product gates.

The source commit is clean at `fb2ecebb9b1238c2c4b3d4c790f935f02fb2a6cc`. The index owner is returned to CDC with no staged changes. Actual refinement count is 1 of 5 available after the initial prompt; no process reset, extra feedback, or auto-refinement is claimed. CDC now owns final independent reproduction and disposition. The main-checkout protected leftover and its exact pin remain untouched.


## CDC final disposition after independent replay

2026-10-25UTC: all SIX TG/R06-S48 DONE at clean fb2ecebb9b1238c2c4b3d4c790f935f02fb2a6cc. The preceding CC proposals/TG-06-open wording records the earlier frozen handback; current independent acceptance, source/record qualifiers and mandatory retrospective are in cdc-verification.md and cdc-retrospective.md. Original nine groups complete:1118runtime+118docs(91execute27compile),four focused both feature modes,28 separate overlapping no-defaultdocs,strict+diffs0; coverage1 retains parent95 red93.718068%/mod208of257. ONE actual refinement/no exception; required49 real terminal/adapter/loop/module-floor and full parent/input/Operator gates remainOPEN. Scope/support inventory and bottom-up66arc54done12open/project25OPEN are independently dispositioned. No integration/maincleanup/release.
