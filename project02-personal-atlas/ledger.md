# Project02 acceptance ledger

Status: all rows open. This ledger describes future implementation acceptance, not completion of the design packet. Evidence strengths follow the collaboration framework: assertion, implementer attestation, independent reproduction, and reconciliation remain distinct. Final statuses are `done`, `deferred`, or `no-op`; deferrals require a reason and re-entry condition. Independent review is required for project closure.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| P02-01 | Arc01 closes with readable names, reliable selection, migration, and coherent switching | Child ledger plus CLI/REPL/MCP shared-selection demonstration using G01/G02 fixtures | correctness-grade | User priority; Arc01 | open | — | Must precede richer data/search delivery |
| P02-02 | Arc02 closes with correct addresses, galaxies, contexts, and explicit missing-data states | Child ledger plus independent G03 fixtures and source-path inspection | correctness-grade | Address defect; Arc02 | open | — | Round-trip-only evidence insufficient |
| P02-03 | Arc03 closes with durable ingestion/history, recovery, and accepted database choice | Child ledger plus G04/G05/G06 results, replay and backup/restore | serious | Arc03 | open | — | DuckDB adoption conditional |
| P02-04 | Arc04 closes with supported rich entities and provenance-bearing relationships | Child ledger plus creature-origin and mixed-context fixtures | correctness-grade | Arc04 | open | — | Unsupported fields documented |
| P02-05 | Arc05 closes with scoped structured search, history, and analytics | Child ledger plus equivalent CLI/REPL/MCP answers and G07 counting checks | correctness-grade | Arc05 | open | — | FTS not required |
| P02-06 | Arc06 closes with durable personal knowledge and interchange | Child ledger plus rename/reimport/restart/export/restore demonstration | serious | Arc06 | open | — | User assertions survive extraction |
| P02-07 | Arc07 closes with release documentation, supported builds, and performance evidence | Child ledger, exact target matrix, reproducible checks and published limitations | correctness-grade | Arc07 | open | — | Coverage denominator and exclusions explicit |
| P02-08 | Friendly selection composes with rich lookup: chosen Xbox save → Chili → correct origin/galaxy | Whole-product journey with independently expected location and source explanation | correctness-grade | Project acceptance journey | open | — | Includes switching away and back |
| P02-09 | History survives a disappearing discovery and distinguishes visit/discovery/observation time | Whole-product present/absent/partial fixtures and recent-world query | serious | User's discovery glitch | open | — | Cannot recover never-observed data |
| P02-10 | Concurrent CLI, REPL, and MCP use preserves one coherent committed view and recovers after owner death | Whole-product process-contention/crash exercise with raw results | serious | Architecture composition | open | — | No stale-owner lockout or partial generation |
| P02-11 | Archive, catalog, registry, and annotations restore together; caches can be rebuilt | Backup/restore to clean app directory; compare identities, facts, annotations, and sources | serious | Durability composition | open | — | Preserve original game files unchanged |
| P02-12 | Every arc finding and inherited project01 issue has an explicit disposition | Final row walk, child-ledger review, source-findings reconciliation | correctness-grade | Work verification | open | — | Deferred items retain re-entry conditions |

Detailed test commands and durable evidence paths will be assigned in child plans after design review. None exist yet; their absence must not be interpreted as successful verification.
