# Slice04 — Address repair contract decision packet

**State: recommended / unresolved.** This packet prepares decisions for CDC and the Operator. It is not approval, an implementation prompt, or a production compatibility decision. Source basis is clean main at a800950ef3558fcf463517696c3c140061e7353d; exact synthetic runs are in [behavior-probes.json](behavior-probes.json), and caller/consumer coverage is in [impact-map.csv](impact-map.csv).

## Evidence labels and present contract

| Label | Evidence and scope | What it supports |
|---|---|---|
| Pinned source behavior | nms-core GalacticAddress packs 48 bits as P-SSS-YY-ZZZ-XXX; reality is a separate u8. from_packed/new mask. PortalAddress maps twelve glyph nibbles. Save PackedGalacticAddress deserializes integer/string forms and calls the core 48-bit conversion. Expanded UniverseAddress has RealityIndex plus typed GalacticAddressObject. Discovery/base conversions supply reality 0; player conversion uses expanded reality. | Current implementation and compatibility surfaces only. |
| Accepted game-input evidence | Prior Slice01 player/game-confirmed W-CHILI, inherited by Slice02 and retained by Slice03, is one discovery witness. Slice03 separately supplies 44 bounded one-input Name+Position expanded-field base/endpoint oracles; it adds no new game-confirmed witness or expanded Chili pairing ([W-CHILI](../../slice02-packed-format-authority/artifacts/witnesses.json), [field assessment](../../slice03-decoded-location-witnesses/artifacts/field-assessment.md)). | A narrow witness set; not universal DD.UA semantics, native-ID equivalence, or nonzero packed-galaxy policy. |
| Synthetic current behavior | 45 explicit nms info --save runs: 18/32 packed-family injections and 9/13 expanded cases exited 0; 18 rejected. Seven CLI controls are now recorded: six prescribed and one supplemental; all exited 0. Fixtures derive only from the author baseline. | Current serde/info and CLI routes for these JSON examples. It does not establish game validity or product correctness. |
| Recommendation | The proposals below are candidates for decision. | Nothing is adopted until CDC/Operator disposition and required evidence are complete. |
| Unresolved | Discovery/base decoder warrant, exact game ranges, reality source, public API and identity migration remain unsettled. | A named owner and re-entry evidence are required before dependent production work. |

The candidate save-packed layout is a distinct 56-bit family with an inserted two-hex-digit galaxy field: P-SSS-GG-YY-ZZZ-XXX (planet bits 52–55, SSI 40–51, galaxy 32–39, Y 24–31, Z 12–23, X 0–11). The 48-bit core portal layout is P-SSS-YY-ZZZ-XXX (planet 44–47, SSI 32–43, Y 24–31, Z 12–23, X 0–11). This correspondence is a field-layout hypothesis for decision and oracle design, not a format declaration. The one accepted Chili witness and bounded base-field comparisons do not establish equal discovery/base applicability or all galaxy values.

Slice03's 44 base pairs are field corroborations, not native-ID/game proof. Chili is one game-confirmed discovery witness; scalar equalities elsewhere are correlations; ordinal 174 is expanded-only. No universal or nonzero packed interpretation is inferred. A successful core roundtrip proves only that the core accepts and emits its own 48-bit portal-frame representation. It does not exercise the save parser, establish how a 56-bit value is partitioned, prove a save field's semantic role, or supply a game oracle.

## D01 — Separate save decode from the portal/core address

**Recommendation (unresolved):** preserve GalacticAddress and PortalAddress as the 48-bit portal-frame contract. Add a save-specific packed value and explicit fallible decoder boundary rather than teaching global from_packed that high bits are a galaxy. Treat DD.UA and base GalacticAddress as separate input families until field-specific evidence authorizes sharing the decoder. Proposed shape only: SavePackedAddress(u64) plus decode_discovery(...) / decode_base(...) -> Result<..., AddressDecodeError> returning coordinates and explicitly sourced reality. Do not ship this signature from this packet.

For a candidate 56-bit layout, split the field positions above, validate no bits remain above bit 55, then construct the unchanged core 48-bit coordinate plus separately attributed galaxy. Do not call the masking constructor on the entire 56-bit value and call the result decoded. First collect independent exact component and portal oracles for discovery and representative paired base examples.

**Alternative and cost:** globally changing GalacticAddress::from_packed preserves a small call surface but changes CLI --ga, REPL, MCP, graph extraction, imports, caches and tests that currently treat the value as 48-bit. It also cannot know which save family or galaxy source applies. Keep rejected unless CDC/Operator explicitly approve a compatibility migration.

**Callers/compatibility:** impact-map rows 1–12 and 17–20; direct save extraction, graph IDs, CLI base display and cache consumers. Existing public 48-bit callers remain unchanged under this recommendation.

**Discriminating oracle:** candidate discovery decoding must reproduce the prior game-confirmed W-CHILI portal/components oracle; it is not a Slice03 expanded-field pair. Separately use a representative paired base and all 44 accepted Slice03 base comparisons. Include accepted and deliberately wrong field partitions. A self-roundtrip is not an oracle.

**Owner / re-entry:** CDC validates evidence scope; Operator owns cross-crate/API choice. Re-enter implementation only after independent discovery and base oracle coverage is accepted and compatibility consequences are dispositioned.

## D02 — Packed JSON kinds, lexical forms, width and failure policy

**Fresh characterization:** both discovery and base accepted integer 0, "0x0", "0x212B0004B2446A", uppercase-prefix form, bare "A", numeric-looking string "10", signed integer -1, over-56-bit hex text, and maximum 56-bit integer. Both rejected fraction 1.5, boolean, null, missing member, empty string, malformed "0xGG" and JSON integer 18446744073709551616 (serde reports a floating-point value outside the accepted integer form). Base info shows the low 48 bits: signed -1 and max56 become FFFFFFFFFFFF; the 56-bit witness displays 2B0004B2446A. Discovery output exposes counts, not the injected address. Exact outcomes are in behavior-probes.json.

**Recommendation (unresolved):** an eventual save-specific decoder should accept only enumerated JSON kinds and lexical forms, parse without floating-point transport, require a nonnegative integer no wider than 56 bits or a strictly specified hexadecimal string of at most 14 digits, and reject out-of-width bits before field extraction. Do not silently cast negative integers, accept decimal-looking strings as hex by accident, mask oversized inputs, or turn missing/null/empty/malformed into zero. Separate a representable 56-bit maximum from valid game values. Decide whether invalid data fails the whole save or rejects one record only; do not hide that choice inside serde defaults.

**Alternative and cost:** retaining permissive current serde/casts is backwards-compatible for inputs such as -1, but turns a sign error into all-one coordinate bits and silently loses high bits. Rejecting every noncanonical historical spelling may break existing save fixtures. Migration evidence is needed before narrowing accepted syntax.

**Callers/compatibility:** PackedGalacticAddress and both required record fields; CLI info, graph extraction and import remain distinct. Current info parser outcomes are not recommended semantic policy.

**Oracle:** table-driven JSON-token and string cases at zero, max56, max56+1, max-u64, negative, float, boolean, null, missing, empty, malformed, upper/lower prefix and excess digits; assert exact typed error category/path or exact decoded components. Repeat at discovery and base boundaries; whole-file versus record-skip behavior must be asserted by the chosen contract.

**Owner / re-entry:** save-format/data authority with CDC; Operator decides user-visible failure scope. Re-enter after historical spelling inventory and a game-backed validity source exist.

## D03 — Expanded values, omission/defaults, ranges and previous sentinel

**Fresh characterization:** valid, zero, partial object, absent UniverseAddress, X=2048, SSI=4095 and SSI=4096 parsed; partial fields become zero and absent wrapper follows model defaulting. info prints raw expanded members; it does not show later core-constructor masks. Null wrapper, Y=128, PlanetIndex=256 and RealityIndex=256 failed typed deserialization. A separate conversion test treats previous location as absent when X/Y/Z/SSI are zero, while not checking planet/reality in that sentinel condition.

**Recommendation (unresolved):** preserve raw parser values long enough to validate exact field-specific constraints before constructing the masked core type. Distinguish Rust primitive bounds from game bit widths and confirmed game-valid ranges. Decide requiredness per wrapper/member; do not infer absent means zero without evidence. Decide SSI 0xFFF versus historically documented 0xFFE, planet 0–15 versus valid in-game subset, signed voxel endpoints, and whether RealityIndex must be tied to a known galaxy range. Make the previous-address empty sentinel explicit over every relevant member including planet and reality, with tests for omitted, zero, contradictory and partial forms. A future fallible library API should use a named matchable error identifying path, field and failure class.

**Alternative and cost:** direct GalacticAddress::new is convenient but masks out-of-range coordinates/planet/SSI and hides invalid input. Strict rejection may reject historical saves if current schema defaults are relied upon; permissive defaults can turn damaged records into plausible origin locations.

**Callers/compatibility:** expanded player current/previous addresses, core constructor, CLI voxel and REPL voxel routes; CLI currently bounds X/Z and SSI while REPL does not. Preserve existing Box<dyn Error> functions unless a separately approved API change is made; EH-04 applies to a proposed new library error, not cleanup here.

**Oracle:** every signed minimum/maximum and one-beyond value, SSI 0xFFE/0xFFF/0x1000, planet 15/16, reality endpoint/one beyond, missing/null/empty/partial members; assert exact components, explicit absence, or error. Include a previous wrapper that differs only by planet/reality to settle sentinel behavior.

**Owner / re-entry:** save-format authority and CDC decide semantic ranges; Operator chooses error contract. Re-enter after primary game schema or independently validated fixture distinguishes representable from valid ranges.

## D04 — Source of reality / galaxy identity

**Recommendation (unresolved):** make reality explicit per decoded address and trace its source. For expanded player state, retain UniverseAddress.RealityIndex. For a packed candidate, take GG only if independently confirmed for that record family. Missing, conflicting, or unsupported sources should produce explicit unresolved/error state; do not substitute active player galaxy or zero for every discovery/base. Keep current player context and per-record origin distinct.

**Alternative and cost:** defaulting to active context avoids missing fields but may relabel older discoveries/bases after travel. Always defaulting to zero preserves current behavior but loses non-Euclid identity. Using packed GG without field evidence is an unsupported assumption.

**Callers/compatibility:** discovery/base conversion, player state, graph extraction, MCP hardcoded reality zero, watcher snapshots and cache. Synthetics can exercise proposed logic but cannot establish actual game records.

**Oracle:** conflicting expanded/current-context and record-level reality; two same-coordinate records with explicit galaxies 0 and 9; missing/malformed reality; compare decoded result against independent per-record game source. Reuse Slice03 locations only within accepted provenance; do not manufacture a nonzero packed witness from synthetic CSV.

**Owner / re-entry:** game-format evidence owner and CDC; Operator decides fallback behavior. Re-enter when discovery and base reality sources are independently attested and conflict policy approved.

## D05 — Public API and interface compatibility

**Recommendation (unresolved):** keep existing core 48-bit conversion, portal glyph, signal-booster, and CLI --ga contracts as baseline. Add distinct save conversion only after D01–D04 are accepted. Audit and decide compatibility changes for public PackedGalacticAddress/to_galactic_address, CLI --ga (currently masks a 56-bit input and has a truncation test), REPL --ga (alias for portal glyph parsing), MCP raw address conversion (reality zero and 12-digit output), and CLI versus REPL voxel range checks. Their input names do not currently establish one shared contract.

**Fresh controls:** convert --ga 0x212B0004B2446A succeeds and displays 2B0004B2446A/SSI 2816 (current low-48 truncation); convert --glyphs 212B04B2446A displays the 12-digit portal and SSI 299; convert --ga 212B04B2446A also displays that portal. These are compatibility observations, not a reason to adopt truncation as save decoding.

**Alternative and cost:** redefine --ga globally as 56-bit save input. That breaks existing 48-bit raw behavior and still leaves REPL/MCP inconsistent. Any behavior change needs explicit migration or a separately named route selected by CDC/Operator.

**Oracle:** golden core/portal/signal-booster outputs; separate save decoder exact components; command-specific CLI/REPL/MCP argument kinds, reality propagation and errors. No live protocol behavior is claimed from static source.

**Owner / re-entry:** Operator owns public compatibility/architecture; CLI, REPL and MCP maintainers estimate after API decision. Re-enter after dependency order and caller migration are approved.

## D06 — Galaxy-qualified system and planet identity

**Recommendation (unresolved):** system identity must include reality plus portal coordinates with planet nibble cleared. Compare tuple/newtype key such as (RealityIndex, system-coordinate) with revised encoded u64. Prefer a semantically explicit key in the proposal because SystemId erases reality, but do not select or implement a serialized key before assessing API/cache compatibility. Carry the identity through systems, address_to_id, node_map, planets/biome indexes, graph edges/routes and delta keys.

**Fresh collision controls:** same portal coordinates in CSV galaxies 0 and 9 report one added/one duplicate in either row order. Two distinct SSI glyphs 212B... and 212C... in the same galaxy report two additions. ImportStats does not reveal which duplicate row remains. Source flow inserts first and skips later SystemId duplicates; winner is inference. Current graph multi-galaxy coverage varies coordinates/SSI and does not test identical-location identity.

**Alternative and cost:** changing SystemId bit encoding may minimize map changes but can alter public numeric meaning and every archive/reference. A tuple changes APIs and derived key types more visibly. Both need explicit migration behavior.

**Callers/compatibility:** identity maps, query joins, routes/edges, watchers and serialized caches. Per-galaxy R-trees do not repair a collision in shared maps.

**Oracle:** identical portal coordinates in realities 0 and 9 must yield two systems, planets, lookup/node entries and new_systems and new_planets additions for the newly present galaxy-qualified key; reverse insertion order; retain distinct-SSI same-galaxy control. Assert route/query lookup returns requested galaxy.

Current SaveDelta exposes new_systems, new_planets, player_moved, new_bases and modified_bases; it has no removal channel (crates/nms-core/src/delta.rs:10–60; crates/nms-watch/src/snapshot.rs:1–61; crates/nms-watch/src/delta.rs:1–69). The bounded future sequence compares an old same-coordinate galaxy-0 snapshot with a new snapshot containing galaxies 0 and 9; after approved key migration, galaxy 9 must appear as a newly added system and planet. Reverse insertion roles/order to verify the corresponding addition. Assert player_moved only if the test deliberately changes the player address. Current identity collision should fail this intended proof. No removal test or claim is made; any removal API remains unresolved, outside current implementation authority, and requires Operator re-entry before implementation.

**Owner / re-entry:** Operator chooses public identity/migration; graph owner proposes key. Re-enter after serialized/public SystemId consequences and migration plan are reviewed.

## D07 — Query association, planet glyphs, base display and names

**Recommendation (unresolved):** keep adjacent defects in their existing corrective tracks. The parent closeout route names separate work for address decoding/planet glyphs, planet-to-system query association, lost names, and supported feature combinations. Route save decode plus portal glyph correctness to address-repair implementation after this decision; route biome-only index association to the separate query-association corrective slice; route dropped RawDiscoveryRecord display names to the separate lost-name slice; leave feature/build combinations on their separate feature track. These are planned categories, not issued implementation assignments. Do not move them to Project02 or fold them into this packet.

Current source: biome-only find associates by planet index alone rather than full originating key; FindResult builds portal glyphs from system address, so its planet nibble may not equal the matched Planet.index; CLI info masks base packed values directly to low48; RawDiscoveryRecord carries generic dm JSON but no name field, to_core_record supplies None, and graph extraction ignores dm. The prior F02 synthetic fixture contains SolarSystem and Planet DM.CN names ([baseline report](../../../artifacts/closeout-2026-10-03/baseline-report.md)); these support a bounded future lost-name test, not a current preserved-name behavior claim. See crates/nms-save/src/model.rs:197–239, crates/nms-save/src/convert.rs:7–43 and crates/nms-graph/src/extract.rs:104–181. Correcting packed decoding alone does not fix these consumers.

**Oracle:** two systems with equal planet indexes and only one matching biome must return its originating system; request a nonzero planet whose index differs from system address nibble and require that exact portal; assert base glyphs from independent oracle; verify name preservation from a synthetic named raw record separately from decoded address. Track each test under its future slice.

**Owner / re-entry:** address-repair, query-association, and lost-name corrective owners under parent route; CDC confirms boundaries. Re-enter each only through its own authorized assignment and exact witness.

## D08 — Cache/watch lifecycle and repair sequence

**Recommendation (unresolved):** sequence future repair as (1) settle field-specific decoder and validation/reality authority; (2) settle galaxy-qualified identity and compatibility migration; (3) implement decode and exact unit-level errors/oracles; (4) propagate through extraction, indexes, query/display and watcher keys; (5) add cache semantic compatibility discriminator or explicit invalidation/rebuild rule; (6) run approved cache, watcher and interface composition gates. Existing cache stores already-decoded addresses and save_version/cached_at; freshness compares only mtimes. An old cache newer than its source can appear fresh after decoder semantics change. Rebuild from cache data does not restore active_galaxy from cached player reality.

Before decoder release, either version semantic cache decoding separately from archive schema or force documented invalidation/rebuild when decoder changes. Confirm fresh source is parsed before serving stale semantic data. Decide saved-user-cache migration; none were accessed.

**Alternative and cost:** rely on archive schema version or modification time alone. That misses same-schema semantic changes and old-decoded-but-newer-than-source archives. Always discarding every cache is simpler but adds startup cost that should be measured.

**Oracle:** cache cold build, warm hit, stale, missing and corrupt recovery; specifically load old decoded archive whose mtime is newer than source and assert it cannot bypass new decoder. Assert rebuild restores active galaxy. Compare watcher snapshots for same-coordinate realities 0/9 and partial-write behavior under existing watch gate. Then exercise parser→model→query→CLI/REPL/MCP composition. Live watch, cache, REPL or MCP behavior was not tested by this packet.

**Owner / re-entry:** cache/watch maintainers and Operator; CDC checks source-derived risk. Re-enter after D01–D07 resolve and migration/invalidation policy is approved. Full composition belongs to the parent closeout demonstration gate.

## Future acceptance-to-test map

| Contract decision | Future exact proof | Existing evidence usable within scope | Separate later gate |
|---|---|---|---|
| Save packed fields / D01 | Reproduce prior game-confirmed W-CHILI portal/components oracle; separately test a representative paired base and all 44 accepted base comparisons; wrong partition fails | [Slice02 W-CHILI](../../slice02-packed-format-authority/artifacts/witnesses.json); [Slice03 field assessment](../../slice03-decoded-location-witnesses/artifacts/field-assessment.md); current core portal oracle | Any new expanded Chili pair; full parser-to-interface composition |
| Packed syntax/errors / D02 | JSON-token matrix, signed min/max, width+1, lexical forms, missing/null/empty/malformed; exact error or components | 45 synthetic current-behavior results characterize today's parser only | Game validity and chosen reject/skip policy |
| Expanded policy / D03 | Signed bounds, SSI 0xFFE/0xFFF, planet 15/16, reality bounds, missing/default and previous sentinel | Existing source/schema and synthetic info observations | Primary game schema/range authority |
| Reality / D04 | Same address with explicit realities 0/9 and independent source, plus missing/conflict cases | Synthetic collision counts only; accepted Slice03 rows where fields apply | Nonzero packed game witness |
| Compatibility / D05 | Golden core/portal/signal-booster and command-specific CLI/REPL/MCP exact outputs/errors | Three fresh conversion controls and pinned source | Live REPL/MCP handshakes |
| Identity / D06 | Two same-coordinate galaxies stay distinct in every index, query, route and watcher key; reversed order | Fresh CSV counts show present collision | Public/serialized key migration approval |
| Query/display/names / D07 | Same planet index in distinct systems; selected planet nibble differs; independent base portal and named record | Pinned source | Separate query-association/lost-name assignments |
| Lifecycle / D08 | Old-decoded cache newer than source; stale/corrupt/missing; active galaxy and watcher cases | Pinned cache/watch source | Full parser/model/query/interface and live update composition |

These are proof obligations for a future authorized repair, not tests run or promised APIs in this slice. No new test framework, trait, dependency, edition or runtime is proposed.


## Iteration01 correction note — 2026-10-04

This current packet corrects the source warrants and watcher oracle after CDC review of 74359a92a48669d5ab0368bb51351ac840f1b9fc. The prior version remains sealed at that commit. W-CHILI is prior Slice01 game-confirmed evidence retained by Slice02/Slice03; the separate 44 one-input base/endpoint expanded-field oracles are bounded to Slice03's save. Chili is not claimed as a Slice03 expanded-field pairing. Name evidence is RawDiscoveryRecord.dm and the fixture's DM.CN values; the current conversion/extraction path drops names as described above. Watcher proof now requires observable additions after an approved galaxy-qualified identity migration. Removal behavior has no current delta channel and is not a proposed implementation requirement. All D01–D08 recommendations and named policy choices remain unresolved; this note adopts no policy.
