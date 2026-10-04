# Slice01 — Address encoding evidence

Version: 1.1. Status: initial CDC review requires correction; slice open. (Was: prepared, not executed.) Workflow: Two-Contributor; Expedited Mode active. Source baseline: main `a800950ef3558fcf463517696c3c140061e7353d`.

## Outcome and scope

Establish a source-backed format/caller contract for addresses already used by the MVP so a subsequent repair can correct F01/F10 without confusing save-packed addresses, portal addresses, expanded coordinates, and CLI inputs. This is evidence/investigation work. It authorizes no production-code, dependency, configuration, test-fixture, or game-save edits.

The output is `artifacts/format-contract.md`, supported by `artifacts/caller-map.csv`, sanitized witnesses/probe receipts, and `artifacts/execution.md`. Cover discovery `DD.UA`, persistent-base packed addresses, expanded player addresses, portal/signal-booster inputs, raw/core conversion, graph extraction, watch snapshots, and query/rendering paths. Existing potentially misleading 48-bit test fixtures and the truncation-accepting CLI test are part of the evidence, not automatic compatibility authority.

## Method and decisions

Inspect all actual callers, then compare each field family against pinned reference code/schema and available local observations. Record documented, observed, inferred, and unresolved interpretations separately. Hex length is insufficient to determine encoding: a zero planet/high system component can make a 56-bit layout numerically fit in 48 bits. Treat `0` and negative integer input semantics as unresolved until supported evidence and existing behavior are recorded.

Replay the prepared address fixture and portal control through the built CLI; keep the mismatch visible. Extract a nonzero-galaxy fixture only when its expected layout has independent support. For base fields, seek an independently expanded/teleport-address witness or a pinned reference decoding route before concluding they share discovery encoding. If evidence is unavailable, report the exact unresolved field family and block its dependent repair; do not force a desired answer.

Recommend the next repair's type/API boundary and affected tests, based on evidence. Preserve the intentional core 48-bit portal contract unless a separately reviewed API change is justified. Invalidate or re-key derived caches if decoded identity changes; determine where that would be wired. Source names/biome association, no-default HTTP, broad coverage and save-account redesign are other tracks, not scope expansion for this slice.

## Exit criteria

All rows in the [ledger](ledger.md) have evidence or explicit unresolved dispositions. The contract maps every caller and schema family, has independent exact-value witnesses and meaningful controls, identifies fixture/test assumptions, and gives a bounded subsequent repair recommendation. CC writes a per-row closing report; CDC independently reviews evidence and scope. A semantic uncertainty can be an honest investigation result but cannot become permission to implement that uncertain policy.

## Artifacts and commits

Supporting outputs live only in this slice's `artifacts/`. At execution end, CC creates `closing-report.md`; independent review creates `cdc-verification.md` when it occurs. Preserve the issued `cc-prompt.md`; follow-ups get numbered sibling prompts. Commit exact produced paths with both required authorship trailers. No main-branch commit is expected or permitted by this evidence assignment.

## Author readiness

Source-grounded: inspected raw/core conversions, graph extraction, all `from_packed`/wrapper call sites, and the public CLI failure/control. Design-complete for investigation: no unsupported format conclusion or API behavior is prescribed; downstream implementation is held until evidence resolves it. Guidance applied: Rust API-37 distinguishes encoding roles; AP-33 prevents unknown/error states from being reported as successful decoding. Executable/falsifiable: existing CLI and bounded queries supply evidence, with exact Chili glyphs and the portal-shaped control distinguishing the observed error. Coherent: prompt/ledger use the same families, source fence, and evidence-only scope. Required sections are bounded to leave headroom for caller/reference inspection; no full-workspace code audit is assigned. This is author self-review, not acceptance.

## Assignment history

- Initial assignment: `cc-prompt.md`, prepared 2026-10-03, subsequently executed by CC and returned at commit `8a5352bc5a9da3b15a51835879b9742c7d4f96f9`. Initial CDC review: four rows accepted, four unresolved under R01; initial attempt preserved.
- Current assignment: `cc-prompt-iteration01.md`, issued 2026-10-03; predecessor `cc-prompt.md`; active for Operator handoff, not yet executed. Correct omitted voxel/constructor input evidence inside the same slice. No source fix or structural scope change. Executed refinement passes: 0 (this is the first issued refinement).

## Iteration01 author readiness and version history

Read and inspected complete CLI/REPL voxel functions and reached core conversion boundaries at pinned main. The correction is source-grounded and design-complete for evidence only: map and describe the existing validation difference, without choosing a new policy. API-37/AP-33 preserve family distinctions and narrowing/error evidence. It is executable using bounded queries, complete source bodies, and three existing CLI probes independently exercised by CDC. Counterexamples expose the omitted paths; no new harness or runtime configuration is needed. The manifest is bounded to current evidence and reached functions, with room for corrections and retained-evidence review. Author self-check, not independent acceptance of CC’s next work.

- 1.1 (2026-10-03): initial CDC review recorded R01; updated assignment history and issued preserved iteration01 to complete the original intake/caller/assumption contract. Original eight rows and downstream production gates unchanged.
