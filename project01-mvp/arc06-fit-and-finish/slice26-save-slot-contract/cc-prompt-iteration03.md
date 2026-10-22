# Slice26 iteration03 — exact remaining test corrections

2026-10-22 UTC, third of five ordinary refinements. Same selected CDC gpt-6.1-sol High / existing sole CC gpt-6-luna High, Autonomous Arc06 only. Read initial+01+02+this immutable sibling; all prior bytes/failures/controls remain. Held HEAD03b6536636f00a74a52f2d783a9ef80a671d191e/composite196e7730f2941f9fe69ac73816ae792e9ffb2f448f32e9d559e7796081f1eba8. Actual final format65974c5941a947f88e21358a38dfee5d/linta8ef38990e6e4b9b9a8531eac05195b4 both0/full raw/status/before==after verified. Earlier File-import/lifetime lint failures retained. No runtime yet.

CDC complete targeted F1–F6/full assertion-sequence walk is closed. Correct ONLY the following two test files, absolute dedicated paths:
- `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-slot-contract/crates/nms-cli/tests/multi_save_integration.rs`
- `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-slot-contract/crates/nms-save/src/locate.rs` TEST section only.
Other THREE files/main.rs/saves.rs/README and ALL executable production bodies/doc comments/dependencies/other tests/policies/scenarios frozen. No source fixture mutation or query/output change.

## T1 — correct only previous-address expected literals

New assert_save_literals previous-address assertions copied original CURRENT tuple into expected previous values. Actual unchanged `data/test/save56_minimal.json` PreviousUniverseAddress is Reality0, X80,Y30,Z-150,SSI38,P0; original current was X100,Y50,Z-200,SSI42. Change ONLY four previous expected literals 100→80,50→30,-200→-150,42→38 in assert_save_literals. Keep previous Reality0/Planet0, ALL current/base/raw record/owner/position assertions and cloned fixture bytes. This is a new test-oracle error, not a production/parser defect or author-supplied previous tuple.

## T2 — restore complete actual edge-group population assertion

locate test's new `actual_edges=(1..=15).map(find-or-none)` projection masks an erroneously returned empty group or repeated group. Before this projection assert `edge_slots.iter().map(|slot| slot.slot()).collect::<Vec<_>>() == [1,2,3,4,15]` EXACTLY, preserving multiplicity/order. This restores the earlier actual-group vector assertion with new Manual-onlyslot4 included. Keep the COMPLETE thirty-file literal tuple table/full fifteen grouping vector/all edge path/newest assertions and all invalid-newer negatives unchanged.

## T3 — complete original target-only observed mutant proof

02 now targets Units row correctly, but omits its explicitly issued assertion that unrelated observed lines/cells stay unchanged. Add a small TEST helper, e.g. `assert_only_table_cell_changed(before:&str,after:&str,row_key:&str,key_column:usize,changed_column:usize,expected_after:&str)` or equivalently local assertions. Compare full before/after `.lines()` vectors: equal line counts, exactly ONE different line; original target line identified from actual parsed cells/key; all other lines byte-equal; parsed cells of changed line equal the complete original cell vector with ONLY the intended cell replaced by the literal expected value. This may ignore final-LF normalization for the deliberately copied mutant only; actual child raw/whole-output assertions remain untouched.

Apply to each EXISTING actual mutant: info SaveName value (key Save Name,column0,valuecolumn1→Save B), info Units (key Units,column0,valuecolumn1→202), missing info key (original Save Name,column0→Other Name), and named-find planet (find actual numeric result-row using original Planet A in column1; changedcolumn1→Planet B). A helper accepting an explicit actual line index/predicate is equally valid and may be clearer for these different table shapes. Do not generate expected observed output or require production formatter/decoder. Preserve effective-change assertions and SAME existing checker rejection/actual positives, plus the existing duplicate-name checker control. No additional child/scenario population.

## T4 — restore dropped fake-listing negative assertions

In existing `test_saves_lists_all_thirty_canonical_fake_files_without_parsing_payloads`, restore original three valid negatives on the already-clean actual text: `!text.contains("31")`, `!text.contains("255")`, `!text.contains("256")`. Keep exact fifteen-row/yes/yes/Auto population/count and empty stderr. These were previously present and must not disappear during correction; no new case or formatting policy.

## Sequence / ownership

Send concise coherent readback then sole CC source/index/gates resumes for this exact two-test-file pass. Every patch absolute dedicated path, main human9line/allheldpins protected. Complete these FOUR corrections together, capture make format→make lint with full raw-first receipts/failures, then IMMEDIATE source/helper/index/gate HOLD and actual source identity/IDs. CDC specific delta review before original unchanged nine runtime/final groups. No discretionary proof expansion, weakening, further source edits after hold or acceptance selfclaim. Thirdof5/no reset/inherited exception; original exactfivepath delivery/returns/independent eleven groups/retro/bubble-up and all higher Arc06 gates remain.
