# Slice09 CC proposed-done return

## Assignment and commit ancestry

- Assignment: `arc06-fit-and-finish/slice09-galaxy-scoped-session/cc-prompt.md` (initial implementation assignment).
- Issuance commit: `70a249671e9a29dfe32493bebfb491a0d7323ae0`; planning intake parent: `9d6e43e725de90737d2738fa7261ce9c35903a1c`; actual planning branch HEAD at intake: `2d73f1192e22a2fae2af123ccd16081eb6fd4a28`.
- Source baseline/direct parent: `f71d607e27e18d7266855a2708449f9d6b90c632`.
- Source return commit: `e9cabd5fe63f19e9fe2e6da840845315d2ee2d18` (`codex/galaxy-scoped-session`), with that exact direct parent and only the six authorized source files. Required co-author trailers each appear once.
- Planning return will be committed on the current `planning` HEAD `2d73f1192e22a2fae2af123ccd16081eb6fd4a28`, preserving the later planning history. Its exact first return and receipt-follow-up commit IDs will be appended as an audit within the permitted return files after Git creates them.

## Work returned

Typed movement now updates the existing physical player state and current reality, and selects the destination galaxy only for cross-reality movement. Replaying an old movement after a deliberate galaxy selection preserves that selection. Same-reality movement keeps a separately selected view. Absent player state stays absent; previous/freighter addresses and all currency fields are preserved.

`SessionState::from_model` initializes its galaxy from the model's selected view. The new `sync_from_model` refreshes selected galaxy and global counts, follows the physical player only for an unset or player-following position, and preserves Base/Address anchors plus filters and warp range. Reset behavior now synchronizes derived values and preserves model selection/player data.

Watcher notifications query each physical address in its own galaxy, use canonical galaxy-qualified `SystemId` for incoming destination and scan names, retain existing note ordering/templates, and synchronize session state after all deltas including empty/discovery-only deltas. The MCP location builder uses the same explicit address-scoped lookup. Actual in-process tool and resource registries return physical-location JSON while the selected view differs and leave model state unchanged.

The real standard channel drain writes after queued deltas and the resulting cache reads/rebuilds with schema 2/semantic 2 unchanged. Tests cover cross-galaxy then same-galaxy movement, a separately selected galaxy 7, complete player-state preservation, qualified system/planet associations, session anchor preservation, and byte-for-byte cache stability after an empty drain.

## Ledger row walk and proposed dispositions

These are CC proposals only. All seven row statuses remain **open** for independent CDC review.

| Row | CC proposal | Evidence |
|---|---|---|
| LM-01 | Proposed done | `test_compute_delta_cross_reality_movement_preserves_qualified_discoveries` constructs typed snapshots in both 0→9 and 9→0 directions, observes exact delta identities and applies them to the model; exact names, planet keys/names/biomes and counts are asserted. Existing qualified collision/replay controls remain. Final focused suite: 717 passed, no ignored/failures. |
| LM-02 | Proposed done | `test_apply_delta_cross_reality_updates_physical_state_and_selection` and `test_apply_delta_same_reality_keeps_selection_and_absent_player` assert complete literal player state, both cross directions, deliberate selection/replay, empty state, absent player and additions. `test_drain_watch_events_same_reality_preserves_selected_galaxy_in_cache` verifies selection 7 with physical reality 9 after cache rebuild. |
| LM-03 | Proposed done | Session tests cover selected 9 with physical 0, absent player selected 7, player-follow synchronization, Base/Address preservation in both movement directions, filter/warp retention, reset position and reset-all without player-state mutation. Watch tests verify live reconciliation after movement and empty delta. |
| LM-04 | Proposed done | Watch tests assert exact 0/9 and 9/0 templates, exact incoming destination and scan name precedence against a conflicting same-ID pre-model name, and `unknown` for foreign-only fallback. Existing note ordering/templates remain. |
| LM-05 | Proposed done | The public `NmsTools::call("where_am_i", ...)` test and public `NmsResources::read(PLAYER_LOCATION_URI)` test parse actual JSON for physical 0/9 in both directions with selected view opposite; both inspect unchanged model state after reads. Resource test matches the pinned `TextResourceContents` variant. Both public registries exercise the shared no-player error path; builder asserts its exact prior text. |
| LM-06 | Proposed done | `test_drain_watch_events_cross_and_same_reality_movements_roundtrip_cache` sends two real mpsc deltas, closes the sender, writes a tempfile cache, reads/rebuilds it, checks full player state and qualified discoveries, observes live explicit anchor preservation versus rebuilt follow-player defaults, then compares full cache bytes after empty drain. A separate test verifies selected 7 survives a same-reality cache roundtrip. Existing no-cache-path channel test confirms deltas still apply. |
| LM-07 | Proposed done | Intake and immutable six-file digest comparison are in `artifacts/execution.md`; exact commands and byte/hash channel captures are in `artifacts/validation.json`; source commit parent/pathset/trailers are recorded above and in execution. Candidate and planning pathsets, planning commit receipts and all open statuses are part of the final return audit. |

## Validation and evidence limits

The final source state passed the required ordered sequence: `make format`; `make lint`; six-package all-features tests; copilot no-default-features library tests; `make test`; workspace all-features build; workspace doc tests; requested six-package docs; strict `nms-cache` rustdoc with `RUSTDOCFLAGS=-D rustdoc::broken_intra_doc_links`; candidate `git diff --check`; planning `git diff --check`. Final counts were 717, 213, and 913 passing tests respectively, each with zero failed and ignored tests. The workspace doc-test command ran nine harnesses with zero doctests. Every attempt, including the earlier failed lint compile and capture-runner prelaunch error, remains in `validation.json` with separate raw stdout/stderr evidence or an explicit prelaunch record.

Existing diagnostics: `proc-macro-error2 v2.0.1` emits its future-compatibility advisory; no-default-feature copilot tests report an HTTP-disabled unused `SocketAddr` import; ordinary docs report existing unresolved-link/HTML warnings in core/graph/query and the copilot `effective_save_file` link. The strict cache docs gate passed. No strict combined core/query rustdoc gate was run because it is retained as an open parent requirement.

This does not prove filesystem notifier delivery, MCP stdio/HTTP/subscription behavior, full cold/warm startup, or repair of historical inconsistent cache state. No raw save decoder was used. Cache schema and semantic revisions stay at 2/2. Unknown destination placeholders/promotion remain with Slice11; REPL galaxy commands and query-reference defaults remain with Slice10. Later DD.UA/base/nonzero-galaxy policy and all parent export/import/docs/distribution/coverage gates remain open.

## Artifact inventory and CDC handoff

- Immutable author input: `artifacts/author-baseline.json` (not modified; all six original source SHA256 values matched at intake).
- CC-created evidence: `artifacts/execution.md`, `artifacts/validation.json`.
- CC return: this `closing-report.md` and evidence/notes-only updates to `ledger.md`; ledger statuses remain open.
- Immutable assignment inputs: `cc-prompt.md`, `slice-plan.md`; predecessor records and parent/Project02 planning untouched.
- CDC handoff: independently reproduce applicable gates and review the contract, exact intake, six source identities, path/parent/trailer receipts, test oracles and evidence limits before deciding any LM row. No acceptance or parent closure is claimed here.

