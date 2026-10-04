# CC iteration01 — complete address-input caller evidence

Execute this evidence-only correction and return to CDC. Project01 / arc06 / slice01; current assignment `arc06-fit-and-finish/slice01-address-encoding-evidence/cc-prompt-iteration01.md`. Predecessor: `cc-prompt.md` (preserved). Initial packet: `8a5352bc5a9da3b15a51835879b9742c7d4f96f9`. Source root `/Users/oubiwann/lab/oxur/nms-copilot`, main `a800950ef3558fcf463517696c3c140061e7353d`; planning root `.worktrees/planning`, branch planning. Confirm actual states and relevant drift before using retained evidence.

**Outcome:** resolve CDC finding R01 and rows AE-01, AE-02, AE-05, AE-07 by completing the public voxel/constructor caller and range-assumption evidence. No production repair or semantic policy decision. AE-03/04/06/08 are independently reproduced within their recorded limits; inspect applicability before retaining them. General packed DD.UA/base/galaxy semantics remain unresolved and block dependent repair.

## Required reading and preflight

| Order | Input / baseline | Scope | Purpose |
|---|---|---|---|
| 1 | This prompt, sibling slice-plan.md, ledger.md, cdc-verification.md | Required-full, current planning state | Current assignment, four unresolved rows, accepted evidence and R01 |
| 2 | Original cc-prompt.md §§Binding boundaries and applied guidance, Validation and return; arc-plan.md §§Workflow, Current corrective preparation | Required-section | Carry forward scope, privacy, gates and assignment authority |
| 3 | artifacts/caller-map.csv, format-contract.md, execution.md; initial closing-report.md | Required-full at CC commit 8a5352b, with current reviewer evidence separate | Correct completeness claims without erasing initial evidence |
| 4 | Source crates/nms-cli/src/convert.rs:6–168 and crates/nms-copilot/src/dispatch.rs:454–535 | Required-section, pinned main | Complete dispatch/input functions and all voxel validation/construction |
| 5 | Source crates/nms-core/src/address.rs:34–209,278–328,352–544; save model.rs:80–165 | Required-section, pinned main | Constructor masks, From/FromStr, portal/signal helpers and expanded conversion |
| 6 | Full production search population for constructor/input conversion routes | Required-data: all Rust sources under crates, exclude external test directories, classify in-file tests/definitions explicitly; query families below | Reconcile every production route and omission, not just from_packed calls |
| 7 | artifacts/cdc-review-receipts.json | Required-data: source/CC fences, every reconciled source hit, all five probes with inputs/results and limits | Review actual observations and retain unaffected exact-value evidence |
| 8 | /Users/oubiwann/.codex/skills/rust-guidelines/guides/02-api-design.md API-37; guides/11-anti-patterns.md AP-33; work-verification/guides/05-independent-verification.md §Evidence During Correction Review | Required-section with necessary definitions | Separate encoding roles and narrowing/error observations from new policy; retain evidence honestly |
| 9 | Other production functions/definitions reached by the expanded search | Conditional: load each complete matched body and necessary constants before classifying it | Prevent another constructor/helper blind spot |

Existing repository/skill workflow obligations apply; the manifest does not waive them. Recover truncation in bounded reads. Record actual read ranges, query populations/counts/exclusions and failures in an iteration-labelled section of `artifacts/execution.md`. Before editing evidence, write a brief source-cited readback explaining the CLI/REPL voxel difference, retained portal contract, allowed evidence writes, and still-held production policies. A missing/conflicting input blocks its dependent work.

## Source-grounded correction recipe

1. Read both complete voxel parsers. CLI `convert.rs:79–128` validates X/Z in -2048..2047 and SSI <=0xFFE, then calls `GalacticAddress::new`. REPL `dispatch.rs:496–533` parses i16/i8/i16 and u16/u8 inputs but supplies them to the same masking constructor without those CLI checks. These are observed differences, not an approved policy or permission to normalize behavior.
2. Expand the workspace query beyond `from_packed` and `to_galactic_address`: include `GalacticAddress::new`, `parse_voxel`, `from_signal_booster`, `from_portal_string`, `from_galactic_address`, `to_portal_address`, `parse_galactic_address`, and `GalacticAddress` From/FromStr implementations and typed parse/From usages. Follow actual production callers. Distinguish definitions, call expressions, tests, fixtures and downstream formatting; no invented automatic Rust parser is required. Retain exact search expressions/counts and reconcile every production hit against a CSV row or a clearly explained helper/definition classification.
3. Update `artifacts/caller-map.csv` to cover CLI and REPL voxel inputs, their constructor/galaxy/planet/range behavior, and any other reached omitted address conversion surface. The current 10 production from_packed and six direct to_galactic_address calls are independently checked; include newly reached families and do not confuse raw grep hits with call counts. Read every added function completely.
4. Correct the format contract's public-input and width/error assumptions and recommendation to reflect CLI/REPL voxel behavior. Inspect primitive-width parse limits, X/Z/SSI checks, and planet nibble masking independently. Keep core portal interpretation, save-field uncertainty, private-data limits and future owner decisions unchanged. Preserve initial observations in dated iteration-labelled prose rather than erasing a failed completeness claim without history.
5. Verify the changed claims with source-body inspection and existing public CLI checks. From source root, run `target/debug/nms convert --voxel 1130,4,-1244 --ssi 299 --planet 2` (exit0, portal 212B04B2446A); `--voxel 3000,0,0 --ssi 0 --planet 2` (exit1, X-range error); and `--voxel 0,0,0 --ssi 4095 --planet 2` (exit1, SSI-range error). Inspect and record actual results. REPL behavior may be established by the complete dispatch/parser/constructor source route; do not invent a live-terminal receipt. A disposable isolated public REPL check is optional only if it cannot touch personal config/history/cache/game data. No new production tests or harness product is requested.
6. Append an iteration01 section to `artifacts/execution.md` and `closing-report.md` identifying this prompt, the corrected evidence, all eight row dispositions, exact changed paths and artifact inventory. Preserve the initial report. Update only unresolved-row evidence notes in ledger.md; keep their statuses open pending CDC review. For accepted rows, cite cdc-verification.md and explain why unchanged inputs/source/criteria permit retention. Report any affected claim that requires fresh evidence instead.

## Applied guidance and proof limits

API-37 (SHOULD) keeps save-packed, portal and expanded/voxel inputs distinct in the evidence classification; it does not settle game encoding. AP-33 (AVOID) requires narrowing/error behavior and unresolved policy to remain explicit; do not turn this discrepancy into an unreviewed API change. Complete constructors plus range counterexamples reject the original incomplete caller claim. A passing CLI input or source-derived REPL result cannot prove generalized save encoding, galaxy sourcing, invalid-input policy, or higher-scale composition.

No dependency/source/game edits means no broad source test or coverage rerun is needed for these Markdown/CSV corrections. If the binary is absent or provenance uncertain, build from pinned source with `CARGO_NET_OFFLINE=true cargo build --workspace --all-features --locked`, record the exit and warning, and retain all failures. Existing no-default-feature and strict-doc failures stay unresolved. Inspect the evidence diff, CSV consistency, links, clean source state, and identifiers before committing.

## Scope, return and Expedited commit

Allowed exact existing files:

```text
project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence/artifacts/caller-map.csv
project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence/artifacts/format-contract.md
project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence/artifacts/execution.md
project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence/closing-report.md
project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence/ledger.md
```

Record extra probe output inside execution.md; no other file is needed. Do not edit issued prompts, slice/arc/project plans, the CDC record/receipts, baseline fixtures, source/tests/dependencies/config, raw/private saves, or game files. No full new game-format research is assigned by this correction. New material changes outside scope go to CDC. Scan allowed outputs for copied numeric owner/account identifiers. List exact files actually changed and commit those with `git commit --only` on planning; preserve unrelated changes. Both trailers exactly once:

```text
Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>
```

Return this assignment identity, commit, corrected claims/proofs, retained evidence and applicability, failures/unrun checks and remaining format/policy gates. CC remains proposed-done until CDC independently accepts the correction; neither arc nor project closes here.
