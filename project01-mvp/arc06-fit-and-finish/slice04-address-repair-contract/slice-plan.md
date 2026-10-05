# Slice04 — Address repair contract decision packet

Version: 1.2, opened 2026-10-04. Status: active, evidence/design preparation only. Two-Contributor Workflow; Expedited Mode. Depends on independently closed Slice03. Source main `a800950ef3558fcf463517696c3c140061e7353d`.

## Outcome and boundaries

Deliver an actionable decision packet for the existing address repair prerequisite: distinguish save-packed families from the public48-bit portal representation, characterize invalid/default states, and map galaxy identity, query, watcher and cache consequences. The packet recommends specific choices with rationale, proof obligations and decision owners. It does not implement or approve those choices. No new database/creature/search/save-identity redesign, new private save, decoder generalization, publication or production mutation. Original milestone acceptance remains unchanged.

Binding inspection scope is in cc-prompt.md: all20 predecessor caller rows plus their named direct consumers, eight decision topics,45 synthetic parser runs and six integration controls. Deliverables under artifacts/: decision-packet.md, impact-map.csv, behavior-probes.json, execution.md; root closing-report.md and ledger.md Evidence/Notes only. author-baseline.json and the issued open set are immutable preparation. No close/review record is pre-created.

## Verification and authority

Seven RP rows govern acceptance of the packet's fidelity, not adoption of a decoder or desired policy. Source facts, accepted one-input observations, synthetic results, author recommendations and unresolved policy must remain distinct. Unknown/unsupported outcomes with concrete owner/re-entry conditions are legitimate investigation findings. CDC independently checks CC outputs, then resolves the consequential design/Operator gates before a separate production assignment. Never turn recommended types/errors/migrations into current APIs or a promised implementation.

## Author readiness

Inspected source conversion/model/address types, SystemId, graph extraction/index/insertion/delta paths, query result glyph construction, CSV import, watcher snapshots/deltas and cache data/freshness/serialization/rebuild; interface conversion bodies and startup load_model; existing CLI masking and multi-galaxy tests. Exact sections are in the prompt. Source remains clean/pinned. Built nms-cli offline/locked successfully and exercised three CLI probes: save-shaped --ga gives2B0004B2446A, portal input gives212B04B2446A, equal portal coordinates in galaxies0/9 import as1 system plus1 duplicate. Existing graph test uses different coordinates/SSI across galaxies and cannot reject that collision. Then exercised all45 manifest parser runs (27 accepted /18 rejected;16 discovery,16 base,13 expanded) and all six controls, including reverse-order collision and distinct-SSI import. The complete recipe is executable; author observations are preparation, not independent acceptance of future CC work.

Applied rust-guidelines API-37/TD-03 (SHOULD) to separate roles in recommendations, AP-33 (AVOID) to expose dropped/defaulted states and EH-04 (MUST for future library APIs) to require matchable errors in any proposed fallible signature. Read exact cited sections; no new trait, dependency, edition, logging or runtime choice required. Existing Rust2024/MSRV1.91 retained. Proportionate proof uses existing CLI and scoped source checks; no reusable validation product or disposable Rust crate needed.

The executable recipe names a complete45-case manifest, field injection paths, existing CLI routes, source-derived discrimination controls and exact consumer inventory. Author has supplied valid synthetic root/base/discovery templates and observed representative routes, not precomputed future CC results. Required reading is bounded source sections plus small evidence projections; expected packet fits one context with correction headroom. Consequential policy is unresolved by design and belongs to CDC/Operator; dependent implementation remains held.

## Assignment history and version history

- Initial cc-prompt.md, issued2026-10-04, returned for review at74359a9; CDC accepts RP-01 and returns six rows for correction. Initial prompt preserved.
- cc-prompt-iteration01.md, issued2026-10-04, predecessor cc-prompt.md; returned for independent review at7d33b1d; substantive corrections accepted, RP-07 exact capture mismatch remains. Executed refinement passes:1.
- cc-prompt-iteration02.md, issued2026-10-04, predecessor cc-prompt-iteration01.md; active RP-07-only raw-build-capture correction. Iteration02 not yet executed.
- Current assignment: `arc06-fit-and-finish/slice04-address-repair-contract/cc-prompt-iteration02.md`.
- Version1.0: opened after Slice03 closes, detailing the existing address-contract-before-repair step; no original scope, architecture, compatibility or acceptance amendment.

## Author packet checks

Checked the exact eleven-path reviewer/open-set diff, seven unchanged predecessor criteria now closed, six previously accepted ledger rows byte-identical, all three issued predecessor prompts and CC/data/receipt files unchanged, seven new rows open, all fifteen original arc rows and twenty-two project rows still open, JSON syntax, all52 local Markdown links, case IDs/counts, whitespace, pinned clean source and unchanged save hash. The new slice has no pre-created close/review report. These checks establish preparation and preservation, not independent acceptance of the new packet or production policy. The author build and51 synthetic executions are recorded separately from the sentence-only Slice03 review, which ran neither decode nor broad production checks.

## Initial CDC review and iteration01 readiness

CDC independently replayed all45 parser cases27/18, five prescribed controls, the missing prescribed sixth control and CC supplement (52 executions total); retained complete injection/object/seed coverage. [Review](cdc-verification.md) accepts RP-01 only. Six rows remain open because delivered coverage substitutes a control, Chili provenance/name source claims need correction, and removed-key watcher proof has no current API. The packet is not closed or adopted.

Iteration01 author inspected the exact distinct_csv and fresh import stderr route, real RawDiscoveryRecord.dm/DM.CN witness and conversion/extraction bodies, and complete SaveDelta/compute_delta observable fields. It supplies a concrete one-control recipe and source-grounded future addition test. No all45 rerun is required when full original result objects/source/baseline are preserved. Scope remains six CC paths; execution and closing history are exact retained prefixes, RP-01 is byte-stable and all production decisions stay held. Domain guideline applications stay at proposal/error/evidence-role level; no code/API design selected.

Version1.1: initial independent review and concrete iteration01 issued inside the existing slice; original criteria, architecture, repair sequence and compatibility/Operator gates unchanged.

## Iteration01 verdict and iteration02 readiness

CDC accepts RP-02–RP-06 after independently checking full51-object retention, exact prescribed import, complete corrected packet, witness/source contracts and S06. RP-01 retains acceptance. Only RP-07 is open: iteration01_build's claimed exact diagnostic omits one word from actual Cargo output. The build itself passes; no product repair is required. Prior failed history and original issued prompts remain preserved.

Author directly captured the current Cargo build, confirmed the diagnostic mismatch (normalizing elapsed duration only), and checked the available raw-stream→JSON byte-equality method. Iteration02 narrows work to four existing CC files and appends one new build object/note; all52 result objects, decision/map files, and six accepted rows are frozen. No substantive contract, code, policy, scope or compatibility choice changes. This is one bounded metadata refinement within the existing slice, not another evidence investigation.

Version1.2 (2026-10-04): iteration01 reviewed and one executed refinement recorded; five additional rows accepted, one exact-capture issue returned in iteration02. Original seven criteria and production gates unchanged.
