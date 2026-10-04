# CC assignment — Slice04 address repair contract decision packet

Execute `arc06-fit-and-finish/slice04-address-repair-contract/cc-prompt.md` in project01-mvp. Two-Contributor Workflow, Expedited Mode; return through the Operator to CDC/Sofie. This is a bounded **decision packet**, not production repair. Slice03 is independently closed. The existing closeout route requires field-specific evidence and consequential policy before implementation. Prepare the concrete choices and compatibility consequences for that decision.

## Fences and authorized paths

Source root `/Users/oubiwann/lab/oxur/nms-copilot`, clean main `a800950ef3558fcf463517696c3c140061e7353d`. Planning root `.worktrees/planning`, branch planning; record actual author-packet HEAD and current status. Check source HEAD/status before and after. Source/required evidence drift blocks dependent analysis; report exact discrepancy, do not repair or substitute inputs.

Only six slice-relative paths may change/commit:

- artifacts/decision-packet.md
- artifacts/impact-map.csv
- artifacts/behavior-probes.json
- artifacts/execution.md
- closing-report.md
- ledger.md — Evidence/Notes and CC proposed dispositions; statuses stay open pending CDC

Preserve author-baseline.json, plans/prompts, predecessor reports/data/receipts, parent records and source exactly. No raw save read or second save/private-home/account crawl, decoded save dump, personal record identity/seed/Position values, source/test/fixture/config/dependency edits, user cache writes, installation, release, game mutation or external message. Synthetic named examples from author-baseline.json are permitted. Temporary fixture files/scripts may live in `/private/tmp` and stay out of Git; no reusable test tool is a deliverable.

## Required reading and intake

All source extents below refer to main's exact pin. Recover truncation before dependent edits; complete body reads and dataset query coverage are separate. Record extent, source state, gaps/failures/conditional applicability plus concise source-cited contract readback in artifacts/execution.md **before** other output edits. The readback must explain save-vs-portal roles, galaxy identity, synthetics-vs-game warrant and why a passing core roundtrip does not prove save decoding.

| Input | Required extent / purpose |
|---|---|
| This prompt, slice-plan.md, ledger.md | Required-full; active assignment, scope and seven criteria |
| artifacts/author-baseline.json | Required-data: complete synthetic templates, all16 packed cases, all13 expanded cases, counts and two CSV controls; inspect all three author observations and limits. Derive fresh results, not copied counts |
| Parent arc-plan.md; project01-mvp/closure-preparation.md | Required-section: Current corrective preparation / bounded closeout sequence / authority; Ordered route steps1–2, Active assignment, Gates and authority |
| ../slice03-decoded-location-witnesses/cdc-verification.md | Required-section: governing banner and complete Iteration02 final independent review, including retention and bubble-up |
| ../slice03-decoded-location-witnesses/artifacts/field-assessment.md | Required-full; accepted bounded claims and unresolved policies; its CC-status language is historical and the final CDC review governs acceptance |
| ../slice01-address-encoding-evidence/artifacts/caller-map.csv | Required-data: every column of all20 seed rows; preserve row coverage, supplement rather than silently replace |
| crates/nms-core/src/address.rs; system.rs; nms-graph/src/spatial.rs | Required-section: address.rs1–544 full core/portal constructors/accessors/conversions; system.rs1–58 SystemId/System; spatial.rs1–40 re-export/point identity |
| crates/nms-save/src/lib.rs; model.rs; convert.rs | Required-section: lib.rs1–65 parse routes; model.rs1–165,197–239,279–340 full named address/record/base types; convert.rs1–108 full conversions plus tests111–208 and previous-sentinel test292–end |
| crates/nms-graph/src/extract.rs; model.rs; import.rs | Required-section: extract.rs104–177 full extract_systems; model.rs20–167 indexes/from_save,256–357 insertion/ensure-player/apply_delta,706–734 current multi-galaxy test; import.rs92–181 full import_csv plus36–83 record/galaxy resolution |
| crates/nms-query/src/find.rs | Required-section: execute_find60–end of body (~175), result type32–58; identity association and portal_hex |
| crates/nms-watch/src/snapshot.rs; delta.rs | Required-section: snapshot.rs1–61 data/from_save/from_file; delta.rs1–69 entire compute_delta |
| crates/nms-cache/src/data.rs; freshness.rs; serialize.rs | Required-section: data.rs full; freshness.rs1–99 entire load/freshness path; serialize.rs1–114 extract/serialization/read/write/rebuild |
| crates/nms-cli/src/{convert,info,import,export}.rs; nms-copilot/src/dispatch.rs; mcp/tools.rs; main.rs | Required-section: CLI convert.rs1–128 plus278–292 and425–435 compatibility tests; info.rs1–121 summary/base glyph route; import.rs full; export.rs1–106 record/run; REPL dispatch.rs454–533; MCP tools.rs646–696; copilot main.rs353–360 load_model. CLI main.rs Info/Import/Export declarations and match arms, located by symbol rather than guessed file |
| Rust-guidelines under /Users/oubiwann/.codex/skills/rust-guidelines | Required-section: SKILL.md Overview/selection/workflows relevant to design; guides/02-api-design.md API-37, guides/05-type-design.md TD-03, guides/11-anti-patterns.md AP-33, guides/03-error-handling.md EH-04; exact strengths and application below |
| Engineering-methods guides/07-implementation-prompt-authoring.md | Required-section: Investigation and semantic evidence assignments; Required reading and CC intake |
| Work-verification guides/05-independent-verification.md | Required-section: Assigned Reviewer Slice Verification; Evidence During Correction Review; distinguish author/CC/reviewer evidence |
| Cargo.toml / Cargo.lock | Required-section/data: existing workspace edition/MSRV and CLI dependencies; no repin or new dependency |

For scoped source sections whose line ends fall inside a symbol, load through its complete closing body. An unexpectedly broader dependency is recorded and resolved before affected work. Historical reference repos/decoder runtimes and raw save population are reference-only; no new fetch/decode needed. If a required guide or source is unavailable, report concrete blocker to CDC.

## Binding inspection method and deliverable shapes

### 1. Reconcile warrant, then characterize current parser behavior

Keep five labels separate: documented/current source contract; accepted one-input observation; synthetic current-behavior result; recommendation; unresolved. The44 base pairs warrant bounded base-field agreement, Chili is one prior game-confirmed discovery witness, scalar equalities are correlations, and ordinal174 is expanded-only. No new universal or nonzero packed interpretation follows. Existing public core contract is48-bit P-SSS-YY-ZZZ-XXX plus separate reality; observed save candidate is14-digit P-SSS-GG-YY-ZZZ-XXX. Treat correspondence as evidence with field-specific applicability, not interchangeable types.

Build the existing CLI from source with `cargo build --offline --locked -p nms-cli` at the fenced root. Record exit/warnings; dependency unavailability blocks behavior replay, not proof of a product failure. Use root target/debug/nms and an explicit temporary --save path for every save command; never trigger auto-detection. No source edit means format/lint/broad suite not needed for this packet.

From author-baseline.json deep-copy synthetic_save for each independent case. Its minimal root is known to parse. For each of16 packed cases, inject the value twice, in separate fixtures:

- discovery: one synthetic_discovery_template under `DiscoveryManagerData.DiscoveryData-v1.Store.Record[0].DD.UA`;
- base: one synthetic_base_template under `BaseContext.PlayerStateData.PersistentPlayerBases[0].GalacticAddress`.

Set values preserving JSON kinds; remove required key for operation=remove. Never convert integers to floats or strings. Execute `nms info --save <fixture>` for each; record accept/reject, exact exit/error and relevant base/summary rows. The32 cases include zero, lexical variants, signed-negative casting, fraction, bool, null, missing, empty/bad hex, over56, over-u64 and max56. Characterization is allowed to find current acceptance of dubious values; do not encode recommended rejection as the baseline expectation. info displays raw expanded values and masks base scalars; it does not observe every core conversion. Label that limitation and trace corresponding core effects from source.

For each of13 expanded cases, start with expanded_valid_wrapper in `BaseContext.PlayerStateData.UniverseAddress`, then apply its declared operation. replace-member modifies the named key in GalacticAddress; replace-reality modifies RealityIndex. Execute info with explicit --save, inspect complete exit/error and Voxel Position/System Index/Planet Index/Galaxy rows. Missing wrapper, null, empty object, valid/zero/partial, X2048, Y128, SSI4095/4096, P16/256 and Reality256 must all be accounted for. info reports raw expanded members: masking in GalacticAddress::new is a separate source-derived consequence, not measured by that output.

45 parse runs total, no sampling. behavior-probes.json contains source revision, build command/result, unique case ID+family, exact injected JSON kind/value or remove operation, argv, exit, stderr, observed stdout projection and explicit observation limits. ANSI stripping is permitted for display projection; retain original stdout digest if omitting full output. A failed operation remains a case result. Do not assert semantic correctness from parse success.

### 2. Six existing integration controls

Run and record through the same freshly built CLI:

1. convert --ga 0x212B0004B2446A: author baseline2B0004B2446A; current truncation oracle, not desired repaired behavior.
2. convert --glyphs 212B04B2446A: author baseline212B04B2446A; correct portal control.
3. convert --ga 212B04B2446A: same portal-shaped48-bit control, separates changing core semantics from adding save decoding.
4. import collision_csv against the empty synthetic_save: author baseline1 added/1 duplicate/0 skipped despite galaxies0/9.
5. reverse the two collision CSV data rows: characterize same loss/count and inspect which first record survives by source flow; CLI ImportStats does not expose stored winner, so never claim a runtime winner observation.
6. import distinct_csv against a fresh empty synthetic_save: changed SSI should distinguish identities, expected current2 added/0 duplicate/0 skipped, derive result freshly.

No import persistence or live REPL/MCP/cache/watch acceptance claim. Import handler creates only an invocation-local model. If results disagree with source/author baseline, record and investigate within the named scope; no repair authorized. These six plus45 parse cases give51 declared executions, with all attempts/exits retained. Semantic save oracle212B04B2446A belongs to prior accepted Chili evidence; current --ga is a separate compatibility path.

### 3. Complete impact map

artifacts/impact-map.csv carries one or more rows for each20 seed caller rows, with explicit seed row number, source revision, path/symbol/full-body range, input family, current behavior, galaxy source/identity key, direct consumers, compatibility consequence, proposed repair/test owner and evidence pointer. Add named consumers: SystemId/PlanetKey/address_to_id/node_map/R-tree; graph insert/ensure_player/apply_delta; import duplicate checks; find associations/planet glyph output and CLI info raw-mask bypass; watcher snapshot/delta keys; cache extraction/rebuild/freshness and REPL load_model. Search named crate source for omitted direct uses of `from_packed`, `to_galactic_address`, `galactic_address.0`, `SystemId` and address maps, and inspect relevant bodies. Record search extent and nonmatches; no unrelated broad audit. REPL/MCP route findings here are static source observations, not live protocol evidence.

The SystemId example is load-bearing: from_address clears planet bits but ignores reality. The existing graph multi-galaxy test uses differing X/SSI, so it cannot reject an identical-location collision. CacheData stores already-decoded addresses and save_version/cached_at; freshness compares mtimes, with no checked decoder revision. Proposed decoder changes therefore need an explicit old-cache policy even if the archive shape stays identical. Inspect rebuild active_galaxy and watcher address/key propagation rather than assuming child tests prove composition.

### 4. Eight-topic decision packet for CDC/Operator

For each topic D01–D08 in decision-packet.md, give current source/accepted evidence, concrete recommendation with exact behavior or illustrative signature, rejected alternatives/tradeoffs, affected callers/compatibility, proposed discriminating oracle, decision owner/status and re-entry. Illustrative signatures are uncompiled proposals. Label authority **recommended / unresolved**, not binding adoption. Local discretion covers prose/CSV row ordering and temporary helper names only.

- D01 field-specific save decode vs48-bit portal/core: recommend a distinct save-packed wrapper/conversion boundary, preserving the core portal layout. Work out extraction/padding/width rules and discovery/base applicability separately; insufficient discovery/general evidence stays explicit.
- D02 packed input policy: exact integer/hex lexical/kind/width bounds, signed-negative, fraction, unsafe numeric transport, zero and missing/null/empty/malformed states. Separate representational56-bit maximum from valid game ranges; characterize whole-save failure vs per-record rejection consequences. Recommend no silent wrap/default of invalid input; CDC chooses the public failure policy.
- D03 expanded policy: typed primitive limits versus game bit widths, required/default/sentinel members, SSI0xFFF vs documented0xFFE, P0–15 vs game-valid planets, previous-address zero sentinel including omitted planet/reality. State validation/checking placement and error shape proposals.
- D04 reality attribution: explicit expanded wrapper, potential packedGG, missing/conflicting sources, active context vs per-record galaxy. Avoid defaulting every discovery/base to current galaxy. Synthetic nonzero candidates cannot establish game sourcing; owner/re-entry required before adoption.
- D05 compatibility/API surfaces: retain published core/portal/signal-booster behavior as baseline; explicitly assess changing public PackedGalacticAddress/to_core conversions, CLI --ga mask tests, REPL --ga portal parsing and MCP defaults. Recommend separate save-specific conversion over globally changing from_packed; any behavior/API amendment is held for CDC/Operator decision.
- D06 galaxy-qualified system identity: recommend a key including reality plus planet-zeroed coordinates, compare tuple/newtype versus revised u64 encoding, serialized/public SystemId consequences and all direct indexes/delta paths. No new key selected/implemented here.
- D07 related query/display correctness: distinguish planet-to-system association, correct planet nibble, info base glyph bypass, cross-galaxy comparisons and name loss ownership. Name which already-promised corrective slice owns each; this packet does not repair them or transfer them to Project02.
- D08 lifecycle and repair sequence: decoder/cache compatibility discriminator or explicit invalidate/rebuild policy, stale/corrupt/old-but-newer-mtime paths, watcher consistency and source migration. Propose bounded dependency order and concrete production readiness gates without preissuing future implementation prompts.

Domain applications: API-37/TD-03 **SHOULD** support distinguishing save-packed, portal and galaxy-qualified ID roles in proposals; AP-33 **AVOID** supports explicit invalid/unknown/default state accounting; EH-04 **MUST** applies to proposed future library errors, not opportunistically changing existing Box<dyn Error> APIs in this evidence slice. New dependencies/traits/concurrency/runtime/edition are unnecessary. Keep current Rust2024/MSRV1.91; no guide default overrides them.

### 5. Future proof and return

Include a compact future acceptance-to-test mapping in decision-packet.md, not a new test framework. Use independent exact components/portal from accepted expanded oracles; a changed decoder vs itself is no oracle. Include Chili, representative paired base, all44 base comparisons as retained bounded evidence; min/max signed fields and width/sentinel errors governed by chosen policy; same location in galaxies0/9 must retain two systems/planets/indexes and distinguish watcher deltas. Cache test must include an old decoded archive newer than its source, not only corrupt bytes. Query test must use matching planet indexes in distinct systems and a selected planet differing from system.address's stored nibble. Separate actual methods available today from future APIs requiring approval. Full protocol/live watcher/cache composition remains later acceptance work, never inferred here.

Write execution.md with pre-edit intake, exact methods/results/denominators, file/state fences, every failed/blocked/unrun attempt and guideline applications. closing-report.md walks all seven rows as CC proposed-done, inventories outputs and bubbles up: packet delivered vs requested; new design facts; honest scope diff. Unsupported policy gets owner/re-entry, not invented warrant. Source/seed/plan/criterion hashes, exact six paths, JSON/CSV parsing, local links/privacy/whitespace/trailers and preserved predecessor data are structural checks, not independent adoption.

Commit exactly the six named paths with git commit --only on planning; do not stage unrelated changes. Both trailers once:

    Co-authored-by: Codex <noreply@openai.com>
    Co-authored-by: Billo AI <ai-engineering@billo.systems>

Return assignment/commit,45+6 execution coverage and outcomes, complete20-row seed coverage/additions, all eight decision dispositions, failures/limits and clean/pinned states. All RP statuses stay open until independent CDC review. Production policy, arc/project composition and required Operator compatibility/architecture gates remain held. A good packet can close with unresolved design questions precisely fenced; it cannot grant permission to implement them.
