# Slice08 closing report — proposed completion

## Assignment and source state

Assignment: `arc06-fit-and-finish/slice08-galaxy-scoped-routing/cc-prompt.md`.

Source implementation was performed on isolated branch `codex/galaxy-scoped-routing` from the accepted Slice07 baseline `7118a486ea43a776aa2451ec0807d43384fb954f`. The planning assignment was issued from planning HEAD `b792257db624d1a1c8edc35068c30481cf69648a` (parent `8aca066a1bf90e2eec29fd12b7bf8101e6740d29`). Main remained at `a800950ef3558fcf463517696c3c140061e7353d`; identity foundation `f57a38f7d1f00d1a3f5aa500ed5f7ce7d8c26eb8` and cache foundation `decd8d6c09c1199a6cdd01fd48aa50f3adef2b1a` remained separate. The source commit is `e1347bc24fafa5049f73854bede444cfa5fe2a8b`, directly parented by the required baseline. Planning commits contain only the four prompt-allowlisted planning paths.

## Outcome by ledger row

All rows are **proposed done for CDC review; status remains `open`**. No acceptance is inferred.

| Row | Proposed disposition and evidence |
|---|---|
| SR-01 | Proposed satisfied: active wrappers reject foreign addresses before distances; four explicit-address APIs use the selected galaxy without mutation; empty galaxy 7 returns empty. See graph query tests listed in `ledger.md`. |
| SR-02 | Proposed satisfied: complete scan, total distance order, qualified numeric key tie-break, planet-index association, inclusive 399/400 radius boundaries, invalid radius and zero/`usize::MAX` limits; collision/tie controls run in both insertion orders. |
| SR-03 | Proposed satisfied: absent IDs are typed before same-ID, distance, radius or limit work; foreign route targets return `NoPath { from, to }`; builders, 2-opt, and hop constraints validate IDs and galaxy membership. |
| SR-04 | Proposed satisfied: disconnected same-galaxy fallback is exactly 800 ly; injected cross-galaxy bridge edges are ignored by A*; KNN, warp-range and incremental edge composition stay galaxy-local. |
| SR-05 | Proposed satisfied: closed route retains the first/last system and exact 2,400-ly line distance; waypoint search advances beyond 60 decoys; reachability returns all 120 systems deterministically and excludes the injected foreign bridge endpoint. |
| SR-06 | Proposed satisfied: current-position mismatch is typed; explicit/base starts scope to their own galaxy without mutation; base-first foreign target handling, validation-before-radius/limit, 220-planet dedup, no implicit 100 cap, and round-trip count are exercised. |
| SR-07 | Proposed satisfied: unchanged genuine schema2/semantic2 fixture (360 bytes, SHA-256 `291b283d06db222bf355c259269328d23fef34d0d020c89012fb922bdf4d31c7`) checked-decodes and rebuilds. A richer current archive has independently expected typed route parity. Schema and semantic revisions stay 2. The one-system-per-galaxy predecessor fixture is not claimed as multi-node route evidence. |
| SR-08 | Proposed satisfied: real `nms route` subprocesses cover nearest-neighbor and 2-opt success, Lush targets distinct from the start, one-count round-trip, and missing-target nonzero/no itinerary; typed formatter tests verify shared names remain galaxy-qualified and legs are exactly 400 ly. Public route/error docs build. |
| SR-09 | Proposed satisfied with a recorded capture limitation: the final nine gates all exit 0 and have separated raw stdout/stderr captures; source and staged planning whitespace checks pass. Some early exploratory failures were not wrapped by the capture helper and their raw bytes were not retained; they are identified without fabricated receipts in `execution.md` and `validation.json`. |

## Changed public behavior and migration notes

- `GalaxyModel` keeps the four existing active-galaxy Vec-returning spatial methods and adds explicit-address counterparts: `nearest_systems_from_address`, `systems_within_radius_from_address`, `nearest_planets_from_address`, and `planets_within_radius_from_address`.
- `RouteError` adds `ForeignGalaxy`. `GraphError` adds `NoSystemsInGalaxy(u8)` and `TargetNotFound(String)`. `shortest_path` now returns typed foreign/missing failures and ignores cross-galaxy shortcut edges. TSP uses `NoPath { from, to }` for a valid foreign target.
- `build_route_from_order`, `two_opt_improve`, and `constrain_hops` now return `Result<Route, RouteError>`. All callers in the permitted graph/query paths were migrated. Repository-wide caller search found no additional non-test callers outside the allowlist; watch and MCP consumers were inspected and left unchanged.
- Route target semantics now scan complete selected-galaxy planet/system populations before dedup and truncation. Biome routing has no implicit 100-target cap. Waypoint routes remain best-effort and may include a direct final leg longer than `max_ly` when progress is unavailable or the guard ends.

## Validation and remaining limits

The final sequence ran in the candidate with `CARGO_NET_OFFLINE=true`: formatting, lint, scoped all-feature tests, full `make test`, workspace all-feature build, graph/query/cache doctests, broad no-deps documentation, strict cache documentation, and source/planning diff checks. The scoped suite reported 663 passed, 0 failed, 0 ignored; full `make test` reported 881 passed, 0 failed. Exact argv, cwd, relevant env, exit code, separate channel lengths/hashes/base64, and earlier captured attempts are in `artifacts/validation.json`.

The broad documentation command exits 0 while reporting existing unresolved `VP[0]`/`VP[1]` links in untouched `crates/nms-graph/src/extract.rs` and unclosed `<label>` examples in untouched `crates/nms-query/src/table.rs`. Strict cache documentation passes. Cargo also reports the pre-existing `proc-macro-error2 v2.0.1` future-incompatibility advisory. No dependency or cache envelope changed.

This slice does not prove real-save galaxy extraction semantics beyond the preserved fixture and test-built models. It does not accept live REPL/MCP routing, startup policy, session switching, duplicate-base policy, save-field oracle, portal UX, or strict numeric policy. The existing cache lifecycle and current in-scope tests passed, but startup and live-interface acceptance remain parent-owned. Parent arc/project rows remain open.

## Artifact inventory

- `artifacts/execution.md` — prompt intake, baseline receipts, implementation record, validation interpretation, and limitations.
- `artifacts/validation.json` — ordered final gates and captured command receipts with raw channel metadata/base64, plus earlier captured attempts and uncaptured-diagnostic disclosures.
- `closing-report.md` — this proposed closeout.
- `ledger.md` — Evidence/Notes additions only; all nine statuses remain open.

## Bubble-up to the arc

1. **Was Slice08 delivered as assigned?** Yes, as CC proposed-done. The authorized source paths, focused integration tests, evidence artifacts, required commits and commit trailers are present. Independent CDC acceptance remains pending for all nine rows.
2. **Did anything unforeseen arise?** The candidate worktree needed an escalated Git metadata write after the sandbox denied its branch lock; a later planning index staging attempt also needed escalation. A compiler typo and one formatter assertion were corrected. The broad docs gate retained pre-existing warnings outside the allowlist. Early unwrapped diagnostic failures have no retained raw-channel bytes; that capture gap is disclosed rather than backfilled.
3. **Does the parent plan need a change, and who owns it?** No parent plan change is justified by this evidence. CDC owns independent row-by-row reproduction and acceptance, plus any later strict combined-doc cleanup or live/session/startup work. The Operator remains the approval authority for any scope or policy change. No parent artifact was edited.
