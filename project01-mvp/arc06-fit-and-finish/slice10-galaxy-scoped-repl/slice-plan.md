# Slice10 — usable REPL galaxy selection and reference defaults

Version:1.0. Created2026-10-16. Status:active; seven RG rows open.

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

Current assignment: arc06-fit-and-finish/slice10-galaxy-scoped-repl/cc-prompt.md. Initial CC implementation issued2026-10-16 after accepted Slice09; predecessor closed09, reason original M7.4 command/reference delivery, disposition active. Five-refinement budget; preserve every issued prompt. No automatic extension or investigation.

## Version history

1.0 (2026-10-16): detail existing next useful command/reference increment from accepted09, including production wiring, user docs, preserved behavior, bounded native proof and honest later integration ownership.


Initial issuance receipt:c45dd92b5303a3501ea9dc50e60f832cd28126d0, direct parentdcd30c4; exact nine planning paths and both trailers once. Actual commit/status captures and prompt SHA256 are in author-baseline.json. This is author readiness/commit evidence only, not implementation acceptance.
