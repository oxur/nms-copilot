# Slice04 — CDC independent verification

2026-10-04. Reviewed CC commit `74359a92a48669d5ab0368bb51351ac840f1b9fc` against clean source main `a800950ef3558fcf463517696c3c140061e7353d`. **Correction required: RP-01 accepted; RP-02–RP-07 remain open.** Slice04, Arc06 and Project01 remain open. No production policy, API, identity or cache migration is adopted.

## Method and reproduced evidence

Read the issued assignment and delivered packet, reconciled the immutable manifest and earlier accepted witness provenance, and inspected pinned source contracts. Built the existing CLI offline/locked successfully, with the existing proc-macro-error2 future-compatibility warning. Used explicit temporary synthetic save paths throughout; no private save, user cache, auto-detection, live protocol or production edit.

[Fresh receipts](artifacts/cdc-review-receipts.json) retain all 52 fresh CLI process results: **45 parser cases + six prescribed controls + one supplemental control**. All 45 declared injection descriptors match the immutable manifest, including JSON kinds and removals. Exits reproduce **27 accepts /18 rejects**, with discovery9/7, base9/7 and expanded9/4. All recorded stdout digests/projections reproduce. Stderr comparisons normalize only JSON line/column locations because independently generated fixture formatting differs; fresh complete stderr remains captured. Parse success is current behavior, not validation of game semantics.

Three conversion controls reproduce raw-save-shaped `--ga` → `2B0004B2446A`, portal and portal-shaped controls → `212B04B2446A`. Collision imports in galaxies0/9 reproduce one addition/one duplicate in either order. Both the prescribed distinct-SSI galaxies0/9 input and CC's distinct-SSI same-galaxy supplement reproduce two additions/no duplicates. Import counts are stderr observations; no runtime stored-winner or persistence claim follows.

All20 seed rows preserve all seven inherited columns; eight supplemental rows exist. The CC commit has exactly its six allowed paths, both trailers once, clean whitespace, unchanged baseline/predecessor/source fences and unchanged criteria. These structural properties do not accept inaccurate source claims or a substituted control. Final automated receipt has563 checks:562 pass and one genuine manifest-control mismatch. Manual findings below are additional semantic review.

Reviewer failures are retained in receipt review_attempts: a helper initially treated padded seed labels01–09 as unpadded1–9 (nine false failures); a subsequent helper assumed an import stdout table instead of actual stderr counts (two false failures). Corrected checks use normalized labels and exact captured stderr, without rerunning unchanged CLI evidence. A batched reading result was truncated; necessary sections were recovered with bounded reads before this verdict. Packaging also initially selected the wrong original arc-ID prefix; corrected to exclude supplemental child IDs and verified all15 original rows remain open. No broad tests, lint/format, new save decode or live REPL/MCP/watch/cache checks were run; no code changed.

## R01 — Prescribed sixth control was replaced

The issued prompt's integration control6 names author-baseline.json `integration_controls.distinct_csv`, whose two rows are galaxies0 and9 with portal values212B04B2446A and212C04B2446A. CC instead records `import-distinct-ssi-same-galaxy`, with both rows in galaxy0 and different synthetic names. Execution.md acknowledges deliberate substitution, but only prose/order/helper names were discretionary. Consequently its claim of full51-run manifest coverage is false. The additional control is useful and reproduced, but cannot stand in for the named one.

Correction: retain all51 existing result objects as historical/reproduced evidence, add a fresh prescribed-control result using exact baseline CSV rows, and classify current coverage as45 parser + six prescribed + one supplemental =52. Preserve absence of a runtime winner observation. Do not erase the substituted run or pretend it was originally prescribed.

## R02 — Oracle provenance and name field are misstated

Decision packet line10 assigns Chili to Slice03's accepted one-save work; D01 and the future proof map claim independent expanded fields for Chili. Slice03's field-assessment explicitly says it adds no game-confirmed witness and retains Slice02's existing Chili evidence. Slice02 witnesses.json `W-CHILI` is prior player/game-confirmed evidence inherited from Slice01. Its portal/components are an independent earlier game oracle, **not an expanded pair newly found in the Slice03 input**. The44 paired-base expanded comparisons are separate bounded evidence. Correct the evidence table, D01 oracle and proof map with resolvable witness/provenance pointers; do not discard either useful witness family or imply a missing expanded Chili pairing exists.

D07 line103 says conversion drops `record.name`. Pinned `crates/nms-save/src/model.rs` RawDiscoveryRecord197–225 has `dd`, `dm`, `ows`, `fl`, `rid`, and no `name`. Its `dm` retains serde_json::Value metadata. The previously committed synthetic name-loss witness supplies `DM.CN` for SolarSystem and Planet records (project01-mvp/artifacts/closeout-2026-10-03/synthetic-address-save.json; baseline-report.md F02). `to_core_record`7–43 supplies None; `extract_systems`104–181 supplies None for system/planet names and does not use dm. Correct current source and future named-record test to this observed path; unknown name representations and general name policy remain unresolved in the separate lost-name track.

## R03 — Watcher removal proof has no current API

D06 promises distinct watcher add/remove keys, and impact-map S06 describes added/removed keys and insert/remove proof. Current `crates/nms-core/src/delta.rs` SaveDelta10–60 and `crates/nms-watch/src/delta.rs` compute_delta14–69 expose new systems/planets, player movement and new/modified bases. They expose no removed-system/planet channel. SaveSnapshot systems/planets are keyed by SystemId/(SystemId,index), so a feasible future identity regression uses old snapshot containing galaxy0 and new snapshot also containing same-coordinate galaxy9; assert a new galaxy9 system/planet addition after an approved identity change, plus reverse ordering and appropriate player movement if included. This distinguishes current collision behavior through an actual existing observable.

Replace unconditional removal proof with feasible addition/movement proof in D06, map S06 and affected future-test prose. Any removal feature can only be an explicitly unresolved proposal with separate authority/re-entry, not a hidden implementation obligation. No identity representation is selected here; no future production test is claimed executed.

## E01 — Build transcript strength needs clarification

CC JSON build stdout/stderr and execution prose read as captured channels, but contain summarized/reworded output. Fresh CDC capture has empty stdout and Cargo Finished/warning/note on stderr. This does not invalidate the independently successful build. Preserve the original report as history, label its transcript as a summary, and append one fresh exact build capture in iteration01. Do not silently replace historical channels or omit the warning.

## Ledger walk and retention

| Row | CDC disposition | Evidence / remaining work |
|---|---|---|
| RP-01 | done | Clean pinned source/assignment, required scoped intake/readback and preservation independently checked. Accepts recorded intake and fences, not every downstream claim. |
| RP-02 | open | R02: separate prior Chili game oracle from Slice03 expanded-base evidence; correct nonexistent raw name field. |
| RP-03 | open | All45 parser objects and five prescribed controls reproduced; retain them. R01: fresh named sixth control required in CC packet; supplement cannot replace it. |
| RP-04 | open |20/20 seed preservation and eight supplemental rows reproduced; retain. R03: S06 must describe existing delta capabilities faithfully. |
| RP-05 | open | All eight recommendations have owners and held adoption; retain unaffected content. Correct R02/R03 source premises and oracle feasibility. |
| RP-06 | open | Retain cache/identity/query discriminating obligations within their bounds; correct Chili/name provenance and watcher observable. |
| RP-07 | open | Six-path diff/privacy/criteria/trailers independently pass. Correct current manifest coverage and report walk; clarify E01 while preserving attempt history. |

## Bubble-up and next assignment

Evidence packet is substantially reproducible but not independently accepted as a whole. Findings are bounded documentation/control corrections inside the same slice, not changed original acceptance, architecture, repair sequence or compatibility policy. The held decision gate remains after packet acceptance. [Iteration01](cc-prompt-iteration01.md) names the remaining work and retained claims. RP-01 stays accepted; predecessor investigations remain closed within their recorded limits.

## Review/handoff structural checks

Final packaging check passes: exactly eight reviewer/handoff paths,46 resolvable local Markdown links, seven sealed input/artifact hashes unchanged, all seven slice criteria unchanged, one accepted/six open RP rows, all15 original arc rows and22 project rows still open, source clean/pinned and whitespace clean. The original CC artifacts/report and initial prompt were not edited. These checks support this bounded handoff; they do not accept the packet or production policy. Commit identity and post-commit worktree state are returned to the Operator after committing.
