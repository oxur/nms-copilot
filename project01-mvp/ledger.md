# Project01 closure ledger

Created 2026-10-03 during closeout recovery. Status: **all rows open**. Baseline captures are attested preparation evidence, not final row acceptance. Six recovered arc ledgers hold 69 initial leaf-review rows and six arc-composition rows; full original-document reconciliation may add distinct missing requirements. No denominator may shrink silently.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| P01-A01 | Research arc accepted against all four preserved research documents | Independent arc ledger/leaf walk and provenance review | correctness-grade | arc01-discovery | open | — | No retrospective acceptance |
| P01-A02 | Foundation arc accepted against all original parser/type/CLI promises | Independent arc ledger/leaf walk and parser-to-CLI demonstration | correctness-grade | arc02-initial-infra | open | — | Address contracts must be correct |
| P01-A03 | Model/query arc accepted with correct associations and displays | Independent arc ledger/leaf walk and exact-world queries | correctness-grade | arc03-map-and-query | open | — | F01–F03 undermine current acceptance |
| P01-A04 | REPL/session arc accepted with configuration, history, and cache lifecycle | Independent arc ledger/leaf walk and session lifecycle demonstration | correctness-grade | arc04-repl-and-session | open | — | Interactive behavior must be witnessed |
| P01-A05 | Routing/watch/MCP arc accepted across supported transports | Independent arc ledger/leaf walk and live composition demonstration | serious | arc05-graph-and-mcp | open | — | Help/unit dispatch are insufficient |
| P01-A06 | Fit/finish arc accepted including later UI refinement documents | Independent arc ledger/leaf walk, package/documentation and UI checks | correctness-grade | arc06-fit-and-finish | open | — | Includes original source-tagging promise |
| P01-A07 | Accumulated closeout work has reviewed value/complexity recommendations and an explicit Operator disposition | Arc07 initial results, complete slice inventory, recommendation review and recorded joint decisions | correctness-grade | Duncan's 2026-10-25 reconciliation request | open | — | Three initial slices; cleanup planning follows the discussion; original criteria remain |
| P01-C01 | Real save-packed addresses produce correct planet/system/galaxy through parser, graph, and all interfaces | Independent golden fixtures, nonzero galaxy case, and typed-caller audit | serious | Original coordinate/model promises; F01/F10 | open | [baseline](artifacts/closeout-2026-10-03/baseline-report.md), attested failure | Caller investigation [accepted](arc06-fit-and-finish/slice01-address-encoding-evidence/cdc-verification.md); primary-source investigation [slice02](arc06-fit-and-finish/slice02-packed-format-authority/slice-plan.md) accepted; decoded witnesses [slice03](arc06-fit-and-finish/slice03-decoded-location-witnesses/slice-plan.md) open after iteration01 CDC review (six accepted, one narrative correction row); this composition row remains open |
| P01-C02 | Biome/name/discoverer/location queries retain exact entity associations and names | Golden multi-system fixture; filtered/unfiltered comparison; name-bearing raw records | serious | Search promises; F02/F03 | open | [probe](artifacts/closeout-2026-10-03/biome-filter-probe.json), attested failure | Counting matches alone insufficient |
| P01-C03 | Routes obey warp range, visit targets, and expose usable glyphs | Independent multi-stop itinerary with measured hop constraints | correctness-grade | Phase 4 composition | open | — | Include unreachable and cross-galaxy cases |
| P01-C04 | Cache is correct on hit/miss/stale/corrupt inputs and after live updates | Restart and file-change demonstrations with exact model/state comparisons | serious | Phases 3/5 composition | open | — | Address fixes may require cache invalidation |
| P01-C05 | CLI/REPL/MCP return consistent current data after a stable file update | Real stdio/HTTP initialize and tools; same update observed in REPL/MCP | serious | Shared-query/live vision | open | — | Diagnose duplicate ingestion paths |
| P01-C06 | Setup, history, completion, map and terminal restoration compose into usable interaction | Isolated fixtures and witnessed terminal lifecycle including cancellation/errors | correctness-grade | Later UI documents | open | — | Never alter real game saves |
| P01-C07 | All original scope changes, omissions, and inherited findings have explicit dispositions | Original-doc-to-source scope diff, row counts, deferral reasons/owners/re-entry | correctness-grade | Silent-drop discipline | open | — | Project02 ownership is not automatic deferral |
| P01-G01 | Full relevant tests and lint/format pass at the final candidate | `make lint`; `make test`; affected regressions with independent exact oracles | correctness-grade | Repository gates | open | [gates](artifacts/closeout-2026-10-03/gates.json), attested baseline pass | Must re-evaluate after repairs |
| P01-G02 | Supported HTTP and no-default-feature builds/tests pass cleanly | All-feature/default/no-default matrix, feature-appropriate startup tests | correctness-grade | Published feature contract; F04 | open | [failure](artifacts/closeout-2026-10-03/no-default-test.log) | Four compile errors at baseline |
| P01-G03 | Broad 95%+ line coverage target is demonstrated with a faithful denominator | Workspace coverage report and module gap review; ignored/excluded/unrun cases disclosed | correctness-grade | Original 95% target; F05 | open | [summary](artifacts/closeout-2026-10-03/coverage-workspace-summary.json) | 81.16%; no target reduction authorized |
| P01-G04 | User/contributor/public API docs and examples match delivered behavior | Strict rustdoc; CONTRIBUTING; example execution and package link/content checks | correctness-grade | Milestone 7.6; F06 | open | [strict docs](artifacts/closeout-2026-10-03/strict-rustdoc.log) | Missing docs and broken markup/links |
| P01-G05 | Packages contain required assets/docs and build/install from a clean boundary | Package list, verified archives in dependency order, clean install and binary smoke | serious | Milestone 7.8; F12 | open | [list only](artifacts/closeout-2026-10-03/package-list.log) | Archive/install checks unrun |
| P01-G06 | Historical publication and current release candidate are distinguished with evidence | Registry/version receipt and clean install; exact source/version mapping | correctness-grade | Milestone 7.8 | open | [blocked probes](artifacts/closeout-2026-10-03/published-0.2.0-curl-attempt.json) | No inferred publication failure; no publish authority exercised |
| P01-G07 | Declared MSRV and supported platform gates cover the final candidate | Rust 1.91 checks; exact-commit CI job evidence on declared OS/features | correctness-grade | Manifest/CI contracts | open | [retained CI](artifacts/closeout-2026-10-03/ci-jobs.json) | Older same-revision default tests passed; MSRV unrun |
| P01-G08 | Original performance promises are measured on stated populations | Nearest-10 on 300 systems <10 ms; cached startup <100 ms; record toolchain/data/method | correctness-grade | Milestones 2.2/3.7 | open | — | Unit tests do not establish latency |
| P01-G09 | Final project assembly is independently reproduced and accepted by Operator | Full row walk, arc reports, attributed independent composition gate and Operator verdict | serious | Project closure discipline | open | — | No success records pre-created |

Statuses become `done`, `deferred`, or `no-op` only with the appropriate evidence/rationale. `done` requires independent reproduction for this ledgered closeout; deferred rows require explicit authority, reason, owner, and re-entry condition. This preparation does not discharge any row.

2026-10-25: Arc06 is paused; Arc07 has only its first three slices planned. The
25 prior project criteria retain their complete meanings and statuses; new
P01-A07 is the 26th open criterion. Recommendations are not source-removal,
scope-waiver or release authority. Later cleanup planning awaits joint decisions.

## Approved repair requirements — 2026-10-04

Added by the Operator's explicit scope expansion after Slice04. Original22 rows remain open and unchanged in criterion; total25 open rows. The earlier P01-C01 progress note is historical: Slice03 and Slice04 are now independently closed preparations, while C01 still needs production/composition proof.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| P01-R01 | Field-specific save conversion preserves correct components, planet glyphs and sourced galaxy while existing 48-bit portal/core behavior is preserved | Independent discovery/game and base/expanded oracles through parser, graph and CLI/REPL/MCP; malformed/omitted/range and reality policies demonstrated | serious | Operator approval2026-10-04; Slice04 D01/D05 | open | — | Boundary approved; D02–D04 exact policy/evidence still required; composes with C01 |
| P01-R02 | Same system coordinates in different galaxies remain distinct through all system/planet consumers | Galaxies0/9 collision fixture in both orders; import/query/route/watch/cache and interface assertions against exact expected entities | serious | Operator approval2026-10-04; Slice04 D06 | open | — | Explicit galaxy-qualified key required; composes with M7.4/C01/C02/C05 |
| P01-R03 | Incompatible derived cache semantics never bypass source rebuilding and future repairs carry explicit compatibility disposition | Fresh incompatible cache startup -> source rebuild -> current warm hit; revision/bump review for every decoder/identity repair and live write-through | serious | Operator approval2026-10-04; Slice04 D08 | open | — | Slice05 supplies protection; subsequent semantic migrations and project lifecycle proof still required; composes with C04 |


2026-10-22 Arc06 Slice21 child progress: all8CA independently closed at b290ea31, Arc39rows/21done/18open. All25 project criteria/statuses remain unchanged/open. AdoptedA original-value parsing/application/cache3→4 is delivered under bounded evidence; synthetic nonzero is not a game-backed nonzero witness and child gates do not substitute final parent/assembly/Operator acceptance. Current child source/retrospective/evidence are linked from Arc06's R06-S21 row.

2026-10-22 Arc06 Slice22 issued under explicitly approved raw-nameA: sixNF/R06-S22open;40arcrows21done19open/project25open. Opt-in foundation only; required23 real application/live/cache5/native and24 full originalM7.1export remain. All25 project criteria/statuses unchanged, no parentacceptance or broader evidence inferred.

2026-10-22 Arc06 Slice22 all6NF/R06-S22 independently closed at753bb3a9,40arcrows22done18open. All25projectcriteria/statuses remainopen; opt-infoundation only, required23application/cache5/native+24originalexport and all parent/highergates retained. FreshCDC12groups/1043runtime+30docs and substantiveCDCretrospective linked from child, no integration/release/game-backed inference.

2026-10-25 Arc06 Slice30 core documentation child accepted: sixCD/R06-S30done under exact approved generated-field A, source40e6/independent actual output proof. All25project rows remainOPEN, including G04/fullM7.6 pending required31–33; Arc06 now48rows35done13open. No parent/integration/release acceptance implied.

2026-10-25 Arc06 Slice31 accepted: sixSD/R06-S31done onb60e, seven independent groups/130runtime+37docs/strictsavezero.49arc36done13open; all25projectrows stayOPEN including fullM7.6/G04/planned32–33, platform/finalrepair/Operator gates.

2026-10-25 Arc06 Slice32 accepted: sixGD/R06-S32done on e30e, seven independent groups/186runtime+13docs/strictgraphzero,50arc37done13open; all25projectrows remainOPEN. FullM7.6/required33query+34remaining/fullreconciliation and every higher gate retained.

2026-10-25 Arc06 Slice33 issued: sixQD/R06-S33OPEN, exact8querypaths/11payloadlayouts;51arc37done14open. All25projectcriteria/statuses remainOPEN, fullM7.6/required34 and allhighergates retained; sourceimplementation/acceptance not yet claimed.

2026-10-25 Arc06 Slice33 accepted: all6QD/R06-S33done at6e5af862, seven independent groups/96runtime+7executingdocs/strictzero;51arc38done13open. All25projectrows OPEN; fullM7.6/required34/fullreconciliation/allhighergates retained.

2026-10-25 Arc06 Slice34 issued: sixWD/R06-S34OPEN, exact5watchpaths/3payloadlayouts;52arc38done14open. All25projectrowsOPEN, fullM7.6/required35/allhigherowners retained, no productimplementation/acceptance atissue.

2026-10-25 Arc06 Slice34 accepted: all6WD/R06-S34done at65ac646d, sevenindependentgroups/42runtime+7docs(3execute/4compile)/strictzero.52arc39done13open. All25projectrowsOPEN; fullM7.6/required35/fullreconciliation/allhighergates retained.

2026-10-25 Arc06 Slice35 issued: sixKC/R06-S35OPEN, exact7cache/compatpaths/8payloadlayouts,53arc39done14open. All25projectrowsOPEN/fullM7.6/required36/M7.5input/allhighergates retained; no productimplementation/acceptanceatissue.

2026-10-25 Arc06 Slice35 accepted: all6KC/R06-S35done at48b6bd79, sevenindependentgroups/57runtime+8docs(5execute/3compile)/strictbothzero.53arc40done13open/all25projectrowsOPEN; fullM7.6/requiredSlice36/allhighergates retained.
