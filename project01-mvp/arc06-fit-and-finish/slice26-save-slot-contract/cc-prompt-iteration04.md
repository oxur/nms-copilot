# Slice26 iteration04 — actual internal-separator table shape

2026-10-22 UTC. Fourth of five ordinary refinements, same selected CDC gpt-6.1-sol High / sole existing CC gpt-6-luna High, Autonomous Arc06 only. Read retained initial+01–03+this immutable sibling. Held HEAD03b/composite0d65c203cc45c09a37e11082550eb262124f23b3ac26eae14896f939b79bd1da/indexempty. Actual nms-save24000f4f418940519202fd71010bb676 exit0:105unit+8address+17validated=130runtime and14docs. Actual focused6cf2cfe68d7248fd94ccfd9e753578ea exit101:2pass/4fail of6. Preserve raw stdout/stderr/exit/end/source and every prior failure; no later gates launched.

## Exact scope / source-grounded cause

ONLY `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-slot-contract/crates/nms-cli/tests/multi_save_integration.rs` may change, for the table-shape corrections below. ALL other FOUR files, executable production, docs, fixtures, names/addresses/selection policy, query/output APIs, cases, dependencies and old tests frozen. No new child/scenario population. Coherent readback then sole source/index/gates resumes. Every patch absolute dedicated path.

All four red tests share a test parser assumption: table_cells requires OUTER│bars, but actual NMS table uses INTERNAL│separators with no outer bars. Retained actual stdout and frozen `crates/nms-query/src/table.rs` show the shape; build_table adds title/header/data/footer rows, padding spaces, and summary rows. Production output is correct on this evidence. CDC held review missed the actual theme boundary; no product defect established, no weakened population or production format repair permitted.

## S1 — actual cells and target-only mutant writer

table_cells: a line with internal `│` is parsed by splitting on EVERY `│` and trimming EACH complete cell, including first and last. Do NOT require outerbars or discard first/last entries. A line with no `│` is None. Example actual ` 1 │ yes │ yes │ Auto (just now) ` => `[1,yes,yes,Auto (just now)]`; account headings/blank lines remain nonrows. Keep Unicode, valid-SGR-only cleaning and all full child output/stream/read-only checks.

replace_info_value must use the same unboxed raw line shape: `line.split('│')`, key in rawcells[0].trim(), replacement value in rawcells[1], preserve other raw cells and ALL other lines, then join with `│`. Current indices1/2 assume an outer empty cell. Preserve T3 `assert_only_table_cell_changed` and its FOUR calls using existing parsed key/value columns0/1 (find planetcolumn1), SAME checker negatives and original actual positives. Inspect ALL table_cells/table_rows consumers and retain their literal semantics; key_value/key_values and numeric-index find/show filters keep their existing correct columns.

## S2 — ALL rows with exact metadata, no marker filtering

With real parsing, titles and footers now appear. For assert_system_list validate and remove exactly the source-derived metadata rows, then compare ALL remaining first-column names with original sorted Vec/multiplicity (A Destination A+System A/B equivalents). Exact four-cell title `[SYSTEMS,"","",""]`, header `[Name,Discovered Planets,Address,Portal Glyphs]`, footer `[Total:,2 Systems,"",""]`. For assert_base_list exact five-cell title `[BASES,"","","",""]`, header `[Name,Type,Galaxy,Address,Portal Glyphs]`, footer `[Total:,1 Bases,"","",""]`; remaining ALL data names exactly Base A/B, row widths intact. No filtering known names, broad removal of unknown rows, missing metadata acceptance or blanket Total-prefix normalization. These are semantic cells from frozen build_table for this literal fixture, not ANSI/space/geometry goldens.

save_sections may keep ALL parsed nonempty title/header/data/footer rows (simplest bounded correction). Update o4_listing_matches's complete expected section vectors accordingly:
- DefaultUser: title `[SAVE,SLOTS,"",""]`, original Number/Manual/Auto/Most Recent header, unchanged slot1Auto/slot2Manual/slot15Manual rows, footer `[Total:,3 Slots,"",""]`.
- st_123: SAME title/header, unchanged manual-only slot1 row, footer `[Total:,1 Slots,"",""]` (actual source count_label is literally Slots, no grammar policy change).
- zz_empty: no table rows, existing exact header order and friendly empty display unchanged.

For fake30 listing assert complete18rows: title index0, originalheader1, fifteen exact data rows indices2..16 with slots1..15/yes/yes/Auto(just now), footer17 `[Total:,15 Slots,"",""]`. Preserve empty stderr/invalid31/255/256 negatives/count/multiplicity. No hidden extra account/row allowed.

## S3 — complete existing consumer/mutant walk

Preserve full info/stats exact keys, find `(planet,system)` populations, show exact mainName/numeric-index planet populations, route destination/opposite checks, export12fields/StringEuclid/multiplicity, exact import CSV stats/streams, all30 matrix children/three flagposition children, explicit no-discovery/errors/static bypass and whole tree bytes/mtime. All literal source assertions/previous80/30/-150/38 unchanged.

drop_default_slot15 and swap_slot1_recent must target actual parsed slot in first cell0 of DefaultUser only; same existing predicates must reject their effective mutants and unchanged unrelated-account assertions stay. No dropped valid negatives. Do not alter supplied expected caller values to satisfy shape bugs or normalize away unexpected populations. No new table/output library, controller, environment or terminal dependency.

## Required sequence / byte applicability

Complete ONLY S1–S3 and full affected consumer/assertion walk, captured make format→make lint, then IMMEDIATE source/index/helper/gate HOLD and exact IDs/identity. CDC targeted review before runtime. Existing130runtime+14docs nms-save proof remains attributable to unchanged locate/save bytes; no unnecessary rerun before corrected focused integration. After release run corrected focused integration FIRST, then original wholeCLI/fullmake/build/doctests/strictdocs/working+fullbaseline diff sequence. All failures preserved, final baseline includes named staged newtest, exactfivepathcommit/frozenreturns/CDCfresh11groups/retro unchanged. Fourthof5/ONE ordinary refinement remains; no further automatic pass beyond five or inherited waiver. Main human9line/allheldpins/no otherarc/integration/install/release authority protected.
