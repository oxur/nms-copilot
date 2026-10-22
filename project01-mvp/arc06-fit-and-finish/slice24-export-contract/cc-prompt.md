# Slice24 CC implementation — complete original M7.1 export contract

2026-10-22 UTC. Standing human-authorized Autonomous Project01 Arc06 ONLY, next required original0037 M7.1 after accepted23/source2f2814/planningbc329c7. Same actual CDC `/root/cdc_arc06` gpt-6.1-sol High / sole existing CC `/root/cdc_arc06/cc_slice17` gpt-6-luna High. You confirmed qualitative headroom/no owned processes for this self-contained unit; preserve accepted23 handback. No new role/model/inquiry/dependency/cache/architecture/otherarc/Project02/integration/release authority. This is a legitimate new unit, initial0of5 ordinary refinements; old23history/cap/exceptions unchanged and no future waiver.

## Coherent intake, ownership, guides

Read this ENTIRE prompt, slice-plan/ledger/source-allowlist, complete original0037 (M7.1 binding; M7.2/7.3 separately retained), source AGENTS, complete export.rs, CLI registration/dispatch and resolve paths, shared query execute_find/filter/order/reference behavior, affected CLI tests and complete README export section. Retain already-read current framework3.0.1/shared/autonomous invariants/PMREADME+operationguides/prompt-authoring/verification/testing/fullRust/AP guidance; refresh any unavailable portion, not substitute this packet. Tests are Rust and require the same quality discipline. Review every changed helper/callsite/assertion/fixture sequence, safe source assignment (actual JSON objects), literal oracles and error/cleanup interactions BEFORE runtime. No new empirical investigation needed.

ONLY source `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/export-contract`, branch codex/export-contract, initial clean/indexempty2f2814c780f5022dea3bc1643d99828ba679e8d0. Planning is separate orphan worktree and CDC-owned. Revalidate source/main/held fences and no active owned process. Mainb616 protected user9line nativeblob2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e; all prior accepted/held/external03e8d04 pins unchanged. EVERY patch uses absolute path mechanically checked against allowlist; EVERY shell explicit cwd. Never mixed-root relative writes. No main/other worktree/index mutation. Supply full readback/currentassignment, source/index/no-owned-process/headroom and planned first action before implementation; CDC's issued assignment authorizes proceed after coherent readback. CC then sole source/index/gate owner; CDC planning/independent review. First format/lint then HOLD all source/index/gates for CDC pinned review before runtime.

Exact THREE paths: new crates/nms-cli/tests/export_integration.rs; existing crates/nms-cli/src/export.rs **cfg(test) only**; README.md **export prose/examples section only**. Freeze all production executable bytes, imports outside testmodule, CLI/query/parser/model/cache/Cargo/dependency/protocol/datafixture/native controllers and all old assertions. Additive tests/docs; if a real production defect emerges, HOLD and report exact reproduction, not repair silently. Original23 all names/checkedparse/lifecycle/cache5 controls stay unchanged. No native harness/extra private diagnostic needed.

## Delivery and source-grounded engineering

Current export already calls parse_validated_save_file/from_validated_save, builds shared FindQuery with biome/infested/within/nearest/named/discoverer/from, emits pretty JSON+newline or csv::Writer records. It uses SYSTEM address P0 in portal/coords, SYSTEM discoverer, PLANET name/biome/infested. Shared query selects current active galaxy or base's galaxy, applies all filters BEFORE nearest, sorts distance then numeric galaxy/system address/planet index. from names are bases, not arbitrary address strings. Same system distance0 even physicalplayerP2; Euclidean otherwise. Names from approved source CN now work. ExportFormat parse is case-insensitive. EmptyCSV currently zero bytes, explicitly permitted by original0037; do not invent header requirement/change behavior. CLI stdout only payload, diagnostic errors stderr; no output before query/parse errors. Preserve current firstduplicate/missing names/source/galaxy/address semantics. No new cache revision.

Use existing assert_cmd cargo_bin_cmd!("nms") actual child, timeout10seconds per child, isolated TempDir/HOME and explicit --save. Capture whole Output status/stdout/stderr (assert success before parse), UTF8 exact; success stderr empty and no stdout ANSI/log/table banner. Existing csv/serde_json are normal deps available to tests; no added package. Entire temp source bytes and parent directory filenames before/after each scenario remain identical (export is read-only), including error runs. Resource ownership via TempDir and assert_cmd bounded child; no detached helpers/threads, globalenv or shared mutable files. Preserve all existing tests/legacy fixtures; new source derives a CLONE of literal save56_minimal JSON only for mandatory root shape, then supplies explicit original-JSON fields/packed strings below. Do not generate fixtures or expected fields with GalacticAddress::packed/from_packed, productionextractor/query/display/ExportRecord/writer. Fixture raw lexical fields must themselves be asserted exactly, not only the output.

### Independent literal fixture / complete twelve-field records

Set Main current UniverseAddress GG0 X0/Y0/Z0/SSI1/P2 (physical P2, projectionP0), remove PreviousUniverseAddress safely via object.remove, preserve valid inactive context from save56_minimal. Replace discovery records entirely with literal pairs, and bases with cloned valid shape whose Name/address assigned explicitly. Root Version4720. OWS shape remains complete (LID/UID/USN/PTK/TS), TS1700000000; system USN below, planet OWS deliberately different e.g. PlanetOwner to prove system discoverer. Raw DM.CN set below except explicitly absent; no raw mutation/adoption policy change.

| System | Literal save56 system UA, GG and tuple | Planet source UAs / data | System CN / system OWS.USN |
|---|---|---|---|
| A | `0x00010000000000`, GG0,(0,0,0,SSI1,P0) | P1 `0x10010000000000`, P2 `0x20010000000000`, P3 `0x30010000000000` | `Alpha, "δ"\nHome` / `Explorer, "δ"\nOne` (actual newline) |
| B | `0x00020004000003`, GG0,(3,4,0,SSI2,P0) | P1 `0x10020004000003` | `Beta` / `Bob Builder` |
| C | `0x00030004000FFD`, GG0,(-3,4,0,SSI3,P0) | P1 `0x10030004000FFD` | CN missing / USN empty |
| Nine | `0x00020904000003`, GG9,(3,4,0,SSI2,P0) | P1 `0x10020904000003` | `Nine Decoy` / `NineOwner` |

A P1 and P3 have IDENTICAL name `Ocean, "α"\nMoon`, VP["0xAB",0] / ["0xAC",0], Lush,false; distinct planet indices yield TWO identical projected export objects that MUST retain multiplicity. A P2 CN absent, VP["0xAD",65537], Toxic,true. B P1 `Meadow`, VP["0xAE",0], Lush,false. C P1 CN absent, VP[] (unknown biome,false). NineP1 `Nine Moon`, VP["0xAF",4],Frozen,false. Atleast one exact duplicate later A P1 row with contradictory CN/biome/OWS is retained as a firstduplicate discriminator: expected effective record stays original, not extra planet; initial source row-order and resulting multiplicity pinned. No second system duplicate needed.

Two bases: `Anchor` address `0x10020004000003` (B/P1/GG0), `Nine Base` address `0x10020904000003` (Nine/P1/GG9); remaining mandatory basefields cloned from valid fixture, unique UID values, no automatic model/native preparation. Assert both literal source addresses. Current origin A; fromAnchor B; fromNine only GG9. No same-voxel differentSSI uncertainty in these cases.

Every JSON row has EXACT these12keys with exact types, no additions/missing/nested fields:
`planet_name,biome,system_name,distance_ly,portal_glyphs,portal_emoji,coords_x,coords_y,coords_z,galaxy,discoverer,infested`.
Expected planet and system texts as above (missing→emptystring), biome Lush/Toxic/empty/Frozen, coords signed integers, galaxy Euclid/GG0 or Eissentam/GG9, discoverer SYSTEM value, infested boolean. distance_ly is numeric f64, exact0.0/2000.0/2400.0 as appropriate; do not coerce strings/bools/null. Portal is SYSTEM P0 uppercase twelve hex, not physical P2/planetP1:
- A `000100000000` / literal emoji `🌅🌅🌅🕊🌅🌅🌅🌅🌅🌅🌅🌅`
- B and Nine `000204000003` / `🌅🌅🌅😑🌅🌜🌅🌅🌅🌅🌅🦕`
- C `000304000FFD` / `🌅🌅🌅🦕🌅🌜🌅🌅🌅🔺🔺🚀`
No production portal/emoji/distance helper computes expected values. Full JSON expectedobjects built from these independently supplied constants are allowed. Whole parsed Value equality enforces keysets/values/types/order/multiplicity. A2 and C have empty planet_name, C empty biome/system_name/discoverer. Initial ordered rows [A1,A2,A3,B1,C1], distances[0,0,0,2000,2000]; A1==A3 exact full objects, array length5. Nine does not leak into current query.

### Closed real-child matrix (run EVERY success case JSON AND CSV)

Expected outputs below refer to whole literal row objects, with origin-adjusted distance only. CSV full decoded record vectors must equal independently expected12strings in exact order (including false/true and floating `0.0`/`2000.0`/`2400.0`); no set/subset/dedup/count-only assertion. JSON and CSV compared to independent expectation, not merely each other.

| Flags besides explicit save/format | Exact ordered expected rows |
|---|---|
| none | A1,A2,A3,B1,C1 |
| --biome Lush | A1,A3,B1 |
| --infested | A2 |
| --named | A1,A3,B1 (named SYSTEM A does not admit unnamed A2) |
| --discoverer eXpLoReR | A1,A2,A3 (case-insensitive substring) |
| --discoverer bUiLd | B1 |
| --within 1999 | A1,A2,A3 |
| --within 2000 | all5 (inclusive boundary) |
| --nearest 4 | A1,A2,A3,B1 (B/C distance tie, numeric SSI tie break) |
| --nearest 0 | empty |
| --biome Lush --named --discoverer bOb --within 2000 --nearest 1 | B1 (all filters BEFORE truncation) |
| --from Anchor | B1(0),A1(2000),A2(2000),A3(2000),C1(2400) |
| --from Anchor --within 0 --nearest 1 | B1(0) |
| --from Nine Base | NineP1(0), Eissentam, same B portal/coords but separate labels/discoverer |
| --from Nine Base --biome Lush | empty (no GG0 leakage) |
| --biome Dead | empty |
| --discoverer Absent | empty |
| --within=-1 | empty, preserve shared legacy negative policy |
| --within NaN | empty |
| --within NaN --nearest 2 | A1,A2, preserve shared documented behavior |
| --within inf | all5 |

Also default omitted --format equals JSON semantics/newline and uppercase JSON/CSV accepted (one populated complete rowvector each). Empty independent fixture with Record[]/validcurrent/bases[] returns `[]\n` JSON, zeroCSV; filter-empty and nearest0 same. No writing header or changing error behavior.

### CSV exact byte/escaping oracle, discriminators and errors

CSV header EXACT newline-terminated:
`planet_name,biome,system_name,distance_ly,portal_glyphs,portal_emoji,coords_x,coords_y,coords_z,galaxy,discoverer,infested\n`.
For baseline full5, additionally assert ENTIRE stdout bytes against one independently written literal golden (not produced by csv::Writer or a clone of productionalgorithm). Comma/quote/newline fields are quoted, inner quotes doubled, real newline stays inside field, LF record terminators, Unicode UTF8 and no BOM/extra banner. A1/P3 identical byte rows both retained, A2 empty name, C missing strings/negativeX/unknownbiome exact. Decode via csv::Reader too, exact header all12columns and exact ordered fieldvectors. Every filtercase still wholevectors; no .lines() row count on quoted newlines.

Make pure validators return Result/boolean (not panic-catch) so meaningful negative controls accept the true observed baseline, then reject a mutated copy for EACH12field (wrong value/type; infested stays boolean unless explicit wrongtypecontrol), dropped/extra key, reordered rows, dropped one duplicate, extra duplicate, wrongP1portal, wrong galaxy/name alias and leaked decoy. CSV validator rejects wrong/header reorder, one-field alteration, removed/extra duplicate; exact golden rejects malformed escaping. These prove oracle discrimination, do not call expected as actual in product tests. Keep valid assertions intact.

Real-child errors in BOTHrequestedformats where applicable: unknownformatxml, invalidbiomeNotABiome, absentbaseMissing Base, nonexistentfile, malformedJSON, typed57bit discoveryUA `0x100000000000000`, missing current UniverseAddress, invalid nearest `--nearest nope` (clap exit2 vs ordinary runtimeexit1). Assert actual status nonzero with exactexpectedcode1/2, EMPTY stdout and stderr contains specific matching diagnostic/category/path (pin actual source-defined wording, never claim absent successpayload only), no panic/successbanner. Null-negative/parserlegacy controls elsewhere remain untouched. Do not turn current validated-save errors into empty successful data. sourcefiles/directories identical before/after all success/error children. Optional no-save/autodetect/global --slot exhaustive M7.3 remains separately owned; explicit --save with --slot0 precedence can be one populated control without touching resolution policy.

In existing export.rs TESTMODULE ONLY add writer-failure tests for BOTH write_json/write_csv using a deterministic Write returning io::ErrorKind::BrokenPipe, assert attemptedwrite and propagated actual error (unwrap_err allowed tests), no swallowed-error/partial-success. Preserve all existing11tests. No production writer change. README exportsection describe twelveflatfields/JSONdefault andCSV/filters/namedPLANETS/basegalaxy selection/systemP0projection/stdoutpayload+stderrerrors/zero-byteemptyCSV; bounded concise text, no claim import work or universal game-backed evidence complete. Read surrounding and complete affected section to avoid stale opposite sentence. No full README rewrites or unrelated wording.

## Capture, required gates and pinned hold

Before FIRSTactualcompiler/gate/diagnostic read full `/private/tmp/nms-slice24-cc-capture.py` and currentattempts. It is proven raw-first kernel copied with new dedicatedroot/record; author controls36f22015f1b64837adcfb61f78b92332 exit7 andc56685ec3b144f659baf5792d616c1d1 exit0 both retain stdout10/stderr6, hashes/endUTC/explicitbeforeafterstatuses/clean2f2814completeidentity verified. Supportonly/no productcredit. All attempts UUIDimmutable, argv/cwd/inheritedenv/originalTMPDIR/startUTC/beforeHEAD+status+completeworkingidentity (diff/untrackedinputs), rawstdoutstderr/exits/end FIRSTsaved BEFOREaftermetadata/postprocessing. Store FULL outer toolobject FIRST, preserve sessionid then correct polling; no inferred exit from empty output. Any actual private syntax/compilerdiagnostic must use capture from its first attempt, but no private helper is needed here. Ordinary source/helper/status/inventory is not a SUTgate or waiver; finaldiffgate captured. Preserve failed/retried attempts and original prefix/JSONvalues.

After edits captured makeformat→makelint, then HOLDsource/index/gates for CDC ENTIREthreepath/closedmatrix/helper/assertion/docs review BEFOREruntime. Compiler selfrepairs withinbounds allowed with full records/renewpair, no production change. CDC corrections immutable siblingcc-prompt-iterationNN;5limit no inheritedexception. After CDC release exact required ordered groups:
```
cargo test -p nms-cli --test export_integration
cargo test -p nms-cli
cargo test -p nms-query
make test
cargo build --all-features --workspace
cargo test --doc --workspace --all-features
RUSTDOCFLAGS='-D warnings' cargo doc -p nms-cli --no-deps
git diff --check
git diff 2f2814c780f5022dea3bc1643d99828ba679e8d0 --check
```
RUSTDOCFLAGS via kernel overrides. Directhost only for established fs/loopback substrate under existingcontract; otherwise retain restrictedfailure then IDENTICAL argv/cwd/env/originalTMPDIR/source hostretry, no loweredtests/concurrency/timebounds. No extra native13, no applicationproductionchanges; accepted23native/runtime remains byte-applicable. No coverage/install/network/remote. New producttest counts observed not predicted. Repeat only changed/failed/unresolvedproof; record-only edits no runtime reruns.

After gates, exactnamedpathlocalcommit only actualallowlistchanges via `git commit --only -F messagefile -- exactnames`, metadataescalation, proper consecutive once-parsed trailers Codex<noreply@openai.com> andBilloAI<ai-engineering@billo.systems>. Verify clean/indexempty/parent2f2814/exactpaths, freeze source/index/gates/noownedprocess. Return ignored workbench/slice24-return/{execution.md,validation.json,closing-report.md}: attributable intake/currentassignment/readback/eachiteration, fullliteralworksheetcasecount/actualrecords/channels/sourcecounts/failures/bytehashes, proposedall6EXdispositions with concreteoracles, namedcommit/trailers/working→committedchronology and precise artifactinventory; answer parentcompliant?/structuralchange?/whatbubbles? Preserve existing23returns/history unchanged. CDC transfers byteexact, independently reviews/reproduces onsettledcommit, accepts/retrospects/closes/bubblesup. CC proposal is not acceptance. Original M7.1mayonly close after fullleafreconciliation/currentindependentproof; originalM7.2/M7.3/import/format/UI/docs/distribution/coverage/performance/game-backednonzero/raw/history/finalsemantic/freshness-identicalrewrite-staleimport/higherindependent/Operator/integration/release gates remain with currentowners. No otherarc/Project02.
