# CDC verification — decoded local location witnesses

## Initial independent review — 2026-10-04

**Verdict: changes required; DW-01/DW-04 accepted, DW-02/DW-03/DW-05/DW-06/DW-07 open.** CDC / Sofie reviewed CC `8d982eab696ef68eaccd437f12ed4ed49ed8d377` against the initial `cc-prompt.md` and seven unchanged criteria. Two-Contributor Workflow; Expedited Mode. Main remains clean at `a800950ef3558fcf463517696c3c140061e7353d`; planning clean at intake. This context authored the assignment, not CC's implementation. [Independent receipts](artifacts/cdc-review-receipts.json) retain the complete query checks, all 44 disagreements with the submitted transcription, fresh correct comparisons, source fingerprints, scope and review failures.

### Independently checked evidence

Freshly read exactly the pinned 2,918,235-byte save; SHA256 and before/after size/mtime/inode agree. Inspected complete decompression/key-mapping functions and sanitation, the installed native LZ4 contract, full relevant pinned decoder bodies/constants and caller inventory. Replayed native raw blocks with exact sizes/capacity and integer/Decimal parsing; 20 blocks, 10,278,541 decoded bytes, one NUL trimmed, 137 invalid UTF-8 sequences, zero duplicate JSON keys and mapped collisions. Primary mapping has eight duplicate rows; first-entry precedence and all mapping hashes/counts reproduce. No typed round trip, save mutation, additional input, source change or original external runtime execution.

Executed complete declared state/kind/range/min/max projections over both contexts and every current/previous spelling, base/endpoint collection/type/Name/Position, discovery DD.UA/DT/VP and numeric/width/precision/duplicate populations. Recounted 3,243 discovery records, 54/207 BaseContext bases/endpoints, 0/17 ExpeditionContext, all 42 scalar-correlated base groups and locators. Both player objects have 254 keys. Full programmatic projections, not a full textual reading of the large JSON or sampling: 1,028 checks, with 44 failed checks localized to the literal transcription and all other checked properties matching. Read assignment, plan, ledger, author-input, close report, execution, field assessment and applicable parent contract text; bounded source reads recovered after oversized output. Complete dataset joins/ordinals are covered by computation; reading receipts remain CC attestation rather than attention proof.

Independently classified all raw associations before candidate comparison. All 49 eligible HomePlanetBase and 74 Base endpoints satisfy field eligibility: 44 bidirectionally unique edges, five unmatched bases, 30 unmatched endpoints; zero ambiguity/exclusions. Incompatible 5/133 records are retained. All three synthetic controls reproduce, including duplicate endpoint 45 raw edges / 43 unique / one ambiguous base / two ambiguous endpoints. Private value scan found no retained names/native identities; known previously supplied filesystem prefixes were excluded. Expanded-only endpoint 174 and direct discovery-link absence remain honestly bounded.

The Companion candidate matches explicit expanded components/reality and portals for 44/44. **The actual pinned NMSCoordinates literal source also matches components/reality and portals for these 44/44.** This does not generalize either source beyond these witnessed inputs. All 218 unique witness IDs, six-path CC scope, unchanged criteria, links, whitespace, fences and both trailers once checked. Main/reference clones unchanged and clean. No broad production build/test/lint/coverage suite was required or run.

### R01 — incorrect literal NMSCoordinates transcription

Serious; affects DW-05/DW-06/DW-07. `Coordinates.cs` at pinned b8f765b uses `dec3 = 0x7F`, `dec4 = 0x7FF`, then `calcX/Z = (rawX/Z + 2047) % 4096`, `calcY = (rawY + 127) % 256`; shifts back by 2047/127. The separate `GalacticToPortal` adds 0x801/0x81. These portal offsets cancel modulo the field width. CC's submitted first-stage values instead behave as if 2049/129 were added, shifting final coordinates/portals by two. The source text and hashes match accepted pins; this is a transcription failure, not evidence the pinned source disagrees.

For W-BASE-000-039, scalar `0x40050003AB8C07`: source intermediates X1030/Y130/Z695, shifted X-1017/Y3/Z-1352, portal **400503AB8C07**. Submitted intermediates 1032/132/697 and portal 400505ABAC09 are wrong. Every one of the 44 literal records differs from a faithful source replay. Recompute these records and flags, correct assessment/report/current outcome summaries, and preserve the initial erroneous claim as history. The Companion outputs, independent expanded oracles, original scalar kinds with normalized display values, relations and locators remain unchanged. Do not infer general identity/format/galaxy policy or original runtime execution from either transcription.

### R02 — header fingerprint identifies library bytes

Correctness-grade; affects DW-02/DW-07. execution.md names the LZ4 **header** SHA256 as `3ce8bec34557e6a5c76129cef6588dc59959caa51e1e54321f45ea7c6e528e07`, which is the library fingerprint. Independently hashed header `/opt/homebrew/opt/lz4/include/lz4.h` is **26b82efc53d1570f3b54eef02e9c4764c1ad374ff03cac04e2ced5ea4d4c552f**. Library hash remains the original. Preserve the incorrect initial execution record; append separately identified fresh header/library hashes and correction. No decode/mapping/population change is warranted by this provenance repair.

### R03 — prose contradicts absent-field population

Correctness-grade; affects DW-03/DW-06/DW-07. execution.md says the player-context UniversalAddress spellings are present malformed strings. The field-assessment’s generic malformed/absent sentence does not clarify the actual field states; make those states concrete while repairing its R01 interpretation. All four alternate player wrappers (`UniversalAddress`, `PreviousUniversalAddress` in each context) are **absent**. All endpoint UniversalAddress wrappers are absent, too. The structured population already records this correctly and independent full projections reproduce it. Correct current interpretation and append execution/report clarification without inventing malformed inputs or changing valid population records. UniverseAddress/PreviousUniverseAddress are separately present expanded objects at player level. A zero member statistic is not a present malformed wrapper.

### Seven-row disposition

| Row | Status | Independent basis / remaining gate |
|---|---|---|
| DW-01 | done | Source/input/scope intake, readback and stable fresh bytes reproduced. |
| DW-02 | open | Decode/mapping/UTF-8/duplicate/precision checks reproduce; R02 prevents accepting the complete provenance record. |
| DW-03 | open | Complete structured population reproduces; R03's prose still contradicts its field states. |
| DW-04 | done | Whole-set raw associations, eligibility/ambiguity/unmatched/incompatible accounting and synthetic controls independently reproduced. |
| DW-05 | open | Expanded oracles and Companion pass; every literal comparison needs R01 correction. |
| DW-06 | open | Policy limits are retained, but source applicability/readiness assessment relies on R01/R03 false claims. |
| DW-07 | open | Scope/privacy/history/JSON/links/trailers pass; R01–R03 prevent a consistent seven-row close. |

Seven opening rows, two accepted, five open; none deleted/deferred/no-op. Unaffected claims above can be retained during correction review only after explicit unchanged-input/applicability assessment, with their original date. Correction must reproduce changed math/claims freshly; no copying desired values as a replay receipt.

### Bubble-up to the arc and return

The bounded decoded-input piece is largely delivered, but its source comparison and provenance/narrative consistency prevent closing it. No original capability, architecture, acceptance or sequencing amendment is needed. Record routine partial progress and issue `cc-prompt-iteration01.md` inside this slice before further advancement. Correcting source evidence must precede any dependent repair design. Original project and arc rows remain open; neither the 44 corroborated associations nor the retained Chili witness authorizes universal semantics or nonzero packed-galaxy policy.

What worked: full raw associations and independent expanded oracles isolate a wrong source transcription without discarding correct population evidence. Actual constants, intermediate stages and byte fingerprints reject plausible descriptions that structural checks alone accept. Reviewer failures/truncation recovery and a normalized-scalar display comparison false alarm are retained in receipts. A helper overwrote a temporary extended receipt; the population projection was rerun and receipts merged before durable capture. No failed review evidence or attempted operation is relabeled successful.
