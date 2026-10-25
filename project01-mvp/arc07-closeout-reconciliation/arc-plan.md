# Arc07 — Closeout reconciliation

Version: 1.1. Created: 2026-10-25. Status: first three slices authorized; Slice01 active.

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

## Version history

- 1.0 (2026-10-25): Duncan requested three bounded initial slices after pausing
  Arc06; later cleanup planning and execution await the findings and joint decisions.

- 1.1 (2026-10-25): Recorded explicit three-slice Autonomous authorization, approved models, ownership, bounded preflight and endpoint; opened Slice01.
