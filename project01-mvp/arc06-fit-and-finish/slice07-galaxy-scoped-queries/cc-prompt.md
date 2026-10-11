# CC implementation — Slice07 galaxy-scoped queries

Issued 2026-10-10. Project01 / Arc06 / Slice07, initial assignment. Execute this production implementation and return to CDC/Sofie for independent review. Two-Contributor Workflow and Expedited Mode continue. This assignment follows independently closed Slice06; it does not close the identity parent, arc or project.

## Outcome, authority and source state

Fix cross-galaxy system-name collisions and the shared find engine's lost planet/system association, inconsistent filtering, incorrect explicit-reference scope and premature nearest truncation. Suppress show distances across galaxies. The Operator approved the identity/correctness expansion on2026-10-04 and approved Slice06's precise historical evidence exception and continuation on2026-10-10. CDC details this bounded query pass inside that direction; the historical exception does not apply to any new capture.

Source baseline is accepted `f57a38f7d1f00d1a3f5aa500ed5f7ce7d8c26eb8` in `.worktrees/galaxy-qualified-identity`. Create `.worktrees/galaxy-scoped-queries`, branch `codex/galaxy-scoped-queries`, from that exact commit; reuse only a checkout already established for this assignment at its expected state. Leave main at `a800950ef3558fcf463517696c3c140061e7353d`, the identity foundation at f57a38f and cache dependency at `decd8d6c09c1199a6cdd01fd48aa50f3adef2b1a` clean and unchanged. Work in the new candidate, never the main or dependency checkouts. Planning artifacts go in the existing `.worktrees/planning` on `planning`; record its actual issuance HEAD. Stop affected work on source/assignment drift or a contradictory binding contract.

Required routing/fallback/waypoint/reachability, movement/session/selected-galaxy UI and live-cache/CLI/REPL/MCP composition follow this child's review, before field-specific save repair. Same-galaxy duplicate system names, duplicate base-name identity, lost save names, low-level active-spatial APIs and planet-specific portal-address construction retain their existing unresolved owners in R06-R02 and original correctness work. Do not represent this pass as solving them. No discovery/base decoder, nonzero save-field sourcing, address math, portal repacking, new grammar/CLI flags, dependencies/manifests, broad docs cleanup, coverage-target change, merge/push/publication or user-save/cache mutation is authorized.

## Required reading and intake

Read complete bounded content before dependent edits. Record assignment/pins, each item's source state and loaded extent, any truncation recovery, conditional applicability, and a concise contract readback in `artifacts/execution.md`. File existence/hash/headings are not reading. Recover omitted tool output with contiguous bounded reads. In a resumed context reload contracts no longer available. A coherent preflight authorizes execution without another permission request.

Reading order and categories:

1. **Required-full:** this prompt, sibling `slice-plan.md` and `ledger.md` at the planning issuance HEAD. These define active assignment, scope and acceptance; no historical prompt supersedes them.
2. **Required-section:** parent `arc-plan.md`, complete “Approved scope and current execution”, “Identity foundation and subsequent composition”, “Slice06 final closure”, and “Slice07 query composition assignment”; parent `project-plan.md`, complete “13. Approved correctness scope expansion — 2026-10-04” section, “Slice06 final closure” and “Slice07 query composition”. Read `closure-preparation.md` “Gates and authority” and final Slice07 section. Historical `0038-phase-7b-model-extensions-multi-galaxy-formats-community.md`, complete Milestone7.4 and Implementation Notes / Multi-Galaxy Impact on Routing section. These preserve the active-default galaxy contract and all broader gates; its illustrative stale signatures are not implementation authority.
3. **Required-full:** repository AGENTS.md at source baseline and installed collaboration-framework, project-management, work-verification, engineering-methods, testing and rust-guidelines `SKILL.md` entrypoints. Current skill root is `/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/`; do not assume the old `~/.codex/skills` paths exist. Follow their applicable routing requirements. **Required-full:** project-management `guides/README.md` before planning operations and engineering-methods `guides/07-implementation-prompt-authoring.md` for the assignment/intake/readback contract. **Required-section:** project-management `guides/03-planning-top-down.md`, “per-slice open set” through “Issuing and executing an iteration”; work-verification `guides/01-ledger-discipline.md`, slice row/status/evidence rules; testing `guides/01-testing-discipline.md` and `03-validation-gates.md` in full.
4. **Required-section:** Rust guides `11-anti-patterns.md` AP-01–AP-10 (first), `02-api-design.md` API-02, `03-error-handling.md` EH-01/EH-04, `05-type-design.md` TD-03, `13-documentation.md` DC-01–DC-04, complete pattern bodies. They support the decisions below. Other domain chapters are conditional on actual affected mechanics; no async/unsafe/type-system redesign is requested. The user's instruction to use applicable parts governs this bounded selection.
5. **Required-full:** source `crates/nms-query/src/find.rs`, `show.rs`, graph `error.rs`, CLI `tests/find_integration.rs`, and sealed sibling `artifacts/author-probe.rs`. **Required-section:** graph `model.rs` runtime through `apply_delta`, existing named lookup/insertion/from-save and dual-galaxy regression test bodies plus necessary fixtures; graph `query.rs` all runtime and filter tests; core `system.rs` SystemId/System/Planet definitions/constructors, `player.rs` PlayerState/PlayerBase constructors, `address.rs` distance_ly/same_region/same_system definitions; cache `serialize.rs` runtime through rebuild_model; copilot `Cargo.toml` dependencies/features and query `display.rs` find formatter and show distance formatting. Inspect all affected callers with `rg`: `name_index`, `system_by_name`, `matches_filter`, `BiomeFilter`, `execute_find`, and show distances. This covers initialization, insertion, archive reconstruction, existing consumers and the tests to extend without loading unrelated route/session algorithms.
6. **Required-data:** sealed `artifacts/author-baseline.json`: query metadata, all three capture commands/exits and exact complete decoded probe stdout/stderr; check all source_sha256 entries and fixture identity. This is an explicit full projection, not a sample or claim to read base64 compiler output semantically. Decode/retain original build/compiler channels when diagnosing them. Check the entire sealed binary fixture (360 bytes, SHA-256 below); no game saves needed. Author output is baseline attestation, not independent acceptance of your future implementation.
7. **Conditional:** if a relevant original requirement or source consumer conflicts with the binding recipe, load its entire necessary section and stop affected implementation for CDC disposition. If checks need sandbox/cache-write escalation, retain the failed attempt, use the supported permission path, and retry without discarding it. **Reference-only:** older evidence slices and all predecessor implementation prompts; they are provenance, not current CC assignments.

Readback must explain why planet index alone cannot recover a system, why a reference in galaxy9 must not be measured against galaxy0, why `nearest` applies after every filter, why existing2/2 raw archives can remain compatible, and which parent claims this slice does not discharge. Cite governing sections and rows, not just a read-list acknowledgment. Reading receipts remain attestation, not comprehension or acceptance proof.

## Binding decisions and implementation spine

### 1. Names retain the galaxy — QG-01

In graph `model.rs`, change the existing public derived index to `HashMap<(u8, String), SystemId>`, keyed by `(system.address.reality_index, name.to_lowercase())`. Wire both `from_save`'s local construction and `insert_system`; `new` remains empty, and archive rebuild automatically uses insertion. Preserve the existing lowercase normalization, duplicate-system no-op and same-galaxy duplicate-name last-insert behavior. No new name-ambiguity policy or base-index change.

Retain `system_by_name(&self, name: &str) -> Option<(&SystemId, &System)>` as the active-galaxy wrapper. Add this borrowed lookup:

```rust
pub fn system_by_name_in_galaxy(&self, name: &str, galaxy: u8)
    -> Option<(&SystemId, &System)>;
```

It looks up the qualified lowercase key and then `systems`; never falls back to a different galaxy. Return borrows into self. Existing show and bare-name route calls inherit the active scope; route's explicit-start naming policy is later work. Document the public index representation change and wrapper/explicit semantics. Do not embed galaxy into portal bits or expose SystemId's private invariant fields.

### 2. One predicate, one association — QG-02/03

In graph `query.rs`, move the existing private `matches_filter(planet, filter)` predicate into public `BiomeFilter::matches(&self, planet: &Planet) -> bool`; update existing graph call sites mechanically. Preserve its exact four-field conjunction: optional biome, subtype, infested true/false, and named_only requiring `planet.name.is_some()`. Empty Some names retain existing behavior. Correct FindQuery's named_only documentation to describe named planets, matching the existing spatial predicate; system names can match name_pattern independently. No low-level spatial selection/algorithm change in this slice.

Replace find's three divergent branches and fixed overfetch with a single borrowed candidate pass over `model.planets`, retaining each complete `(SystemId, planet_index)` key. Resolve its containing `System` with that key; never search systems by planet index and never rebuild identity from name or numeric coordinate alone. A missing containing system is skipped as in the current filter_map; this is not an orphan-model validation feature. Normal model insertion/delta reconstruction maintain these associations.

Filter galaxy before calculating distance. Apply `BiomeFilter::matches`, existing case-insensitive planet-OR-system name substring and system discoverer substring, and the optional radius to each candidate. Normalize query name/discoverer once outside the loop. For finite radii use the existing rejection `distance > radius`. Preserve the existing exceptional radius behavior explicitly: negative radii (including negative infinity) yield no matches; positive infinity adds no cutoff; NaN with no nearest limit yields empty after reference resolution, while NaN with a nearest limit imposes no radius cutoff. This preserves the old branch difference without choosing a new validation/error policy; numeric-input validation remains separately owned. Do not describe these legacy cases as validated useful radii. Add small boundary regressions for these retained cases.

### 3. Reference and selected scope — QG-04

Resolve exactly as existing ReferencePoint: CurrentPosition requires player position, Base requires an existing base, Address uses that address. Preserve NoPlayerPosition and BaseNotFound precedence even for `nearest=0`.

Binding reference table:

| Reference | Galaxy searched | Mismatch behavior | State mutation |
| --- | --- | --- | --- |
| CurrentPosition | model.active_galaxy | If player's address galaxy differs, return typed ReferenceGalaxyMismatch | None |
| Base(name) | base.address.reality_index | Explicit base overrides active selection for this call | None |
| Address(addr) | addr.reality_index | Explicit address overrides active selection for this call | None |

Extend the existing non-exhaustive thiserror `GraphError` with:

```rust
#[error("reference galaxy {reference} differs from active galaxy {active}; use an explicit address or base")]
ReferenceGalaxyMismatch { reference: u8, active: u8 },
```

Only CurrentPosition produces this variant, after resolving player presence. No coordinate projection, active-galaxy mutation, empty-success substitution or cross-galaxy distance. Return an empty successful result when a valid explicit galaxy contains no planets, or when no candidates pass. This retains0038's default-active selection while allowing references that carry their own reality to select it; later session/galaxy commands must compose with this policy.

### 4. Exact filtering before ranking — QG-05

Build borrowed candidates `(PlanetKey, &Planet, &System, distance_ly)` with the resolved single galaxy. Sort by ascending distance (`f64::total_cmp`), then numeric `SystemId::reality_index()`, `SystemId::system_address()`, then planet index. Apply `nearest` truncation once, after all filters/radius. Only then clone selected Planet/System into existing FindResult and preserve its current system-address-based portal_hex construction. No new result fields or serialized identity changes.

Recommended sketch, not compiled production code:

```text
resolve_reference_and_scope() -> address, galaxy or GraphError
normalize_query_patterns_once
if radius is NaN and nearest is absent: return empty after reference resolution
for (complete_key, planet) in model.planets:
    system = model.system(key.system) or skip
    if key/system galaxy != scope: continue
    if not biome_filter.matches(planet): continue
    if name/discoverer predicates fail: continue
    distance = from.distance_ly(system.address)  // same galaxy now
    if optional radius and distance > radius: continue
    retain borrowed candidate with complete key
sort distance then numeric qualified key and planet index
truncate only if nearest requested
clone selected results; preserve legacy portal formatting
```

A complete candidate scan is binding for this pass: O(P + K log K) time and O(K) borrowed candidates, P total indexed planets and K surviving candidates; no O(P*S) association recovery, fixed oversampling or repeated full System clones before limiting. R-tree optimization is later performance work and must preserve exact results. Do not replace the current graph spatial methods globally to make this recipe shorter. No concurrency/lifetime ownership changes; synchronous immutable query, borrowed candidates die before returning owned results. Public graph fields retain their existing visibility for compatibility; their normal insertion/reconstruction invariants are assumed. Arbitrary inconsistent external field mutation is not validated by this pass.

### 5. Show only meaningful distances — QG-06

In query `show.rs`, preserve active-galaxy name/packed lookup and base-specific qualified system lookup. For both system/base results, compute `distance_from_player=Some(distance)` only when player address and selected item's address have the same reality_index; otherwise None (also None without player). No artificial zero, infinity or cross-galaxy warp distance. Existing formatter omits the row for None. No show grammar change; a missing named system in the active galaxy still returns SystemNotFound through the current name/hex resolution behavior.

### 6. Derived-cache assessment and parity — QG-07

Binding assessment: retain schema2/semantic2. This pass changes reconstructed indexes and query execution, not CacheData's fields, scalar address interpretation, SystemId identity, selected-galaxy meaning or raw stored system/planet/name population. CacheData stores all systems/names and rebuild_model inserts them afresh. There is no persisted bad association/name index/query result to migrate. Bumping merely because the query algorithm changed would rebuild valid raw data without improving it. This is a scoped compatibility assessment, not an exception to the continuing requirement to bump when stored meaning changes.

Prove the assessment with the actual prior-reader archive generated atf57a38f: sealed `artifacts/predecessor-cache-schema2-semantic2.rkyv`,360 bytes, SHA-256 `291b283d06db222bf355c259269328d23fef34d0d020c89012fb922bdf4d31c7`, header hex `4e4d5343414348450200000002000000`. Copy unchanged into the allowed native fixture path, never patch a current writer's header and call it a predecessor. It contains two systems named Shared in galaxies0/9 at voxel0/SSI1, planet1 named Zero(false)/Nine(true), Lush, discoverer Explorer; player0 with currencies7/8/9; active9; save_version4720. cached_at is incidental. This is typed source-flow evidence, not nonzero discovery-field authority.

In copilot's new integration test (its existing dependencies include graph/query/cache/core, so no manifest changes), use public deserialize/rebuild_model on this genuine archive; assert exact raw populations, both qualified names, active9/player0, header2/2 and expected query/show behavior. Compare against a freshly constructed typed model with the same fixture inputs and a current serialize/deserialize/rebuild roundtrip. Compare declared projections excluding timestamp/incidental graph traversal: qualified names, complete result identities/planets/distances/order and show absence of cross-galaxy distance. This proves stored-data compatibility and fresh/warm query parity, not full startup/live/MCP composition. Existing prior1/1 rejection tests and lifecycle tests must still pass. If implementation needs a raw-data semantic/layout change contrary to this assessment, stop for CDC revision/disposition instead of silently retaining or bumping revisions.

Private helper names, test helper structure and local iterator style are CC discretion. The behavior, public signatures, state/failure boundaries, compatibility decision, candidate completeness and ordering are binding. Alternate consequential APIs/policies require CDC review before implementation.

## Acceptance-to-test mapping

Extend native tests; derive literal expectations from these setups, not another invocation of the repaired function. Parameterization is fine; report actual assertions/case counts. Existing weak count-only tests do not prove these contracts.

| Row | Setup, action and exact observable oracle | Incorrect implementation rejected |
| --- | --- | --- |
| QG-01 | Insert Shared systems with equal coordinates/planet1 in galaxies0/9, both insertion orders. Select0 then9: case-insensitive wrapper and explicit lookup return the corresponding qualified ID and exact different planet/name; unknown galaxy7 returns None. Repeated same-system insertion stays no-op. Exercise from_save construction's index shape with its available fixtures (save extraction currently lacks names), plus insertion/cache reconstruction. | Global last-name overwrite, bare coordinate identity, only one construction site fixed |
| QG-02 | Two distinct systems in the same galaxy, each planet index1, both Lush and distinct planet names. Unbounded biome query returns exactly both complete planet/system pairs; for each assert selected planet equals its containing system's matching-index planet and literal SystemId. Repeat both insertion orders. Add an identical-coordinate galaxy9 counterpart and assert no wrong galaxy pair. | Index-only reconstruction and hash-order-dependent pairing |
| QG-03 | Named and unnamed planets, two subtypes, infested true/false and mixed biomes. Assert exact matches for each four-field predicate alone, plus their conjunction and deliberate no-match. Run valid unbounded/radius/nearest/combined-radius-nearest variants with identical expected filtered population before limits. Test Some(false), not only true; named system alone does not satisfy named_only. | Non-spatial path ignoring subtype/infested/named, filters differing by branch |
| QG-04 | Player0/active0 default excludes9. Explicit address9 and uniquely named base9 while active0 return only9 for every spatial mode with literal distances0/400ly from voxel0/1; active selection/player remain unchanged. Player0/active9 default returns ReferenceGalaxyMismatch{reference:0,active:9}; no player returns NoPlayerPosition; missing base returns BaseNotFound. Valid empty galaxy7 returns empty; nearest0 still resolves reference/errors. | Active-tree leakage, projection, fabricated cross-galaxy distance or hidden state mutation |
| QG-05 | Origin(0,0,0,SSI1), three near nonmatches and a farther matching planet at X4: nearest1 with name pattern Keep returns that exact match at1600ly. Separately assign near systems discoverer Other and the X4 target Explorer; discoverer Explorer alone must return the X4 target at1600ly. Radius1600 includes it,1599 excludes it, both constraints compose. Equal-distance candidates X-1 named Alpha and X1 named Zeta, each SSI1 with planets1/2, assert X1 planet1, X1 planet2, X-1 planet1, X-1 planet2 (the signed negative voxel packs above positive1), both insertion orders; nearest0 empty. | Fixed2N overfetch, filtering after truncation, hash or alphabetical tie ordering |
| QG-06 | Show Shared under active0/9 returns its correct galaxy. Player0 showing system/base9 produces None and rendered output omits Distance; same-galaxy show at X1 is Some(400.0), same system Some(0.0), no player None. Packed show and base containing-system identity remain qualified. | Cross-galaxy same-coordinate false0 and hidden name overwrite |
| QG-07 | Decode unchanged sealed2/2 archive, verify literal population/player/active/version, execute scope/name/filter/show assertions; equivalent fresh typed model and current roundtrip have identical declared projections. Existing prior1/1 rejection/cold-warm lifecycle suite passes. | Mistaken claim that old index is persisted, stale index rebuild, lost scope on warm load |
| QG-08 | In CLI find integration, strengthen public parser→model→query exact associations/filter assertions and add actual cargo_bin_cmd nms find --save tempfile --infested test: clone existing synthetic fixture, set second planet VP[1] to65536, assert only its known system address appears, first excluded, success exit. At query formatter level assert paired names and omitted invalid show distance. Public docs state scoping, errors, named_only, completeness/ties, public name-index type and retained compatibility. | Library-only execution described as subprocess proof, weak empty/count-only assertions, undocumented API behavior |
| QG-09 | All prescribed gates below pass; source/planning/dependency/fixture/commit fences, exact capture of every attempt and complete return per row. | Green-suite closure without honest evidence, collateral source change or historical exception reused |

Use the verified core variants BiomeSubType::LushRoomTemp and BiomeSubType::LushHumid for the subtype fixture. Use finite valid radii for the new contract; invalid-number validation is not discharged. The CLI fixture remains a parser regression for the current layout and cannot establish real-game generalized DD.UA or galaxy semantics. Reuse existing synthetic JSON and public constructors; no user data, new replay framework, broad mutant suite or validator product.

## Source and planning boundaries

Source candidate allowlist (only necessary edits; enumerate actual subset in return):

- `crates/nms-graph/src/model.rs` — qualified names, API docs, constructor/insertion tests.
- `crates/nms-graph/src/query.rs` — BiomeFilter::matches, mechanical predicate callers/tests only; low-level spatial behavior held.
- `crates/nms-graph/src/error.rs` — typed reference mismatch.
- `crates/nms-query/src/find.rs` — complete candidate pipeline, scope/ranking/docs/native regressions.
- `crates/nms-query/src/show.rs` — same-galaxy distance/docs/native regressions.
- `crates/nms-cli/tests/find_integration.rs` — stronger actual pipeline and subprocess regression.
- `crates/nms-copilot/tests/query_cache_integration.rs` — new public cache/query compatibility test.
- `crates/nms-copilot/tests/fixtures/predecessor-schema2-semantic2.rkyv` — exact sealed360-byte copy.

No Cargo manifests/lockfile or dependency updates. Candidate parent must bef57a38f. Don't cherry-pick/move accepted worktrees or merge main. Any necessary additional path or behavior change is a concrete stop/escalation, not implicit authorization from test failure. `make format` might touch unrelated files: inspect and preserve unrelated changes rather than commit them.

Planning CC write allowlist, relative to this slice directory:

- `artifacts/execution.md` — intake/readback, actual steps, discrepancies/failed attempts, source and fixture identities, compatibility assessment.
- `artifacts/validation.json` — each command's argv/cwd/relevant env, direct exit status, exact separate stdout/stderr bytes (base64 plus lengths), actual test summaries, unrun/blocked/ignored/warnings disclosed.
- `closing-report.md` — assigned prompt, per-row proposed dispositions, exact commits/parents/pathsets/trailers/pins and limits.
- `ledger.md` — Evidence/Notes cells only; every Status stays open pending CDC.

Sealed author inputs, this prompt, slice/parent plans, predecessor review records and original source checkouts are immutable to CC. Do not create CDC acceptance/verification files. If correcting your own report, preserve earlier failed evidence and date superseding prose; no silent overwrite of an unsuccessful attempt.

## Validation, evidence and commits

After changes, execute in the isolated candidate with CARGO_NET_OFFLINE=true:

1. `make format`, then `make lint` before tests.
2. `cargo test --offline --locked -p nms-graph -p nms-query -p nms-cli -p nms-cache -p nms-copilot --all-features` (actual focused counts, no prescribed count that hides additions).
3. `make test` (workspace/all-features gate).
4. `cargo build --offline --locked --workspace --all-features`.
5. `cargo test --offline --locked -p nms-graph -p nms-query -p nms-cache --doc --all-features`.
6. `cargo doc --offline --locked --no-deps --all-features -p nms-core -p nms-graph -p nms-query -p nms-cache`.
7. `RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links' cargo doc --offline --locked --no-deps -p nms-cache`.
8. `git diff --check`, exact fixture SHA/bytes, scoped source/path/parent/trailers and clean-worktree fences.

No coverage number is claimed by this pass; the existing95%+ project target and full original feature/docs/package/performance/composition gates remain. Prior Slice06 combined strict-doc check fails on four unchanged core/address.rs bracket links, ordinary scoped docs warn on those plus graph extract bracket/query table HTML; known proc-macro-error2 future warning. Preserve these baseline issues; changed docs must not introduce new unresolved warnings/errors. Do not call baseline failure waived or fix unrelated docs under this allowlist. A new relevant gate failure blocks proposed completion until repaired in scope or explicitly dispositioned. All attempts, including sandbox-denied and command/checker failures, retain original channels/status; the GI-08 exception applies only to its named historical call.

Capture before invocation, not retrospectively from tool-truncated or combined display. Use the existing evidence-file shape/small subprocess capture, not new product tooling. Inspect complete relevant outputs, recover truncation, retain actual test denominators and ignored/unrun cases. A successful shell wrapper does not establish every underlying command's success.

Expedited Mode: after coherent implementation and all required checks, commit source candidate with named allowed paths using `git commit --only`; then separately commit the four planning CC paths. Use the supported metadata-write permission path when needed. Every assistant-authored commit includes exactly once:

```text
Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>
```

Audit actual pathsets, parent commits and trailers, then prove clean candidate/planning/main/dependencies and sealed fixture/hash. No unrelated dirty work discarded. Return concise CC report with source/planning full SHAs, validation counts and warnings/failed attempts, per-row proposed-done/open dispositions and remaining scope. All nine rows remain open for CDC review; child acceptance never closes R06-R02/R03 or Project01. Operator relays the report; do not message other chats without separate human authorization.
