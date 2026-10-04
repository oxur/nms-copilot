# Slice04 — Address repair contract decision packet

Version: 1.0, opened 2026-10-04. Status: active, evidence/design preparation only. Two-Contributor Workflow; Expedited Mode. Depends on independently closed Slice03. Source main `a800950ef3558fcf463517696c3c140061e7353d`.

## Outcome and boundaries

Deliver an actionable decision packet for the existing address repair prerequisite: distinguish save-packed families from the public48-bit portal representation, characterize invalid/default states, and map galaxy identity, query, watcher and cache consequences. The packet recommends specific choices with rationale, proof obligations and decision owners. It does not implement or approve those choices. No new database/creature/search/save-identity redesign, new private save, decoder generalization, publication or production mutation. Original milestone acceptance remains unchanged.

Binding inspection scope is in cc-prompt.md: all20 predecessor caller rows plus their named direct consumers, eight decision topics,45 synthetic parser runs and six integration controls. Deliverables under artifacts/: decision-packet.md, impact-map.csv, behavior-probes.json, execution.md; root closing-report.md and ledger.md Evidence/Notes only. author-baseline.json and the issued open set are immutable preparation. No close/review record is pre-created.

## Verification and authority

Seven open RP rows govern acceptance of the packet's fidelity, not adoption of a decoder or desired policy. Source facts, accepted one-input observations, synthetic results, author recommendations and unresolved policy must remain distinct. Unknown/unsupported outcomes with concrete owner/re-entry conditions are legitimate investigation findings. CDC independently checks CC outputs, then resolves the consequential design/Operator gates before a separate production assignment. Never turn recommended types/errors/migrations into current APIs or a promised implementation.

## Author readiness

Inspected source conversion/model/address types, SystemId, graph extraction/index/insertion/delta paths, query result glyph construction, CSV import, watcher snapshots/deltas and cache data/freshness/serialization/rebuild; interface conversion bodies and startup load_model; existing CLI masking and multi-galaxy tests. Exact sections are in the prompt. Source remains clean/pinned. Built nms-cli offline/locked successfully and exercised three CLI probes: save-shaped --ga gives2B0004B2446A, portal input gives212B04B2446A, equal portal coordinates in galaxies0/9 import as1 system plus1 duplicate. Existing graph test uses different coordinates/SSI across galaxies and cannot reject that collision. Then exercised all45 manifest parser runs (27 accepted /18 rejected;16 discovery,16 base,13 expanded) and all six controls, including reverse-order collision and distinct-SSI import. The complete recipe is executable; author observations are preparation, not independent acceptance of future CC work.

Applied rust-guidelines API-37/TD-03 (SHOULD) to separate roles in recommendations, AP-33 (AVOID) to expose dropped/defaulted states and EH-04 (MUST for future library APIs) to require matchable errors in any proposed fallible signature. Read exact cited sections; no new trait, dependency, edition, logging or runtime choice required. Existing Rust2024/MSRV1.91 retained. Proportionate proof uses existing CLI and scoped source checks; no reusable validation product or disposable Rust crate needed.

The executable recipe names a complete45-case manifest, field injection paths, existing CLI routes, source-derived discrimination controls and exact consumer inventory. Author has supplied valid synthetic root/base/discovery templates and observed representative routes, not precomputed future CC results. Required reading is bounded source sections plus small evidence projections; expected packet fits one context with correction headroom. Consequential policy is unresolved by design and belongs to CDC/Operator; dependent implementation remains held.

## Assignment history and version history

- Initial cc-prompt.md, issued2026-10-04, active for fresh CC execution, then Operator relay to CDC/Sofie. Executed refinement passes:0.
- Version1.0: opened after Slice03 closes, detailing the existing address-contract-before-repair step; no original scope, architecture, compatibility or acceptance amendment.

## Author packet checks

Checked the exact eleven-path reviewer/open-set diff, seven unchanged predecessor criteria now closed, six previously accepted ledger rows byte-identical, all three issued predecessor prompts and CC/data/receipt files unchanged, seven new rows open, all fifteen original arc rows and twenty-two project rows still open, JSON syntax, all52 local Markdown links, case IDs/counts, whitespace, pinned clean source and unchanged save hash. The new slice has no pre-created close/review report. These checks establish preparation and preservation, not independent acceptance of the new packet or production policy. The author build and51 synthetic executions are recorded separately from the sentence-only Slice03 review, which ran neither decode nor broad production checks.
