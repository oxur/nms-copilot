# Project02 — A personal atlas that remembers

Date: 2026-10-03

Status: Draft for discussion; research informs proposed decisions, not implemented capabilities.

## Purpose

Make NMS Copilot easy to point at the right save, then build a durable, explainable atlas across that player's saves and expeditions. A user should select “main - Xbox,” find a creature's home world, understand which save supplied the information, and revisit discoveries that later disappear from the game's current records.

The atlas can preserve only evidence it has observed or the user has supplied. It cannot promise to recover discoveries that vanished before a snapshot was captured.

## Confirmed priorities and proposed scope

The user requires friendly accounts and saves **before** creatures, detailed expedition support, and richer searching. Full-text search is lower priority. Rust library research and a comprehensive design are authorized; implementation is not part of this design pass.

Place the save/account work in **project02 Arc01**. It establishes identity and selection rules needed throughout the larger project and can ship independently of a database. Keep project01's delivery history intact. Carry the known discovery-address defect into Arc02, before adding richer location features; an urgent standalone fix may ship earlier without moving the UX behind database work.

Proposed project scope extends beyond the initiating planet/creature search: multi-save selection, provenance, historical observations, durable annotations/imports, relationships, structured queries, analytics, recovery, and consistent CLI/REPL/MCP behavior. The particular entity set and later arc boundaries remain reviewable.

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

## Roadmap

These are capability arcs, not implementation assignments. “Depends on” means an acceptance gate, not that research cannot overlap.

| Arc | Capability and boundary | Depends on | Exit evidence |
|---|---|---|---|
| 01 — Accounts, saves, and selection | Shared inventory/identity service; in-game names and local aliases; readable chooser/list/status; explicit selection; migration of existing configured paths; safe switching; diagnostics. No database prerequisite. | Existing MVP | CLI and REPL demonstrations with duplicate names, multiple roots, missing metadata, unavailable files, and a synced cross-platform save. Shared selector and MCP scope tests. |
| 02 — Trustworthy addresses and observations | Separate address encodings; preserve galaxy; model source/context, absent/null/malformed fields, timestamps, and extraction versions. Establish sanitized fixtures and raw-record locators. | 01 | Independent expected-address fixtures including Chili; multiple galaxies and boundary values; mixed-context/partial-data tests. |
| 03 — Durable catalog and history | Gate DuckDB adoption; archive stable snapshots; idempotent ingestion, migrations, retention controls, backup/restore, process ownership, and replayable projections. | 02 and database trial | Crash/restart, simultaneous interface access, replay, migration failure, and storage-limit demonstrations. |
| 04 — Rich atlas entities and relationships | Names/owners/dates; companion instances and species evidence; main/expedition contexts; discoveries, bases, teleport destinations, and supported related records. Field support documented individually. | 03 | End-to-end Chili journey and fixtures proving supported relationships, conflicting evidence, and unknown cases. |
| 05 — Search, history, and analytics | Typed filters, scope/date controls, relationship traversal, history comparisons, aggregates, and consistent CLI/REPL/MCP results. Exact/prefix/substring names first. | 04 | Recent-world, creature-origin, cross-save, and analytics journeys with stable pagination and explainable provenance. |
| 06 — Personal knowledge and interchange | Durable notes, tags, collections, attachment references, community imports, and portable exports; distinguish user assertions from save evidence. | 03; composes with 05 | Restart/round-trip/backup demonstrations; an annotation survives reimport and renaming; external records retain source and galaxy. |
| 07 — Composition and release | Verify the whole experience, performance, migration/recovery instructions, supported-platform packaging, and release documentation. | 01–06 | Independent project-level journey reproduction, all ledger rows dispositioned, and documented limitations. |

FTS/Tantivy is a reserved extension after structured search proves a need. It is not a project completion gate. Source-format adapters for unsynced native console containers, cloud account APIs, game-save editing, automatic OCR, semantic/vector search, and a graph database are outside this proposal.

## Architecture direction

Use three complementary layers: retained source snapshots, a relational catalog of observations and relationships, and rebuildable query projections. Keep `petgraph`/`rstar` for spatial and routing operations. Prefer DuckDB for the catalog **if** the native-build, process-sharing, and interactive workload trial passes. Keep SQLite as a fallback decision, not a second simultaneously authoritative database.

Arc01 uses a small, versioned local profile registry so the UX ships first. Later catalog records refer to its stable IDs; they do not take ownership of aliases or selection preferences. See [design](design.md) and [library research](artifacts/research-and-library-decisions.md).

## Definition of done

The project is complete when all seven arc capabilities compose into the acceptance journeys above; ordinary use needs readable names rather than raw IDs; each result carries source/save/context/freshness; historical evidence and user knowledge survive restart/reimport; locations retain the correct galaxy; and failure/recovery behavior is demonstrated across the supported release targets.

Each arc must supply its child verification and a composition demonstration. The [project ledger](ledger.md) remains open until an independent reviewer reproduces the required evidence. This design pass does not close project01 or assert project02 implementation completion.

## Decisions to settle before implementation

1. **Startup policy:** proposed default is reopening the last explicitly selected logical save, with visible active-save status and an easy chooser. The user's preference question is pending; this default is provisional. Scripts never prompt.
2. **DuckDB:** adopt only after Arc03's focused trial; Arc01 proceeds without it.
3. **History storage budget:** measure real snapshot sizes, then set an explicit default limit before enabling continuous archival. Initial behavior at the limit should preserve existing history and report archival paused; automatic pruning requires a separately specified policy.
4. **Platform commitment:** macOS is the first demonstrated host. Check existing distribution targets before promising Windows/Linux support with the new native dependency; native Xbox storage is a separate adapter problem from synced `.hg` files.

Risks are tracked with their gates in the research document: unreliable identity inference, incomplete game records, format drift, expensive metadata scanning, sync-time ambiguity, native dependency packaging, and multiple application processes.
