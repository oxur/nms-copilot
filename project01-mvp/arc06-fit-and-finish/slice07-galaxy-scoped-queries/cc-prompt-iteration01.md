# CC Iteration01 — complete Slice07 regression proof and return

Issued 2026-10-10. Execute this bounded correction, then return to CDC/Sofie. Two-Contributor Workflow / Expedited Mode continue. Current source `b2d27017fafb7a85ad880b20faea31ad496e23a0` in `.worktrees/galaxy-scoped-queries`, branch `codex/galaxy-scoped-queries`; its parent remains accepted `f57a38f7d1f00d1a3f5aa500ed5f7ce7d8c26eb8`. Reuse this candidate, do not recreate from the old baseline. Planning review baseline was `91522bb6e02954516acd93d92b985dc0d4b7be10`; record actual issuance HEAD containing this prompt and review. Preserve main `a800950ef3558fcf463517696c3c140061e7353d`, foundation f57a38f and cache `decd8d6c09c1199a6cdd01fd48aa50f3adef2b1a` clean/pinned.

**Outcome:** finish QG-03's per-predicate/mode proof, QG-08's rendered pairing and new error docs, and QG-09's complete closing return. QG-01/02/04/05/06/07 are independently accepted at b2d2701; retain their rows unchanged. No runtime defect was reproduced in this pass. Production behavior, public API shape, error text, stored meaning and schema2/semantic2 remain binding and unchanged. No successor slice, save-field policy, main movement, merge, push or publication is authorized. The GI-08 historical exception applies to its original named call only.

## Required reading and intake

Before edits, load complete required content in bounded reads, recover truncation, and append dated intake/readback to `artifacts/execution.md`. Record assignment/source state, actual extents/conditional applicability, read gaps and their recovery. Explain why the conjunction alone does not fulfill the promised per-predicate matrix, why global name presence does not prove rendered pairings, and why accepted runtime/cache evidence can be retained while changed tests/docs require new checks. No permission loop after coherent preflight.

1. **Required-full:** this prompt, current sibling slice-plan.md, ledger.md and cdc-verification.md. **Required-full:** preserved cc-prompt.md as the underlying unchanged binding design/acceptance/gate contract, current closing-report.md and artifacts/execution.md. This iteration supersedes its create-worktree/source-baseline instructions and narrows permitted edits; it does not weaken its behavior or validation.
2. **Required-section:** parent arc-plan.md “Slice07 query composition assignment” and “Slice07 initial review and Iteration01”; project-plan.md “Slice07 query composition” and “Slice07 initial review and Iteration01”; closure-preparation.md “Gates and authority” and final Slice07 review section. These retain parent requirements and current sequencing.
3. **Required-full:** source AGENTS.md, installed collaboration-framework, project-management, work-verification, engineering-methods, testing and rust-guidelines SKILL.md entrypoints, following applicable routing; current root `/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/`. **Required-full:** project-management guides/README.md before planning, guides/04-closing-slices.md for inventory/three-answer bubble-up, engineering-methods guides/07-implementation-prompt-authoring.md, testing guides/01-testing-discipline.md and guides/03-validation-gates.md. **Required-section:** project-management guides/03-planning-top-down.md “Issuing and executing an iteration”; work-verification guides/01-ledger-discipline.md slice status/evidence rules, guides/05-independent-verification.md “Evidence during correction review”. Rust guides/11-anti-patterns.md AP-01–AP-10 first, guides/13-documentation.md DC-01–DC-04, complete pattern bodies. Other domain chapters conditional on actual affected mechanics; no runtime redesign.
4. **Required-full:** candidate crates/nms-query/src/find.rs, crates/nms-cli/tests/find_integration.rs and crates/nms-graph/src/error.rs. **Required-section:** core system.rs Planet/System constructors and definitions; player.rs PlayerState constructor; graph model.rs new/insert_system/player position/active selection; query display.rs format_find_results including column order and truncation. These are the public fixture/formatter route. Initial prompt's unchanged implementation recipe is the retained design, not an assignment to reopen all runtime code.
5. **Required-data:** all 33 existing artifacts/validation.json attempt records: argv/cwd/env/exits and both complete decoded channel length/hash checks; inspect complete diagnostics for failures and gate summaries/warnings. Reviewer artifacts/cdc-review-receipts.json: all nine gate exits/summaries/warnings, audit exit and complete decoded audit stdout/stderr, public probe compiler/run exits and complete probe output. Check all six current supporting artifact names and exact sealed/native predecessor SHA/360 bytes. This is declared structured coverage, not a full semantic read of base64 compiler boilerplate. All prior objects/metadata remain historical inputs.
6. **Conditional:** if source/contract drift, a changed runtime path or an additional necessary file appears, stop affected work for CDC. If sandbox denies a write, capture failure and use supported escalation before retry. **Reference-only:** older slice prompts/evidence outside the retained inputs; no historical instruction replaces this assignment.

## Binding correction recipe and literal oracles

### QG-03 — exercise every predicate in every mode

In `crates/nms-query/src/find.rs`, change only tests/helpers. Existing `test_find_composes_all_planet_filters_in_every_spatial_mode` tests five individual/conjunction cases unbounded, then only the conjunction through four modes. Move parameterization so **each** case executes `(within_ly, nearest)` equal to `(None,None)`, `(Some(10_000.0),None)`, `(None,Some(10))`, `(Some(10_000.0),Some(10))`.

Keep the same typed fixture and explicit galaxy0 origin, and exact ordered literal populations:

| Predicate | Expected names |
| --- | --- |
| Lush alone | Named, `<unnamed>`, Cool |
| LushHumid subtype alone | `<unnamed>`, Cool |
| infested Some(false) alone | `<unnamed>`, Cool |
| named_only alone | Named, Cool, Toxic |
| Lush AND LushHumid AND Some(false) AND named_only | Cool |
| Deliberate biome Lava no-match | empty |

Six cases times four modes give **24 exact population checks**; these are assertion cases, not 24 invented Rust tests. Keep Some(true) coverage elsewhere and existing name/discoverer/ranking tests. Preserve the named system/unnamed planet exclusion test; it can use the same four-mode loop. Expected literals come from the unchanged fixture, never another call's output. The radius and nearest limit include the entire matching population, so mode comparisons test predicates rather than incidental truncation. No runtime branch or algorithm change is permitted.

### QG-08 — rendered rows and public error contract

In `crates/nms-cli/tests/find_integration.rs`, retain existing parser pair assertions and actual cargo_bin_cmd infested inclusion/exclusion exactly. Add a small typed **public query → formatter** regression, since the parser fixture supplies no names. Construct a model with player/origin galaxy0, two distinct systems using existing public constructors, each with planet index1 and Lush:

- voxelX0/SSI1: system `Cedar`, planet `Bud-A`;
- voxelX1/SSI1: system `Maple`, planet `Bud-B`.

Execute the normal public find query, assert exactly two results and literal planet/system names/qualified identities (distances0/400 if checked), then call `format_find_results(..., Theme::none())`. For each literal planet name locate exactly one rendered data line; Bud-A's line must contain Cedar and exclude Maple, Bud-B's line must contain Maple and exclude Cedar. The four names are below existing column widths and share no identifying substring, avoiding truncation or false matches. Merely asserting all four names somewhere in the full table would let swapped rows pass. Reuse imports/helpers where appropriate; private test names/helpers are local discretion. Describe this as typed query/formatter proof, not parser name extraction or a second CLI process.

In `crates/nms-graph/src/error.rs`, add rustdoc to the new public ReferenceGalaxyMismatch variant and both fields: it represents current-position reference reality differing from selected active reality; `reference` is the player's address reality and `active` is the model's selected reality; explicit address/base references select their own reality instead. Preserve variant shape, field types and exact existing error display. DC-01–04 require scoped public summaries; avoid a crate-wide lint/docs cleanup or documentation of unrelated pre-existing items.

### QG-09 — complete the canonical return

Append a dated Iteration01 correction to execution.md and closing-report.md, preserving each entire original file as an exact prefix. Keep original observations/proposed statuses historical and explicitly state current verdict. Identify both assignment paths and exact new commits/parents/pathsets/trailers/pins, failed attempts, actual test totals and limits.

Inventory **actual** durable supporting artifacts under artifacts/ with ownership/purpose: the three sealed author inputs, CC execution.md/validation.json and CDC cdc-review-receipts.json. Root plans/ledger/prompts/closing/reviewer records are first-class, excluded. The unchanged binary copy under native source tests is a test fixture, not an extra planning artifact home. Create no new CC artifact files.

End the correction with **Bubble-up to the arc**, explicitly answering: (1) assigned query portion of R06-R02/R03 delivered versus remaining acceptance; (2) unforeseen structural findings/dependencies/deferrals, or a reasoned none; (3) specified-versus-delivered scope and every outstanding item. Preserve required routing/spatial/session/live/save-field owners. No arc structural change is presently indicated; CC reports findings and CDC decides, rather than changing parent plans or inferring closure.

In validation.json preserve every existing value and attempt object; append an `iteration01` object containing new captures and current source/planning fence metadata. Do not overwrite initial source_commit/previous captures or repin sealed inputs. A new source commit may change identity metadata **only inside the new iteration object**. In ledger.md change Evidence/Notes only on QG-03/08/09; their Status remains open. Six accepted rows remain byte-identical, including acceptance evidence.

## Exact write boundaries and validation

Source only: `crates/nms-query/src/find.rs` (tests/helpers only), `crates/nms-cli/tests/find_integration.rs` (tests/helpers/imports only), `crates/nms-graph/src/error.rs` (rustdoc only). New source commit parent must be b2d27017. No production logic/signature/schema/fixture/revision/dependency change. Planning CC only: artifacts/execution.md, artifacts/validation.json, closing-report.md, ledger.md under this slice. CDC-owned review, receipts, prompt, plans and all sealed artifacts are immutable to CC.

Because new native regression tests affect the validation claim, run **all initial prescribed gates** after the corrections with CARGO_NET_OFFLINE=true in the candidate: make format; make lint before tests; focused all-feature five-package tests; make test; offline/locked workspace all-feature build; graph/query/cache all-feature doctests; ordinary scoped core/graph/query/cache docs; strict cache broken-link docs; diff/fixture/path/parent/trailer/clean fences. Exact argv appear in cc-prompt.md “Validation, evidence and commits”; retain that complete section. Record actual denominators, zero-test commands, ignored/unrun/warnings and **every attempt's exact separate channels/direct exits** before invocation. Baseline warnings remain owned; no broad docs fix or coverage claim. Review retention of six accepted rows is subject to CDC's actual diff/applicability check, not CC self-acceptance.

Commit source's actual named permitted paths with git commit --only, then separately the four permitted planning paths; use the supported Git metadata permission path. Both required co-author lines exactly once, adjacent in the final trailer block:

```text
Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>
```

Audit source parent/actual pathsets/trailers, retained row/prefix/value identities, fixture SHA and clean candidate/planning/main/dependencies. Do not rewrite historical commits or issued prompts. Return full SHAs, actual validations/failures/limits, six accepted retained rows and three corrected rows proposed-done/open for CDC. No cross-chat message; Operator relays the packet.
