# Slice09 — galaxy-scoped live session state

Version: 1.0. Created 2026-10-11. Status: active; seven rows open.

## Goal and authority

Complete the next bounded portion of the approved R06-R02/R03 identity composition: a typed live movement must update the physical player state, model galaxy selection, REPL session and location consumers coherently. The original M7.4 contract defaults selection to the player's galaxy while permitting independent selection; cross-galaxy live movement resumes the destination view, same-galaxy movement retains the selected view. CDC settles this bounded consumer policy within the already approved multi-galaxy repair; it does not establish raw save-field authority.

[cc-prompt.md](cc-prompt.md) contains the binding behavior, source-level recipe, literal tests, required reading, gates and return boundaries. All seven [ledger](ledger.md) rows require independent CDC acceptance. Two-Contributor Workflow resumes: separate CC implements; Sofie/CDC reviews work she did not implement. Expedited Mode commits/advances ordinary authorized work without reducing proof or approval gates. Slice08's approved historical exception is confined to Slice08.

## Source and scope

Source baseline: accepted Slice08 f71d607e27e18d7266855a2708449f9d6b90c632. Planning intake: closure9d6e43e725de90737d2738fa7261ce9c35903a1c; issuance is the commit introducing cc-prompt.md, resolved and recorded at intake. CC creates/reuses a clean isolated .worktrees/galaxy-scoped-session / codex/galaxy-scoped-session from that source pin. Main6d183bde, routingf71d607, query7118a486, identityf57a38f and cachedecd8d6 remain clean/pinned; Project02 untouched.

In: shared model movement fields/selection; producer-to-consumer qualified additions and movement proof; session initialization/reset/live synchronization; explicit-reference preservation; qualified live notification names; MCP player-location shared builder and actual in-process tool/resource reads; real drain-to-cache write/read/rebuild observations. Six exact source paths and four CC return paths are enumerated in the prompt. No dependency or public signature change; a small SessionState method may be added as prescribed there.

Out with owners: Slice10 (CDC, after this acceptance) owns REPL galaxy command/index/name resolution, help/completion/parser/mutable dispatch, selected-view and reference defaults for find/route, status/prompt behavior and actual REPL command integration. Slice11 (CDC, after Slice10) owns actual filesystem watch additions/movement, MCP stdio/HTTP/subscription lifecycle, startup/cache C04 composition and legacy/current warm behavior, unknown-player-system placeholders/promotion and persisted inconsistent prior player state diagnosis/disposition. These follow-on units are roadmap owners, not issued assignments. After them, CDC owns field-specific DD.UA/base/nonzero-galaxy policy evidence and repair with its required compatibility transitions. All original export/import/format/docs/distribution/coverage and parent composition requirements remain open. No merge/main movement/push/release/parent closure is authorized here.

## Verification and artifacts

Native tests observe complete identities/state and independent literal names, including both galaxy directions, empty/absent player, deliberate selection different from physical location, same-packed-coordinate collisions and both insertion orders. In-process typed producer, channel drain and actual public MCP registry reads are integration evidence within those paths; they do not prove real filesystem events or wire subscriptions. Current cache extraction/write/read/rebuild preserves existing2/2 representation and stored meaning, with those exact limits. Historical malformed cached state is not retroactively corrected by this repair; Slice11 must diagnose and disposition it before parent acceptance.

Artifact home: artifacts/. Author-owned author-baseline.json is immutable source-inspection evidence, not a tested result. CC creates artifacts/execution.md and artifacts/validation.json, then closing-report.md at actual return; no prefilled review/close records. CDC creates cdc-verification.md only when independently reviewing. Scope/receipt checks support behavioral gates and do not replace them.

## Author readiness

Source observations and pin digests: artifacts/author-baseline.json. Source investigation loaded model construction/insertion/selection/apply_delta and existing delta tests; complete session/watch/resource modules; snapshot production shape, compute_delta and relevant test constructors; MCP location builder/handler/registry/response helpers; cache extract/write/read/rebuild and scoped nearest counterpart. No baseline Rust probe was executed; this is source-grounded author preparation. Recipe uses actual non-exhaustive constructors, no consuming ownership trap, and public registry APIs with released write guards before awaited reads.

Engineering-methods From Slice to Implementation Prompt read completely; PM scales/layout/top-down/closure/confirmation applied. Rust anti-patterns AP-02/12/18/29 support borrowed lookup, narrow player borrow, deliberate ownership only and canonical qualified SystemId; exact application is in the prompt. Required reading is bounded to the current consumer contract; unrelated source bodies/history are reference-only. Estimated20–30k reading/source/test packet leaves implementation and correction headroom. No unresolved decision is delegated to CC; contradictory evidence stops the affected work for CDC.

## Assignment history

Initial assignment: arc06-fit-and-finish/slice09-galaxy-scoped-session/cc-prompt.md; issued2026-10-11; predecessor: closed Slice08; reason: required identity consumer composition after routing; disposition: active CC implementation work. Current assignment is that path. Five-refinement budget applies; no automatic extension. Author review above establishes readiness, not implementation or independent acceptance.

## Version history

1.0 (2026-10-11): opened the next bounded movement/session consumer slice after formally closing Slice08; explicit follow-on owners retain all remaining scope and original gates.
