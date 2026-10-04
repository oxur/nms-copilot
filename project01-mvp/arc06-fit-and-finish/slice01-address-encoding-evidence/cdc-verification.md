# CDC verification — address encoding evidence

**Current verdict: accepted after iteration01; all eight rows done.** The initial failed review below remains historical; the final review section governs the current candidate.

## Initial review, 2026-10-03

**Verdict: changes required; slice remains open.** Reviewer: CDC / Sofie, the planning/review context, separate from the CC author of commit `8a5352bc5a9da3b15a51835879b9742c7d4f96f9`. Two-Contributor Workflow; Expedited Mode remains active. Reviewed assignment: `cc-prompt.md`, against the original eight AE rows. Source: main `a800950ef3558fcf463517696c3c140061e7353d`, clean before/after. Planning: CC packet `8a5352b`, inspected directly against `3fcf3ce`.

This reviewer authored the original assignment and baseline preparation, but did not implement CC's evidence packet. Its new source checks and CLI replays independently review that packet. This does not independently accept the reviewer’s higher-scale closeout assembly.

## Reproduced evidence

[CDC receipts](artifacts/cdc-review-receipts.json) retain exact commands, exits, fixture digests, complete CLI results, the inspected source-search population, and commit scope. Fresh all-feature locked offline build passed (0.65 seconds), with the existing `proc-macro-error2` future-incompatibility warning. No tests were rerun; no source changed, and this is evidence review rather than a code fix.

Both public-path exports matched CC's complete JSON outputs. Discovery `0x212B0004B2446A` produced `2B0004B2446A`, differing from the independently supplied game-visit oracle `212B04B2446A`. The portal-shaped control produced `212B04B2446A`. That independently reproduces the current defect, not a generalized decoder or a supported 48-bit save encoding.

The caller CSV has 18 data rows. Independent source inspection confirms 10 production `from_packed` call expressions and six direct `to_galactic_address` calls, excluding definitions and in-file tests. The expanded search also finds three production `GalacticAddress::new` calls: save model, CLI voxel parser, and REPL voxel parser. The last two input paths are missing from the claimed complete map and read coverage. Search receipts retain definitions separately; raw hit counts are not call counts.

The pinned MBINCompiler and libNom.io clones are clean at the stated revisions. Read the complete expanded-address/base schema bodies and the cited libNom test section. They support field structure and base scalar width, not the unresolved packed layout. Inspection of the relevant reference search and bodies found no applicable packed decoder. No private witness was supplied to this review. No stronger base/nonzero-galaxy conclusion is accepted.

Inspected raw scalar serde and all save conversions, graph extraction, SystemId, model insertion, find result construction, watch snapshot conversion, and cache fields/freshness/extraction. Confirmed: signed integer cast; missing-expanded defaults; required scalar deserialization; forced galaxy zero; query glyphs from system address; and cache freshness without decoder version. Galaxy collisions remain a source-derived risk, not a live collision reproduced here. The current multi-galaxy test uses different coordinates/SSI and does not settle identical portal addresses in distinct galaxies.

## Finding R01 — omitted voxel conversion paths

`crates/nms-cli/src/convert.rs:79-128` and `crates/nms-copilot/src/dispatch.rs:496-533` are production address-input conversions reached by their respective convert commands. The CSV labels CLI inputs as portal/signal/raw and REPL inputs as portal/signal/GA alias, omitting voxel input and its constructor route. CC's read coverage stops at CLI line 85 and REPL line 500, before those complete functions.

This is a substantive compatibility omission. CLI validates X/Z in -2048..2047 and SSI at most 0xFFE before `GalacticAddress::new`; REPL accepts the Rust integer widths and calls the masking constructor without those checks. Three additional independent CLI probes verify a valid Chili voxel input succeeds, X=3000 exits 1, and SSI=4095 exits 1. REPL narrowing is established by source inspection; no new live REPL result is claimed. Planet narrowing and core conversion entry points must be accounted for in the corrected assumption map. Do not select a new policy or repair these functions in this slice.

## Per-row disposition

| Row | CDC disposition | Evidence / limit |
|---|---|---|
| AE-01 | open | Source fence and readback are substantially supported; complete reached input-function reading is missing for R01. Recover and record the bounded sections before the correction. |
| AE-02 | open | Direct packed-call reconciliation is sound, but universal production caller completeness fails R01. Map all voxel and reached constructor/parser routes, distinguish definitions/tests, and reconcile the expanded population. |
| AE-03 | done — reproduced | Source/schema observations and exact witness applicability checked. General DD.UA, base layout and galaxy source explicitly unresolved; that is permitted investigation output, not a semantic waiver. |
| AE-04 | done — reproduced | Fresh CLI failure/control outputs exactly match CC. Oracle provenance is the supplied independent game visit; no new game visit claimed. |
| AE-05 | open | Existing save/raw assumptions checked, but CLI/REPL voxel range/narrowing differences are absent. Complete the enumeration under R01; policy remains unresolved. |
| AE-06 | done — reproduced | Inspected misleading fixtures/test, graph/query/watch identity dependencies and cache invalidation sites. Representation impacts confirmed by source; potential galaxy collision and planet-glyph mismatch are not claimed as new dynamic witnesses. |
| AE-07 | open | Save-repair recommendation is bounded, but its completeness claim depends on the omitted public input routes. Correct the recommendation/held-policy list against the completed map; generalized format decisions still gate production repair. |
| AE-08 | done — reproduced | Exact nine-file CC diff stays within slice allowlist; original prompt unchanged; seven CC supporting artifacts accounted for. Clean source and identifier scan checked. Intake command errors preserved by CC attestation; not independently rerun as historical attempts. |

Four rows accepted, four open; no deferrals/no-ops and no rows removed. Accepted conclusions remain limited to investigation criteria. Future source/input drift requires affected reverification.

## Artifact and scope checks

The CC commit has the seven supporting files listed in its report, plus ledger and closing report; both required commit trailers appear exactly once. No main-source changes, dependency changes, fixture changes, private snapshots, or game writes. An independent scan of all eight current supporting artifacts (including CDC receipts) found no 15+-digit numeric account/owner values. The receipt is reviewer-produced evidence, separate from CC's inventory. Earlier prompts and the CC initial report are preserved.

## Bubble-up and plan decision

The supplied evidence slice can legitimately leave format questions unresolved, so the unknown generalized discovery/base encodings are not a failed acceptance criterion by themselves. R01 is a failure to complete the assigned caller/intake scope. Correct it inside the same slice with `cc-prompt-iteration01.md`; do not open or implement the dependent address repair.

No structural arc-plan amendment is authorized or needed for this correction. Add a routine progress entry recording review and the held gate. Discovery/base/galaxy format authority and any proposed architectural split remain future CDC design work under the original closeout route; they are not silently approved by this report. Arc/project ledger rows remain open. Higher-scale independent and Operator gates are unchanged.

## What worked

The public-path failure plus portal-shaped control exposes a defect concealed by current fixtures. Explicitly unresolved field-family rows prevent one location witness from becoming an unsupported general algorithm. Exact source/commit fences and a narrow evidence diff make the corrective pass small and allow unaffected reproduced evidence to be retained.

## Return and evidence retention

Execute the current numbered correction before returning for CDC review. Preserve this initial failed-review section. For AE-03/04/06/08, retain the evidence only while their source, inputs, criteria and applicability remain unchanged; document that assessment on return. Corrections to a shared Markdown file do not by themselves require rerunning every unaffected probe. Any altered claim or newly implicated dependency needs fresh evidence. Production fixes, arc/project closure, and new compatibility policy remain outside this assignment.

## Iteration01 independent review and final close — 2026-10-03

Reviewed CC commit `c94a2080d907e4d904066ddf0ff614513544837d` against current assignment `cc-prompt-iteration01.md`, its slice plan and eight original rows. Same CDC reviewer, separate from CC author; source main remains clean at `a800950ef3558fcf463517696c3c140061e7353d`. Exact five-file correction inspected against `8f56eba`; both commit trailers verified once. Initial issued prompt, numbered prompt and prior independent receipts unchanged. Supporting inventory expands only by the reviewer’s [iteration01 receipts](artifacts/cdc-iteration01-receipts.json).

### Fresh checks and retained evidence

Read the complete correction delta, complete voxel parsers and resolvers, reached core constructor/From/FromStr/portal helpers and expanded save conversion. Re-ran all expanded search expressions and classified production bodies versus test/definition hits. CSV has 20 data rows. Three production `GalacticAddress::new` calls and two voxel entry calls are accounted for. Definition-only core helper routes are classified in execution.md, rather than invented runtime callers. The 10 packed calls and six direct wrapper calls remain accounted for. Raw matched-line counts are in the new receipt and match the expanded queries.

One nonblocking counting correction: execution.md says the six direct `to_galactic_address` calls accompany two definitions; the actual population contains **three definitions** (expanded save object, packed save wrapper, and core PortalAddress), nine raw matches total. The six call expressions and their classification are correct. This reviewer record supplies the exact reconciliation; no production route is missing and no criterion is weakened.

Independently repeated the three CLI probes: Chili voxel input exits 0 with `212B04B2446A`; X=3000 exits 1 with X-range error; SSI=4095 exits 1 with system-index-range error. Complete stdout/stderr/exits retained. REPL primitive parse followed by masking is source-confirmed; no live REPL result claimed. No production test suite, broad coverage or build rerun was required by this unchanged-source evidence correction; No new build is claimed; the earlier independent build retains its original date.

Prior evidence for AE-03/04/06/08 remains applicable: exact source revision, criteria and relevant fixture/reference/implementation inputs unchanged; the correction adds public voxel classification and does not alter game-format witnesses, export behavior, downstream identity/cache code or accepted limits. Compared old exact-value outputs and reviewer receipts directly with `8f56eba`. Checked the new five-file scope and current sanitation afresh for AE-08; the old scan is not presented as covering later edits. No new finding invalidates the prior method; no gate requires a fresh unchanged discovery/control replay here. Earlier evidence remains dated as initial review.

### Final row walk

| Row | Final status | Independent basis |
|---|---|---|
| AE-01 | done | Complete bounded intake and pre-edit readback now include both voxel bodies; checked against actual source and current assignment. |
| AE-02 | done | Reconciled expanded searches, 20 CSV rows, constructor/parser definitions and complete reached production input routes. R01 resolved; definition-count correction explicitly recorded above. |
| AE-03 | done | Retained initial independently checked witness/unresolved-family evidence; applicability assessed above. |
| AE-04 | done | Retained initial exact CLI failure/control, input digests and oracle provenance; no claim of a fresh game observation. |
| AE-05 | done | Verified primitive limits, CLI range checks, REPL/core masks, planet narrowing and separate galaxy propagation; new counterexamples checked. Unsupported game/sentinel meanings remain explicit. |
| AE-06 | done | Retained checked fixtures and graph/query/watch/cache effects; unaffected source/inputs and method. |
| AE-07 | done | Revised bounded recommendation includes voxel routes and holds harmonization and other unresolved policies; checked against completed map. |
| AE-08 | done | Fresh correction diff/source status, CSV/identifier checks and inventories; preserved previous outputs/prompt history and accepted historical scope. |

Eight opening rows, eight final done rows; no removal, deferral or no-op. Evidence reaches reproduced through current or explicitly retained independent checks. This closes the investigation, **not the address defects** or any general format/policy decision.

### Bubble-up and next work

The slice delivered its assigned current format/caller inventory and exact failure/control, with unknown generalized DD.UA/base/galaxy interpretation allowed by its original criteria. No silent drop: production fixes, nonzero live witness, base/discovery equivalence and policy decisions were never represented as delivered. The earlier failure and correction remain visible.

Update the arc before advancing: record slice01 accepted and scope slice02 to primary decoder/field-family evidence and bounded location witnesses under the already authorized evidence-before-repair route. This is continuation of investigation, not an architectural split or scope amendment. No production repair is opened while its semantic prerequisites remain unresolved. Original milestone and composition rows, independent higher-scale gate, coverage target and Operator project acceptance stay open.

What worked: concrete voxel counterexamples fixed the completeness gap without rebuilding unrelated evidence. Exact unchanged-input comparisons support proportionate retention. A pinned persistent-base reference lead supplies a concrete route for the remaining research, rather than treating a missing witness as a desired semantic answer.
