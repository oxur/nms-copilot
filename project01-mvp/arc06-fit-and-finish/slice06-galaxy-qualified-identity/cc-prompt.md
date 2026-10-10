# CC Slice06 — implement the galaxy-qualified identity foundation

Execute this **production CC implementation**, then return through the Operator for independent CDC/Sofie review. Exact assignment relative to Project01: `arc06-fit-and-finish/slice06-galaxy-qualified-identity/cc-prompt.md`. Two-Contributor Workflow and Expedited Mode continue; scope approval already exists. Eight rows GI-01–08 begin open. Do not self-accept them or implement the subsequent query/routing composition pass.

## Authority, dependency and preflight

Slice05 is independently closed at source `decd8d6c09c1199a6cdd01fd48aa50f3adef2b1a`; closure planning commit `ba6859d` (resolve complete SHA at intake). Preserve main clean at `a800950ef3558fcf463517696c3c140061e7353d` and `.worktrees/cache-semantic-compatibility` clean at the accepted dependency. Create `.worktrees/galaxy-qualified-identity`, branch `codex/galaxy-qualified-identity`, from **decd8d6**, not main. An existing worktree may be reused only if it belongs to this assignment and matches the expected baseline; never reset another checkout or repin silently. Planning is `.worktrees/planning` on planning; record actual issuance HEAD containing this prompt. No merge, push, publication, release or main movement.

The Operator approved an explicit reality-plus-system key, preserving 48-bit portal math, and cache semantic evolution. CDC details this into two bounded passes: foundation now; query/routing/interface composition afterward, before field-specific save repair. All parent criteria remain required. This is routine decomposition within the approved direction, not an acceptance amendment. No extra permission request follows coherent preflight. Source drift, unavailable required material or need for a consequential contract/path change goes to CDC before affected work; progress on independent authorized work meanwhile.

## Required reading and intake

Aliases in this manifest identify paths, not shell environment variables: P=`/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/planning/project01-mvp`; S=P/arc06-fit-and-finish/slice06-galaxy-qualified-identity; R=the new source worktree above at decd8d6; K=`/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills`. The old `~/.codex/skills` alias in preserved older prompts is absent on this host; do not rewrite those prompts.

| Order | Exact input and extent | Category / purpose |
| --- | --- | --- |
|1 | S/cc-prompt.md, slice-plan.md, ledger.md | Required-full; active contract, scope and all eight rows |
|2 | P/project-plan.md §13; P/arc06-fit-and-finish/arc-plan.md §Approved scope and current execution, §Slice05 final independent closure and §Identity foundation and subsequent composition; P/closure-preparation.md §Ordered route to closure and §Current identity assignment | Required-section; approved direction, dependency and all remaining gates |
|3 | Repository AGENTS.md; K/project-management/guides/README.md; K/project-management/guides/03-planning-top-down.md §The per-slice open set through §Issuing and executing an iteration; K/engineering-methods/guides/07-implementation-prompt-authoring.md §Required reading and CC intake; K/work-verification/guides/03-row-closure.md | Required-full for AGENTS/README/row closure, complete named sections otherwise; execution/intake/Expedited rules. Load other applicable governing material required by these skills |
|4 | K/rust-guidelines/SKILL.md Overview, Document Selection Guide, API workflow and Critical Rules; guides/11-anti-patterns.md AP-01–06 and AP-33; guides/02-api-design.md API-37; guides/05-type-design.md TD-03/05; guides/13-documentation.md DC-01/02; K/testing/guides/01-testing-discipline.md and 03-validation-gates.md §Proportionate Proof | Required-section except full testing discipline. Complete selected entries including rationale; skills may require further applicable sections, load before affected work. Key invariants/serde/docs and behavioral proof |
|5 | R/crates/nms-core/src/system.rs; R/crates/nms-graph/src/model.rs; R/crates/nms-cache/src/{data.rs,serialize.rs,lib.rs}; R/crates/nms-cache/tests/load_integration.rs | Required-full at baseline; primary changes and all existing cache regression obligations |
|6 | R/crates/nms-core/src/address.rs GalacticAddress declaration, new/from_packed/packed/planet_index; R/crates/nms-graph/src/{spatial.rs,import.rs}; R/crates/nms-query/src/show.rs; R/crates/nms-watch/src/snapshot.rs declaration/from_save and delta.rs compute_delta | Complete named definitions/bodies, full spatial/import/show; distinguish preserved portal/game boundary, import/show ownership and actual snapshot addition API |
|7 | R/crates/nms-graph/src/route.rs RouteError/Display plus all numeric SystemId construction/access test bodies; edges.rs connect_new_system and legacy constructor test; R/crates/nms-query/src/display.rs format_route and all legacy SystemId test bodies; R/crates/nms-cli/tests/show_integration.rs show_system_with_planets; pipeline_integration.rs test_pipeline_every_system_has_address_to_id_entry | Required-section; mechanical compatibility migration only; complete changed tests before editing, not just grep hits |
|8 | R/crates/nms-graph/src/query.rs nearest_systems/systems_within_radius/nearest_planets/planets_within_radius; R/crates/nms-query/src/find.rs execute_find; route.rs resolve_start/resolve_named_targets/resolve_biome_targets/execute_route; R/crates/nms-copilot/src/watch.rs drain_watch_events/apply_and_notify/system_name_near | Required-section; inspect known remaining behavior. These files are not authorized for runtime repair in this pass; report boundary contradictions to CDC |
|9 | S/artifacts/author-baseline.json, author-probe.rs, predecessor-cache-schema1-semantic1.rkyv | Required-data for baseline pins/source hashes, complete2-order probe projection and captures; full probe text; verify entire binary digest and16-byte header, do not decode/reinterpret it as current. Author observations are attested, not future acceptance |
|10 | R/Makefile format/lint/test recipes; Cargo.lock rkyv/serde package stanzas; source-wide identity construction/access and CacheData literal sites | Required-section plus required-data: search all crates/*.rs, inspect complete matching identity caller bodies with lossless output partitions. Record denominator/false positives/exclusions. No blind global `.0` replacement |
|11 | P/arc06-fit-and-finish/slice05-cache-semantic-compatibility/cdc-verification.md final review; earlier packed-format/field evidence | Reference-only provenance; no private save or historical investigation replay required |

Before dependent edits, load complete required text and data projections into the executing context, recover every truncation with bounded contiguous reads, and record source state/extents/queries/outcomes in **artifacts/execution.md**. Hashes or headings alone are not reading; summaries do not replace required source text. Give a concise source-cited contract readback covering key/serde invariant, portal distinction, cache transition, held query/game semantics and proof limits. Unavailable/conflicting inputs hold dependent work until reconciled. Revalidate baseline/contracts after context recovery.

## Binding design and implementation spine

### 1. Canonical domain key — nms-core/src/system.rs (GI-02/07)

Replace `pub struct SystemId(pub u64)` with an explicit named-field value:

```rust
// Binding shape; representative sketch, not a compiled production patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct SystemId {
    reality_index: u8,
    system_address: u64,
}
impl SystemId {
    pub fn from_address(addr: &GalacticAddress) -> Self { /* below */ }
    pub fn reality_index(&self) -> u8 { self.reality_index }
    pub fn system_address(&self) -> u64 { self.system_address }
}
```

`from_address` stores `addr.reality_index` and `addr.packed() & 0x0FFF_FFFF_FFFF` (only44 coordinate/SSI bits; planet bits removed). Equality/hash use **both** fields. Keep these fields private. No implicit-galaxy scalar constructor, `From<u64>`, opaque high-bit repacking or raw tuple compatibility shim. Mechanical test constructions use `SystemId::from_address(&GalacticAddress::from_packed(value, explicit_galaxy))`, retaining values and explicitly choosing0 for historical Euclid fixtures. Keep existing exports in core and graph/spatial.

Serialize as a JSON object with numeric `reality_index` and `system_address`, exactly those field names; it is intentionally a public serialized API change from the old scalar. Deserialize must maintain the invariant too: use a private `#[derive(Deserialize)] #[serde(deny_unknown_fields)]` field helper and manual `Deserialize` for SystemId. Require both fields, u8 reality and u64 coordinates; reject any bits outside `0x0FFF_FFFF_FFFF` using `serde::de::Error::custom` (stable message: `system_address must contain only the 44 system-coordinate bits`). Do not mask invalid serialized input or default a missing galaxy. Reject legacy integer, missing fields, unknown fields, wrong scalar types, reality256, planet/high bits and coordinate overflow; negative values fail serde unsigned decoding. No new public error type/dependency or legacy reader. Constructor normalization from a valid core address differs intentionally from rejecting malformed serialized key data.

Add `Display` with decimal galaxy, colon, and zero-padded uppercase12-digit system coordinate, e.g. **`9:050003AB8C07`**. This is an identity diagnostic, not a portal glyph string and not a new CLI parser syntax. Document key invariants, accessors and serde/tuple/API consequences. No new archive trait is needed: CacheData stores addresses, not SystemId. Derive additional traits only if an actual caller requires them; numeric-order tests can sort by `(id.reality_index(), id.system_address())`.

### 2. Model and mechanical consumers — GI-03/07

In graph/model.rs change `address_to_id` to `HashMap<SystemId, SystemId>` and insert `(sys_id,sys_id)` in both from_save and insert_system. Keep the existing field rather than opportunistically redesigning the index API; document that its key is now qualified. Every entry must have key=value, pointing to the matching system. Tests and pipeline assertion use the full ID. Existing systems/planets/biome/node/graph/spatial keys already use SystemId and acquire galaxy sensitivity through the type. Preserve insertion idempotence for the same qualified key, and existing data/topology policies.

Migrate all numeric constructors/accesses in graph/model, spatial, route/edges tests, query/display tests, and CLI show/pipeline tests. Disambiguate tuple PlanetKey `.0` and route iterator tuples: they are not the former numeric ID payload. graph/route.rs `RouteError::Display` should use `system not found: {id}` and `no path from {from} to {to}`, with qualified Display. Its runtime routing algorithms, fallback and return signatures remain unchanged in this foundation pass.

query/display.rs `format_route` fallback **must format `system_address()` as12-digit hex** before `hex_to_emoji`, preserving Address/Portal Glyphs columns. Do not put the qualified Display string into glyph conversion. The existing system-address case remains `system.address.packed()`. Tests prove a missing-system hop in galaxy9 still yields12 glyph-address digits, while error diagnostics identify galaxy9.

Do not edit GalacticAddress math/serde, save scalar decoders, extracted discovery galaxy, typed SaveRoot or game data. extract_systems currently supplies galaxy0 from scalar discoveries; this slice does not establish its correct nonzero source. Global name_index/base-name identity and active-spatial selection remain as currently implemented for the subsequent query/routing pass. No inferred universal multi-galaxy capability claim.

### 3. Existing import/show surfaces — GI-04

Graph import_csv already decodes12-digit portal input with explicit resolved galaxy and deduplicates by SystemId; retain the runtime body, add exact both-order import tests. For identical glyph coordinates in0/9, expect2 added,0 duplicates,0 skipped, and replay yields0 added,2 duplicates,0 skipped. Different planet glyphs in the same galaxy remain duplicate systems. Keep galaxy-name fallback, biome/source-tagging/persistence behavior unchanged and disclosed; they are separate original work.

query/show.rs hex fallback replaces raw tuple construction with `SystemId::from_address(&GalacticAddress::from_packed(packed, model.active_galaxy))`. Preserve existing lexical parsing and name-first precedence (no new string grammar/range policy). Thus unqualified packed system lookups resolve in the selected galaxy, including clearing a supplied planet nibble. No fallback to whichever other galaxy exists. Named-base association already calls from_address(base.address); prove distinct bases at same coords0/9 attach to their proper system. Public ShowQuery shape remains unchanged. Duplicate global system names remain explicitly for the next pass.

### 4. Watcher key propagation — GI-05

Keep compute_delta and SaveSnapshot production bodies unchanged: their existing SystemId keys inherit the new identity. In nms-watch/src/delta.rs tests build actual old/new SaveSnapshot values with public fields, using literal explicit-core addresses (or start from existing snapshot and isolate this controlled population). Old contains galaxy0 at coordinates C/planet1; new adds galaxy9 at C with its distinct planet1. Keep player addresses equal and no base changes so counts are attributable. **Call compute_delta(old,new)** and assert precisely one new system and one new planet carrying the galaxy9 key; then apply that result to a GalaxyModel holding galaxy0 and verify both full keys/planet names/biomes and2 systems/2 planets. Reapply the same delta and verify no duplicated indexes/planets. A manually fabricated SaveDelta alone does not prove snapshot differencing. Test both old0→add9 and old9→add0. No removal API or active-galaxy-on-movement policy is introduced.

### 5. Cache archive and semantic transition — GI-06/07

Add public `active_galaxy: u8` to CacheData, storing **model.active_galaxy** in extract_cache_data and restoring it explicitly in rebuild_model. Preserve selected galaxy independently of player position, including no player. Do not derive it from player_state or the first system. Existing indexes/edges are reconstructed via qualified system insertion. No concurrent write/content-hash/performance redesign.

Set schema revision to **2** (CacheData layout adds a field), semantic revision to **2** (qualified identity and selected-galaxy restoration change model meaning). Both current revisions exact-match; magic/header offsets/error precedence/aligned checked decoding, existing public function signatures and startup best-effort lifecycle stay as accepted in Slice05. Public CacheData struct literals now require the active_galaxy field; document this source-compatibility consequence along with the archive change. Update public lib/module docs and every literal current-header test to2/2; retain revision1 as historical predecessor, never silently label its semantics repaired. Tests must select incompatible versions relative to2, not accidentally treat2 as mismatched. Preserve all existing Slice05 cases:0/7/8/15 lengths, magic, schema-first and mismatch-before-payload precedence, malformed current payload, misalignment, legacy, source failure/no overwrite, stale/missing/bypass, write failure and exact populated transport. For direct negatives use schema0/1/3 with semantic2, semantic0/1/3 with schema2; both3 expects SchemaMismatch(found3,expected2). Exact current header is `NMSCACHE\x02\0\0\0\x02\0\0\0`.

Copy the sealed **actual decd8d6-produced predecessor** from S/artifacts/predecessor-cache-schema1-semantic1.rkyv byte-identically into `crates/nms-cache/tests/fixtures/predecessor-schema1-semantic1.rkyv` and use include_bytes! in integration tests. It was generated by the author-probe at schema1/semantic1 with sentinel save_version9999, selected9 (lost on old restoration), and a synthetic galaxy9 system. Verify its SHA-256 against author-baseline.json. Do not generate purported predecessor bytes by current serializing and changing header fields: that cannot witness the old archive layout. The old payload is never decoded by the new reader.

Predecessor direct read/deserialize expects **Incompatible(SchemaMismatch{found:1,expected:2})**, respecting schema-first order. Write it to a temporary cache newer than a valid synthetic source fixture; explicitly assert is_cache_fresh, then load_or_rebuild returns false/source4720, correct cold semantics, literal2/2 rewrite; assert fresh again then warm true/source4720 and same semantics. Make the source's active UniverseAddress.RealityIndex9 using structured fixture JSON mutation (existing active context Main), independently checking cold active9. Its discovery records remain currently decoded0: report that honest mixed population; do not alter decoder to make a desired two-galaxy parsed fixture. Also test a schema2/semantic1 cache with sentinel9999 (current archive payload) for typed SemanticMismatch and actual rebuild/warm, so semantic checking remains discriminating despite simultaneous schema bump.

Separately test synthetic **two-galaxy model transport** through extract/serialize/deserialize/rebuild and a verified-fresh compatible startup read: both orders0/9, both planet1 marker names/biomes, qualified indexes, selected9 preserved even when player address is0 or absent. Direct transport and compatible warm read demonstrate stored model integrity; they do not prove raw game-save extraction supplies both galaxies. Use independent literal field checks in addition to full keyed comparisons. Keep source/cache/temp paths synthetic only.

## Applied guidance, oracles and bounds

| Source/strength | Design application | Proof |
| --- | --- | --- |
| Rust API-37/TD-03 SHOULD | Separate a named domain key from primitive portal coordinates; explicit two components, borrowed existing constructor |0/9 inequality/hash coexist; same-galaxy different planet sameID; SSI and galaxy255 boundaries |
| TD-05 MUST / AP-06 AVOID | Private canonical fields and checked serde prevent bypassing planet-zero invariant | Exact object plus integer/missing/wrong/high-bit negatives through real serde |
| DC-01/02 MUST | Actual public docs explain tuple/serialized/index and cache-layout/API changes, accessor purpose and2/2 process | Source inspection and scoped strict rustdoc links |
| Testing discipline / Proportionate Proof | Real public insert/import/show/snapshot/cache paths and independent marker/count/header observations | Existing suites extended with regression cases; no new reusable validator product |

Representative discriminator: two addresses `0x150003AB8C07` in realities0 and9 must produce **different IDs**, each with system_address literal `0x050003AB8C07`; another planet nibble in the same reality must give the same ID. Insert distinct names Zero/Nine and Lush/Toxic planets both index1, in each order. Require systems2, planets2, graph nodes2, node_map2, address_to_id2, spatial total2 with one in each galaxy; correct per-key data and biome memberships. Repeat insertion must not alter counts or overwrite the accepted same-key data. Select0/from0 then9/from9 and use existing nearest/planet queries to observe the corresponding key/data. Missing selected galaxy gives empty spatial results and packed show SystemNotFound, never the other galaxy's same coordinate. This tests existing selected-galaxy behavior; **an explicit reference differing from active galaxy is held for the subsequent pass**.

Serde positive expected `{ "reality_index": 9, "system_address": <numeric value of 0x050003AB8C07> }`; compare serde_json::Value to independently constructed literal object, not a second serializing call. Round-trip retains both fields, legacy scalar rejected. JSON galaxy255 accepted,256 rejected. Test canonical0 coordinates accepted and planet/high bits rejected. Constructor use across planet nibble0/15 preserves coordinate ID. No new from_packed/save decoder recipe.

Active cache discriminator: model selected9, player0 (and separately None), restored selected9. Matching9 alone cannot distinguish a wrong implementation that derives galaxy from player. Exact2/2 header and pinned actual1/1 predecessor distinguish schema evolution; current2/1 distinguishes semantic rejection. Source4720 vs poisoned9999 and false→true distinguish fallback from serving old data. Assert deterministic freshness at each dependent load.

This pass does not close R06-R02/R06-R03, P01-R02/P01-R03, Arc06 or Project01. Remaining **required subsequent identity work**: duplicate-name scoped lookup; explicit-reference/selected-galaxy query semantics; existing unbounded biome association bug; cross-galaxy route/fallback/waypoint behavior; movement/session notification selection; live update cache and CLI/REPL/MCP composition. Preserve all original correctness/feature/95%-coverage/docs/package/MSRV/performance/distribution gates and field-specific decoder policy. No deferred/no-op status is preauthorized for any current row. If tests expose a gap in the specified foundation, resolve in scope; if a held behavior is needed to satisfy a current criterion, return the contradiction to CDC rather than weaken proof.

## Exact permitted paths

Source edits/commit only the following16 entries (15 existing +1 fixture). Scope annotations are binding:

1. crates/nms-core/src/system.rs — type/API/serde/docs and tests above
2. crates/nms-graph/src/model.rs — qualified address index/docs and key regression tests; no other runtime policy
3. crates/nms-graph/src/spatial.rs — mechanical legacy ID tests only
4. crates/nms-graph/src/route.rs — error Display and mechanical constructor/access tests only; no routing algorithm/signature/policy changes
5. crates/nms-graph/src/edges.rs — mechanical missing-ID test only
6. crates/nms-graph/src/import.rs — additional import tests only
7. crates/nms-query/src/show.rs — packed lookup scope/docs and specified show tests only
8. crates/nms-query/src/display.rs — numeric ID accessor fallback and mechanical/regression tests only
9. crates/nms-cache/src/data.rs — active_galaxy archive field/docs only
10. crates/nms-cache/src/serialize.rs — extraction/restoration/revisions/docs and affected tests
11. crates/nms-cache/src/lib.rs — public cache revision/process/compatibility docs only
12. crates/nms-cache/tests/load_integration.rs —2/2 updates, predecessor/semantic lifecycle and transport tests
13. crates/nms-cache/tests/fixtures/predecessor-schema1-semantic1.rkyv — exact sealed binary copy only
14. crates/nms-watch/src/delta.rs — actual differencing/application tests only
15. crates/nms-cli/tests/show_integration.rs — mechanical accessor update only
16. crates/nms-cli/tests/pipeline_integration.rs — mechanical qualified-index assertion only

No manifests, lockfile, core address/delta fields, raw save/extract decoder, nms-query find/route, graph query, name/base index redesign, watch snapshot/runtime, copilot/session/MCP/CLI runtime, private game/cache/config, main or predecessor worktree changes. Discovering another essential numeric caller/path is a concrete scope drift report to CDC before editing that path; do not smuggle it into formatting.

Planning CC edits/commit exactly four under this slice:

- artifacts/execution.md — intake, implementation, known-limit and attempt record
- artifacts/validation.json — actual source commit; each command's argv/cwd/env/exit, exact stdout/stderr base64 bytes and lengths, parsed test summaries; all failures/retries/blocked/unrun kept
- closing-report.md — eight-row proposed disposition, checks/limits, complete artifact inventory and bubble-up/silent-drop diff, assignment identity, source commit and actual planning intake
- ledger.md — Evidence/Notes only; all criteria/statuses stay open for CDC

Do not change author inputs, issued prompt, slice/parent plans or reviewer evidence. Temporary tools/logs under /private/tmp are permitted, no new validator product. First-class docs stay slice root, supporting outputs artifacts/.

## Validation, commits and return

From new source worktree, **make format then make lint before tests**, CARGO_NET_OFFLINE=true for make gates. Run `cargo test --offline --locked -p nms-core -p nms-graph -p nms-query -p nms-watch -p nms-cache`, then `make test`, `cargo build --offline --locked --workspace --all-features`, and `cargo test --offline --locked -p nms-core -p nms-cache --doc`. Run `cargo doc --offline --locked -p nms-core -p nms-graph -p nms-query -p nms-cache --no-deps` for changed public docs, and `RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links' cargo doc --offline --locked -p nms-cache --no-deps` for the cache links. Independently inspect new core/graph/query links; do not suppress or introduce diagnostics. Author baseline strict combined docs exits101 at unchanged core/address.rs lines25 and495 (four unresolved bracket links). Preserve that evidence and track strict core/workspace docs for the original documentation repair; neither the ordinary scoped build nor strict cache-only pass discharges it. Ordinary author docs also preserve three graph/extract.rs bracket warnings and two query/table.rs HTML-tag warnings, besides four core bracket warnings; all exact channels are in author-baseline.json. Any new diagnostic in changed docs holds the affected row. Baseline824 workspace tests is a reference, not a result to force; derive all actual new denominators,0 failures/ignored expected. Additional current assertions need focused tests; do not merely run an old named filter matching zero cases. Record existing proc-macro-error2 warning and every failed/blocked attempt. Scoped documentation success does not close the known failing strict workspace-doc gate. No broad coverage/no-default/MSRV/package/performance gate is waived or gratuitously repaired here.

Inspect all changed source directly and run whitespace/JSON/digest/scope checks. Preserve other work and use **git commit --only -- <exact files>** for a source commit on the new branch and separate planning commit. New fixture/artifact files must be added before an explicit-path commit. Every assistant-authored commit carries these contiguous trailers exactly once:

    Co-authored-by: Codex <noreply@openai.com>
    Co-authored-by: Billo AI <ai-engineering@billo.systems>

Return both complete SHAs, exact pathsets, gate counts/outcomes/limits and all eight row proposals through the Operator. Verify commit parents, trailers, clean fences, exact binary copy, unchanged main/dependency/prompt/author inputs and no unexpected formatting. CC work remains **proposed-done** pending independent CDC. The selected0/9 synthetic witnesses and cache transport do not establish universal raw discovery/base/nonzero reality semantics or complete multi-galaxy user behavior. No merge/push/release or higher-scale closure.
