# CC Iteration03 — correct the closed-route test setup and finish Slice08

Issued2026-10-11 by Sofie/CDC. Execute this refinement, then return through the Operator for independent review. Current assignment: `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration03.md`; predecessor `cc-prompt-iteration02.md` issued4c3d405f90152159b0f56232d6c348b512638dfa. This is refinement pass3, following Iteration02's interim stop. **SR-08 remains accepted; SR-01–07/SR-09 remain open.** No other row is accepted by this diagnostic review.

## Disposition and source-grounded correction

CDC reproduced the exact failing native test. Its `iteration02_route_query_oracles::query` helper in `crates/nms-query/src/route.rs` sets **warp_range:None**. The failing closed call passes that query unchanged. `execute_route` invokes `constrain_hops` only for Some(warp_range), as designed. Thus the observed800-ly return leg is an **unconstrained** route, not proof of failed400-ly constraint handling. Preserve the failure; append a superseding correction to the incorrect production-discrepancy/400-ly statements in the CC stop records. The test also selected B and C, not only C as one execution paragraph states.

CDC independently compiled the public query with ordinary typed100-Lush-start/B/C model, player0/active0 and colliding0 control. Both NN and2opt produce:

- `None`: IDs[A,B,C,A], flags[F,F,F,F], legs[0,400,400,800], cumulative[0,400,800,1600], total1600, targets2.
- `Some(400.0)`: IDs[A,B,C,B,A], flags[F,F,F,T,F], legs[0,400,400,400,400], cumulative[0,400,800,1200,1600], total1600, targets2.

The existing uncommitted typed rich-cache constrained closed regression also passes. No production repair is authorized or required by this failure. This disposition reconciles the fixture with the unchanged original contract; it does not relax waypoint expectations or adopt a strict maximum-hop guarantee for unbridged models.

In `planet_heavy_start_selects_two_unique_targets_with_literal_open_and_closed_routes`, construct the constrained query **at the callsite**, retaining the shared helper's None default for other tests:

```rust
let constrained = RouteQuery {
    warp_range: Some(400.0),
    ..query(target.clone(), a, algorithm, Some(2), true)
};
assert_eq!(constrained.warp_range, Some(400.0));
let closed = execute_route(&model, &constrained).unwrap();
```

This is a source-checked sketch, not a claimed compiled native patch. Keep the full existing five-hop constrained literal. Add the four-hop None control separately, for both algorithms, so omitting the option fails the constrained test and does not change unconstrained behavior. Assert configured inputs before calling the API; do not merely infer configuration from an expected result. Align the planet-heavy setup with the already stipulated player0/active0/colliding0 controls using ordinary inserted systems, and snapshot state around calls. Current stopped fixture instead supplies addressA9 in PlayerState and no0 collision systems. For the new pure-foreign truncation test, set **within_ly:Some(0.0)** at the callsite to exercise the stipulated radius-removal threat; the shared helper's None does not cover that case. Keep the maxNone105 test unconstrained (no warp) as prescribed. Use AGENTS test naming when editing/adding native tests.

Finish **all remaining Iteration02 native recipes, evidence corrections and ordered gates**, not only this one assertion. Its SR-01/02, SR-03/04/05, SR-06, SR-07, SR-08 retention/SR-09 and ordered capture/return sections remain binding, with this correction to actual test configuration. Do not weaken literals, silently replace dense/control populations or waive historical evidence. Runtime, errors, serialization and schema/semantic2/2 remain held unchanged.

## Continuation authority and exact boundaries

Repository `/Users/oubiwann/lab/oxur/nms-copilot`; candidate `.worktrees/galaxy-scoped-routing` / `codex/galaxy-scoped-routing` HEAD21494b963a5ffbfc5b283c5865a06fa3da096335. **Reuse the existing uncommitted four-file Iteration02 test work**, not a clean-reset/new candidate. It is authorized continuation input, fingerprinted in CDC receipts `iteration02_stop.audit.dirty_input_hashes`; source prefix audit confirms production unchanged. Source commit directly parents21494b9 and includes the preserved Iteration02 additions plus this test correction. No Iteration02 source commit exists.

Planning `.worktrees/planning` previously69ff089662eb081c7a23d2621053f599a0831af9 has four uncommitted CC records; CDC separately commits its own review/assignment paths. Preserve CC's four files byte-for-byte through intake, then append authorized corrections. Do not demand a clean planning/source tree, stash/delete the in-progress work, or include CDC files in the CC commit. Resolve current issuance via `git log -1 --format=%H -- project01-mvp/arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt-iteration03.md`, and record actual planning HEAD/parent at continuation. Dirty fingerprints identify stopped input; new edits legitimately supersede those hashes and are recorded separately.

Only four source paths: crates/nms-graph/src/query.rs and route.rs, crates/nms-query/src/route.rs (**test modules only**), crates/nms-copilot/tests/route_cache_integration.rs (tests/helpers). Preserve production prefixes against21494b9; all other source paths, CLI accepted file, edges/error files and genuine fixture remain immutable. Only four CC planning paths inside slice: artifacts/execution.md, artifacts/validation.json, closing-report.md, ledger.md Evidence/Notes only; retain SR-08's accepted row and all first six cells/eight-column format. CDC receipts/prompts/plans and parent records remain read-only to CC.

Main6d183bde79dd7328c32d4739dca249f5f1f9cd6c, cachedecd8d6c09c1199a6cdd01fd48aa50f3adef2b1a, foundationf57a38f7d1f00d1a3f5aa500ed5f7ce7d8c26eb8 and accepted-query7118a486ea43a776aa2451ec0807d43384fb954f stay clean/pinned. No subagents, production edit, dependency/fixture/cache change, merge/main movement/rebase/release/push/publication. Existing approval covers correction; proceed on coherent intake. If a **correctly configured** literal test exposes a production discrepancy, capture actual input/output and stop affected repair for CDC.

## Required reading and intake

Append current intake/readback in existing execution.md before dependent edits. Required-full: this prompt, current slice-plan.md/ledger.md, preserved Iteration02 prompt. Required-section: cdc-verification.md “Iteration02 interim-stop disposition”; current execution/closing Iteration02 intake/stop/addenda; source query route.rs `execute_route`, complete `iteration02_route_query_oracles` module, core RouteQuery definition; cache test typed_rich_projection/current_route_cache_matches_exact_open_and_closed_typed_projection. These resolve the discrepancy and input shapes. The remainder of Iteration02's bounded reading manifest still applies to finishing its work; retain reliable original intake with provenance, reload governing sections no longer available, and recover truncated required text. No retrospective original-reading claims.

Required-data: current validation.json full channel population (CDC now derives236, after later status receipt copies; earlier228 was an earlier snapshot), all pre-Iteration02 values retained, actual captured gate failures/stops/unrun04–09 and present limitations. Inspect current `iteration02_stop` CDC audit/summary plus full public-probe source/output and failure receipts; confirm dirty/source/fixture/parent identities before edits. Do not dump repeated successful workspace output to substitute for queries. Record query results/counts/exclusions separately from text reading. No fresh claim that all gates pass.

Relevant applied guidance: testing/guides/01-testing-discipline.md Failure Triage and Test Behavior And Contracts; engineering-methods/guides/07-implementation-prompt-authoring.md section5 and Corrective iterations; project-management README/iteration filename and workflow sections; work-verification/guides/05-independent-verification.md correction evidence. Resolve under the installed skill root from Iteration02. Read applicable Rust routing/type/CLI guidance as specified there if no longer reliably available. Here that guidance preserves typed qualified IDs and genuine integration proof; no API/lifecycle redesign is needed. Readback must distinguish supplied None from intended Some400, branch taken, literals for both modes, actual selected B/C, retained evidence and remaining owner gates.

## Evidence, gates and return

Preserve entire stopped execution/closing text as exact prefixes and every existing validation value; append Iteration03, including an explicit retraction of the failed-test production-bug interpretation. Keep original exit101/full channels and unknown historical gaps. New current input/gate identities are separate from historical evidence; no exceptions approved. New uncaptured inspection/record-write limitations from Iteration02 remain separately disclosed, not absorbed into old groups.

After correcting tests, rerun **all nine gates afresh in original order**, make format then lint then the prescribed five-package scoped all-feature suite, make test, offline locked workspace all-feature build, graph/query/cache doctests, ordinary four-crate docs, strict nms-cache docs with actual RUSTDOCFLAGS recorded, source/planning diff checks. Exact commands/env/feature matrix and failure handling are in Iteration02 “Ordered gates, capture and return”; no reduced gate set or inferred pass. Capture every command from the first attempt, child argv/cwd/overrides/exit and separate full base64/length/SHA channels before proceeding. Derive fresh totals; all historical warnings/limits/04–09 unrun remain past records.

Commit source and **only the four CC planning records** separately after completion, including preserved uncommitted Iteration02 work. Use named-path git commit --only and capture all attempts/receipt followups. Each assistant commit ends once with adjacent trailers:

Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>

Return all four initial/iteration assignment paths explicitly in closing report, source/planning full SHAs/parents/pathsets, actual counts/gates/limits, retained SR-08 and proposed-open eight rows, honest inventory/bubble-up and plain relative current prompt/report. No slice/arc/project closure. Required session/live/full startup then evidence-gated save repair and original closeout owners remain.

Author readiness: actual dirty helper/callsite and production branch inspected; exact native failure reproduced; existing configured cache regression passed; public NN/2opt None versus Some400 compiled/executed with literal projections and state assertions. Temporary probe first selected incompatible feature-build libraries; captured failure corrected using Cargo's matching compiler-artifact outputs, successful probe retained. An initial short exact test filter ran zero tests and is not proof; full name then reproduced failure. Dirty/prefix/236-channel/sealed controls audited. Recipes use inspected typed fields and struct update, leave production immutable, preserve current dirty work and have discriminating paired inputs. Original acceptance scope unchanged; current supplement adds bounded5–10k reading to reliable prior intake, or the existing30–45k packet on fresh context. Historical evidence disposition remains pending, no new approval barrier.
