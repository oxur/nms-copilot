# CC Slice04 iteration01 — Correct control coverage and source warrants

Execute `arc06-fit-and-finish/slice04-address-repair-contract/cc-prompt-iteration01.md` in project01-mvp, return through Operator to CDC/Sofie. Two-Contributor Workflow; Expedited Mode. This refines the issued [cc-prompt.md](cc-prompt.md) after CDC review of74359a92a48669d5ab0368bb51351ac840f1b9fc. Current assignment must match slice-plan.md. This is evidence/decision-packet correction only; no production policy or repair.

## Fences, scope and retention

Source root `/Users/oubiwann/lab/oxur/nms-copilot`, clean main `a800950ef3558fcf463517696c3c140061e7353d`. Planning root `.worktrees/planning`, branch planning; record actual reviewer-handoff HEAD and status. Check both before/after; source/required-evidence drift blocks affected work, report it rather than substituting inputs. RP-01 is independently done and its ledger row must stay byte-identical. RP-02–RP-07 remain open. Do not change criteria or status cells.

Only the original six slice-relative paths may change/commit:

- artifacts/decision-packet.md
- artifacts/impact-map.csv
- artifacts/behavior-probes.json
- artifacts/execution.md
- closing-report.md
- ledger.md — open rows' Evidence/Notes only

Preserve author-baseline.json, issued prompts, plans, CDC verification/receipts, predecessor data and parent records. Preserve original execution.md and closing-report.md **as exact prefixes**, append explicitly superseding iteration sections. Current decision-packet.md may correct inaccurate prose with a dated correction note; original version stays sealed in74359a9. No raw save/private-home/account read, private identifiers/seed/Position values, decoded save dump, source/test/config/dependency edits, cache writes, installation, publication, game mutation or external messages. Synthetic templates and already committed evidence projections below are allowed. Temporary files/helpers remain in `/private/tmp`, outside deliverables. Required two commit trailers remain unchanged.

## Required reading and pre-edit intake

Read before affected edits; recover truncation and distinguish complete document reads from structured population checks. Append intake/source-cited readback to execution.md **before other output edits**, explaining R01–R03, historical-vs-current counts, game-vs-expanded oracles, dm vs name and available delta observables. Record exact extents, hashes/pins, gaps, failures and conditional applicability.

| Input | Required extent / purpose |
|---|---|
| This prompt, original cc-prompt.md, slice-plan.md, ledger.md, cdc-verification.md | Required-full; active assignment, constraints, all findings and seven-row walk |
| artifacts/decision-packet.md, artifacts/execution.md, closing-report.md | Required-full; locate every affected current claim, preserve historical prefixes |
| artifacts/author-baseline.json | Required-data: complete synthetic root/templates, full integration_controls including exact distinct_csv; all16 packed+13 expanded IDs/kinds/operations for retention checks; SHA2560c3d0208a79f23cd8ff44ea101ad5d4909b2c0921aef4435d8291b3d11e30bdf |
| artifacts/behavior-probes.json at74359a9 | Required-data: all51 full result objects, IDs/families/inputs and current metadata; retain all51 objects unchanged, not merely their counts/digests |
| artifacts/cdc-review-receipts.json | Required-data: summary, failed_checks, manual_findings, review_attempts and exact prescribed `import-distinct-prescribed` result plus build channels; other52-run bodies reference-only because unchanged claims have independent retention evidence |
| artifacts/impact-map.csv; Slice01 artifacts/caller-map.csv | Required-data: all28 current rows/all columns; all20 seed rows/all seven original columns, and S06 complete |
| ../slice03-decoded-location-witnesses/artifacts/field-assessment.md | Required-full; exact bounded44-base warrant and statement that no new game-confirmed witness was added |
| ../slice03-decoded-location-witnesses/cdc-verification.md | Required-section: governing banner, final Iteration02 verdict and retained applicability |
| ../slice02-packed-format-authority/artifacts/witnesses.json | Required-data: complete W-CHILI object and provenance, raw/expected values and limits; no new save read |
| ../../artifacts/closeout-2026-10-03/baseline-report.md and synthetic-address-save.json | Required-section: F01/F02 behavior findings; Required-data: complete DD/DM projections of both discovery records to verify synthetic DM.CN name witness; unrelated fixture fields reference-only |
| Main crates/nms-save/src/model.rs; convert.rs; nms-graph/src/extract.rs | Required-section: model.rs197–239 RawDiscoveryRecord/DiscoveryData, convert.rs7–43 full to_core_record, extract.rs104–181 full extract_systems, through complete closing bodies |
| Main crates/nms-core/src/delta.rs; nms-watch/src/{snapshot,delta}.rs | Required-section: delta.rs10–60 SaveDelta/PlayerMoved/helpers; snapshot.rs1–end from_save (~76), watcher delta.rs1–69 full compute_delta, necessary definitions; future observable feasibility |
| Parent arc-plan.md; project01-mvp/closure-preparation.md | Required-section: Current corrective preparation / bounded closeout sequence / Authority; Active assignment / Gates and authority |
| /Users/oubiwann/.codex/skills/engineering-methods/guides/07-implementation-prompt-authoring.md; work-verification/guides/05-independent-verification.md | Required-section: Investigation and semantic evidence assignments, Required reading and CC intake, Corrective iterations; Evidence During Correction Review |
| /Users/oubiwann/.codex/skills/rust-guidelines | Required-section: API-37/TD-03/AP-33/EH-04 sections named in initial prompt, apply only to affected proposal framing; no Rust edits |

Initial prompt constraints still govern privacy, source roles, unresolved policies, structural checks and commit method. Its original45+6 initial manifest is preserved; this iteration explicitly authorizes **one additional result** while retaining the substituted same-galaxy observation as a supplement. No unspecified investigation expansion.

## Required corrections and acceptance method

### R01 / RP-03, RP-07 — Complete the prescribed control without erasing observations

Build existing CLI at source root `cargo build --offline --locked -p nms-cli`. Use explicit root target/debug/nms. Retain original build object; append new top-level `iteration01_build` with exact argv/exit/stdout/stderr capture, and a `historical_build_note` that original build channel text was a summary (E01). A build dependency failure is a blocker, not product failure; retain actual warning. Never fabricate channels from CDC output.

Deep-copy immutable synthetic_save into a new temporary JSON file; write `integration_controls.distinct_csv` verbatim to temporary CSV, preserving galaxies0/9, names and glyphs. Run `nms import <csv> --save <explicit fixture> --source synthetic`. Add one unique result ID `import-distinct-prescribed` with exact rows, argv, exit, complete stderr, stdout projection and original stdout digest, consistent existing result shape. Current pinned expectation is exit0, stderr `Imported 2 systems (0 duplicates, 0 skipped)`; derive freshly, retain unexpected results and investigate only bounded cause. Import has no persisted model/winner observation.

All original51 result objects must equal the74359a9 objects **in full by ID**, including parser inputs, argv, stderr/projections/digests and original same-galaxy control. Keep unique IDs. Update current method/summary honestly to45 parser + six prescribed + one supplemental =52 records (27 accepted/18 rejected parser cases; derive exit totals). Add explicit ID classification arrays or equally precise metadata: six prescribed IDs include the three conversions, two collision orders and new prescribed import; old `import-distinct-ssi-same-galaxy` is supplemental. Original51-run claims in execution/closing prefixes are historical and superseded in appended sections. Do not rerun all45 unchanged cases merely because a shared JSON file gains one result; CDC retains independently reproduced claims if objects/source/baseline remain fixed.

### R02 / RP-02, RP-05, RP-06 — Repair provenance and real name path

Correct decision-packet evidence table, D01 oracle and future mapping: Chili is the prior game-confirmed witness inherited by Slice02 W-CHILI from Slice01, retained by Slice03; it has independently confirmed portal/components but no newly observed Slice03 expanded pairing.44 paired-base expanded oracles are separate one-input evidence. Cite resolvable evidence paths/IDs and retain their bounds. Do not demand a nonexistent expanded Chili pair as already supplied evidence; obtaining any new pair would be a separately held proof obligation.

Replace `record.name` and implied raw display-name field with actual RawDiscoveryRecord.dm / synthetic DM.CN witness. Trace actual conversions supplying None and graph extraction ignoring dm; cite complete source bodies and prior F02 fixture. Future lost-name test can use synthetic SolarSystem/Planet DM.CN records and exact known names through existing parser/extraction/query output when its separate assignment is authorized. General name representations, selection/conflict policy and production fixes remain unresolved. Correct every affected table/prose claim; do not hide the distinction in one footnote.

### R03 / RP-04, RP-05, RP-06 — Use observable watcher proof within existing scope

Correct D06 and impact-map S06 to available delta channels. Keep all20 inherited seed rows/all seven original columns unchanged. Current SaveDelta has new_systems/new_planets/player_moved/new_bases/modified_bases and no removal channel. Proposed future discriminating sequence: old snapshot includes same-coordinate galaxy0; new includes0+9 with equal planet index. After approved key migration, compute_delta must report newly added galaxy9 system/planet; reverse insertion roles/order and assert appropriate player_moved if that test is included. Current same-location identity collision must fail this intended proof. Static recipe only: no new runtime watcher acceptance claim.

Remove unconditional add/remove proof and unsupported current removed-key descriptions throughout packet/map; an optional removal API proposal must be explicitly unresolved, outside current implementation authority, with owner/re-entry. Do not invent an accessor/channel or quietly add scope to make the oracle executable. Keep cache/identity/query obligations that CDC did not invalidate; serialized/public representation and semantic invalidation policy are still choices.

## Return, validation and disposition

Append iteration01 sections to execution.md and closing-report.md identifying this exact assignment, fresh build/control results, E01 summary clarification, all corrections, every failed/blocked/unrun attempt, retention checks, and per-row RP-02–RP-07 CC proposed dispositions. Explain packet delivered versus requested, changed source claims versus new policy, and honest52-record coverage. Initial closing report is historical; its statement that all rows remain open is superseded by current RP-01 acceptance. Keep RP-01 ledger row byte-identical; other six remain open for CDC.

Validate JSON/CSV, all52 IDs/full old51-object preservation, full20-row inherited-column equality, corrected S06 and every affected source claim, local links/privacy/whitespace, exact six-path diff, all original execution/closing prefix bytes, baseline/source/predecessor/issued-prompt/CDC preservation and required trailers exactly once. A temporary checker may be used; not a deliverable. Record results only after actual execution. Structural checks are not independent acceptance. No source edit means no broad tests/format/lint required.

Commit exactly six allowed paths using git commit --only on planning; no unrelated staging, push or release. Use one contiguous trailer block:

    Co-authored-by: Codex <noreply@openai.com>
    Co-authored-by: Billo AI <ai-engineering@billo.systems>

Return iteration identity/commit, fresh one prescribed-control result plus retained45+5+1 coverage, eight corrected recommended/unresolved topics, six open row proposals, failures/limits and clean/pinned states. CDC independently reviews changes and decides retained applicability. Slice04/Arc06/Project01 remain open until their own gates; policy adoption and implementation remain held.
