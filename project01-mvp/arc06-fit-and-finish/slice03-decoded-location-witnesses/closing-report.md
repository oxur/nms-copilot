# Slice03 CC closeout report

Date: 2026-10-03. Assignment: `arc06-fit-and-finish/slice03-decoded-location-witnesses/cc-prompt.md`. CC disposition: **proposed done, pending CDC independent review**. This report does not close Slice03, Arc06, or Project01 until CDC accepts the evidence and unresolved project gates remain with their owners.

## Fences and outcome

Source main remained clean at `a800950ef3558fcf463517696c3c140061e7353d`. The planning worktree was branch `planning`, intake HEAD `4a64b64253b1c6b8eeae4d9cbe9b753d97d6fc6e`, parent `150216e72a8cb51bf4a05b7e6c2761a68d4a7083`. The only input was `local-save-A` (`workbench/save-games/save.hg`), 2,918,235 bytes, SHA-256 `ee786384b19ae3a7974cb3130cf16b8823f3dc2a930bd55cfa092015a244c89e`. Fresh after-read metadata and digest matched the fence exactly. No second input, private-home crawl, save mutation, product change, or external thread message was used.

The native raw-block decode consumed 20 LZ4 blocks and all input bytes, yielding 10,278,541 bytes. It trimmed one trailing NUL and replaced 137 invalid UTF-8 byte sequences. Duplicate-aware JSON parsing found zero duplicate JSON keys. Mapping followed inspected source precedence, retained unknown keys, found eight duplicate rows in the primary mapping file, and produced zero mapped-key collision objects/occurrences. Arbitrary-precision integers and Decimal fractions were retained; no typed `SaveRoot` round trip or integer narrowing was used.

The complete scoped populations were freshly recounted: 3,243 discoveries; BaseContext 54 bases and 207 endpoints; ExpeditionContext zero bases and 17 endpoints. Exact wrapper/member state, spelling, type/range/width, reality and candidate distributions, exclusions and duplicate counts are in [population.json](artifacts/population.json). The strict raw relation accepted 44 unique exact Name+Position pairs among 49 eligible HomePlanetBase records and 74 eligible Base endpoints; five bases and 30 endpoints were unmatched, with zero ambiguous edges and zero eligibility exclusions. Incompatible type records were retained by ordinal. All 44 accepted pairs matched the Companion candidate to the independently expanded oracle; the pinned NMSCoordinates literal transcription matched 0/44. The prompted example matches the expanded portal `400503AB8C07`; ordinal 174 remains an unmatched expanded-only RealityIndex 18 observation. Discovery/base scalar equality yields 42 correlations only, not linkage; discovery VP did not yield any exact endpoint Position edge. The independent game-confirmed witness “Chili” remains the only game-confirmed location evidence.

## Seven-row disposition

| Row | CC disposition | Evidence and result |
|---|---|---|
| DW-01 | proposed done | This report and [execution.md](artifacts/execution.md) retain unchanged source/input fences, intake/readback and matching before/after digest/metadata. |
| DW-02 | proposed done | [population.json](artifacts/population.json) records the raw decode route, full mapping provenance, sanitation, duplicate/collision counts and precision caveats. No typed projection was used. |
| DW-03 | proposed done | The full population artifact accounts for discovery, both contexts, all requested current/previous spellings, field states/kinds/ranges, and exact collection denominators; absent/null/malformed are distinct. |
| DW-04 | proposed done | [location-witnesses.json](artifacts/location-witnesses.json) includes each accepted, unmatched, ambiguous, excluded and incompatible ordinal; all three synthetic controls are recorded. |
| DW-05 | proposed done | All 44 eligible raw associations have separate explicit-field oracles and both source candidate outcomes; the nonzero ordinal-174 record remains expanded-only. |
| DW-06 | proposed done | [field-assessment.md](artifacts/field-assessment.md) maps claims to witnesses, pins, relationship classes, caller scope, a bounded adapter design for CDC consideration and all remaining policy owners. |
| DW-07 | proposed done | Exact scope, privacy, JSON and count checks, source/input state, artifact inventory, failures and this seven-row walk are recorded here; structural check receipts are recorded below after execution. |

Every “proposed done” is a CC attestation pending independent CDC reproduction. It is not acceptance, production readiness or closure of an upstream gate.

## Artifact inventory and scope

The two already-authored supporting inputs, unchanged, are `artifacts/author-input.md` and `artifacts/author-probe-summary.json`. The authorized CC outputs are exactly these six paths:

1. `artifacts/population.json`
2. `artifacts/location-witnesses.json`
3. `artifacts/field-assessment.md`
4. `artifacts/execution.md`
5. `closing-report.md`
6. `ledger.md`

No other tracked path is authorized in this work. The public JSON retains anonymous ordinals/JSON pointers, permitted location scalars/components, type/count facts, relation flags and source provenance only; it contains no raw names, Positions, record bodies, seeds, owner/account/native-ID values, or decoded save data. The private decode/query/reference temporary files are removed after structural checks. No project, arc, parent plan, source, test, fixture, configuration, acceptance criterion, or policy file was edited.

## Failures, deviations and unrun work

The first source `wc` path and one relative Git root were wrong and were corrected. Exact NMSCoordinates fetch attempts first returned cache/fetch failures; the same pinned bodies were later retrieved and SHA-verified, with no repin. An initial private Decimal-to-JSON serialization changed number types, so its dependent Position query was discarded before evidence analysis; the corrected private text preserved numeric lexemes and all final counts were rerun. Early disposable query attempts had syntax errors and key errors (`BaseContext`, `reality_index`); they wrote no authorized artifacts and were superseded. The final query generated the two JSON files from the corrected decode.

No Rust implementation, test, build, lint, coverage run, second-save comparison, or runtime execution of the external source implementations was performed. No broad workspace validation is required by this evidence-only assignment. CDC still needs to independently replay the evidence and accept or revise the proposed dispositions.

## Structural receipts and review gates

Structural checks passed before commit: both JSON artifacts parse; the 44 accepted witness records reconcile with population counts and all candidate/source comparison flags; all 35 eligible unmatched and 138 incompatible-type ordinals are retained; the three synthetic controls and ordinal-174 witness are present; the witness JSON has no raw Name, Position, identity, seed, or record-body keys; the artifacts contain no UTF-8 replacement character; the planning diff is exactly the six authorized paths; source HEAD and the input byte count/SHA remain at their fences. Commit trailers are checked against the created commit before return. Semantic assertions and source transcription remain subject to CDC review.

### Remaining gates

- CDC must independently reproduce this one-input decode, population, relation predicate, source comparison and structural checks; CC evidence alone is not acceptance.
- A one-input observation does not decide discovery/base semantics, field-specific authority, zero/missing/malformed/negative/unsafe/over-width behavior, galaxy/reality attribution, planet/system limits, fallback/default/error states, public APIs, native identity, cache transitions, migrations or compatibility.
- No production or adoption decision is made. Project and arc gates remain open until their assigned authority reviews and records them.

## Iteration01 CC correction record — 2026-10-04

This dated addendum supersedes only the affected current conclusions above; the initial closeout remains an unmodified historical prefix at CC commit `8d982eab696ef68eaccd437f12ed4ed49ed8d377`. Current planning intake was clean at reviewer HEAD `246b7f00003c1aa2465a56eb8957da943487882f`; source main stayed clean at `a800950ef3558fcf463517696c3c140061e7353d`. The only save was freshly read again: 2,918,235 bytes, SHA-256 `ee786384b19ae3a7974cb3130cf16b8823f3dc2a930bd55cfa092015a244c89e`; before/after size, mtime_ns, inode, and mode matched. No scope/input/source/reference changed.

### Fresh correction results

The raw native route was rerun from `workbench/save-games/save.hg`, not the previous private decode: 20 blocks, all 2,918,235 compressed bytes, 10,278,541 decompressed bytes, one trailing NUL trimmed, 137 UTF-8 replacement characters, zero duplicate JSON-key objects/occurrences, zero mapped collisions. Integer precision and `Decimal` fractional parsing were retained. The 49 HomePlanetBase/74 Base eligibility and exact Name+Position relation was freshly reconstructed before decoding. It yields the same 44 raw edges, all bidirectionally unique, and exactly the same 44 ordinal pairs as the preserved witness set.

The faithful NMSCoordinates source transcription now produces the independently expanded P/SSI/X/Y/Z, RealityIndex and portal for all 44: components/reality 44/44, portal 44/44. The first-stage `HexToAll` offsets are `0x7FF`/`0x7F`; its separate `GalacticToPortal` stage uses `0x801`/`0x81`. The deliberate negative control substitutes those later offsets into the first stage and keeps the portal stage unchanged; it fails components/reality and portal agreement 44/44. In the example witness, correct outputs are first-stage 1030/130/695, source shifted -1017/3/-1352, portal `400503AB8C07`; negative-control values reproduce the initial bad 1032/132/697, -1015/5/-1350 and `400505ABAC09`. Every corrected derived object matches the faithful `actual` object in CDC's retained receipt and replaces the initial wrong object; the Companion candidate, oracle, association, raw scalar kind/display, IDs, locators, controls, nonzero witness, discovery correlations and all other fields remain unchanged.

Fresh wrapper counts confirm in both contexts current/previous `UniverseAddress` are objects; current/previous `UniversalAddress` are absent. Endpoint `UniverseAddress` is an object for 207/207 BaseContext and 17/17 ExpeditionContext endpoints; endpoint `UniversalAddress` is absent for all 224. `population.json` already encodes these correct states and is unchanged byte-for-byte. The initial prose describing present malformed alternate strings was wrong.

Fresh separate LZ4 fingerprints: header `/opt/homebrew/opt/lz4/include/lz4.h` SHA-256 `26b82efc53d1570f3b54eef02e9c4764c1ad374ff03cac04e2ced5ea4d4c552f`; versioned library `/opt/homebrew/Cellar/lz4/1.10.0/lib/liblz4.1.10.0.dylib` SHA-256 `3ce8bec34557e6a5c76129cef6588dc59959caa51e1e54321f45ea7c6e528e07`. The prior execution text had identified the latter as the header hash; the original line stays in its historical prefix and the correction is explicitly recorded in the appended execution section.

### Current row walk and evidence retention

| Row | Current CC status | Evidence and correction-review applicability |
|---|---|---|
| DW-01 | done, retained | CDC initial verification independently reproduced source/input fences. Fresh input/source state still matches; iteration did not alter source, save, decoder dependencies or fence. Retained with original date; this is not a new acceptance. |
| DW-02 | open, proposed correction | Fresh raw decode and mapping projection reproduce; header identity is now separately hashed and the old library-as-header statement is preserved as an identified historical error. CDC must review the appended execution receipt before row acceptance. |
| DW-03 | open, proposed correction | Structured `population.json` is byte-identical and CDC's whole-population evidence remains applicable. Fresh all-context/current-previous/endpoint spelling checks resolve R03 as absent alternates; assessment/report prose is corrected. CDC review remains pending. |
| DW-04 | done, retained | CDC independently accepted the full association, exclusions, controls and ordinal accounting. Same pinned bytes/mapping/relation predicate remain; none of the corrected literal outputs affects the join. Retained with original date and source. |
| DW-05 | open, corrected evidence | All 44 faithful source outcomes and both flags were freshly derived from raw scalar inputs and explicit endpoint oracles; the wrong-first-stage negative control fails all 44. CDC must review the changed complete delta. |
| DW-06 | open, revised assessment | Field applicability now reports both pinned source candidates agree on the witnessed 44 base associations; alternate wrappers are explicitly absent. Discovery/base, galaxy and production policies remain unresolved; CDC review pending. |
| DW-07 | open, correction validation | Only the five authorized existing files changed; population bytes and all non-literal witness data are preserved. Full delta, privacy and ledger checks are recorded in the validation section below. Commit trailer and post-commit status checks will be reported with the commit receipt. CDC review pending. |

Rows DW-01 and DW-04 remain exactly as accepted in the existing ledger and CDC record. The five open rows stay open; CC evidence is proposed, not independent acceptance. The original seven criteria are unchanged.

### Retained evidence, artifacts and limits

Retained without relabeling as new: CDC's 2026-10-04 accepted DW-01/DW-04 record, the unchanged input/source/mapping pins, complete population and relation projections, Companion results, independent expanded values, synthetic relation controls, discovery scalar correlations, and the 42 address-equality groups. The CDC receipt preserves its 44 initial-vs-faithful source objects and failure details. Fresh work applies only to the literal calculation/flags, the wrapper spelling recheck, fingerprint separation and current narrative. The source pins remain NMSCoordinates `b8f765b03082cf5e130a538896db6d67481f9bae` and Companion `729c319a15f3be66d01587984ef6fff96a50401d`; no repin or external runtime execution occurred.

The five changed paths are `artifacts/location-witnesses.json`, `artifacts/field-assessment.md`, `artifacts/execution.md`, `closing-report.md`, and `ledger.md`. `artifacts/population.json` remains byte-identical. Existing author input, author probe summary, CDC verification/receipts, prompts, plan, source files and parent records are unchanged. No raw names, Positions, owner/account/native IDs, seeds, decoded records or save bytes were added. The temporary private replay script/result are removed after checks. No production, build, test, lint, coverage, second-save or policy work was performed. Slice03/Arc06/Project01 remain open pending CDC and higher-scale gates.


### Structural and privacy validation before commit

The exact witness delta is 44 corrected derived objects plus 88 literal comparison flags (396 changed leaves); all other witness values are structurally identical to initial CC commit `8d982eab696ef68eaccd437f12ed4ed49ed8d377`. The corrected objects match the fresh replay and both literal flags are true for all 44. Population JSON is byte-identical to the initial artifact (SHA-256 `e6e2fef7038d271225a5df8f85684632fb7ecad6ef0c13a014358fedf4195b18`); both JSON files parse. Execution and closing report retain their complete initial bytes as prefixes. The strict privacy scan found no exact saved Name, Position array or string identity values in artifacts; the only saved-name substring match was already present before this iteration. Seven criteria and statuses match reviewer intake; DW-01 and DW-04 lines remain identical, and the other five rows remain open. The pre-commit worktree contains exactly the five authorized paths. No broad build/test/lint/coverage was run. Commit trailers and clean-tree status will be verified after commit and returned with its SHA.


## Iteration02 CC correction record — R04 / DW-07 (2026-10-04)

This pass corrects one evidence-narrative sentence under `cc-prompt-iteration02.md`; the immediately preceding report, execution, witness, and CDC receipt history remains intact. On reviewer intake, candidate `13409d1e32a86f0d16cae8163c696f1e37693c5c` was at planning HEAD `92c08c3dda1369da0da215f471669beb92c672d5`; source main remains clean at `a800950ef3558fcf463517696c3c140061e7353d`. The unchanged input is `local-save-A`, 2,918,235 bytes, SHA-256 `ee786384b19ae3a7974cb3130cf16b8823f3dc2a930bd55cfa092015a244c89e`. This pass did not decode the save or recalculate candidates.

The stable-ID, full-object comparison recorded in the appended execution section resolves the direction unambiguously: current corrected literals equal the initial CDC `actual` objects 44/44 and the CDC `expected` objects 0/44; the initial witness literals equal `expected` 44/44 and `actual` 0/44. The `actual` objects are the faithful source transcription; `expected` preserves the initial incorrect witness values. The Iteration01 CDC receipt independently records the same direction and all 44 negative controls; this CC pass used the current and initial immutable objects to reproduce the equality counts.

The sentence beginning “All 44 fresh NMSCoordinates results equal the CDC receipt's faithful `actual` objects and the original witness objects' CDC-marked incorrect `expected` objects” in the preserved Iteration01 execution prefix is superseded by the correction in the following section. It incorrectly attributes both equalities to the corrected values. The closing report's earlier replacement wording remains a correct control and has not been rewritten.

### Row walk and retained evidence

| Row | CC disposition | Evidence / applicability |
|---|---|---|
| DW-01 | done, retained | CDC Iteration01 independently reproduced the source/input fence and readback. Source, input, and criteria remain unchanged; retain its original 2026-10-04 evidence date. |
| DW-02 | done, retained | CDC Iteration01 reproduced raw decode, mapping, sanitation, precision caveats, and separate header/library identities. No decode or dependency changed in this narrative pass. |
| DW-03 | done, retained | CDC Iteration01 independently reproduced complete population projections and alternate-wrapper absence; population and current spelling interpretation are unchanged. |
| DW-04 | done, retained | CDC Iteration01 reproduced the complete raw association set, eligibility, exclusions, and controls; association inputs and predicate are unchanged. |
| DW-05 | done, retained | CDC Iteration01 reproduced all 44 corrected source comparisons and rejected all 44 wrong-offset controls. This pass confirms receipt direction against current and initial objects; no numerical evidence changed. |
| DW-06 | done, retained | CDC Iteration01 accepted the corrected source applicability assessment and held policy limits. The assessment and scope are unchanged. |
| DW-07 | open, CC correction proposed | Current/initial/receipt full-object direction counts reproduce; the prior execution sentence is explicitly superseded below. CDC must review this three-file delta before acceptance. |

DW-01 through DW-06 retain the independent CDC Iteration01 decision and evidence date; this CC pass did not relabel them as fresh acceptance. DW-07 remains open. The seven criteria, acceptance scope, and production/API/compatibility/galaxy/cache gates are unchanged. Slice03, Arc06, and Project01 remain open pending independent CDC review and their existing higher-scale gates.

### Files, checks, and limits

The only modified paths are `artifacts/execution.md`, `closing-report.md`, and `ledger.md`; in the ledger only DW-07 Evidence and Notes changed. The complete pre-iteration execution and closing-report bytes are preserved as prefixes, including the incorrect sentence. All 218 witness IDs/classes remain unchanged; the comparison used all 44 raw-corroborated records, not a sample. Population and location-witness JSON, field assessment, CDC receipts, six accepted ledger rows, seven criteria, plans, prompts, source and save remain unchanged. The current report has no new raw identity values or decoded records. This was a focused standard-library JSON equality check and text/ledger validation only; no build, test, lint, coverage, decoder, external runtime, policy work, or external message was run.
