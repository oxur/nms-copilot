# Slice10 — usable REPL galaxy selection and reference defaults

Version:1.3. Created2026-10-16. Status:closed; all seven RG rows accepted at e615f254 with the approved bounded RG-07 historical exception.

## Useful increment and authority

Implement the original M7.4 REPL galaxy command and the already planned selected-view/reference behavior. A traveller can inspect/switch galaxies, set a base reference, then search or route consistently, with relevant help, completion, status and prompt. Approved parent R06-R02/R03 identity composition and the existing Slice10 owner govern; no new investigation is authorized or necessary. Framework2.7.2 usable-increment and investigation rules now apply in the existing two-contributor workflow: separate CC implements, Sofie/CDC independently reviews; Expedited Mode continues.

Actual prerequisite is the independently accepted movement/session/query/routing candidate e9cabd5fe63f19e9fe2e6da840845315d2ee2d18, not whole Project01 closure. Planning prerequisite: Slice09 close dcd30c47a6611f3e40b2feef428e9e09c5ebe145. CC creates/reuses isolated .worktrees/galaxy-scoped-repl / codex/galaxy-scoped-repl from that exact source. Issuance is the introducing commit for cc-prompt.md, resolved at intake. Delivery promised here is a reviewable candidate with real production REPL wiring and documentation plus native component integration proof. Main remains6d183bde; CDC/Operator own later authorized integration and final-baseline drift/composition checks. A candidate is not an installed release.

## Scope and concrete contract

[cc-prompt.md](cc-prompt.md) supplies exact binding decisions/recipe/tests/gates. Eight allowed source paths: commands/dispatch/session/completer/main/prompt and dispatch_integration.rs under nms-copilot, plus README.md (scopes in prompt). Add Galaxy action; migrate dispatcher to mutable model and actual read-lock caller to bounded write lock; switch only view selection, preserve physical player and explicit anchors; correctly distinguish PlayerPosition from explicit Base/Address for find/route; update current status/help/completion/docs and tests. No new set-position address syntax or query/core/cache/runtime transport API.

Preserve successful find/route/show/stats/convert/set/reset/map/exit behavior, all256 reference data, qualified0/9 names, explicit independent active selection, physical MCP location, full player-state preservation, filter/warp defaults and watcher anchor behavior. Public dispatch now requires &mut GalaxyModel; this is the bounded mutable-command consequence already identified by the parent, all in-tree callers migrated and rustdoc updated; no unrelated API/framework/version/dependency work.

Out: real terminal-driver observation, filesystem watcher/MCP transports/subscriptions/full startup-cache/unknown-system and old inconsistent cached-state disposition remain Slice11/CDC before parent acceptance. Native parser→dispatcher→formatter/prompt/completer tests plus actual production caller/build establish this bounded command integration; they are not a live PTY/wire claim. Later raw-field policy/repair and original export/import/format/docs/distribution/coverage/independent/Operator gates remain. No source merge/push/publish/parent closure.

## Verification, artifacts and readiness

Seven [ledger](ledger.md) rows start open; no close-set files precreated. Native tests use colliding0/9 fixtures with opposite distance/name outcomes, command errors/empty7/indices0 and255/names and quoted spaces, exact player state and anchors after every relevant transition. Keep useful prior tests and readable REPL README examples. Supporting artifacts default to artifacts/: immutable author-baseline.json pins source observations; CC creates execution.md/validation.json and later closing-report.md. CDC creates review records only during review.

Author readiness: loaded current complete prompt-authoring guide, usable-increment/investigation/contributor rules, and actual commands/parser, relevant dispatcher bodies/resolver/help and all callers, complete prompt and integration tests, completer production helpers, session synchronization/status, query reference/error contracts, model selection APIs and README REPL section. Version-sensitive clap/reedline APIs match existing source; signature and lock transition are explicit. Applied Rust CLI-05/06 (MUST) to clap optional positional; AP-12/29 (AVOID) to one shared mutable model and typed references; testing Proportionate Proof to actual native paths and existing Cargo gates. Public error/feature boundaries and stronger literal oracles are binding in prompt; no unresolved architecture delegated. No new Rust probe performed or claimed. Approx40–55k reading/source/test packet leaves correction headroom; supporting receipts serve delivery, not a new verification product.

## Assignment history

Current assignment: none; Slice10 closed after independent review and explicit Operator historical-evidence approval. Last executed assignment: arc06-fit-and-finish/slice10-galaxy-scoped-repl/cc-prompt-iteration01.md. Initial CC implementation cc-prompt.md issued2026-10-16 after accepted Slice09; predecessor closed09, reason original M7.4 command/reference delivery, disposition returned and independently reviewed2026-10-16; superseded as active by iteration01. Five-refinement budget; preserve every issued prompt. No automatic extension or investigation.

## Version history

1.0 (2026-10-16): detail existing next useful command/reference increment from accepted09, including production wiring, user docs, preserved behavior, bounded native proof and honest later integration ownership.


Initial issuance receipt:c45dd92b5303a3501ea9dc50e60f832cd28126d0, direct parentdcd30c4; exact nine planning paths and both trailers once. Actual commit/status captures and prompt SHA256 are in author-baseline.json. This is author readiness/commit evidence only, not implementation acceptance.

## Independent review and refinement01 — 2026-10-16

Version1.1: CDC accepts RG-01/02/03/04/06 at0346b193; RG-05/07 remain open per cdc-verification.md. All native gates independently pass724/215/920; supplemental reference checks40/40. Public completion loses the typed multi-word value ending in space (two failing observations); current evidence inventory/environment semantics require clarification. No scope, architecture, criterion or sequence change.

Iteration01 issued2026-10-16; exact path arc06-fit-and-finish/slice10-galaxy-scoped-repl/cc-prompt-iteration01.md, predecessor cc-prompt.md, reason F01 completion repair and F02 current evidence clarity, disposition returned by CC and independently reviewed; implementation accepted, historical environment decision pending. Resolve issuance from the commit introducing that immutable prompt; record actual intake HEAD. First refinement in the five-refinement budget, not an extension. Source direct parent0346b193; only completer.rs complete_galaxy_context and directly relevant cfg(test) may change. Four CC return paths remain execution.md/validation.json/closing-report.md/ledger.md, with only RG-05/07 Evidence/Notes editable in ledger. Five accepted rows held byte-identical. Original JSON values/history and report/execution prefixes preserved. No new investigation, waiver, merge, parent acceptance or installed release. Existing Slice11/later owners and every original gate remain.

Author readiness: inspected actual pinned helper and quote/span/name helpers plus native tests, parser, relevant dispatcher/session/reference/caller paths, all changed production source and README scope. Loaded current framework/PM iteration mechanics, prompt-authoring correction rules, work-verification retention, Rust and testing guidance. Applied AP-09/12/29 (AVOID) by retaining defensive span bounds, existing typed shared paths and the bounded helper, testing behavioral/proportionate proof through actual Suggestion.span reconstruction then parse_line; existing helper/API verified from compiled public probes. Repair cause and concrete raw-tail/span recipe are resolved before handoff. Witnesses are bounded review proof, not a maintained tool. Current prompt reading packet is smaller than initial implementation and leaves room for correction evidence.

## Iteration01 review and pending historical-evidence decision — 2026-10-16

Version1.2: F01 repair at e615f254 independently accepted; RG-05 done. Five accepted rows/evidence retained under exact helper/tests-only delta and fresh3/726/217/922 tests/all gates. Six rows done, RG-07 open solely for original1–53 environment metadata. Current correction intake/gates/source/commits/history independently verified; issuance pointer corrected by CDC to73a238c8 (c32dc420 is actual intake/receipt).

Iteration01 issued2026-10-16, predecessor cc-prompt.md, reason F01/F02 correction, returned at e615f254 with planning584c4ba9, independently reviewed2026-10-16. First refinement executed; no iteration02 issued because no further source repair is needed. Operator owns pending proposal in cdc-verification.md: permit only the unrecoverable historical environment metadata while retaining unknowns and original bytes, relying on fully captured fresh independent candidate proof. Proposal not adopted; no criterion or parent gate changed. Current state awaits that explicit decision before closure/advance. All original scope, Slice11/later owners, held source/main fences and parent counts remain. No inquiry, source integration or release authorized by this update.

## Final closure — 2026-10-16

Version1.3: Operator approved the exact RG-07 original1–53 environment-metadata exception (“I approve and give the waiver :-D”). Unknowns/bytes preserved, all seven rows done at e615f254; final row walk/artifact inventory/silent-drop/bubble-up in cdc-verification.md and Operator/CDC closing-report addendum. Iteration01 disposition accepted/closed after one refinement, not pending. No current/future or parent waiver/source integration. Actual current candidate proof retained; next planned live/lifecycle increment belongs to Slice11, with every original obligation retained.
