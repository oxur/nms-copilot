# CDC verification — decoded local location witnesses

**Current verdict after iteration01: six rows accepted; DW-07 remains open for R04 narrative clarification.** Historical initial review below remains preserved.

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

## Iteration01 correction review — 2026-10-04

CDC / Sofie independently reviewed `13409d1e32a86f0d16cae8163c696f1e37693c5c` against preserved `cc-prompt-iteration01.md`. [Fresh receipts](artifacts/cdc-iteration01-receipts.json) record the exact five-file correction, independent raw replay, all 44 corrected comparisons and all 44 wrong-offset controls. Main is clean/pinned at a800950ef3558fcf463517696c3c140061e7353d; planning/reference clones clean at review intake. Source/dependency/mapping/reference hashes and input digest unchanged. Seven criteria and accepted DW-01/DW-04 rows unchanged during CC correction; no implementation or policy change.

R01–R03 are resolved. Freshly decoded exactly the pinned raw bytes, preserving Decimal/integer precision, unknown fields, mapping precedence and sanitation. All 255 current replay checks pass. All44 faithful literal objects match raw expanded oracles and the initial CDC receipt's faithful `actual` objects. A deliberately wrong first-stage transcription fails both full component/reality and portal agreement for44/44 and reproduces all initial incorrect `expected` objects. Fresh corrected objects match those initial-bad `expected` objects **0/44**. Thus the literal source math, negative-control rejection and comparison flags reproduce. Both source comparisons agree on this witnessed one-save population only; no original external runtime/general decoder or packed nonzero-galaxy meaning follows.

Independently hashed header and library bytes separately; they agree with the correction and distinguish R02's old misattribution. Fresh membership/type projections over both contexts' current/previous wrappers and all207/17 endpoints agree with corrected absent-alternate prose and unchanged structured population. R03 is resolved. Corrected field assessment now states both source candidates agree and retains all representation/relationship/policy limits. Nonzero ordinal174 remains expanded-only; discovery/base scalar equalities remain correlations.

Reverse-normalizing only44 authorized literal objects and88 comparison flags makes the entire current witness JSON equal the initial packet; all218 IDs and other values preserved. population.json byte-identical with SHA256 e6e2fef7038d271225a5df8f85684632fb7ecad6ef0c13a014358fedf4195b18. Execution/report preserve their exact initial prefixes; current assessment identifies its historical errors. Scope, whitespace, trailers once, unchanged criteria and intake statuses pass. Previously reproduced full population, raw associations, oracles, Companion outputs, synthetic relation controls and DW-01/DW-04 evidence remain applicable because their input bytes, mappings, source, predicates, criteria and independent provenance are unchanged. Their original2026-10-04 evidence is retained; changed outcomes above are freshly checked. No broad build/test/lint/coverage or original reference-runtime execution was required or run.

### R04 — contradictory receipt-direction sentence

Minor evidence-narrative defect; affects **DW-07 only**. execution.md line72 begins: “All 44 fresh NMSCoordinates results equal the CDC receipt's faithful `actual` objects and the original witness objects' CDC-marked incorrect `expected` objects”. This says the corrected results equal both different datasets. Independent full comparisons establish44/44 matches to faithful `actual`, **0/44** to initial-bad `expected`. The original initial witness objects match the initial-bad `expected` objects44/44; they are not the corrected results. Adjacent execution text and closing-report correctly describe replacing the wrong objects, so this does not undermine the accepted math or require another save/decoder investigation. It does leave the canonical execution narrative internally inconsistent.

Append an explicit dated correction identifying that sentence as superseded. Preserve history; clearly distinguish corrected witnesses, initial witnesses and the initial CDC receipt's labels. Check those equalities over all44 immutable objects, record the counts and append the short report correction. Only execution/report and DW-07 evidence/notes require CC edits. This is an evidence-label clarification, not a new acceptance criterion or production finding.

| Row | Current status | Independent basis / remaining gate |
|---|---|---|
| DW-01 | done | Initial independently reproduced fences/readback retained under checked unchanged applicability. |
| DW-02 | done | Raw decode/mapping/sanitation/precision evidence retained and freshly checked; header/library identities now correct. |
| DW-03 | done | Complete prior population retained; fresh spelling projections and current absent-state prose agree. |
| DW-04 | done | Prior whole-set association/eligibility/control evidence retained; fresh same44 pairs reproduce. |
| DW-05 | done | All44 corrected literal observations and comparison flags independently reproduce; wrong-offset control fails all44. |
| DW-06 | done | Corrected source applicability and readiness limits agree with witnessed math; consequential policy remains held. |
| DW-07 | open | Structural/history/scope checks pass, but execution line72 needs R04 clarification and independent review. |

Seven opening rows, six accepted, one open; none omitted/deferred/no-op. Initial failed source/provenance/prose findings remain visible. The slice, arc and project remain open. Routine partial progress only: no architecture/scope/acceptance or original sequencing amendment. Issue `cc-prompt-iteration02.md` inside this slice, narrowing to R04; do not reopen accepted numerical evidence or advance dependent repair design until the slice closes.

What worked: exact source stages and full independent comparisons distinguish a resolved numerical correction from a remaining sentence-level inconsistency. A small immutable-data equality check can settle it without repeating unaffected save/population/source work.
