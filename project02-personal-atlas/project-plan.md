# Project02 — A personal atlas that remembers

Version: 1.5 — 2026-10-26 (initial design: 2026-10-03)

Project: `project02-personal-atlas`. Status: design, no active implementation arc.
Depends on: the existing MVP and independently accepted Project01 correctness foundations before dependent delivery. Blocks: none declared. Related: `project01-mvp`.

The Operator authorized this augmentation following the journal/culture design discussion. Confirmed requirements are distinguished from proposed mechanisms and unresolved research. No Project02 implementation is claimed or assigned.

## Purpose

Make NMS Copilot easy to point at the right save, then build a durable, explainable atlas across that player's saves and expeditions. A user should select “main - Xbox,” find a creature's home world, understand which save supplied the information, and revisit discoveries that later disappear from the game's current records.

The atlas can preserve only evidence it has observed or the user has supplied. It cannot promise to recover discoveries that vanished before a snapshot was captured.

It also serves as an adventure notebook and an atlas of a traveller's cultures: write experiences while playing, attach screenshots, preserve distinct narratives across saves, and analyze the territories that emerge from discoveries. The motivating play pattern is solitary black-hole travel, choosing a compelling world, then exploring its system and an expanding surrounding volume, repeated across four main saves on two accounts. The reported 500–1000 ly buffers describe play, not a configurable culture-definition radius.

## Confirmed priorities and proposed scope

The user requires friendly accounts and saves **before** creatures, detailed expedition support, and richer searching. Full-text search is lower priority. Rust library research and a comprehensive design are authorized; implementation is not part of this design pass.

Place the save/account work in **project02 Arc01**. It establishes identity and selection rules needed throughout the larger project and can ship independently of a database. Keep project01's delivery history intact. Consume the independently accepted Project01 address/galaxy/cache repairs in Arc02 and extend their observation contracts where necessary. **Supersedes the 2026-10-03 proposal to carry the existing address defect into Project02:** the Operator subsequently required those repairs in Project01 Arc06. Project02 neither duplicates that assignment nor discharges its gates.

Proposed project scope extends beyond the initiating planet/creature search: multi-save selection, provenance, historical observations, durable annotations/imports, relationships, structured queries, analytics, recovery, and consistent CLI/REPL/MCP behavior. The particular entity set and later arc boundaries remain reviewable.

Confirmed additions from the 2026-10-11 discussion:

- Discovery is the atomic claim; no separate claimed/cherished flag is required. Base presence/counts are useful evidence of attachment, without imposing a sentimental score.
- Each save has a distinct narrative. A shared world does not merge visits, bases, or journals; a synced replica does not create a new traveller. Recorded account owner and narrative traveller are different concepts until evidence links them.
- Cultures belong to particular travellers and develop spatial definition through accumulated discoveries. Different travellers' adjacent territories stay distinct. Names and arbitrary collective nouns (culture, empire, cluster, etc.) describe them.
- Cohesion/completeness should be derived from the arrangement and ownership of systems. A user-selected neighborhood radius or ship warp range must not define culture identity. Mathematical methods and score calibration remain research, not settled algorithms.
- Two historically distinct cultures may later adjoin; retain their names, histories, and dated connection evidence. Do not silently merge them.
- All applicable atlas queries support explicit cross-save/account scope while active play and journal authorship remain independently scoped. FTS remains optional.

Mechanism details and acceptance examples live in [journals/media](artifacts/journals-and-media.md) and [cultures/exploration](artifacts/cultures-and-exploration.md).

Confirmed transfers from Duncan’s 2026-10-26 Project01 reconciliation decisions:

- **Search, detail and statistics:** expose the existing shared name predicate consistently in CLI/REPL/MCP; add standalone planet details including seed; provide distance histograms, discovery-type and named/unnamed statistics with meaningful flag semantics; include biome overview and the active save context’s Game Mode in `info`. Existing system/base queries and basic statistics remain the starting point.
- **Working configuration and output:** make the currently inert default galaxy, `tsp_algorithm`, `find_limit`, `emoji_glyphs`, `table_style` and `save.format` settings operational; provide consistent color controls, true ANSI-free/plain piped output, and general structured JSON beyond export. Configurable themes/semantic biome colors retain their separate [B02](../project01-mvp/backlog.md#b02-configurable-themes-and-biome-wiring) deferral; this transfer does not reschedule it.

These are **next-project delivery commitments**, owned by the explicitly assigned Project02 contributors with CDC independent review and Duncan prioritization/acceptance. Re-entry is actual Project02 delivery: friendly account/save selection remains FIRST, followed by Arc01 settings/output foundations; Arc04/05 carry planet detail and search/statistics. Existing-query wiring may follow Arc01 directly and has **no catalog/database prerequisite**. Only richer catalog/history/entity capabilities retain their existing dependencies. See [source findings](artifacts/source-findings.md#2026-10-26-two-explicit-project01-transfers) and future-open P02-21/22. No implementation, new arc/slice or prompt is issued by this amendment.

### REPL parity and onward integration scope — 2026-10-26

Duncan also transfers the existing CLI command set’s REPL parity to this NEXT project ([B08](../project01-mvp/backlog.md#b08-cli-command-parity-in-repl-next-project), future-open P02-23). Keep friendly saves FIRST: add REPL `saves` early in existing Arc01 alongside readable account/save inventory and selection; place REPL `export`/`import` in existing Arc06 interchange; place REPL `completions` in later Arc07 polish. Here `completions` generates shell scripts; existing interactive TAB completion is a separate implemented capability. Preserve shared command/domain semantics rather than invent another query engine. **Owner/re-entry:** explicitly assigned Project02 contributors with CDC independent review and Duncan acceptance, at those actual delivery boundaries. No new arc, prompt or implementation assignment.

**Project03 reservation only:** named-client/gameplay integrations move AFTER Project02 to a reserved `project03`, owned by Duncan for targets/priority and a future explicitly assigned contributor for delivery ([B09](../project01-mvp/backlog.md#b09-named-client-and-gameplay-integrations-project03-reservation)). Claude Desktop, Claude web and other Anthropic apps are explicitly excluded. Keep current generic stdio/HTTP MCP implementation and protocol evidence; these are distinct from external-client/gameplay acceptance. Re-enter after Project02 when Duncan chooses integration targets and authorizes the bounded project. No Project03 directory, full plan, source assignment or automatic implementation is opened.

Last-query results and extraction into a separate `nms-mcp` crate are **required Project01 closeout deliveries**, not transferred here. Duncan has confirmed actual most recent successful query result within the current REPL session; cross-restart result history remains future. Result delivery/API and crate dependency design still need implementation readiness/independent acceptance; no completion is claimed. Existing cache fallback, representation/history and SaveParser/library questions receive no new scope waiver or disposition from an information request.

### Notify backend recovery transfer —2026-10-26

Duncan explicitly transfers notify backend-error recovery to this NEXT project ([B12](../project01-mvp/backlog.md#b12-notify-backend-reconnection-next-project), future-open P02-24). Existing Arc02 observation intake, AFTER friendly Arc01 account/save selection FIRST, owns bounded re-registration/resumption and shutdown/cancellation/duplicate-event/ingestion correctness. This is distinct from already-tested ordinary file replacement/deletion/corruption recovery and the unresolved Windows CI6h timeout. No catalog/database prerequisite is added for this existing watcher behavior. **Owner/re-entry:** explicitly assigned Project02 contributor, CDC independent review and Duncan acceptance during actual Arc02 delivery. No backend-recovery implementation, successful proof, new arc/slice/prompt or immediate source assignment.

## Motivating acceptance journeys

| Journey | Required behavior |
|---|---|
| Choose the Xbox main save from a Mac | Show the in-game name, useful identity evidence, last-saved platform, and context; do not force numeric account IDs into ordinary interaction. |
| Switch between household saves | Resolve duplicate names explicitly; keep player identities, local account folders, and saves separate. Never select the first account merely because it sorts first. |
| Recover Chili's home world | Follow companion → origin planet → system → galaxy; return trustworthy portal glyphs with source/context and uncertainty. |
| Find recent lush or infested worlds | Filter by biome and infestation separately, with an explicit date basis and scope. Distinguish first observed by Copilot from discovered or visited in-game. |
| Inspect disappearing discoveries | Compare observations and retain earlier evidence. “Absent from this snapshot” does not erase historical records or prove the object ceased to exist. |
| Use knowledge regularly | Reopen the chosen save, update from stable files, retain notes/imports, explain conflicting names, and query through all three interfaces. |
| Explore relationships and patterns | Join companions, discoveries, bases, locations, and observations; summarize counts and changes without duplicating shared-world objects or counting every snapshot as a new discovery. |
| Capture an experience while playing | Infer the target from the selected save's latest accepted snapshot, show freshness, freeze that target while composing, allow Escape cancellation, and retain the note after reimport. |
| Browse a local adventure journal | List notes for all supported subjects in the current system, grouped by subject type, or view a chronological journal; filter by subject or explicit cross-save scope. |
| Keep screenshots with the story | Import files or explicitly capture a supported image clipboard; list/show images, report physical disk usage, and restore managed media with the atlas. |
| Revisit a world in a different save | Share its world identity but show two attributed narratives and base histories; broad queries never change the active traveller. |
| Understand a culture | Show its discoverer, named history, arbitrary collective noun, unique system counts, bases, region evidence, and mathematically derived structure with coverage limits. |
| Plan the next exploration | Find interior gaps/candidates, connecting corridors, and outward frontier candidates; distinguish unknown data from verified discovery status and report both frontier and travel distances. |

## Roadmap

These are capability arcs, not implementation assignments. “Depends on” means an acceptance gate, not that research cannot overlap.

| Arc | Capability and boundary | Depends on | Exit evidence |
|---|---|---|---|
| 01 — Accounts, saves, and selection | Shared inventory/identity service; in-game names and local aliases; readable chooser/list/status; REPL saves parity alongside selection (P02-23); explicit selection; migration of existing configured paths; safe switching; diagnostics. AFTER friendly selection: working inherited settings, consistent color/plain/JSON output foundations (P02-22). No database prerequisite. | Existing MVP | CLI and REPL demonstrations with duplicate names, multiple roots, missing metadata, unavailable files, and a synced cross-platform save. Shared selector and MCP scope tests; distinct play/query/authorship scopes and traveller/save/replica identity contract; settings-to-consumer behavior and output-mode equivalence checks after selection. |
| 02 — Trustworthy addresses and observations | Separate address encodings; preserve galaxy; model source/context, absent/null/malformed fields, timestamps, and extraction versions. Establish sanitized fixtures and raw-record locators; discovery-owner attribution and coordinate-resolution evidence. Consume accepted Project01 repairs; notify backend-error recovery with bounded re-registration/resumption and lifecycle/duplicate correctness (P02-24). | 01 | Independent expected-address fixtures including Chili; multiple galaxies and boundary values; mixed-context/partial-data tests; controlled backend-error recovery/resume, shutdown and duplicate-event/ingestion checks, separate from ordinary replacement recovery. |
| 03 — Durable catalog and history | Gate DuckDB adoption; archive stable snapshots; idempotent ingestion, migrations, retention controls, backup/restore, process ownership, and replayable projections. | 02 and database trial | Crash/restart, simultaneous interface access, replay, migration failure, and storage-limit demonstrations; short journal writes alongside ingestion and long analytics. |
| 04 — Rich atlas entities and relationships | Names/owners/dates; companion instances and species evidence; main/expedition contexts; discoveries, bases, teleport destinations, and supported related records. Standalone seed-bearing planet detail from existing data (P02-21); field support documented individually. | 03 for richer catalog entities; existing-data planet detail may follow 01 without a database gate | End-to-end Chili journey and fixtures proving supported relationships, conflicting evidence, and unknown cases; independent expected planet seed/detail output. |
| 05 — Search, history, and analytics | Typed filters, scope/date controls, relationship traversal, history comparisons, aggregates, and consistent CLI/REPL/MCP results. Exact/prefix/substring names first; inherited shared-name wiring, distance/type/named-unnamed statistics and flags, info biome/Game Mode scope (P02-21). | 04 for rich queries/history; existing-query wiring/statistics may follow 01 without 03/database | Recent-world, creature-origin, cross-save, and analytics journeys with stable pagination and explainable provenance; literal name/filter/statistics/info oracles across all three interfaces where applicable. |
| 06 — Journals, media, and interchange | Expand the original personal-knowledge arc: scoped multiline notes, chronological/grouped views, tags/collections, managed images, captions/links, usage accounting, community imports, portable exports, and REPL export/import parity (P02-23). | 03; richer subjects compose with 04/05 | G08/G09; frozen-target capture/cancellation, distinct narratives, media integrity/usage, restart/reimport/rename/export/restore, source/galaxy preservation. |
| 08 — Traveller cultures and territory | Discovery attribution; stable culture identity, custom noun/name/history, per-traveller affiliations, region evidence, base/system summaries, and historical connection records. | 02/03/06; rich attribution composes with 04/05 | G10/G14; adjacent different travellers remain distinct; same-traveller cultures retain history when they adjoin; ambiguous ownership stays explicit. |
| 09 — Culture geometry and exploration | Research surrounding-system availability and coordinate precision; derive graded cohesion/completeness, gaps/frontiers/corridors and ranked exploration suggestions without a user-chosen culture radius. | 05/08 and G11/G12 research decisions | G11–G13; adversarial synthetic geometry, source coverage/uncertainty, reproducible analyses, and supported in-game candidate witnesses. Capability gaps require explicit disposition, not silent scope reduction. |
| 07 — Composition and release | Retain the original release arc ID, now composed after the added arcs: whole experience, performance, migration/recovery, supported-platform packaging, and release documentation and REPL shell-script completions parity (P02-23; distinct from TAB completion). | 01–06, 08, 09 | Independent project-level journey reproduction, all ledger rows dispositioned, and documented limitations. |

Arc IDs remain stable: Arc06 is expanded, Arcs08/09 are added, and the existing Arc07 release arc moves later in dependency order. No issued arc/slice prompts exist in Project02 or are renamed. Arc01 remains the first delivery. Arc06's basic system/planet notebook can follow Arc03 without waiting for every rich entity in Arc04. Candidate-source/geometry research can start alongside Arc02/03 so feasibility is known before Arc09 implementation; a negative result does not block the notebook or culture history.

FTS/Tantivy is a reserved extension after structured search proves a need. It is not a project completion gate. Source-format adapters for unsynced native console containers, cloud account APIs, game-save editing, automatic OCR, semantic/vector search, and a graph database are outside this proposal.

## Architecture direction

Use three complementary layers: retained source snapshots, a relational catalog of observations and relationships, and rebuildable query projections. Keep `petgraph`/`rstar` for spatial and routing operations. Prefer DuckDB for the catalog **if** the native-build, process-sharing, and interactive workload trial passes. Keep SQLite as a fallback decision, not a second simultaneously authoritative database.

Arc01 uses a small, versioned local profile registry so the UX ships first. Later catalog records refer to its stable IDs; they do not take ownership of aliases or selection preferences. See [design](design.md) and [library research](artifacts/research-and-library-decisions.md).

## Definition of done

The project is complete when all nine arc capabilities compose into the acceptance journeys above; ordinary use needs readable names rather than raw IDs; each result carries source/save/context/freshness; historical evidence and user knowledge survive restart/reimport; locations retain the correct galaxy; journals and managed images restore with their original authorship and links; culture identity survives changing geometry; exploration results distinguish coverage, ownership, and uncertainty; and failure/recovery behavior is demonstrated across the supported release targets. Full undiscovered-system enumeration remains a required investigation and gated capability: inability to support it needs an explicit Operator-approved disposition with a re-entry condition before project closure.

Each arc must supply its child verification and a composition demonstration. The [project ledger](ledger.md) remains open until an independent reviewer reproduces the required evidence. This design pass does not close project01 or assert project02 implementation completion.

## Decisions to settle before implementation

1. **Startup policy:** proposed default is reopening the last explicitly selected logical save, with visible active-save status and an easy chooser. The user's preference question is pending; this default is provisional. Scripts never prompt.
2. **DuckDB:** adopt only after Arc03's focused trial; Arc01 proceeds without it.
3. **History storage budget:** measure real snapshot sizes, then set an explicit default limit before enabling continuous archival. Initial behavior at the limit should preserve existing history and report archival paused; automatic pruning requires a separately specified policy.
4. **Platform commitment:** macOS is the first demonstrated host. Check existing distribution targets before promising Windows/Linux support with the new native dependency; native Xbox storage is a separate adapter problem from synced `.hg` files.
5. **Discovery attribution:** determine whether native owner evidence distinguishes saves sharing an account; unresolved links remain unknown or explicitly user-attested, never automatically credited to every save.
6. **Culture mathematics and candidate supply:** G11/G12 must establish available system populations, coordinate resolution, boundary/scale selection, and meaningful separate measures before a formula or geometry dependency is adopted. No user radius defines a culture.
7. **Capture interaction:** validate command grammar, multiline submit/cancel, clipboard hosts, media limits, and display fallbacks in G08/G09. Command examples are proposals.

Risks are tracked with their gates in the research document: unreliable identity inference, incomplete game records, format drift, expensive metadata scanning, sync-time ambiguity, native dependency packaging, and multiple application processes.

## Workflow and authority

Continue the established Two-Contributor Workflow and Expedited Mode: Sofie/CDC plans and reviews; separately assigned CC implements; the Operator carries handoffs. This augmentation authorizes planning edits and their scoped commit, not source implementation or a new assignment to the contexts working on Project01. Independent implementation acceptance uses `cdc-verification.md`; project closure also needs independent composition and Operator acceptance. Research hypotheses cannot become production policy merely by appearing in this packet.

Reuse the existing `planning` worktree, `project02-personal-atlas/`, and its project-level `artifacts/` design-paper convention. Prepare arc/slice open sets only near execution. All project ledger rows remain open.

## Version History

- **1.0 — 2026-10-03:** Initial seven-arc design packet (version label recorded retrospectively); friendly saves first, conditional DuckDB, durable history and personal knowledge, optional FTS.
- **1.1 — 2026-10-11:** Operator design discussion, not a child close, supplied adventure journals/media, distinct save narratives, discovery-based traveller cultures, graded geometric cohesion, late adjacency, and cross-save exploration. Expand Arc06; add Arcs08/09; retain Arc07 ID and move its release dependency gate after them. Add research gates and acceptance rows without closing any. Earlier conversational proposals for manual claim/cherished flags, shared multi-traveller culture affiliation, and configurable-radius culture definitions are superseded. Explicitly reconcile the old Project02 address-repair placement with the subsequently authorized Project01 Arc06 responsibility. No Project01 records or source are changed.
- **1.2 — 2026-10-26:** Duncan explicitly transfers Project01 reconciliation groups2 (search/detail/statistics/info) and3 (working configuration/output) into this NEXT project. Consolidate in existing Arc01 after friendly selection and Arc04/05; no database prerequisite for existing-query work. Retain all existing arcs, dependencies for richer capabilities,20 ledger IDs/statuses and B02’s separate theme deferral; add future-open P02-21/22. No source work or implementation assignment authorized.
- **1.3 — 2026-10-26:** Duncan transfers existing CLI-command REPL parity to this project (Arc01 saves early, Arc06 export/import, Arc07 shell-script completions); add future-open P02-23 while preserving all22 prior criteria/statuses. Reserve external-client/gameplay integration scope for Project03 AFTER this project, explicitly excluding Claude Desktop/web/Anthropic apps; no new project directory/plan. Last-results and separate nms-mcp crate remain mandatory Project01 deliveries, not completed or transferred here. Information requests do not waive other scope.
- **1.4 — 2026-10-26:** Correct the last-results note: current-session most-recent-successful result semantics are now agreed, delivery remains Project01 responsibility/open; cross-restart result history is future, with no persistent backend/new command invented. Project02’s23 criterion IDs/statuses and delivery scope remain unchanged.
- **1.5 — 2026-10-26:** Add Duncan’s notify backend-reconnection transfer in existing Arc02 after friendly Arc01 selection, with future-open P02-24. Bounded recovery/resumption/lifecycle/duplicate correctness is distinct from ordinary replacement recovery and Windows CI. Preserve all23 prior IDs/statuses and existing arc order; no implementation or new arc. Strict-positive2opt proof remains Project01 required work; visualization B11 has no fixed project/deadline.
