# CC Slice04 iteration02 — Preserve exact build output

Execute `arc06-fit-and-finish/slice04-address-repair-contract/cc-prompt-iteration02.md` in project01-mvp, return through Operator to CDC/Sofie. Two-Contributor Workflow; Expedited Mode. Predecessor [iteration01](cc-prompt-iteration01.md) reviewed at CC `7d33b1d14f8d47f46a91b08965f8033b4bc60b1a`. This refinement addresses **RP-07 only**. RP-01–RP-06 are independently accepted and frozen. Verify this active assignment against slice-plan.md before acting.

## Fences and authorized changes

Source `/Users/oubiwann/lab/oxur/nms-copilot`, clean main `a800950ef3558fcf463517696c3c140061e7353d`. Planning `.worktrees/planning`, branch planning; record actual reviewer-handoff HEAD/status. Check pins/status before/after, report drift and stop dependent work rather than substitute inputs. Initial and iteration01 prompts/records remain preserved.

Only four slice-relative paths may change/commit, narrowed from the prior six:

- artifacts/behavior-probes.json — add `iteration02_build` and a dated `historical_iteration01_build_note` only; all existing values/objects, including52 results and both earlier build objects, unchanged
- artifacts/execution.md — append iteration02 intake/results; all prior bytes an exact prefix
- closing-report.md — append this assignment's RP-07 CC proposal/bubble-up; all prior bytes an exact prefix
- ledger.md — RP-07 Evidence/Notes only; status stays open; all six accepted rows and criterion cells byte-identical

Preserve decision-packet.md and impact-map.csv completely, baseline, issued prompts/plans, CDC review/receipts, predecessor data and parent records. No source/test/config/dependency edits, new parser/import/watcher/protocol/cache run, raw-save/private-home/account read, decoded dump, private identifiers/seed/Position values, installation/release/game mutation, external message or reusable validation product. Temporary capture files/helpers in `/private/tmp` are permitted. No policy is selected or implemented.

## Required reading and pre-edit intake

Read before dependent edits; recover truncation and record complete-read versus data-query coverage separately. Append intake to execution.md before other output edits, identifying active assignment, pins, remaining mismatch, accepted retention, actual stdout/stderr capture method and gaps/failures. A transcript generated from expected text is not a raw capture.

| Input | Required extent / purpose |
|---|---|
| This prompt, slice-plan.md, ledger.md | Required-full; one-row correction and frozen accepted work |
| cdc-verification.md | Required-section: governing current-verdict banner and complete Iteration01 review, including E01, row walk/retention/bubble-up |
| cc-prompt-iteration01.md | Required-section: Fences/scope/retention, R01 build-capture contract, Return/validation; original prompt privacy/policy constraints carried forward above |
| artifacts/behavior-probes.json at7d33b1d | Required-data: all top-level metadata/build objects, all52 complete result objects by ID for exact retention; no execution replay |
| artifacts/cdc-iteration01-review-receipts.json | Required-data: sole failed check with exact differing channels, fresh_build, limits/review_attempts; other checks reference-only for unchanged accepted work |
| artifacts/execution.md, closing-report.md | Required-full; preserve all previous bytes and explicitly supersede incorrect raw-capture claims as history |
| Parent arc-plan.md; project01-mvp/closure-preparation.md | Required-section: Slice04 review progress, Current bounded closeout sequence / Authority; Active assignment / Gates and authority |
| /Users/oubiwann/.codex/skills/work-verification/guides/05-independent-verification.md | Required-section: Evidence During Correction Review; raw evidence versus unchanged accepted claims |
| /Users/oubiwann/.codex/skills/engineering-methods/guides/07-implementation-prompt-authoring.md | Required-section: Corrective iterations and Required reading/CC intake; do not invent receipts |

No Rust implementation/proposal changes occur in this pass; earlier domain applications remain historical. No additional source bodies, game data, external decoder or dependency lookup are necessary.

## Concrete repair method and discriminating fidelity check

The iteration01 JSON and execution transcript omit `what` from Cargo's note (`to see the problems were` versus actual `to see what the problems were`). Fresh CDC build succeeds and confirms the warning. This is an exact-capture discrepancy, not a failed build. Preserve those objects/report sections as history and add a dated `historical_iteration01_build_note` explicitly superseding the earlier exact-capture label: the diagnostic differs from fresh raw output, earlier object remains historical, and iteration02 supplies current raw evidence. Do not hand-correct its text or present the old object as a verified original transcript.

Run once from the pinned source root:

    cargo build --offline --locked -p nms-cli

Use direct process capture with stdout/stderr retained separately as bytes (for example Python subprocess.run with capture_output=True); store temporary raw streams before constructing JSON. Serialize `iteration02_build` **directly from the returned process argv/exit and decoded captured bytes**, not copied or retyped Cargo text. Preserve whitespace, diagnostic wording and trailing newlines. Capture failures/environment-dependent warning changes honestly; expected build exit0 is a comparison, not permission to manufacture output. No import/parser replay is required.

Read the written JSON back and assert that encoded stdout/stderr equal the original captured stream bytes exactly, without whitespace/word/time normalization. Record each stream byte count and SHA-256 in execution.md along with argv/exit, warning presence and this round-trip equality result; a full manual quote is unnecessary. This rejects precisely the earlier missing-word failure. If decoding is not lossless, retain raw bytes temporarily and report the issue before calling the decoded fields exact. Do not create a reusable tool/test framework. No source edit means broad tests/format/lint are unnecessary.

## Retention, report and return checks

Compare against7d33b1d: all52 result objects equal in full by ID; original build and iteration01_build objects and existing metadata unchanged; exactly two added top-level keys. Current52 classifications/counts remain45+6+1 and parser27/18; preserve both accepted decision files byte-for-byte and all six accepted ledger rows. Preserve old execution/closing byte-prefix lengths and SHA-256. Record each failed/blocked/unrun attempt. Validate JSON, links/privacy/whitespace, exact four-path diff, pins/input/prompt/CDC preservation and trailers. A stronger fresh channel capture does not retroactively make historical mismatched text exact.

Append closing-report iteration02 identifying this exact prompt, fresh build/fidelity evidence, RP-07 CC proposed-done/open for CDC, six accepted rows retained and artifact inventory unchanged. Bubble-up: requested exact raw capture versus delivered; any discrepancy and scope diff; no policy/implementation/higher-scale acceptance. Ledger RP-07 remains open for review.

Commit exactly the four allowed paths with git commit --only on planning; preserve unrelated staging and no push/release. Both trailers once in a contiguous block:

    Co-authored-by: Codex <noreply@openai.com>
    Co-authored-by: Billo AI <ai-engineering@billo.systems>

Return assignment/commit, actual build exit/stream hashes/byte-equality checks,52-object and six-row retention, failures/limits and clean/pinned states. Do not report Slice04/Arc06/Project01 closed or any policy adopted. CDC independently reviews the new capture and decides final RP-07 acceptance.
