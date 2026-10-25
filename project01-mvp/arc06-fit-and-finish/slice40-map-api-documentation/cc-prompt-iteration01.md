# Slice40 first actual refinement — complete initial documentation findings

2026-10-25 UTC. FIRST ACTUAL refinement1/5; four remain. Initial prompt/plan/readback/initial patch/raw attempts preserved. Same sole CC /root/cdc_arc06/cc_map_docs gpt-6-luna High; CDC /root/cdc_arc06 gpt-6.1-sol High; original Arc06-only Autonomous authority. No new model/reviewer/API/scenario/policy/exception.

Initial actual offline format5eceeb519f904c72bb2b85cc61b5d700 and lintdeac9d72105a4508b2d9290b010904c9 both exit0, complete raw channels/end/status/equal source identityc7cfc502f00b5b7cf65aa28f464202705dda89294f0b0dea52b9c2f14c5ec49f/diff269666eceeb95d61f8a1b45e03ca794c801d71c6184aaf78a6657825afe1aa65 directly verified. No runtime/doctest/strictdocs/auditor/diff gate released or run. Findings below are STATIC documentation defects/omissions, not invented compiler failures or production defects.

Complete current four-file public documentation/production/new E1–E4 assertions and whole changed diff walked; full renderer tail recovered. Exact initial21546-byte patch/head/status/individual-file hashes are DATA in artifacts/initial-held-diff.json, not gate credit. Source remains HEAD00cfa740/indexempty, exactly4modified paths. Baseline file hashes for each guarded edit are in that capsule. Every write still requires resolved dedicated root + exact current allowlist + expected actual bytes/count, explicitcwd/absolute target.

## Frozen work / exact correction scope

Only existing Rustdoc/adjacent blanks in the original4mapfiles may change in the groups below. ALL production/API/attributes/ordinarycomments/tests/dependencies/layout/otherpaths/helpers/kernel freeze stays binding. All existing E1 constructor/state/header/complete label vector, E2 geometry/zoom/zero-grid assertions, E3 complete zoom/density populations, and E4 entire renderer code/assertions are SOUND and FROZEN. Reordering a whole existing example section is allowed below; no code/assertion mutation, new example, forcepanic, model/test fixture or runtime scenario. Entry no_run code may move unchanged, with the missing closing fence added. Scope remains EXACT4sourcepaths.

## F1 real examples and canonical section order

Current mod.rs module opens a no_run fence but never explicitly closes it; run_map's item docs have Errors→Panics→Examples, and the final Examples contains only prose referring to an example elsewhere. Current render item docs likewise put Panics BEFORE Examples. Original prompt/Rust DC04 explicitly require Examples→Errors→Panics; DC06's illustrative ordering does not override this binding order.

Recommended concrete repair: preserve module teaching/navigation and link run_map; move the COMPLETE existing terminal entry code/prose from the module to run_map's item # Examples, convert whole-line doc prefix as needed, add its explicit closing triple-backtick line. Keep its exact model/session/run_map/?/io::Result content and **no_run** directive, with one copy only. Put this complete Examples section BEFORE existing complete Errors and Panics. No loss of initialization/cleanup/masking/unguarded-panic teaching or new terminal calls.

Move render's COMPLETE existing Examples/prose/code section BEFORE its existing complete Panics section. E4 remains EXECUTING, boxed-error main unchanged, exact full player-cell tuple and whole legend row unchanged. Do not mark it no_run, weaken the row/padding predicate or alter positions/colors/expected values. No incidental code formatting/layout edits.

## F2 actual renderer dimensions, overlays and failure propagation

Current render module says to resize state “to the usable map area”; this is wrong for the actual MapState::resize API, which receives OUTER terminal/frame width and height and subtracts two columns/five rows itself. Current state.resize summary “Set usable dimensions” is also ambiguous about inputs. State plainly that callers supply complete terminal/frame dimensions (for E4,80×24), and the method derives usable grid; preserve saturation/zero-axis cursor behavior. Do not tell callers to subtract reservations twice. Render uses frame inner dimensions for binning and state.grid_size separately for cursor/status math, so keep them synchronized; there is no validation/automatic state resize inside render.

Complete the original W4 prose that is absent from initial docs, without changing code or adding a scenario: map/status/legend layout, help last; zero map-inner width or height skips map contents; state A–Z labels are yellow, Local zoom adds names, density is cyan/darkgray, cursor reversed, player@green overwrites a base cell, legend uses stored labels/player presence. Collision spreading down adjacent rows applies at ALL zoom levels; only adding names is Local-specific. A last-row collision can overwrite an earlier base—no unconditional separation guarantee. Source uses string byte lengths for label clipping/skipping, not Unicode display-width; existing UTF8 byte-boundary panic teaching remains, no fix/Unicode safety promise/theme feature.

Author precision correction: original W4 phrasing “Local adds names/spreads collisions downward” could associate spreading with Local; actual while-loop precedes the zoom-dependent label selection, so explicitly separate them as above. This does not change product policy or add proof scope; preserve original issued text and record the qualification.

handle_input has # Errors but omits its applicable forwarded numeric panic boundary: a movement key calls MapState::move_cursor, whose intermediate i16 addition can overflow with checks enabled for extreme public cursor/delta values. Add concrete # Panics/link to that actual method after Errors, not a general new error policy/forcedpanic/input test. Existing help precedence, key event-kind behavior, resize semantics and concrete IO errors stay complete.

## F3 state qualifications under existing public-field contract

MapState::new documents case-sensitive sorted names but omits the issued no-tie-order promise: equal names can retain unspecified HashMap-derived order. Add that qualification; keep26/A–Z/snapshot/filter/model-v-session/player/position teaching and E1 exact. No duplicate policy change/new fixture.

cursor_voxel says origin is center minus half viewport extent on each axis without qualifying zero-sized axes. For a positive dimension that is correct. With dimension0, divisor uses1 but half dimension is0: formula origin is center on that axis, and an independently retained nonzero cursor can still offset the result. State this real formula/limit; don't claim zero-grid cursor always equals center or add a test/expected value. viewport_bounds correctly documents collapsed zero-axis bounds and remains intact.

Author precision correction: original W3's origin shorthand likewise needed the positive-dimension qualification despite its separate zero-dimension discussion. Record it as author imprecision, not solely a CC invention. Preserve the stronger E2 zero-grid cursor/None controls and all current complete examples.

## Execute / hold / applicability

Read original prompt's unchanged capture/ownership/return/gates together with this complete scoped correction. Brief coherent F1–F3 readback, then CDC acknowledgement before edits. Correct only named Rustdoc groups → captured offline makeformat THENmakelint → IMMEDIATE source/index/gates HOLD. No postpair owned edits or runtime until targeted full corrected-doc/example review/release. Current4CC records remain exact prefix; all actual failed guards/compiler/validator attempts retained, no direct duplicate diagnostics.

On release original NINE groups apply unchanged: all-feature doctests FIRST then no-default, actual require_escalated host library/originalTMPDIR, actual STRICT zero docs (-D warnings -D missing_docs; no residual/exception), four-file ordered non-doc/opposedextent auditor, both diffs. Fresh CDC originalnine on finalcleancommit; no fullworkspace/newnative or extra discretionary proof. Exact4namedpath normal-hook localcommit/parsedonceconsecutive trailers/raw stored-body bytes([10,10])/sixMP+3bubbleup complete return/independent acceptance+retro+bottom-up. All6MP OPEN pending actual proof. Required41/fullM7.6/authenticNomNom/alloriginalhigher owners retained; no main cleanup/integration/push/install/release/otherarc/Project02. FIRST actual1/5/fourleft, no budget reset.
