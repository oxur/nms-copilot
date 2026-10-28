# NMS Copilot — project roadmap and backlog

Version: 1.3 — 2026-10-27. Owner: Duncan, with Sofie maintaining the record.

Duncan approved the original six-project grouping after Project01 closure and subsequently added Procedural Narratives as an unnumbered group. This records delivery boundaries and retained wishes; it does not start implementation, commission research, choose dependencies, or authorize publication. Future projects remain unnumbered except the existing Project03 reservation. Ordering follows prerequisites and Duncan's priorities, not a directory sort.

## Repository convention and authority

This file is the repository-wide roadmap/backlog, at the root of the existing orphan `planning` worktree, beside `projectNN-<slug>/` directories and any future repository-wide status directory. There is no `status/` directory in this checkout at intake. Collaboration-framework 3.0.1 specifies project/arc/slice records but no canonical repository-wide backlog; this file is the local convention adopted for Duncan's request to record the grouping at this level.

Keep high-level project boundaries, prospective features, priorities, dependencies and re-entry conditions here. Detailed delivery plans and acceptance live in their owning project/arc/slice records. Link original evidence and decisions rather than copying their packets. Moving an item into a project requires explicit scope agreement; a backlog entry is not an execution assignment. Maintain destinations here alongside reconciled project plans, retaining stable references and decision history. Do not silently discard or renumber prior requirements.

Enduring discovery notes for prospective work live in `backlog/<slug>/README.md` beside this file, on the same `planning` branch/worktree. Duncan proposed this category on 2026-10-27 to preserve discussion and source leads before project planning. This file is its index; create additional files within a topic only when useful. These topic directories are not numbered projects or execution assignments. When work gains a delivery home, link the owning plan and retain the original notes and evidence distinctions.

## Accepted project grouping

| Group | Scope | Finish line | Dependencies / boundary |
|---|---|---|---|
| **Project02 — Your saves and searchable atlas** | Friendly accounts/saves, explicit selection and safe switching; operational inherited settings/output; reliable intake and backend reconnection; durable observations/history; supported discoveries and companion origins; structured queries/analytics; existing CLI-to-REPL parity | Choose the intended save, find previously observed worlds/creatures, explain the supporting evidence and preserve that knowledge across updates/restarts; restore the authoritative data this project introduces | Consume accepted Project01 foundations. No notebook/media, culture geometry or undiscovered-star enumeration gate. Existing-query improvements do not require a catalog. |
| **Project03 — Client and gameplay integrations** (reserved) | Specifically selected external clients and an initial gameplay/conversation integration, using existing MCP capabilities | Agreed integrations work end to end; additional clients/mechanisms require scope decisions | After Project02, as already reserved. Duncan chooses targets. Claude Desktop, Claude web and other Anthropic apps remain excluded. Does not inherently require the later notebook/culture projects. |
| **Adventure notebook** (unnumbered) | Multiline notes; grouped/chronological views; tags/collections; managed screenshots; captions; explicit clipboard/file capture; attachment lifecycle and portable journals | Record experiences while playing and restore text, images, authorship and associations after moving/restoring the atlas | Consumes atlas identity/storage and supported subjects. File input and native clipboard/display need distinct acceptance. No culture mathematics requirement. |
| **Procedural Narratives** (unnumbered) | [NMS GM and procedural storytelling](backlog/nms-gm-and-procedural-storytelling/README.md): geocaching, clue chains, mystery expeditions, generative tables and attributed GM guides; [Gameplay session history](backlog/nms-gm-and-procedural-storytelling/README.md#gameplay-history-and-intent): recent visits, travel evidence and low-noise summaries; [Screenshot sensing and companion presence](backlog/nms-gm-and-procedural-storytelling/README.md#screenshots-as-observations): notification, image analysis, classification and possible in-game GM chat | Play an agreed hunt or story with persistent quest state, evidence-backed progress and a useful player handbook; develop through actual play toward deeper mysteries | Builds on supported atlas/history, notebook and selected gameplay integrations. Project02 retains its existing history scope; this row does not expand it or require culture geometry. Source/license research, capture/notification/control feasibility, exact sequencing and delivery scope remain to be agreed. |
| **Traveller cultures** (unnumbered) | Stable named cultures; custom nouns; founding/history; evidence-bearing discovery affiliations; distinct narratives; base/system summaries; historical connections and qualified region information | Describe and revisit cultures without merging histories or confusing visits/bases with discovery ownership | Consumes atlas identity/provenance and notebook links where used. Stable culture identity must remain useful without geometry. Region derivation is conditional on supporting research. |
| **Spatial evidence and exploration research** (unnumbered) | Candidate-star availability; coordinate precision; region derivation where needed; boundary/cohesion/completeness methods; relevant library evaluation | A bounded, evidence-backed feasibility decision, supported populations/limits and an implementation contract | Separate research from delivery promises. Agree concrete questions, consumer, budget, stopping conditions and unavailable-result disposition before commissioning. Can inform later projects without blocking Project02. |
| **Culture analysis and exploration tools** (unnumbered) | Accepted geometry; reproducible culture analyses; interior gaps/frontiers/corridors; ranked exploration suggestions | Agreed exploration journeys return useful, explainable results against independently checked examples | Consumes accepted research and culture/atlas inputs. No promise of complete undiscovered-system enumeration without supporting evidence. Reuse existing routing. |

The full notebook, culture and exploration capabilities move out of Project02. Their requirements are retained, not cancelled. New project numbers, exact sequencing and detailed plans remain to be agreed. Project03's reservation does not make it a prerequisite for all later work.

## Project02 shape — reconciled in v2.0

These accepted capability boundaries are formalized in the [current Project02 plan](project02-personal-atlas/project-plan.md#six-arc-roadmap), with an explicit old/new arc map. They are not opened arcs or source assignments.

| Proposed order | Capability | Stop boundary |
|---|---|---|
| 1 | Recognizable saves and safe selection: inventory, aliases, remembered selection, explicit overrides, coherent switching and REPL `saves` | No database prerequisite or broader UI redesign. Startup preference and exact selector behavior still need confirmation. |
| 2 | Working controls and consistent interfaces: inherited settings, human/plain/JSON output, existing-command REPL parity, name filtering and existing-data detail/statistics | No configurable theme system or rich-entity extraction. Existing export/import parity is included; journal/media interchange belongs later. |
| 3 | Reliable observation capture: source/context/time evidence, stable intake and bounded watcher reconnection | Consume accepted Project01 repairs; extend address handling for newly supported fields where needed. No repeat of the original repair programme. |
| 4 | Durable atlas and history: accepted database, idempotent ingestion, snapshots, migration, retention, backup/restore and concurrent interface access | No media store, journal editor or geometry engine. Trial the actual atlas workload; future journal/geometry workloads are not Project02 acceptance gates. |
| 5 | Discoveries, companions and historical search: explicit entity/field support, origin traversal, recent-world filters, history comparisons and cross-save reads | No generic promise to expose all save data. Define a finite supported entity/query set before implementation. |
| 6 | Whole-product acceptance and delivery: agreed journeys, recovery/performance checks, documentation and integrated usable delivery | Composition and agreed residual gaps; no open-ended fit-and-finish mandate. Publication remains a separate authority. |

Maintain a useful end-to-end increment in each arc; apply interfaces, documentation and checks as behavior lands. Roughly 4–8 slices is a sizing expectation, not a quota. Material growth calls for reconsidering the boundary with Duncan. Detail arcs near execution, rather than pre-authoring every future slice.

## Accounting for the existing Project02 ledger

The [reconciled ledger](project02-personal-atlas/ledger.md) retains all24 original IDs and their original rows as dated history. Current population is26 criteria:19open and7explicitly deferred, with B03/B04 added as P02-25/26. The [six-arc plan](project02-personal-atlas/project-plan.md) explicitly maps the former nine arcs; later notebook/culture/exploration delivery no longer gates Project02. No implementation criterion is done or waived. The mapping below accounts for every original criterion and retained cross-project obligation.

| Existing criterion | Destination / retained meaning |
|---|---|
| P02-01 | Project02: friendly selection and switching. Reserve separate play/read/authorship identities without implementing the later notebook. |
| P02-02 | Project02: trusted new observation fields and context; consume accepted Project01 address/galaxy foundations. |
| P02-03 | Project02: durable ingestion/history and accepted database decision. |
| P02-04 | Project02: bounded supported rich entities and evidence-bearing relationships. |
| P02-05 | Project02: scoped structured search, history and atlas analytics. |
| P02-06 | Notebook: journals/media/tags/collections and their interchange. Project02 retains existing CLI/REPL import/export parity and agreed atlas interchange; define any new import formats explicitly. |
| P02-07 | Each delivery project owns its documentation, supported builds, performance and composition. Project02 closure no longer depends on the notebook or culture/exploration projects. Publication is separately authorized. |
| P02-08 | Project02: chosen save to Chili's correct origin/galaxy, with switching and source explanation. |
| P02-09 | Project02: retained disappearing-discovery evidence and honest date/absence semantics. |
| P02-10 | Project02: coherent concurrent CLI/REPL/MCP catalog access and owner-death recovery. |
| P02-11 | Every project restores all authoritative state it introduces. Project02 covers registry/archive/catalog; notebook adds journals/media; culture project adds authored culture state. |
| P02-12 | Project02 reconciles its inherited scope and findings; moved features retain destinations and conditions here. This is not a requirement to implement every backlog item. |
| P02-13 | Traveller cultures: identity/history/affiliation and summaries. Region claims remain qualified by research. |
| P02-14 | Research establishes feasible geometry/population capabilities; exploration-tools project delivers the accepted implementation scope. No enumeration blocker on Project02 closure. |
| P02-15 | Notebook: frozen target/author, composition/cancellation, drafts and exact journal views. |
| P02-16 | Notebook: managed images, display/fallbacks, physical usage and reference-aware lifecycle. |
| P02-17 | Project02: explicit cross-save read scope and distinct observations without changing selection. Notebook adds separate journal authorship/narratives. |
| P02-18 | Project02 preserves supported/unresolved discovery-owner evidence. Traveller cultures consume it without inferring ownership from visits or bases. |
| P02-19 | Research selects and checks mathematical measures; exploration tools implement the accepted methods and uncertainty contracts. |
| P02-20 | Research evaluates region semantics/derivation; traveller cultures present supported region evidence separately from culture identity. |
| P02-21 | Project02: inherited name search, planet detail/seed, statistics/flags and info context. |
| P02-22 | Project02: operational inherited configuration, consistent colors/plain/JSON. Configurable themes remain separate. |
| P02-23 | Project02: existing CLI-command REPL parity, including `saves` first and existing import/export/completions. New notebook interchange stays with notebook delivery. |
| P02-24 | Project02: bounded notify backend reconnection with lifecycle and duplicate-event correctness. |

## Shared design constraints

- A shared world does not merge save/traveller histories. An evidenced synced replica does not create another traveller; unresolved identity remains explicit.
- Discovery attribution, visits, bases and authored narratives remain distinguishable. Cross-save queries do not silently change active play or authorship.
- Every project backs up/restores its own authoritative data; later storage responsibilities extend that contract without retroactively blocking earlier closure.
- Conditional DuckDB adoption and the SQLite fallback remain design decisions, not installed dependencies. No second authoritative catalog is implied.
- Culture identity survives changing geometry and late spatial connections. No user radius or ship warp range defines a culture.
- Missing observations cannot establish globally undiscovered stars, exact geometric positions or territorial completeness. Research outputs must identify their supported populations and limits.
- Preserve useful Project01 code and independently accepted evidence. Apply direct user interaction checks early; do not commission new readiness/evidence machinery merely because prior work was expensive.

## Retained backlog and re-entry

Original evidence, owners and detailed conditions remain in [Project01's backlog](project01-mvp/backlog.md). Current allocation below records Duncan's accepted grouping; Project01's backlog has a dated routing note and preserves the original deferral evidence.

| Item | Destination / trigger |
|---|---|
| B01 — Clean installation | Existing evidence accepted for now; re-enter for an actual publication blocker or a future project's concrete install need. No automatic install campaign. |
| B02 — Configurable themes and biome colors | Future UI work when Duncan schedules concrete scope; outside Project02's required configuration/output repairs. |
| B03 — Conditional 32-bit LZ4 offset overflow | Schedule a bounded correction in Project02, satisfying the existing next-two-project deadline. Meaningful malformed-length/platform evidence; no universal parser hardening programme. |
| B04 — Jump-count range/cast/sum arithmetic | Schedule a bounded correction in Project02, satisfying the existing next-two-project deadline. Resolve consequential policy with Duncan; preserve the distinction from route construction. |
| B05 — Best-effort overlong final route leg | Future routing work when scheduled or directly encountered. No newly assigned deadline or automatic exploration-project scope. |
| B06 — Search/detail/statistics/info | Project02, after friendly selection; existing-data work has no database prerequisite. |
| B07 — Working configuration/output | Project02 after friendly selection; configurable themes stay separately deferred. |
| B08 — Existing CLI-command REPL parity | Project02; `saves` first, then existing import/export and shell-script completions. Interactive TAB completion is already implemented. |
| B09 — Named-client/gameplay integration | Existing Project03 reservation after Project02, with targets selected by Duncan and Anthropic apps excluded. |
| B10 — Legacy GoatFungus/SpookyHash | Actual problematic legacy/export input or concrete compatibility use case. No fixed project. |
| B11 — Reachability visualization | Future explicitly scheduled routing/visualization use case. Not automatically added to exploration tools. |
| B12 — Notify backend reconnection | Project02 observation intake after friendly selection; distinguish backend recovery from ordinary file replacement recovery. |
| Authentic NomNom verification | Re-enter when an attributable full export is available; no manufactured verification claim. |
| FTS / Tantivy | Optional search extension when a concrete structured/name-search limitation justifies it. No project-completion gate or adopted dependency. |
| Cross-restart last-query history | Future feature only if scheduled; current-session `last` is delivered by Project01. No new persistent history requirement for Project02. |
| OCR, semantic/vector search, cloud account APIs, native console containers and save editing | Outside the approved delivery roadmap. Record a separate use case/scope decision before any adoption. |

## Next planning work and decision history

- **2026-10-27:** Following Project01 closure, Duncan explicitly requires splitting the broad Project02 vision into multiple projects, then approves the proposed six-project categorization and bounded Project02 shape. Preserve Project03's reservation; leave other project numbers unassigned. B03/B04 are allocated to Project02; earlier conditional/future deferrals retain their qualifications.
- **2026-10-27:** Duncan requests a durable high-level backlog beside project/status records. Adopt this root `backlog.md`, link it from Project02 and mark the old roadmap/ledger as awaiting reconciliation. No future project directories or status system are created.
- **2026-10-27:** Reconcile Project02 v2.0 plan/ledger/design and dated research consumers; retain24 original ledger rows as history, record7transfers and add2arithmetic rows (19open/7deferred). Refresh accepted Project01 inheritance and explicitly map old/new arcs.
- **2026-10-27:** Duncan requests preservation of the earlier coplayer/GM discussion after a read-only coverage check finds only partial representation in Project02 artifacts. Adopt `backlog/<slug>/README.md` for enduring discovery notes; capture GM/storytelling, source-material leads, session-history detail and screenshot/chat possibilities without changing the approved project grouping. This is a discussion synthesis, not a verbatim transcript or completed source/license review.
- **2026-10-27:** Duncan corrects the destination after the notes were published on `main` in `a866d6d`: the roadmap and detailed notes belong on `planning`. Add the unnumbered Procedural Narratives row to Accepted project grouping, carrying the three linked topics into its Scope column; supersede the separate discovery-topic table. Preserve the detailed note under `backlog/nms-gm-and-procedural-storytelling/`, restore local planning links and remove the duplicate index/note from `main`. No existing project acceptance criteria change.
- **Next:** Agree remaining Arc01 startup/selector/identity choices, prepare that arc near execution and select any execution arrangement explicitly. Later entity/query field coverage and catalog trial budgets are settled at their owning boundaries. Later projects receive plans when priority/prerequisites make that useful. This record issues no implementation prompt, research trial or contributor/model assignment.
