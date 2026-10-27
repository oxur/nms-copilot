# Slice11 initial CC checkpoint — route proof

## CC intake and readback

Read the complete `cc-prompt.md` (13 lines), `slice-plan.md` (15 lines), and `ledger.md` (9 lines) at planning revision `5ad6d7cef8a108970c7ad53c5ba5b8fc3daf4670`. The source worktree was clean at `5f3b18eadddd57d54a92c9ec456b0f1e98b0869c` before edits. Read the required `route.rs` test imports/helpers (704–755), route/nearest-neighbor/2-opt implementation (315–485), and existing 2-opt tests (1270–1310), plus Rust anti-patterns AP-02/AP-04, testing gate selection/proportionate proof, and implementation-prompt authoring required-reading/intake sections.

Readback: implement only one inline, test-only closed-tour regression. An independent five-point coordinate oracle must establish that every nearest-neighbor choice is unique, greedy order is `[A,B,D,C,E,A]`, and greedy distance is `400*(8+2*sqrt(13))`. Exercise both direct `two_opt_improve` and public `tsp_two_opt`; each must be strictly shorter by over 1,000 ly and match `400*(9+sqrt(13))` within `1e-6`, while retaining both A endpoints, exactly six hops, exact sorted `[A,A,B,C,D,E]` membership, galaxy 9, and no waypoints. Do not pin one of the tied optimal orders, alter production code, or weaken the literal oracle. Required validation is format, lint, exactly one selected focused test, then the full `nms-graph` crate test suite, all offline/locked where specified. This is checkpoint 1 only; QS-02 and QS-03 remain open, and root owns source Git/index/commit/push/CI.

## Implementation and validation

Added exactly one regression test in `crates/nms-graph/src/route.rs`; no production code or other source path changed. The literal fixture produced the required greedy route and both direct and public optimizer paths passed the exact improved-cost, strict-gain, endpoint, membership, galaxy, and no-waypoint assertions.

- `CARGO_NET_OFFLINE=true make format` — exit 0.
- `CARGO_NET_OFFLINE=true make lint` — exit 0; Clippy and format check passed. Cargo emitted the existing future-incompatibility warning for `proc-macro-error2 v2.0.1`.
- `cargo test --offline --locked -p nms-graph test_two_opt_strictly_improves_unique_nearest_neighbor_closed_tour` — exit 0; exactly 1 test selected and passed (176 filtered unit tests; integration targets selected 0).
- `cargo test --offline --locked -p nms-graph` — exit 0; 177 unit tests, 7 `discovery_names` integration tests, 3 `validated_save` integration tests, and 13 doctests passed (200 total).
- `git diff --check` — exit 0; source diff is limited to the assigned test.

No supporting artifacts were needed; the complete command summaries are recorded here.

## Checkpoint disposition

QS-01 pending independent review. QS-02 and QS-03 remain open. Supporting artifacts: none anticipated; command output is recorded below, with native output potentially streamed/truncated by the terminal.
