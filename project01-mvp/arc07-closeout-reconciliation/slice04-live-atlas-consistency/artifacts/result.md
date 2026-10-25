# Slice04 CC candidate result

Status: **proposed for CDC review; not accepted**. The source candidate implements the owned map snapshot and the bounded application graph consistency repair. AT-01 and AT-02 are proposed done based on scoped evidence; AT-03 remains open because native exit/terminal restoration awaits CDC's direct-PTY replay and the workspace gate is not green.

## Change

- The map captures owned display state and system addresses at entry. Rendering and the modal loop consume that snapshot, using the same address-based rendering path as the public borrowed renderer. Main drops its model read guard before entering the modal loop.
- Application coordinator updates and cache startup repairs rebuild the existing default KNN-10 edges once when system population grows. Node-neutral updates preserve graph topology. Cache archive format and low-level graph APIs are unchanged.
- Public coordinator/cache documentation states the application-owned growth policy. The cache integration oracle now expects the supported rebuilt graph after player-system growth, while retaining raw metadata and foreign-galaxy negative controls.

## Witnesses and validation

The two inherited baseline attempts are retained: the first reachability assertion failed because its expected vector used traversal order `[A,C,B]`; after sorting by identity, consumer comparison passed and the canonical edge assertion failed because incremental A–B was 1600 ly while a full default rebuild also contained A–C and B–C at 800 ly. No route outage is claimed; the same-galaxy shortest-path API has a direct fallback. The repaired fixture has literal 800 ly witnesses and galaxy-9 decoys remain disconnected.

The added cache witness runs the unknown-player case through cold/no-cache startup, a warm cache ensure, and a warm roundtrip. Each has two nodes, one literal edge at `676_721.744_884_853` ly, and the same player/discovery membership. The independent expected model uses the existing default rebuild only after player-key growth. The witness's distance was checked against the literal coordinate delta; it does not call the production distance helper to construct its own expected value.

The owned-snapshot test compares entry rendering, then mutates model/session state and verifies the captured view stays fixed. The production capture test acquires the model write lock after capture. Scoped checks passed:

- `make format`, then `make lint` (rerun after final source edits; both passed).
- Focused coordinator graph parity, coordinator component tests (4/4), cache unknown-player parity (1/1), and the full cache integration test (31/31).
- Copilot map tests (41/41), production capture lock-release test (1/1), owned snapshot test, no-default map library tests (5/5), and no-default cache unknown-player test (1/1).
- Strict affected rustdoc/doctests with `RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links'`: copilot 33/33 and cache 6/6.
- `cargo build --locked --all-features -p nms-copilot --bin nms-copilot` succeeded and produced the source-qualified normal HTTP binary.
- `make test` was run once: **247 passed, 7 failed**. All seven failures were filesystem-watcher timeouts under the restricted sandbox. The affected watcher/coordinator tests were then rerun with native permissions: coordinator 13/13 and the retained-watcher write/drop test 1/1. This does not convert the workspace gate to green.

The no-default package-level build also failed on pre-existing HTTP feature-gating references: `main.rs` reaches `spawn_mcp_background_with_updates` and `Transport::Http` without the feature, while `mcp_live_integration.rs` uses `ExitStatus` after its import is gated. CDC reproduced those blockers at baseline `fb2ecebb`; the relevant no-default library map/cache tests passed. Feature-gate repair is outside Slice04.

The owned PTY/MCP run initialized the loopback server, listed `where_am_i`, read player voxel X=100, opened the map, changed the save to X=101, and observed X=101 through MCP while the map still displayed `Cursor:`. Pressing `q` returned to the ordinary REPL; `status` printed `Position: player position (Euclid, galaxy 0)`. The runner then timed out because it incorrectly expected the status output to include the voxel coordinate. A predicate-correction replay failed before map entry because the locally authored PTY reader did not answer CPR. These outcomes are recorded in `artifacts/native-smoke.json`. CDC's independent direct-PTY/CPR check remains necessary for ordinary process exit and terminal restoration.

## Remaining gate

AT-03 stays open. The workspace `make test` population is still 247/7, and this CC candidate does not attest full workspace success or terminal restoration. The independent native replay and final full-workspace gate belong to Arc07's later acceptance work. No threshold or population was narrowed.

Source candidate commit: `8f10d4cbe04f7d781667cc4e37280c333a67ee8d` (`fix: reconcile live atlas graph and map snapshots`). Planning record commit: to be filled after the exact-path commit. Acceptance remains CDC-owned.
