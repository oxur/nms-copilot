# Arc07 — Closeout reconciliation

Version: 1.13. Created: 2026-10-25. Status: finite cleanup underway;04–06 reviewed with explicit remaining gates. Initial three retained as accepted. Arc/project acceptance remains open.

## Purpose and authority

Finish the interrupted map check, assess the accumulated Project01 work, and give
Duncan a concrete keep/simplify/revert recommendation before deciding the remaining
closeout work. This is the bounded arc requested on 2026-10-25, not a change to the
collaboration framework. Arc06 stays paused and unclosed.

Initially plan **three slices only**. After their findings and recommendations are
presented, **stop execution and further arc planning for discussion with Duncan**.
Only after that discussion will this arc be amended with the agreed cleanup scope
and remaining slice sketches. There is no automatic Slice04 or restart of Arc06.

## Baseline and scope

- Accepted working candidate: `fb2ecebb9b1238c2c4b3d4c790f935f02fb2a6cc`, in
  `.worktrees/workspace-coverage-contract`.
- Current main: `b616ab2cfa9c67dfb6361bb1355ca9e6680d18cc`; its nine-line unused
  `HttpChild::wait_for_stderr` addition remains untouched pending the decision.
- Planning baseline: `6f824923af925d49cc1e36cf094e8560d9d62ea7`, in the existing
  orphan `planning` worktree.
- Assess Arc06 Slices01–48 and the unissued Slice49 draft, their source changes,
  relevant original Project01 promises, and present integration state. Include
  dependencies and changes already upstream so recommendations address actual work.
- Recheck these mutable facts before execution. Do not rebase or alter the source
  stack merely to begin the assessment.

Existing acceptance records are evidence to reuse, not instructions to repeat
every historical gate. Original project criteria and unresolved failures remain
visible; none is waived by this plan. In particular, the recorded 93.718068%
workspace coverage is below the existing 95% target. Chasing that target is not an
initial-phase task. Conclusions about this limited phase cannot close Project01.

## Autonomous execution authorization and current preflight

Duncan approved on 2026-10-25: “Perfect! Go ahead and do these in Autonomous Mode, and then lets confer for slices 4 and later :-)”. Authority ends after the first three reviewed results; execution AND further planning then pause. CDC `/root/cdc_arc07` is requested/applied GPT 6.1 Sol High, CC is requested GPT 6 Luna High, through Codex/OpenAI tools; backend identity beyond reported tool configuration is not asserted. Two contributors: CC executes and authors results; CDC independently inspects/reproduces and records `cdc-verification.md`. Root transports status and guards scope only. No additional reviewer context.

Preflight: planning `4d498081200e2faee0bd069f7ff41110f2bf2a99`, source clean `fb2ecebb9b1238c2c4b3d4c790f935f02fb2a6cc` on `codex/workspace-coverage-contract`; main `b616ab2cfa9c67dfb6361bb1355ca9e6680d18cc` and its known helper untouched. Old Arc06 Slice49 is untracked/unissued and protected. Source mutation belongs only to CC in the dedicated source checkout, with explicit handoff before CDC checks; planning index has one writer at a time (CDC preparation/review, CC assigned-result return). Python3 PTY/ioctl and existing binary/fixture are available; native socket/PTY permission follows tools if required. No external action or pipeline is authorized. Existing guard/input/render proof is retained at accepted source; actual owned-PTY observation addresses the missing usability/restoration boundary. Full95/platform/packaging/higher acceptance gates stay open.

## Initial slices

| Slice | Deliverable | Depends on |
|---|---|---|
| 01 — Finish the interrupted map check | A working-map result with a short statement of what was observed and any remaining defect | Accepted Slice48 source; existing owned-terminal tooling |
| 02 — Assess the accumulated work | One inventory covering every completed slice and the unissued draft, with source-grounded findings grouped by capability | Slice01 result and existing source/review history |
| 03 — Recommend what survives | A concise decision report proposing keep, simplify, revert, or defer, with dependencies and consequences | Slice02 inventory/findings |

### Slice01 — Finish the map check proportionately

Use an isolated fixture and owned PTY to exercise real REPL map entry, movement,
zoom, help, resize, map exit and continued REPL use. Check terminal restoration at
an observable boundary; distinguish Reedline's own raw mode from map cleanup and
compare terminal modes again after normal process exit. Reuse the accepted guard
and input tests for their already-proven error paths.

Use existing terminal facilities and retain the actual result/output with a short
observation note. Do not build a terminal emulator, general capture framework,
coverage-profile pipeline, or exact-cell golden-screen suite. A small direct
public-entry probe is permitted only if the REPL obscures the restoration boundary.
Do not touch game saves or the user's foreground terminal.

If the check finds a small map defect, repair and check it within this slice. If
it exposes a larger design question or cannot be observed with available tools,
record the failed check and a concrete unresolved finding for Slices02–03; do not
turn it into another research programme. Format and lint precede tests when Rust
changes; run affected tests and the native smoke check. A meaningful runtime change
also needs the existing workspace gate once on the resulting candidate. Retain
applicable previous proof instead of rerunning unrelated gates for record changes.

The old Arc06 Slice49 draft is unissued background, not this assignment. Its full
original D0042/coverage obligations remain recorded for the later scope decision;
this smoke check does not claim to discharge them all.

### Slice02 — Assess all accumulated work

Make one complete pass over the 49 slice entries and their actual source deltas,
using existing reports/tests to guide source inspection. A compact inventory must
identify each entry's purpose, implementation/evidence commits, changed areas,
integration state, useful outcome, maintenance cost, proof limits and proposed
disposition. Mark Slice49 as unissued with no implementation. Cover correctness,
user experience, APIs, dependencies, cache compatibility, tests, docs and supporting
tools; assess cross-cutting dependencies rather than treating slices independently.

Compare delivered behavior with the Project01 purpose and original requirements.
Distinguish valuable code from expensive execution: excessive effort does not by
itself make good documentation or tests worth deleting. Identify unnecessary
abstractions, duplication, brittle tests, dependency costs and unneeded runtime
policy only where source evidence supports the finding. Include the known stray
main helper in this assessment.

Budget: one complete inventory/review pass plus targeted checks needed to support
material findings. Reuse existing raw observations; do not recreate all historical
captures, rerun every slice suite, or open exploratory subprojects. Mark genuine
unknowns for discussion rather than silently extending the inquiry.

### Slice03 — Recommend keep, simplify, revert, or defer

Write a short recommendation report, backed by the single Slice02 inventory:

- **Keep:** concrete correctness, usability or maintainability value and its proof.
- **Simplify:** the unnecessary complexity, smallest replacement and behavior to preserve.
- **Revert/drop:** exact changes or unissued proposals, why removal helps, downstream
  dependencies, compatibility/data risks and the checks needed after removal.
- **Defer/discuss:** unresolved evidence or scope questions that need Duncan's decision.

Separate runtime changes, tests, shipped documentation and non-shipped supporting
tools. Preserve historical evidence and commits; recommending a source revert is
not permission to rewrite history or delete records. Consolidate decisions by
capability, with every slice mapped, rather than presenting 49 independent votes.
Identify actual release blockers and the choices needed to establish a finite
Project01 closeout. Do not prescribe a new release definition or silently waive
existing gates. Present the report and **pause**.

## Delivery, review and limits

Use the established planning layout and existing contributor/model selections
(CDC GPT 6.1 Sol High; CC GPT 6 Luna High) when execution is resumed. No new
contributor or independent-review layer is introduced. Prepare each slice's normal
open set only when it becomes the next assignment; no execution prompt is issued
by this planning change. Retain existing independent acceptance requirements and
use proportionate checks for changed behavior. Reports must distinguish observations,
inferences and recommendations; a review must check the underlying source/evidence.

Keep supporting output to the map result, one work inventory and one decision
report in their owning slices' `artifacts/` homes, using existing records by link.
No additional evidence system is a deliverable. No source revert, main cleanup,
merge, push, install, publish, framework change, Project02 work or additional slice
is authorized by the initial phase.

## Decision gate and later work

After Slice03, confer with Duncan about the 49 entries and the remaining original
requirements. Record those decisions here, update affected existing criteria with
explicit authority, then sketch only the remaining agreed cleanup slices in this
same arc. Project01 closes only after that cleanup and its applicable final gates;
neither this initial plan nor the recommendation report declares closure.

## Adopted decisions and remaining-obligation gate — 2026-10-25

After the initial handoff Duncan said “Agreed! Proceed! :-D” to the concrete
recommendations. The following decisions are adopted; the earlier three-slice
endpoint and its records remain historical facts:

- Use a fixed map snapshot and release the shared model read guard before modal
  interaction, allowing coordinator/cache/MCP progress. Live map refresh is later
  work, outside this correction.
- Preserve the original graph-consistency promise. Compare incremental state
  with a full rebuild through real consumers; if they differ, apply the smallest
  necessary repair. No routing redesign is selected.
- Keep the reviewed correctness layers, useful docs and discriminating oracles;
  simplify only duplicated test mechanics. Remove only the known stray main
  helper, preserving the committed/called candidate version. Drop old Arc06
  Slice49’s execution contract while preserving its draft bytes/history and
  every unresolved parent obligation.

**The next gate is this table’s disposition, before any remaining scope or
Slice04+ sketches/prompts.** Root promised the concrete finish/defer table before
resuming execution. Agreement with behavior recommendations does not approve a
specific deferral, numeric-target amendment, external action or an old backlog.
This follow-up authorizes only this additive plan record/table; no source work,
CC dispatch, tests/native reruns, install, rebase, push, merge or publishing.

Preflight for this record: planning began at `517fcb6919a10a4f2096527149eb2bd23ae64f8d`;
source remains clean `fb2ecebb9b1238c2c4b3d4c790f935f02fb2a6cc`; main is
`b616ab2cfa9c67dfb6361bb1355ca9e6680d18cc` with protected helper blob
`2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e`. Old49 remains untracked/unissued.
No new inquiry was undertaken. The table reuses the accepted
[49-entry inventory](slice02-work-inventory/artifacts/work-inventory.md),
[decision report](slice03-decision-report/artifacts/decision-report.md),
[Project01 criteria](../ledger.md), and [Arc06 criteria](../arc06-fit-and-finish/ledger.md).
All 26 project criteria and 12 Arc06 criteria retain their open dispositions.

“Finish” below is a recommendation for the subsequent approved scope, not work
issued now. **Both proposed deferrals require Duncan’s explicit adoption** with
the affected clauses recorded; until then those obligations remain open.

| Existing obligation | Owning rows | Recommendation and concrete remaining work | Boundary / owner and re-entry |
|---|---|---|---|
| Correctness and live composition | P01-C01–C05, R01–R03; R06-R01–R03, C-06 | **Finish before closure.** Implement the adopted snapshot/graph corrections; reconcile final address/name/galaxy, cache semantic/freshness/history and shared CLI/REPL/MCP behavior using accepted proof plus affected real-consumer checks. Correct atlas identity and current data are the product’s purpose. | CDC owns reconciliation. Missing genuine/game-backed/nonzero input is an explicit input/decision stop; synthetic proof does not silently replace it. No broader redesign or automatic new investigation. |
| Authentic NomNom format acceptance | M7.5; P01-A06, C07 | **Proposed deferral of only NomNom acceptance while an attributable full-save export is unavailable.** Keep the existing adapter and truthful support limits; do not claim format equivalence from code inspection. | Duncan owns supplying/authorizing the input; CDC owns reopening M7.5 when an attributable export/version is available. Then compare actual format and perform the bounded parser/interface check. |
| Existing UI promises | D0040–D0043, M7.10; P01-C06, A04, A06 | **Finish bounded reconciliation/verification** of existing banner/configuration, setup/history/completion, themes/table output and map behavior. Reuse native S01 and accepted guard/input/render proof; repair only concrete gaps against original clauses. This establishes usable existing interaction. | CDC owns the original-clause walk. A substantial unimplemented original feature stops for explicit disposition; it does not automatically expand the work. New polish/themes or live map refresh need later scope. No broad “all UI” deferral. |
| Final tests and coverage | M7.7; P01-G01, G03 | **Finish applicable gates; keep 95% and existing module requirements open.** Record the faithful final workspace denominator and module gaps after authorized changes. Existing result is 24,213/25,836 =93.7180678%. Retain all-command coverage obligations and meaningful regression oracles. | If the final result remains short, **stop for Duncan’s choice**: separately bounded behavior-driven hardening or an explicit criterion amendment. This table commissions neither unbounded hardening nor a lower/excluded denominator. |
| User/API docs and examples | M7.6 (accepted child scope); P01-G04 | **Finish final-candidate alignment**, strict docs/examples and package links/assets affected by the selected fixes. Reuse accepted S29–44 proof where unchanged; their cost is no reason to discard useful teaching. | CDC owns applicability and changed-contract checks. No wholesale documentation rewrite or replay of every historical capture. |
| Local package/install readiness | M7.8 readiness clauses; P01-G05 | **Finish before closure.** Supply missing license assets under the already declared terms, inspect archives/dependency order, build from a clean package boundary and smoke an isolated local install. These are concrete distribution-readiness promises. | Future scoped authorization must cover the local install; none is run by this record. Local readiness does not authorize registry publication or a push. |
| Publication provenance and a new release | M7.8 publication clauses; P01-G06, C07 | **Finish historical version/source/registry provenance; propose deferring new crates.io publication** to a separately authorized release. Distinguish demonstrated historical publication from the current candidate’s readiness; do not infer absence from blocked probes. | Duncan owns the release gate. Re-enter new publication only with a ready candidate, chosen version and explicit external-action authority, including required push/tag/pipeline effects. Deferring that original publication clause needs explicit adoption; readiness alone cannot close it. |
| Feature, MSRV and platform contract | M7.7; P01-G02, G07 | **Finish existing supported checks**: declared Rust 1.91 and default/all-feature/no-default behavior, plus exact-candidate evidence on declared OS/CI populations. Older default-only CI is insufficient. | CDC owns the matrix; unavailable toolchains/hosts become concrete blockers for Duncan’s choice. A push needed for hosted evidence has a separate external-action gate; this table gives no push/CI-effects permission. No blanket verification deferral. |
| Original performance promises | P01-G08 | **Finish the two stated measurements**: nearest 10 on 300 systems under 10 ms and cached startup under 100 ms, with toolchain/data/method and actual results. A small direct measurement is sufficient to assess the promises. | CDC owns the bounded measurement. A miss stops for a scoped repair/explicit criterion decision; no benchmark framework or optimization campaign is implied. |
| Scope reconciliation and final acceptance | P01-A01–A07, C07, G09; C-06 | **Finish the final row/leaf-scope walk and composed candidate demonstration**, retaining applicable accepted evidence, failures and explicit deferrals. Then obtain the prescribed independent and Operator acceptance. | CDC assembles; the existing independent/Operator gates remain separate. Child acceptance and this decision table do not close a project or waive a requirement. |

After Duncan dispositions the table, record the selected finish/defer clauses and
owners in their existing ledgers, amend only the agreed remaining Arc07 scope,
then sketch the necessary usable increments. **No Slice04+ directory or prompt is
created by this follow-up.** Execution remains stopped until that gate is met.

## Table adopted; finite cleanup authorization — 2026-10-25

Duncan’s “I agree!” explicitly adopted the two proposed deferrals and the rest of
the table. This supersedes the table-disposition hold above; the earlier pause
was observed and remains historical. M7.5 is deferred until an attributable
NomNom full-save export/version is available (Duncan input owner, CDC reopens the
bounded comparison). Only **new crates.io publication** is deferred until a ready
candidate, chosen version and Duncan’s explicit release/external-action authority.
Local package/isolated-install readiness and historical version/source/registry
provenance remain required; M7.8 and P01-G06 are mixed obligations and stay open.
All other table obligations remain in closeout scope. No 95% or module-floor
waiver was adopted.

The earlier fixed-snapshot map, original graph-consistency/minimal-repair,
keep/narrow test-mechanics consolidation, remove-only-stray-helper and drop-old49
execution decisions carry through this Arc07 cleanup. Standing Autonomous Mode
and approved CDC SolHigh / sole existing CC LunaHigh settings continue for this
finite endpoint. Source work remains on the existing dedicated non-default
candidate branch, without a preliminary rebase. Only restoration of the exact
known stray main edit is allowed on main, after blob/exact-diff guards; any drift
must be preserved. Old49 stays untracked, byte-preserved and never issued.

| Remaining increment | Usable result and boundary |
|---|---|
| 04 — Live atlas consistency | Fixed entry map snapshot releases the shared lock; bounded incremental/full-rebuild real-consumer comparison and smallest necessary graph repair. Preserve routing policy, existing API/terminal behavior and cache compatibility. |
| 05 — Narrow support cleanup | Consolidate only concrete duplicated test mechanics while retaining discriminating cases/oracles; restore only the guarded nine-line stray main edit. No new harness or cleanup campaign. |
| 06 — Existing interaction/docs and local distribution | Reconcile existing UI promises, align changed docs/assets, and establish isolated package/install readiness. A substantial missing original feature or unavailable package/input is an explicit affected gate, not permission for feature growth. |
| 07 — Final candidate validation | Existing feature/MSRV/platform, faithful coverage/module and two original performance checks. Reuse applicable accepted proof. Any coverage/performance shortfall or unavailable host/toolchain stops affected acceptance for a concrete human choice; no automatic hardening/optimization slice. |
| 08 — Scope/composition handoff | Reconcile original obligations and adopted deferrals; independently review the composed candidate and report completed capability plus genuine remaining gates. Preserve higher-scale independent and Operator acceptance; no automatic project success, integration or release. |

Detail only the next increment. This is a small finite sequence, not five blank
contracts to fill by momentum. Explain any necessary growth before opening more
work; do not silently add09+. Hold only missing input/criterion/external-action
proof while completing independent ready authorized work. No new process,
emulator/capture/benchmark framework, dependency install, routing redesign,
feature polish, Project02/new arc, threshold/exclusion weakening, or retrospective
rebuilding of49 evidence packets. Scratch install roots are authorized for local
verification; user-installed binaries/global environment are protected. Push,
merge/rebase/integration/tag, external CI/pipeline effects and publication still
need explicit separate authority after a concrete candidate is ready.

## Current cleanup disposition

Current cleanup candidate: source8f10d4cbe04f7d781667cc4e37280c333a67ee8d.
Slice04 AT-01/02 are independently accepted: owned fixed map releases model
lock, live MCP/cache progress continues, app exit0/terminal restoration and
literal application/rebuild graph parity are observed. AT-03 remains open:
reported whole-workspace247/7 was not green, individual historical failure
names/complete commands are unavailable, and no-default package cfg blockers
remain. Final07 owns the actual native full-candidate/feature commands; no
historical unknown is reconstructed. Independent ready05 work may proceed
under the adopted affected-proof hold, without calling04 fully complete.
Actual native runner failures remain separate from positive app observations.

Subsequent05 candidate8d572529445d80e404c1d45813c8b4a5b4b732fc shares only
duplicated HTTP test transport. Main's exact nine-line stray restoration is
independently accepted and main is clean atb616ab2. Live MCP24/24 independently
passes. NS-01/03 remain open: the owned startup integration fails its old
seed-only graph expectation after04's adopted ensure/rebuild behavior.07 must
align that literal oracle and rerun the actual target; no oracle was rewritten
in05 or failed target called green. Ready agreed06 work proceeds separately.

Slice06 source6b47cd3d758e6b44e28041ae52eceba0439c35db adds the bounded
existing-wizard fallback and truthful docs/portable license assets. Independent
startup4/history-component3 and strict affected rustdoc pass; all10archives,
licenses/current VCS and five historical0.2.0 checksum/source mappings checked.
ID-03 is accepted; wizard/config/historical subclaims are accepted. Native
completion/history/clean-exit composition is limited by actual retained probes,
not inferred from components. Theme disposition remains pending. Plain package
verification fails at the unpublished sibling hash; the archive-only core/save
check is partial and scratch install stops at uncachedclap_derive4.6.7. The
restricted full test remains247pass/7timeouts, not native success. ID-01/02/04/05
and R07-S06 remain open.07 proceeds with ready original validation/minimal
carried fixes; no further packaging/terminal investigation is commissioned.

Final07 candidate2b7fc9eba87529181ba609e52ac6a04a8ba6b1b0 is independently
reviewed: host lint/full1129runtime+119docs pass; true no-default package0 and
qualified existing generated-field-A docs apply.04 AT-03/05 NS-01,03 and06 ID-05
carries close;04/05 are complete. Coverage24466/26073=93.83653588002915% and14
below90 files remain RED. Installed1.91/Linux/Windows proof is unavailable.
Bounded nearest300/headless warm20-system timings pass on this debug host,
without a REPL/general-latency claim.06 native interaction/theme/install and
original genuine-save/higher acceptance gates remain. Source2b7fc9e's separated
co-author lines parse only Billo; preserve SHA/history and hold metadata handling
for later authorized integration. No further07 correction/source/measurement
work;08 is the finite scope/composition handoff, not automatic project success.

## Version history

- 1.0 (2026-10-25): Duncan requested three bounded initial slices after pausing
  Arc06; later cleanup planning and execution await the findings and joint decisions.

- 1.1 (2026-10-25): Recorded explicit three-slice Autonomous authorization, approved models, ownership, bounded preflight and endpoint; opened Slice01.

- 1.2 (2026-10-25): Slice01 independently accepted with actual owned-PTY map/continued-REPL and shell-boundary restoration observations; native evidence limits retained. Source unchanged. Open Slice02 single inventory pass; no structural scope change.

- 1.3 (2026-10-25): Slice02 complete49-entry inventory independently accepted. Original open criteria preserved; map-lock/no-edge/update/support-cost findings feed only03 recommendations. Open final initial-phase Slice03; no source cleanup or further slice authorized.

- 1.4 (2026-10-25): Slice03 decision report and close returned by CC, all49 mapped and original gates preserved. CDC review pending; no04 or later plan opened. After review, pause for Duncan's discussion.

- 1.5 (2026-10-25): Slice03 recommendations independently accepted; all49 mapped, three results composed into initial handoff and actual stop recorded. No source changes or criterion waivers. Keep/simplify/drop/defer remain recommendations; human joint decisions are required before any later scope or planning. CC metadata/trailer-correction deviation retained in Slice03 review.

- 1.6 (2026-10-25): Recorded Duncan’s adoption of fixed-snapshot map, original graph-consistency/minimal-repair and keep/narrow-cleanup/drop-old49 decisions. Added the requested existing-obligation finish/proposed-deferral/external-gate table; no deferral or numeric waiver adopted and no execution or04+ planning resumed.

- 1.7 (2026-10-25): Duncan adopted the table and its two narrow deferrals. Recorded mixed publishing/readiness obligations, continuing finite Autonomous authority and04–08 usable-increment sketch; only the next actual assignment will be detailed. Original numeric/input/external/acceptance gates remain.

- 1.8 (2026-10-25): Recorded04 independently reviewed map/graph behavior and explicit open AT-03 carry to final07; no whole-gate success, threshold waiver or extra scope. Ready agreed cleanup continues separately.

- 1.9 (2026-10-25): Recorded05 reviewed transport/main restoration and exact owned graph-oracle carry to07. No broad cleanup or validation narrowing;06 remains the next agreed increment to ground.

- 1.10 (2026-10-25):06 reviewed with assets/wizard/historical subclaims accepted and exact remaining native interaction/theme/install/full-gate limits; proceed ready07 without new inquiry.

- 1.11 (2026-10-25): Issued compact07 final validation contract at source6b47cd3d; minimal carried feature/oracle repair only, actual host gates/faithful95 and module90/two direct latencies, unavailable1.91/platform inputs explicit. No08 skeleton or new scope.

- 1.12 (2026-10-25): Independently reviewed07/final host1248 closes04/05 gate carries and06 regression carry; faithful coverage/platform/input/native/install/metadata qualifications remain. Only08 handoff remains; no further07 refinements.

- 1.13 (2026-10-25): Issued records-only08 finite capability/gate handoff on unchanged2b7fc9e; reuse accepted inventory/current07review, no new audit/probe/source work. Canonical endpoint pause follows independent handoff review.
