# Slice05 acceptance ledger

Created 2026-10-04. All seven rows open. CC may propose done with attested evidence, but keeps status open until CDC independently reproduces. Five refinement iterations maximum; no silent criterion changes. Evidence home: artifacts/.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| CS-01 | Assignment/source fences, full required intake and source-cited readback are established | Read execution intake; verify source worktree commit, planning issuance and exact path fences | correctness-grade | Approved scope and prompt | open | — | Pending CC and independent CDC review |
| CS-02 | Current writes/reads use the exact schema 1 / semantic 1 envelope and retain payload data | Literal16-byte assertion, populated fixture field comparisons, checked aligned/misaligned slice reads | serious | Envelope contract | open | — | Pending CC and independent CDC review |
| CS-03 | Legacy/truncated/magic/schema/semantic incompatibility is typed and precedes payload decoding | Negative direct deserialize/read_cache tests and error chaining; current bad payload has Deserialize | serious | Failure order contract | open | — | Pending CC and independent CDC review |
| CS-04 | Newer incompatible caches rebuild from source and become usable current warm caches | Legacy/schema/semantic sentinel9999 vs source 4720 through load_or_rebuild, verify false then true and source fields | serious | Startup and D08 | open | — | Pending CC and independent CDC review |
| CS-05 | Cache rejection never returns bad data on source failure; stale/corrupt/missing/bypass lifecycle remains correct | Source parse error/no overwrite; corrupt current payload recovery; missing/stale; no_cache existing-cache byte preservation | serious | Lifecycle contract | open | — | Pending CC and independent CDC review |
| CS-06 | Public compatibility and ongoing revision process are documented accurately | Review changed API docs, separate schema/semantic rules, old-cache rebuild and later semantic bump obligation | correctness-grade | Required process and public API | open | — | Pending CC and independent CDC review |
| CS-07 | Required validation and scoped candidate/evidence delivery are faithful | make format then make lint before focused/full tests; doc/build checks; exact commits/trailers, outcomes and seven-row return | correctness-grade | Repository/Expedited gates | open | — | Pending CC and independent CDC review |
