# Slice05 — cache semantic compatibility

Version: 1.1. Created 2026-10-04. Status: open; initial CDC review accepted four rows, three require iteration01.

## Goal, authority and boundaries

Implement the first dependency of the Operator-approved three-direction expansion recorded in project-plan.md §13 and arc-plan.md v1.11. A mtime-fresh cache must also have the current envelope/schema/semantic revision before it can serve a model. Legacy or incompatible data is disposable: startup reparses the source and writes a current cache. This slice supplies protection and a continuing revision rule, not a repaired save decoder or galaxy identity.

CDC/Sofie designed this packet and will independently review CC implementation. Two contributors; Expedited Mode; no extra permission request after coherent preflight. Main stays clean at `a800950ef3558fcf463517696c3c140061e7353d`. CC creates isolated `.worktrees/cache-semantic-compatibility` on `codex/cache-semantic-compatibility` from that commit; planning remains `.worktrees/planning` on `planning` (preparation parent `082270d720d7e69669c97711328b72b9db6d8309`). The issuance commit containing this open set is the initial planning state, to be recorded by CC. No merge/push/publication authorization is implied.

Artifacts: `artifacts/` in this slice. CDC sealed inputs: author-baseline.json and author-probe.rs. CC outputs: artifacts/execution.md, artifacts/validation.json and root closing-report.md; CC may annotate ledger evidence/proposals while keeping statuses open. Do not create a CDC verification record before review.

In scope: serialize/deserialize cache envelope, typed compatibility errors and exports, startup documentation/integration tests and revision-process docs. No dependency/manifest changes. Out of scope: game decoding policy, SystemId migration, active_galaxy restoration, source-content hashes, multi-writer atomicity redesign, mmap/zero-copy performance redesign, repairing no-default/docs/coverage/package failures. These stay owned by the original closure route or subsequent identity slice. Existing direct cache API signatures and best-effort write lifecycle remain; bytes intentionally change to a versioned internal format.

## Binding cache contract

The public serialize/deserialize functions, and therefore write_cache/read_cache, operate on an envelope followed by the unchanged rkyv CacheData archive. Envelope exactly16 bytes:

| Offsets | Meaning | Initial bytes/value |
| --- | --- | --- |
| 0–7 | Magic | ASCII `NMSCACHE` |
| 8–11 | Archive schema u32 little-endian |1, bytes01 00 00 00 |
| 12–15 | Model semantic revision u32 little-endian |1, bytes01 00 00 00 |
| 16 onward | rkyv payload | Current locked 0.8.15 CacheData encoding |

Revisions are internal constants in serialize.rs, separate from game SaveRoot.version, package version and cached_at. Initial semantic 1 labels the existing interpretation, with its known address/identity defects; it does not claim those are repaired. Require exact equality for both revisions. Future serialized layout/codec/config changes assess schema; future address interpretation/identity changes assess semantics and bump semantic revision when meaning changes even if archive layout does not. Both may need to change. No version guessing, reinterpretation, dual-reader legacy acceptance or silent downgrade.

Deserialize ordering: (1) length<16 -> typed truncated header error, (2) bad magic -> typed invalid magic, (3) schema mismatch -> typed schema error, (4) semantic mismatch -> typed semantic error, (5) only then checked rkyv payload validation/deserialization. Compatibility failure wins even when the payload is invalid. Current header with invalid payload -> CacheError::Deserialize, never an unchecked reader or panic for bad bytes. Copy payload into `rkyv::util::AlignedVec::<16>` before checked from_bytes, so arbitrary borrowed byte slices do not depend on caller allocation alignment. This extra bounded copy is a correctness choice; no startup speed claim is made here.

Add `CacheError::Incompatible(CacheCompatibilityError)`. New public error type is Debug/Clone/Copy/PartialEq/Eq, `#[non_exhaustive]`, implements Display/Error, and is reexported beside CacheError. Exact variants: `TruncatedHeader { actual: usize }`, `InvalidMagic`, `SchemaMismatch { found: u32, expected: u32 }`, `SemanticMismatch { found: u32, expected: u32 }`. CacheError Display must identify incompatibility, actual/expected revision where applicable; Error::source exposes the nested compatibility error. Keep existing Serialize/Deserialize/Io variants and messages. No new library dependency or opaque compatibility string. Adding the CacheError variant intentionally affects exhaustive external matches; document this under the approved compatibility change and retain later packaging/version gates. Repository callers do not exhaustively match it.

Startup remains synchronous: with caching enabled, try a newer cache; read rejection warns and falls through to parse_save_file, from_save, then existing atomic-temp/rename write. Successful parse returns was_cached=false and the source save version; next fresh compatible startup returns true. Parse failure propagates without serving rejected data or overwriting that cache. Write failure remains warning plus successfully parsed model; retain existing best-effort contract. `no_cache=true` bypasses all cache reading and writing, including an existing incompatible cache. `is_cache_fresh` remains an mtime predicate; document that semantic compatibility is checked separately. Do not introduce content hashing or concurrent writer guarantees.

## Implementation and proof

The [CC prompt](cc-prompt.md) contains the complete file/symbol recipe, reading manifest, behavioral oracles, exact paths and validation commands. Seven acceptance rows in the [ledger](ledger.md) are required. Direct byte/header tests distinguish format policy; load_or_rebuild integration tests distinguish source fallback from returning poisoned cached data. Existing tests retain their assertions. No new reusable validator product is requested.

## Author readiness — attested, not independent acceptance

- Source-grounded: complete cache crate source and existing load_integration.rs inspected; bounded actual startup/watch callers and source-wide error/call scan inspected. Cargo.lock pins rkyv 0.8.15. author-baseline.json gives source hashes, reading extents and exact fresh execution channels.
- Design-complete: contract above fixes header, version matching, error order, alignment, fallback/failure and bypass rules. Public byte/error compatibility is explicit. Future game semantics remain outside this slice.
- Guideline-applied: prompt table applies EH01/04, AP02/33, TD07 and DC01; the wider approved boundary/identity directions apply API37/TD03 without implementing them here. Manual Error impl reuses existing crate style; no speculative dependency.
- Executable: checked from_bytes and AlignedVec APIs verified in locked local source. Temporary FileTimes snippet compiled/executed successfully. Author-probe.rs compiled against emitted nms_cache rlib; cold source 4720 then newer poisoned cache 9999 returned false/true respectively, demonstrating the actual bypass. The proposed envelope/error recipe is a design, not a compiled production patch.
- Falsifiable: legacy/schema/semantic-poison tests must return source 4720 instead of9999; wrong revision plus invalid payload must produce compatibility errors before codec errors; a current header plus bad payload must produce Deserialize. Both discrimination and subsequent warm hit are observed through real public APIs.
- Coherent/portable: one active prompt with bounded manifest and intake/readback home; source and planning allowlists are separate; sealed predecessor files preserved. Estimated required context20–30k tokens including applicable governing guides/source, plus implementation/recovery headroom; no broad history or private save inspection required. Recover truncated normative reads instead of shrinking them.

## Assignment history

| Issued | Prompt | Predecessor | Reason | Disposition |
| --- | --- | --- | --- | --- |
|2026-10-04 | `arc06-fit-and-finish/slice05-cache-semantic-compatibility/cc-prompt.md` | No predecessor within this new slice; Slice04 closed | First production dependency under approved expansion | returned for refinement |

Initial cc-prompt.md is preserved and returned for refinement; current assignment is cc-prompt-iteration01.md. Closing requires seven independently reproduced rows; this open set does not close R06-R03/P01-R03, the arc or project. Later semantic transitions and complete cache/watch/project composition remain separate proof.

## Version history

- 1.0 (2026-10-04): opened after Operator's scope expansion; independently designed cache protection before identity and evidence-gated save repair. No production code or success report pre-created.

## Initial review and iteration01 — 2026-10-04

CDC independently reproduces runtime protection,31 cache tests and824 workspace tests plus format/lint/build/doc gates at source8d2cf00/planning4f6ca66. CS-01/02/05/07 are accepted; CS-03/04/06 remain open for empty-envelope coverage, durable freshness preconditions and compatibility/process documentation. Runtime success is retained; the three delivery gaps hold closure. [CDC review](cdc-verification.md) and [Iteration01](cc-prompt-iteration01.md) govern current refinement.

Assignment history continuation:

| Issued | Prompt | Predecessor | Reason | Disposition |
| --- | --- | --- | --- | --- |
|2026-10-04 | `arc06-fit-and-finish/slice05-cache-semantic-compatibility/cc-prompt-iteration01.md` | initial cc-prompt.md | Initial review R01–R03, no runtime repair redesign | active |

Author readiness for iteration01: actual source/docs/helper call sites inspected and full binding plan/initial prompt/reports loaded; omitted assertions and missing docs are exact deltas. Fresh public-API probe proves length0 and actual-fresh schema/semantic rebuild behavior. Recipe is restricted to existing test loop/helper and public docs; no API/runtime decision is left to CC. DC01/02 and testing's behavioral/precondition guidance apply; unchanged runtime/semantic revisions are binding. Existing native gates plus crate-only rustdoc suffice, with no new verification product. Required-reading manifest/intake and preservation exceptions are explicit in the new prompt; estimate10–15k scoped reading plus ample implementation/recovery headroom. These are author readiness/self-checks, not acceptance of CC's future work.

Version1.1 (2026-10-04): routine initial review and first refinement recorded. Four rows accepted; three remain open; source/runtime/semantic and original project scope unchanged.
