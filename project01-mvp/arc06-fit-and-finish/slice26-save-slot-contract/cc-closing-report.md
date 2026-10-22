# Slice26 save-slot contract — CC return

## Candidate and scope

The assigned five-path candidate is committed locally at `84a5abde8a772f031cb1f1620a2bc3af144ba285`, parent `03b6536636f00a74a52f2d783a9ef80a671d191e`. The commit contains exactly `README.md`, `crates/nms-cli/src/main.rs`, `crates/nms-cli/src/saves.rs`, `crates/nms-cli/tests/multi_save_integration.rs`, and `crates/nms-save/src/locate.rs`. The commit has the required consecutive parsed coauthor trailers, with each trailer once. Its subject says proof is pending because the corrected integration test still has one failure. Index and worktree are clean after commit.

The changes preserve the original slot-selection policy: the first name-sorted account remains the only account searched for an explicit slot, default selection remains globally newest, and explicit save paths retain precedence. The implementation routes Import through the existing resolver, bounds numbered discovery to save2 through save30, and distinguishes an actually empty account from I/O failure. The new tests cover the literal thirty-file discovery table, three account kinds, exact Save A/B caller matrix, slot/flag position, listing/error behavior, complete output populations, and target-only negative controls. README changes are limited to Multi-Save Support.

## Validation and unresolved failure

The final captured format/lint pair passed. The focused integration command, whole CLI package tests, and full `make test` all fail only in `test_actual_cli_routes_select_literal_save_rows_for_all_ten_save_backed_commands`. The retained assertion reports 43 lines before versus 42 after in `assert_only_table_cell_changed`. The focused run had 5 passes and 1 failure. Whole CLI had 129 passes and 1 failure. Full `make test` had 177 passes and 1 failure across 178 runtime cases. The captured source identity was unchanged across those failures. No source correction has been made in response.

The all-feature workspace build passed. Workspace doctests passed all 33 cases. Strict save/CLI documentation with warnings denied passed. The working-tree diff check passed; after staging only the new test, the full baseline diff check passed and included that test. The earlier focused `nms-save` gate remains applicable at 130 runtime passes and 14 doctest passes because its source files are unchanged. Exact raw attempts, full child channels, argv, source identities, and timestamps are in `/private/tmp/nms-slice26-cc-attempts.json`; the execution chronology is in `execution.md` and the summarized gate record is in `validation.json`.

## Proposed status

CC proposes the implementation and tests for review, but does not claim Slice26 acceptance. All six M7.3 acceptance rows remain open pending CDC's independent review and reconciliation. The focused integration and aggregate test gate remain red, so the save-slot proof is incomplete. Higher M7.3 and project gates remain outside this return's acceptance claim.

## Iteration06 current result

The final authorized one-line test-helper correction is `replace_info_value(...).lines()` to `.split('\n')` in `crates/nms-cli/tests/multi_save_integration.rs`, on top of red-candidate commit `84a5abde8a772f031cb1f1620a2bc3af144ba285`. The exact focused test passed 6/6, whole CLI passed 172/172 across 12 harnesses, and authorized-host `make test` passed 1082 runtime tests plus 31 doctests. Post-correction working and full-baseline diff checks both passed. The prior red records describe the pre-correction commit and remain preserved in the execution log and capture record. This is CC-proposed evidence; the six M7.3 rows remain subject to CDC acceptance. A new one-path correction commit is pending final metadata and clean-tree verification.

## Final iteration06 handback

The exact `.split('\n')` correction is committed as `bfa2b3710269b8110a858227ad79202ffdd1d513` on top of `84a5abde8a772f031cb1f1620a2bc3af144ba285`; the commit changes only `multi_save_integration.rs` and has the two required parsed trailers in order. The exact focused test passed 6/6, whole CLI passed 172/172, and authorized-host `make test` passed 1082 runtime tests plus 31 doctests. Current working and full-baseline checks passed, and the final tree/index are clean. All attempts are in the existing capture record. CDC owns independent acceptance; all six M7.3 rows remain open pending that disposition.
