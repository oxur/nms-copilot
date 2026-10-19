# Slice15 CC initial implementation return

Assignment: Project01 Arc06 Slice15, `arc06-fit-and-finish/slice15-complete-live-save-updates/cc-prompt.md`; initial assignment, no predecessor, 0/5 refinements. Issued in planning commit `aec768f4c82a084fbd9150b61bc0a02a53f8c11f` (parent `aebef82393cff80600a5b49407d676663e1a903a`); planning HEAD at intake was `aa04d859ea71811d8ff5e15bf20ae84bce62195d`. Dedicated source branch `codex/complete-live-save-updates` began clean at exact baseline `130034e8a6f2e77605058620fea2fb0cae43c608`. Implementation was performed by the assigned CC; CDC retained planning ownership. No push, merge, install, release, or application integration was performed.

## Delivered source increment

The source adds the runtime `nms_core::SaveUpdate` and additive `nms_watch::start_watching_updates` API. Both legacy `SaveDelta` and full update subscriptions use a shared worker. The complete state and version come from the same `SaveRoot` parse as the delta; full state-only/version-only changes are observable, unchanged input is suppressed, and legacy delta behavior remains. The full stream has capacity one with ordered retry/cancellation. Handles own debouncer, shutdown signaling, and worker join. `GalaxyModel::apply_save_update` applies additive atlas changes, then treats complete PlayerState as physical authority while applying the specified selected-view preservation rules. Tests exercise real watched files, legacy behavior, state-only changes, malformed recovery, filtering/replacement, queue cancellation/order, shutdown, contradictory synthetic movement, and imported atlas retention. README/rustdoc state the usable library capability and the remaining application/coordinator/cache boundary.

Only the six authorized tracked paths changed:

- `crates/nms-core/src/delta.rs`
- `crates/nms-core/src/lib.rs`
- `crates/nms-graph/src/model.rs`
- `crates/nms-watch/src/lib.rs`
- `crates/nms-watch/src/watcher.rs`
- `README.md`

No manifests, dependencies, fixtures, save parsing/raw fields, cache schema, main/MCP/REPL code, or placeholder policy changed. The source result does not claim normal/headless single ownership, full live-cache correctness, application notifications, placeholder promotion, or performance.

## Validation and evidence limits

The final stable source passed all 13 required commands on the authorized host with the prompt's original inherited `TMPDIR`. The latest complete capture is `validation.json` root `attempts`, with separate raw channels, timestamps, cwd, environment, overrides, exit status, and observed harness counts. Focused counts: core 87/87, graph 171/171, watch 32/32; doc-test selections 0. Cache 19/19, route-cache integration 4/4, MCP-live integration 7/7, no-default-feature copilot lib 220/220; workspace `make test`, all-feature build/doctests/docs, cache broken-link-deny docs, and `git diff --check` all exited 0. Full command-by-command evidence is in `validation.json`.

Historical capture is incomplete. The JSON preserves two earlier superseded full 13-gate waves and the initial captured format/lint successes plus a focused aggregate failure (exit 101, with six watcher timeouts visible in the preserved stderr). It also enumerates known but uncaptured earlier lint/compiler failures, standalone watcher/isolated diagnostics, and later sandbox aggregate failures, with unavailable command/source/timestamp/environment/exit/channel fields marked unknown. No transcript was used to reconstruct channels; a current passing run is not claimed to recover historical evidence. The identical authorized-host retry is captured in latest `attempts[2]`. CU-07 remains open for CDC disposition; no evidence-completeness waiver is asserted.

## CU row proposals (CDC has not accepted these)

- CU-01: proposed-done, open pending CDC — additive complete runtime event/watch API and retained legacy public API; source/API review plus focused compilation/tests.
- CU-02: proposed-done, open pending CDC — same-parse state/version and delta; currency-, previous-address-, version-only, unchanged, and movement/full-state tests.
- CU-03: proposed-done, open pending CDC — exact PlayerState, selected view and imported maps under same/cross/absent-prior and contradictory-movement cases.
- CU-04: proposed-done, open pending CDC — native selected-save, filtering/replacement, malformed recovery/baseline and legacy stream tests; historical native failures retained.
- CU-05: proposed-done, open pending CDC — deterministic queue order/cancellation and native idle/post-update handle shutdown/disconnect tests; owned join implementation.
- CU-06: proposed-done, open pending CDC — README/rustdoc scope/content review and required documentation gates.
- CU-07: open — incomplete per-attempt historical capture as described above; gate results and delivery path await CDC independent review and disposition.

All seven ledger rows were open at intake. These are CC proposals, not planning changes or acceptance. Required rows about the application coordinator, normal/headless cache writer, notifications, placeholder policy, broader history/raw-field repair/final transitions, and original C04/R06-R02/R03/M7.4/export/import/format/docs/distribution/coverage/higher-scale/Operator/integration/release obligations remain outside this slice and open.

## Return artifacts

Candidate directory: ignored source workbench `workbench/slice15-return/`. Stage exactly `closing-report.md`, `execution.md`, and `validation.json`; `capture_gates.py` and intermediate JSON snapshots are local diagnostic material, not deliverables. CDC should transfer the three staged files byte-preserved to the canonical slice plan and artifacts and record provenance. No planning file was edited by CC.

## Source commit and handback

Source commit `e254466ffc0e2ab7c2a6ed23f355c3cf6d5bc8ee`, parent `130034e8a6f2e77605058620fea2fb0cae43c608`, message `Add complete parsed live save updates`. `git show` confirmed both required trailers exactly once: `Co-authored-by: Codex <noreply@openai.com>` and `Co-authored-by: Billo AI <ai-engineering@billo.systems>`. The commit contains exactly the six allowlisted paths enumerated above; no extra tracked paths. Post-commit tracked worktree status is clean on `codex/complete-live-save-updates`. CC now returns stable source ownership to CDC for independent diff/evidence review and reproduction. No ledger row is accepted by this return.
