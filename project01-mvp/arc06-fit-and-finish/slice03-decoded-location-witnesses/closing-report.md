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
