# Project02 — Your saves and searchable atlas

Version: 2.0 — 2026-10-27. Project: `project02-personal-atlas`.
Status: reconciled project design; no active implementation arc or issued prompt.
Depends on: accepted Project01 parser, galaxy-qualified model/query/cache and application-coordinator capabilities at the applicable source revision. Blocks: the existing Project03 integration reservation's after-Project02 gate; later projects consume the particular atlas inputs they need. Related: [repository roadmap/backlog](../backlog.md), `project01-mvp`.

Duncan approved splitting the former nine-arc vision into six projects and authorized this reconciliation. This document now defines Project02's bounded delivery. The notebook, traveller cultures, spatial research and exploration tools have retained destinations in the repository backlog; Project03 keeps its integration reservation. Their production delivery or feasibility does not gate Project02 closure. Technical mechanisms remain proposals where stated. This update authorizes planning, not source implementation, research trials, publication or automatic advancement into another project.

## Purpose and finish line

Choose the intended save by a recognizable name, find previously observed worlds or creatures, understand the evidence behind the result, and preserve that knowledge across save updates and application restarts. The motivating journey is selected save → Chili → origin planet → system → galaxy/portal, with honest uncertainty and provenance.

The project ends when its six arcs compose into the finite journeys and ledger below, the resulting capability is integrated on its agreed delivery baseline, and independent composition review plus Duncan's acceptance pass. It does not wait for a notebook, image store, culture classifier, undiscovered-star enumeration, external client or new publication. The atlas cannot recover evidence that vanished before it was observed or supplied.

## Accepted starting point

[Project01 is closed at its amended scope](../project01-mvp/cdc-verification.md). Accepted source: `01b601f49253987b1b9c596ad0fb1e01cb757dda`, planning close `8a56a405af459d945b6de508086b6d4c1bd0c131`. This reconciliation checked the clean source candidate and its parser/coordinator/MCP entrypoints; it did not rerun Rust tests or audit the whole source tree.

Consume checked `SaveParser`/`NmsSaveParser`, field-specific validated address handling, galaxy-qualified identity and routing, compatible-cache rebuilds, the shared live-save coordinator, extracted `nms-mcp`, current-session `last`, repaired completion and accepted map behavior. These are existing capabilities, not new Project02 tasks. Historical inspections in [source findings](artifacts/source-findings.md) remain pinned history; current inheritance is described in that document's dated update. Project01's coverage, documentation, installation and startup qualifications remain scoped to that acceptance, not universal guarantees or automatic Project02 waivers.

Before source delivery, verify the actual branch contains the required accepted capabilities and record drift/integration ownership. Acceptance of a candidate is separate from main integration; this planning update does not assert that PR14 has merged. Retain the existing Linux/macOS/Windows CI populations and declared Rust 1.91 MSRV unless Duncan explicitly approves a change. Native console containers remain outside synced-file support.

## In scope and bounded data coverage

Friendly accounts/saves come FIRST and ship without a database. Preserve explicit physical-file selection; introduce readable inventory, aliases, disambiguation, remembered selection and safe switching. Account owner, recorded player, logical save, replica, snapshot and main/expedition context remain distinguishable. Cross-save reads never silently switch the active save. Reserve narrative identity without implementing journals.

Deliver the inherited B06/B07/B08/B12 commitments: working configuration/output, existing-query detail/statistics, existing-command REPL parity and notify backend reconnection. Add the explicitly allocated B03/B04 bounded arithmetic corrections. Neither correction authorizes a general parser or routing redesign. The stronger best-effort route-leg guarantee remains B05 outside this project.

The initial extraction boundary is the following finite set; exact supported field paths and accepted fixtures are specified in Arc03/05 before dependent implementation:

| Family | Project02 behavior | Limit |
|---|---|---|
| Systems and planets | Galaxy-qualified identity, supported names, locations and planet seed/detail; source-specific evidence | Similar names/seeds do not establish shared identity. No population of unobserved stars. |
| Discovery records | Preserve record type, available name/value/owner/time evidence and resolved or unresolved object links | Raw retention does not promise typed querying for every flora/mineral/other payload. Publish a field-support table. |
| Companions | Save/context-scoped pet instance, nickname/species evidence and supported native-origin linkage | Do not infer stable pet identity from slot alone or universal species identity from a name. No breeding/genetics/editor features. |
| Bases and teleport destinations | Retain their existing location/name/provenance behavior and supported historical relationships | Do not expand into base construction, settlement/fleet management or automatic ownership transfer. |
| Main/expedition contexts | Preserve source scope, distinguish active/retained contexts and query supported evidence across them | No expedition objectives/rewards/progression tracker or universal lifecycle simulation. |

Project02's query families are readable inventory/selection; existing name/detail/info/statistics; companion-origin lookup; recent-world filters; cross-save comparisons; present/absent/changed history; and declared entity/claim/observation aggregates. Filters use supported name, save/account/context, biome, infestation, source/owner evidence and explicit date basis. Missing fields remain unknown; biome and infestation are separate. No unrestricted SQL/MCP, arbitrary graph traversal language, semantic search or FTS requirement. New families require an explicit scope decision.

## Acceptance journeys

| Journey | Observable finish |
|---|---|
| Choose among the household's four saves | In-game names/local aliases are useful across two accounts and Xbox-origin/Steam-origin replicas. Last-writing platform is not origin. Duplicate/missing names and unavailable sources resolve visibly; scripts never prompt or silently choose another account. |
| Switch without mixing sessions | Save/context, model, reference location, completions, watcher and cache generation move together. Failure retains a usable prior session; late old-save events cannot overwrite the new one. |
| Use the existing atlas consistently | Required settings affect consumers with documented precedence; plain/piped output is ANSI-free, structured JSON represents the same domain result, and existing commands have REPL parity with shared semantics. |
| Find Chili's origin | Select a save, follow supported companion evidence to planet/system/galaxy and return independently expected portal glyphs, uncertainty and source/context. Switch away/back without losing the association. |
| Find recent lush or infested worlds | Match biome and infestation independently; select date basis/range and save/context explicitly. First observed is not automatically discovered/visited time. Missing evidence is explained. |
| Retain disappearing discoveries | Present → absent → present observations remain queryable. Partial/failed extraction cannot prove deletion. Re-import/re-extract does not multiply discoveries or erase prior sources. |
| Query and recover the atlas | CLI/REPL/MCP read equivalent committed generations; contention, owner death, interrupted import/migration and storage limits preserve recoverable state. Restore registry/archive/catalog to a clean app directory and rebuild disposable projections. |

Use direct native interaction with Duncan during friendly-selection delivery, then exercise the later search journeys as they become usable. Component tests and actual consumer checks complement one another; do not postpone every user interaction to the final arc.

## Six-arc roadmap

No arc directories, slice plans or assignments are opened by this roadmap. Each arc owns usable entrypoints, documentation and validation for its changes. Approximate sizing is 4–8 coherent slices with context/review headroom, not a quota. Unexpected expansion or repeated correction without convergence returns to Duncan for a scope decision; it does not automatically add slices.

| Arc | Useful capability and destination | Required accepted inputs | Exit evidence and stop boundary |
|---|---|---|---|
| **01 — Recognizable saves and safe selection** | Shared inventory/identity/registry service; names/aliases; chooser/list/status; exact overrides; migration; switching; REPL `saves` and MCP scope | Accepted parser/model/session baseline; agreed startup preference | Four-save/duplicate/missing/conflicting/copied/synced fixtures; CLI/REPL/MCP resolution; safe failed switch/migration and real native chooser witness. No catalog, general configuration overhaul or notebook. |
| **02 — Working controls and consistent interfaces** | Operational inherited settings and color/plain/JSON; existing name predicate, seed-bearing planet detail, statistics/info; remaining existing CLI-command REPL parity; B04 jump arithmetic correction | Arc01 selection/result scope; existing query/routing capabilities | Literal config precedence and output equivalence, exact name/seed/count/bin/flag/Game Mode results; existing import/export/completions parity; meaningful invalid-range/overflow consumer tests. No configurable themes, new rich extraction or stronger B05 route guarantee. |
| **03 — Reliable observation capture** | Source/context/time and absent/null/malformed states; raw locators/extraction versions; stable capture; bounded notify recovery; B03 compressed-offset correction | Arc01 source/identity contract and accepted Project01 address/coordinator behavior; Arc02 not a technical prerequisite for this arc | Independent new-field/galaxy/time/partial-data fixtures; backend failure→re-register→subsequent update, cancellation/shutdown and duplicate controls; malformed-length boundary/platform proof for B03. No database, historic-address re-investigation or star-population research. |
| **04 — Durable atlas and history** | Accepted catalog engine; snapshot archive; idempotent ingestion/re-extraction; migrations/retention; coherent multi-interface access; backup/restore | Arc01 registry and Arc03 stable observations; agreed bounded database trial | Actual atlas-workload build/target/contention/restart/replay/migration/storage-limit checks and restore to a clean directory. No journal/media or speculative geometry workloads. |
| **05 — Discoveries, companions and historical search** | The declared rich entity set and field-support table; provenance-bearing joins; creature origins, recent-world/history/cross-save queries and atlas aggregates | Arc03 field/source contracts and Arc04 committed catalog; Arc02 existing shared output/query behavior | Independently expected Chili and recent-world examples; exact absent/partial/conflicting and counting populations; equivalent CLI/REPL/MCP answers, stable pagination and unchanged active save. No all-save-data or undiscovered-system promise. |
| **06 — Whole-product acceptance and delivery** | Compose agreed journeys on the integrated delivery baseline; supported-target/build/performance/recovery evidence and documentation | Applicable accepted results from Arcs01–05 | Independent project composition, current ledger row walk and Duncan acceptance. Apply bounded actual-consumer checks, retain limits, disposition findings. No general fit-and-finish mandate or automatic new publication. |

Arc01 is first. Existing-data improvements in Arc02 need no catalog; Arc03 also needs no database. This is a delivery roadmap, not permission for parallel contributor work. Later arcs consume the accepted capabilities they actually need rather than reopening every earlier gate.

### Explicit mapping from the pre-split roadmap

The former arc numbers were project-level design only: no Project02 child prompts or arc directories were issued. This version openly changes their roadmap meanings; historical ledger origins and research references retain their dates.

| Former roadmap (v1.5) | Current destination |
|---|---|
| Arc01 — accounts plus appended general settings | Arc01 selection; settings/output move to current Arc02. |
| Arc02 — addresses/observations plus watcher recovery | Current Arc03; consume accepted Project01 repairs rather than duplicate them. |
| Arc03 — catalog/history | Current Arc04; notebook/media workloads move to their project. |
| Arc04 — entities | Current Arc05; existing-data planet detail moves to Arc02. |
| Arc05 — search/history/analytics | Current Arc05; existing predicates/statistics/info move to Arc02. |
| Arc06 — journals/media/interchange | Adventure notebook; existing CLI/REPL import/export and agreed atlas interchange stay in Arc02/04. |
| Arc07 — composition/release/completions | Current Arc06 composition/delivery; shell-script generation parity moves to Arc02. Publication stays separate. |
| Arc08 — traveller cultures | Unnumbered traveller-cultures project. |
| Arc09 — culture geometry/exploration | Separate unnumbered spatial-research and exploration-tools projects. |

## Architecture and decisions at their owning boundary

Keep authoritative profile registrations/aliases/selection separate from retained raw save snapshots and normalized catalog observations. Existing graph/cache and optional future search indexes are rebuildable projections. Public domain results do not expose database rows, Arrow types or connection lifetimes. Reuse existing parser, coordinator, query and CLI/REPL/MCP boundaries; add focused modules/crates only when their responsibility justifies it. See [current design](design.md).

Prefer DuckDB only if its actual build, interactive workload and process-sharing trial passes. SQLite remains a fallback decision; do not ship two authoritative catalogs. A DuckDB owner/worker/local endpoint is a proposed response to its constraints, not an architecture to implement before the trial justifies it. If it costs more than the atlas benefits, choose the simpler passing catalog. Library versions in [research](artifacts/research-and-library-decisions.md) are dated leads to refresh at adoption.

Resolve decisions when their consumer approaches:

| Boundary | Decision / stop condition |
|---|---|
| Before Arc01 implementation | Startup preference, final friendly selector grammar, identity evidence/replica-link rules and supported root inventory. Proposed default is reopening the last deliberate selection; user preference remains pending unless separately recorded. Unknown identity is allowed; no online lookup or automatic cross-root merging. |
| Before Arc02 consumers | Existing-setting precedence and exact output/statistic flag semantics; B04 invalid-range/overflow policy where consequential. No unrelated config-location/logging migration. |
| Before Arc03 field/recovery changes | Finite supported field paths/fixtures, missing-state and time semantics, bounded recovery/cancellation rules and B03 failure behavior. |
| Before Arc04 trial/adoption | Agree question/workload, explicit effort limit, host/target matrix, decision thresholds and stop/fallback conditions. Set archive data layout/default limit before continuous archival; preserve history and visibly pause at the limit. Define cross-process access against the chosen engine. |
| Before Arc05 | Freeze supported entity/field/query coverage within the finite families above. Unsupported associations remain searchable unresolved evidence; capability additions return to scope discussion. |
| Before Arc06 close | Revalidate integrated source/CI and finite native journeys; distinguish debug warm headless readiness, REPL first prompt, full-save startup and cold starts. Accept or resolve explicit evidence limits without relabelling one measurement as another. |

The old geometry/candidate/clipboard research gates belong to later projects and are not prerequisites for these decisions. No trial, benchmark, dependency update or outside-data acquisition starts from this planning pass.

## Definition of done and authority

All six current arc capabilities compose into the journeys above; required source is integrated on the agreed delivery baseline; and [all current Project02 acceptance rows](ledger.md) are independently dispositioned with Duncan's acceptance. Current-project delivery criteria remain open; explicitly transferred rows retain a destination, reason and re-entry condition. No amount of future notebook/geometry work is required to close the accepted atlas scope. Backup/restore covers registry, archive, normalized catalog and any other authoritative data actually introduced here; future projects extend their own contract.

Retain supported-target CI, relevant MSRV/feature/package checks, meaningful regression oracles and current-project coverage/documentation decisions. Plan these as behavior lands; final acceptance checks composition and residual gaps. Fixes, evidence limitations and optional improvements receive separate dispositions. Existing B01/B02/B05/B09/B10/B11/NomNom/FTS qualifications stay in the [shared backlog](../backlog.md). B03/B04/B06/B07/B08/B12 are explicitly assigned here. No graph database, save editing, native console-container adapters, cloud account API, OCR or semantic/vector search is included.

This is planning-only work by Sofie. The default Two-Contributor implementation/review separation remains available; no Project02 role/model dispatch or Autonomous endpoint is selected by this amendment. Record the actual contributor arrangement, explicit model/settings and authorized endpoint before Autonomous execution; Project01's closed arc authorization does not automatically carry into Project02. Independent implementation acceptance uses the selected workflow's normal records; project closure needs independent composition and Duncan's acceptance. Publication and external actions retain their separate authority.

Use the existing orphan `planning` worktree and `project02-personal-atlas/`. Root `backlog.md` holds cross-project scope/destinations; project records govern Project02 delivery. Keep future design papers in their current historical homes until deliberately maintained; this update creates no later project directories or speculative slice plans.

## Version History

- **1.0 — 2026-10-03:** Initial seven-arc design packet (version label recorded retrospectively); friendly saves first, conditional DuckDB, durable history and personal knowledge, optional FTS.
- **1.1 — 2026-10-11:** Operator design discussion, not a child close, supplied adventure journals/media, distinct save narratives, discovery-based traveller cultures, graded geometric cohesion, late adjacency, and cross-save exploration. Expand Arc06; add Arcs08/09; retain Arc07 ID and move its release dependency gate after them. Add research gates and acceptance rows without closing any. Earlier conversational proposals for manual claim/cherished flags, shared multi-traveller culture affiliation, and configurable-radius culture definitions are superseded. Explicitly reconcile the old Project02 address-repair placement with the subsequently authorized Project01 Arc06 responsibility. No Project01 records or source are changed.
- **1.2 — 2026-10-26:** Duncan explicitly transfers Project01 reconciliation groups2 (search/detail/statistics/info) and3 (working configuration/output) into this NEXT project. Consolidate in existing Arc01 after friendly selection and Arc04/05; no database prerequisite for existing-query work. Retain all existing arcs, dependencies for richer capabilities,20 ledger IDs/statuses and B02’s separate theme deferral; add future-open P02-21/22. No source work or implementation assignment authorized.
- **1.3 — 2026-10-26:** Duncan transfers existing CLI-command REPL parity to this project (Arc01 saves early, Arc06 export/import, Arc07 shell-script completions); add future-open P02-23 while preserving all22 prior criteria/statuses. Reserve external-client/gameplay integration scope for Project03 AFTER this project, explicitly excluding Claude Desktop/web/Anthropic apps; no new project directory/plan. Last-results and separate nms-mcp crate remain mandatory Project01 deliveries, not completed or transferred here. Information requests do not waive other scope.
- **1.4 — 2026-10-26:** Correct the last-results note: current-session most-recent-successful result semantics are now agreed, delivery remains Project01 responsibility/open; cross-restart result history is future, with no persistent backend/new command invented. Project02’s23 criterion IDs/statuses and delivery scope remain unchanged.
- **1.5 — 2026-10-26:** Add Duncan’s notify backend-reconnection transfer in existing Arc02 after friendly Arc01 selection, with future-open P02-24. Bounded recovery/resumption/lifecycle/duplicate correctness is distinct from ordinary replacement recovery and Windows CI. Preserve all23 prior IDs/statuses and existing arc order; no implementation or new arc. Strict-positive2opt proof remains Project01 required work; visualization B11 has no fixed project/deadline.

- **2.0 — 2026-10-27:** Duncan-approved multi-project split, recorded in root backlog, replaces the former nine-arc closure boundary with six capability arcs. Explicit old/new arc map; friendly saves first; retained next-project transfers and allocated B03/B04; finite data/query families and staged decisions. Consume closed Project01 at accepted01b601f, including SaveParser/last/separate MCP/coordinator; no duplicate repair or unresolved-current-CI claim. Seven old ledger criteria move to named future destinations; the remaining/current criteria stay open, with two new arithmetic rows. Later notebook/culture/research/exploration delivery no longer gates Project02. No source work, trial, new arc or contributor assignment.
