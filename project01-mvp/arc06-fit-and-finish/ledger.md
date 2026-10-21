# arc06-fit-and-finish — recovered acceptance ledger

Created 2026-10-03 during closeout preparation. Original milestone documents remain normative historical inputs; no historical acceptance is invented. All rows are open pending scoped evidence and independent review. Baseline successes do not close individual capabilities.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| M7.1 | `nms export` command: Export filtered results as JSON, CSV. `--biome`, `--within`, `--format` flags; Scriptable data extraction | `cargo test -p nms-cli` plus independent demonstration of the stated deliverable and original milestone reconciliation | correctness-grade | [README.md:205](../README.md) | open | — | No result inferred from test count alone |
| M7.2 | Shell completions: `clap_complete` for bash, zsh, fish; Install instructions in README | `cargo test -p nms-cli` plus independent demonstration of the stated deliverable and original milestone reconciliation | correctness-grade | [README.md:206](../README.md) | open | — | No result inferred from test count alone |
| M7.3 | Multi-save support: Detect and list all save slots. `--slot` flag or interactive selector; Choose between save slots | `cargo test -p nms-save` plus independent demonstration of the stated deliverable and original milestone reconciliation | correctness-grade | [README.md:207](../README.md) | open | — | No result inferred from test count alone |
| M7.4 | Multi-galaxy support: Separate spatial index per galaxy, cross-galaxy queries, galaxy switching in REPL; Full 256-galaxy support | `cargo test -p nms-graph` plus independent demonstration of the stated deliverable and original milestone reconciliation | correctness-grade | [README.md:208](../README.md) | open | — | No result inferred from test count alone |
| M7.5 | NomNom format: Parser for NomNom export format (if distinct from standard deobfuscated JSON); Additional format support | `cargo test -p nms-compat` plus independent demonstration of the stated deliverable and original milestone reconciliation | correctness-grade | [README.md:209](../README.md) | open | — | No result inferred from test count alone |
| M7.6 | Documentation: README with screenshots/examples, rustdoc for all public APIs, CONTRIBUTING guide; Publish-ready docs | `cargo test --workspace --all-features` plus independent demonstration of the stated deliverable and original milestone reconciliation | correctness-grade | [README.md:210](../README.md) | open | — | No result inferred from test count alone |
| M7.7 | Integration tests: End-to-end tests with fixture save files covering all commands; CI green with full coverage | `cargo test --workspace --all-features` plus independent demonstration of the stated deliverable and original milestone reconciliation | correctness-grade | [README.md:211](../README.md) | open | — | No result inferred from test count alone |
| M7.8 | crates.io publishing: Publish `nms`, `nms-core`, `nms-save`, `nms-graph`, `nms-copilot` to crates.io; `cargo install nms-copilot` works | `cargo test --workspace --all-features` plus independent demonstration of the stated deliverable and original milestone reconciliation | correctness-grade | [README.md:212](../README.md) | open | — | No result inferred from test count alone |
| M7.9 | Community data import: Import external coordinate lists (NMSCE, community CSV) as "external" discoveries; Enriched galaxy model | `cargo test -p nms-graph` plus independent demonstration of the stated deliverable and original milestone reconciliation | correctness-grade | [README.md:213](../README.md) | open | — | No result inferred from test count alone |
| M7.10 | Color themes: Configurable color schemes for terminal output, biome color mapping; Pretty by default, customizable | `cargo test -p nms-copilot` plus independent demonstration of the stated deliverable and original milestone reconciliation | correctness-grade | [README.md:214](../README.md) | open | — | No result inferred from test count alone |
| D0040 | Configurable Banner Support for nms-copilot REPL: reconcile the complete original document with current source and any recorded amendments | Read [0040-configurable-banner-support-for-nms-copilot-repl.md](0040-configurable-banner-support-for-nms-copilot-repl.md); inspect its source witnesses and reproduce its promised behavior | correctness-grade | Historical document D0040 | open | — | A heading/inventory read is not semantic acceptance |
| D0041 | REPL Setup Wizard & Config Improvements: reconcile the complete original document with current source and any recorded amendments | Read [0041-repl-setup-wizard-config-improvements.md](0041-repl-setup-wizard-config-improvements.md); inspect its source witnesses and reproduce its promised behavior | correctness-grade | Historical document D0041 | open | — | A heading/inventory read is not semantic acceptance |
| D0042 | Interactive ASCII Galaxy Map (with ratatui): reconcile the complete original document with current source and any recorded amendments | Read [0042-interactive-ascii-galaxy-map-with-ratatui.md](0042-interactive-ascii-galaxy-map-with-ratatui.md); inspect its source witnesses and reproduce its promised behavior | correctness-grade | Historical document D0042 | open | — | A heading/inventory read is not semantic acceptance |
| D0043 | Adopt oxur-cli tables for all tabular output: reconcile the complete original document with current source and any recorded amendments | Read [0043-adopt-oxur-cli-tables-for-all-tabular-output.md](0043-adopt-oxur-cli-tables-for-all-tabular-output.md); inspect its source witnesses and reproduce its promised behavior | correctness-grade | Historical document D0043 | open | — | A heading/inventory read is not semantic acceptance |
| C-06 | Exports, slot/galaxy support, format adapters/imports, documentation, distribution, and later UI refinements meet their original contracts. | End-to-end arc demonstration; walk every row and all original document requirements, including non-table requirements | correctness-grade | Project plan and historical leaf documents | open | — | Requires independent composition evidence |

## Corrective investigation child rows

These add to, rather than replace, the 15 original recovered arc rows above. Closing investigation children does not close the original capabilities.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-S01 | Current address caller/format investigation independently accepted | Eight-row slice ledger and CDC final review | correctness-grade | Closeout F01/F10 | done | [final review](slice01-address-encoding-evidence/cdc-verification.md) — reproduced | Investigation closed; production defects and semantics still open |
| R06-S02 | Primary packed-field authority and bounded witness investigation independently accepted | Slice02 ledger, source applicability and independent exact-value checks | correctness-grade | Slice01 unresolved DD.UA/base/galaxy findings | done | [CDC final review](slice02-packed-format-authority/cdc-verification.md) — reproduced | Seven investigation rows accepted; wider semantics/production remain open |
| R06-S03 | Decoded local-save population and location-witness assessment independently accepted | Slice03 ledger, pinned byte replay and raw association/oracle checks | correctness-grade | Slice02 unavailable-pair re-entry | done | [CDC final review](slice03-decoded-location-witnesses/cdc-verification.md) — reproduced | Seven child rows accepted; one bounded input, no universal policy |
| R06-S04 | Address repair decision packet independently accepted without production adoption inference | Seven-row Slice04 ledger; source/behavior/impact and consequential policy review | correctness-grade | Existing address contract prerequisite / Slice03 bubble-up | done | [CDC final review](slice04-address-repair-contract/cdc-verification.md) — reproduced | Seven preparation rows accepted after iteration02. Policy/API/identity/cache choices and all original arc/project capabilities remain held/open. |

## Approved repair requirements and implementation child — 2026-10-04

The Operator approved scope expansion after Slice04. All15 original rows and four accepted investigation/preparation children remain; total23 arc rows (four done,19 open). Earlier descriptions of policy being held describe their historical review state; current approval is in arc-plan.md v1.11. Delivery remains unaccepted.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-R01 | Distinct fallible save boundary produces correct field-specific addresses while 48-bit portal/core conversion remains compatible | Exact independent game/base/expanded oracles and parser-to-interface regressions; adopted input/reality/error policy checked | serious | Operator approval2026-10-04; D01/D05 | open | — | Save semantics remain evidence-gated; preparation is insufficient |
| R06-R02 | Galaxy-qualified identity retains same-coordinate systems/planets without collisions in all consumers | Both-order galaxies0/9 fixtures through graph/query/import/route/watch additions and cache restoration | serious | Operator approval2026-10-04; D06 | open | — | Explicit key, not opaque numeric repacking; no new watcher removal API |
| R06-R03 | Cache compatibility composes with every decoder/identity semantic transition and normal lifecycle | Slice05 review plus later revision-bump and old-cache/live-update proofs on the final repair candidate | serious | Operator approval2026-10-04; D08 | open | — | Header protection alone does not prove later migrations or full C04 lifecycle |
| R06-S05 | Versioned semantic cache protection independently accepted | Seven-row Slice05 ledger, source diff, header/error and real load_or_rebuild tests | correctness-grade | R06-R03 first implementation dependency | done | [CDC final review](slice05-cache-semantic-compatibility/cdc-verification.md#iteration01-final-independent-cdc-review--2026-10-10) — reproduced | Seven child rows accepted at decd8d6 after iteration01; later semantic-transition/composition parent remains open |

## Identity foundation child — 2026-10-10

Prior23 rows and their criteria/dispositions remain;24 total, six done/eighteen open. R06-R02/R03 require the subsequent identity composition and later semantic-transition/live evidence as well as this child.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-S06 | Galaxy-qualified key/index and cache foundation independently accepted | Eight-row Slice06 ledger; both-order collisions, canonical serde, import/show/snapshot and predecessor cache proofs | correctness-grade | Approved R06-R02/R03 bounded implementation | done | [CDC final closure](slice06-galaxy-qualified-identity/cdc-verification.md#operator-approved-exception-and-final-closure--2026-10-10) — reproduced with approved evidence exception | Eight child rows accepted at f57a38f; required parent identity composition remains open |

## Query composition child — 2026-10-10

Prior24 rows and all criteria/dispositions retained;25 total, six done/nineteen open. Parent identity/cache and original capability criteria remain open through required subsequent routing/session/live and save-repair composition.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-S07 | Galaxy-scoped query associations/names/filters/reference/ranking/show and raw-cache compatibility independently accepted | Nine-row Slice07 ledger, native literal regressions, actual CLI and prior2/2 archive parity | correctness-grade | Approved R06-R02/R03 subsequent composition / original query correctness | done | [Final CDC review](slice07-galaxy-scoped-queries/cdc-verification.md#iteration02-independent-review-and-final-closure--2026-10-11) — all nine child rows independently accepted at7118a486 | Child closed; remaining composition and original/expanded parent gates stay open |

## Slice08 required spatial/routing composition child — 2026-10-11

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-S08 | Galaxy-scoped spatial/routing composition independently accepted | Nine-row Slice08 ledger; native exact routes, literal neighborhoods, actual CLI and raw-cache parity | correctness-grade | Approved R06-R02/R03 identity composition and original correctness | done | [Final closure](slice08-galaxy-scoped-routing/cdc-verification.md#operator-approved-sr-09-exception-and-final-closure--2026-10-11): independent CC current proof plus Operator-approved specific SR09 evidence amendment, at f71d607. | Nine child rows closed; listed historical data remains unknown. Required session/live/startup/save and original parent gates remain open; no source integration. |

Slice08 closure2026-10-11:26 rows/eight done/eighteen open; all25 other entire rows and criteria retained. Operator exception applies only to the seven listed Slice08 past observations; no parent/current/future validation waiver.


## Required session consumer child — 2026-10-11

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-S09 | Galaxy-scoped movement/session/location consumers independently accepted | Seven-row Slice09 ledger and actual typed producer/drain/cache/MCP registry proofs, current native gates | correctness-grade | Approved R06-R02/R03 identity composition and original M7.4 | done | [Independent CDC review](slice09-galaxy-scoped-session/cdc-verification.md), e9cabd5;717/213/913 and all gates reproduced2026-10-16. | Seven child rows closed as reviewed candidate; all broader integration/lifecycle/save and original parent gates stay open. |

27 rows/eight done/nineteen open. All prior26 entire rows and criteria/dispositions retained. New child is a bounded owner within the approved required identity consumer route, not a replacement for parent acceptance.

Slice09 independent closure2026-10-16:27 rows/nine done/eighteen open. Parent composition remains open; source candidate e9cabd5 not integrated into main.


## Required usable REPL command child — 2026-10-16

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-S10 | REPL galaxy selection and query-reference increment independently accepted | Seven-row Slice10 ledger; actual parser/dispatch/render/completion paths, production wiring/docs and native gates | correctness-grade | Original M7.4 and approved R06-R02/R03 consumer composition | done | [CDC final closure](slice10-galaxy-scoped-repl/cdc-verification.md#operator-approved-historical-exception-and-final-closure--2026-10-16), candidate e615f254 — reproduced plus explicit accepted historical amendment | Seven child rows accepted; Operator waiver only original1–53 environment metadata. Main integration, live/lifecycle and original parent gates remain open. |

28 rows/nine done/nineteen open; all prior27 entire rows/criteria/dispositions retained. This child details the existing owner, not a replacement for parent composition.

## Live MCP/watch implementation child — 2026-10-16

Adds one child to28 prior rows;29 total/ten done/nineteen open. All prior criteria/statuses unchanged.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-S11 | Reliable real MCP save tracking independently accepted | Slice11 seven-row ledger, native stdio/HTTP/filesystem subscription proof and CDC review | correctness-grade | Planned live/lifecycle consumer composition after Slice10 | done | Slice11 cdc-verification.md iteration01 / artifacts/cdc-review-receipts.json iteration01_review ; final Slice11 iteration02 acceptance239313 / cdc-verification.md | Broader startup-cache/unknown-state/config composition remains planned12 before parent acceptance; no original scope dropped. ; 2026-10-17: four child rows accepted, LW-03/05/07 open; current assignment iteration02 test/evidence correction. ; 2026-10-17 formal child closure: all seven rows done; broader planned12/parent scope remains open. |
| R06-S12 | Cached physical player-state integrity and authoritative recovery independently accepted | Slice12 seven CI rows, byte/file codec rejection, genuine fresh-cache fallback/warm and preservation controls, CDC review | correctness-grade | Existing historical-state/cache composition owner after Slice11 | done | Slice12 cdc-verification.md / artifacts/cdc-review-receipts.json — all seven rows accepted at a101bd44 | Remaining full startup/live C04/unknown/watch configuration/duplicate-producer composition explicitly planned13 before parent acceptance. ; 2026-10-17 two child rows accepted/five open, iteration01 test/doc/evidence correction; no parent acceptance.  Superseding2026-10-17: Iteration01 independently accepted, no parent composition acceptance. |

2026-10-17 Slice12 final child closure:30 rows/12 done/18 open; original parent and planned13 obligations remain.


## Slice13 — existing watcher settings increment

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-S13 | Existing watcher enabled/debounce settings applied in actual MCP startup modes with default API compatibility | Seven WS rows, actual main configured wiring, typed forwarding, disabled/enabled native children and independent CDC review | correctness-grade | Existing configuration/composition owner after Slice12 | done | Slice13 cdc-verification.md / artifacts/cdc-review-receipts.json — reproduced at16c0787 | All full startup/live-cache/unknown-system/duplicate-producer/broader history remainder required planned14/CDC; no parent acceptance  Superseding2026-10-17 seven child rows independently accepted; no parent composition acceptance. |

2026-10-17:31 arc rows/12 done/19 open; seven WS child rows open, project25 open.

2026-10-17 Slice13 closed:31 arc rows/13 done/18 open; all original composition gates remain.


## Slice14 — native headless cache startup increment

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-S14 | Initial headless startup cache composition independently accepted through real transports | Seven HC rows, eight literal cases per stdio/HTTP, opposed source/cache identity and exact persisted effects, independent CDC review | correctness-grade | Existing C04/startup cache composition owner after Slice13 | done | Slice14 cdc-verification.md / artifacts/cdc-review-receipts.json — all seven HC rows accepted at130034e | Full remaining normal/live/write-through/unknown/duplicate-producer/history/parent composition required planned15/CDC  2026-10-17 four HC rows accepted/three open; iteration01 test/evidence correction, no parent acceptance.  Superseding2026-10-17 child closed; no full C04 or parent acceptance. |

2026-10-17:32 arc rows/13 done/19 open; seven HC rows open; project25 open.

2026-10-17 Slice14 closed:32 arc rows/14 done/18 open; every original remaining owner/gate preserved.


## Slice15 — complete parsed live save update API

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-S15 | Complete same-parse supported source-state/version watch→graph API independently accepted with legacy compatibility and owned producer lifetime | Seven CU rows, native full-state/state-only/version-only/failure/recovery/legacy checks, graph authority/selection/atlas tests and independent CDC review | correctness-grade | Operator-adopted complete-state ownership direction; source-grounded bounded API increment | done | child all7 CU done at11cf7e6: final CDC verification, fresh rebased gates, approved initial-capture exception69928e5 and retrospective | Required16 common coordinator/headless live cache;17 normal REPL/background lifetime/PTY; unknown/history/original parent gates retained |

2026-10-19 Slice15 open:33 arc rows/14 done/19 open; parent capabilities remain open.

2026-10-19 Slice15 closed:33 arc rows/15 done/18 open.

## Slice16 — common coordinator and headless live cache

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-S16 | Common app-owned full-update apply/cache coordinator and actual headless stdio/HTTP live-cache composition independently accepted | Eight LC rows, lock/order/error/no_cache/lifetime controls and native full-state/cache/observer matrix plus CDC review | correctness-grade | Adopted single-owner direction / source-grounded headless increment | done | [Final CDC closure](slice16-headless-live-cache/cdc-verification.md#operator-approved-lc08-exception-and-final-slice16-closure--2026-10-20-utc) at e0939229 — all8LC accepted, explicit bounded LC08 amendment | Normal SAME-owner REPL/background/PTY remains required17; unknown/history/original parent gates retained |

2026-10-19 Slice16 opened:34 arc rows/15 done/19 open; all original parent capabilities remain open.

2026-10-20 Slice16 final closure:34 arc rows/16 done/18 open; all original criteria and required17/parent obligations preserved.


## Required normal composition child — 2026-10-20 UTC

Previous34rows/16done/18open retained. NewR06-S17 gives35rows/16done/19open; all25projectrows remainopen.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-S17 | Normal REPL and shared background MCP use SAME accepted coordinator with owned lifecycle/current session/full native cache composition | Eight RC rows, actual normalPTY/HTTP/cache boundary matrix, retained headless/legacy and independent CDC review | correctness-grade | Adopted one-owner direction / required remainder from accepted16 | done | [approved final closure](slice17-repl-live-cache/cdc-verification.md#operator-approved-rc-08-historical-exceptions-and-final-closure--2026-10-20-utc) | All8 child rows done on61f1d98: RC01–07 independently reproduced; RC08 exact Operator-approved historical exceptions only. Unintegrated/uninstalled; parent/policy gates retained. |

2026-10-20 Slice17 closed:35 arc rows/17 done/18 open; project25 open. Exact approved child capture exceptions do not close original parent/policy/integration/release obligations.


## Slice18 explicit placeholder/cache foundation

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-S18 | Explicit model synthetic provenance/promotion/import and3/3 checked cache foundation independently accepted | Eight UP rows, exact whole-index/cache/raw negatives/old predecessor fallback/current regressions/CDC review | correctness-grade | Operator-approved policy A | done | [approved CDC closure](slice18-placeholder-cache/cdc-verification.md#operator-approved-up-08-exception-and-final-slice18-closure--2026-10-20-utc) — all8UP accepted on e592ecf | Exact12 historical status observations excepted by Operator; required19 application composition and original parent gates retained. |

36arcrows17done19open/project25open; all previous35entire rows and criteria/dispositions retained.

2026-10-20 Slice18 current independent review: seven child UP rows accepted/one open; R06-S18 remains open. Arc36rows17done19open/project25open. Exact bounded historical decision in CDC packet; no exception or next dispatch inferred.
2026-10-20 Slice18 final closure:36arcrows18done18open/project25open; exact approved exception only, required19 retained.


## Slice19 required application unknown-system composition

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-S19 | Common startup/live ensure, exact physical labels and actual normal/stdio/HTTP unknown→real→compatible restart independently accepted | Eight UL rows, literal full model/cache/location/file controls, real transports/nativePTY and CDC review | correctness-grade | ApprovedA / required19 after18 | done | [final CDC acceptance](slice19-unknown-system-lifecycle/cdc-verification.md#final-independent-acceptance--2026-10-21-utc) — reproduced | Clean4cb745b7; all8UL accepted, independent19ordinary/native10/full1009. Prior failures/unknown teardown cause retained; all original parents remain open |

2026-10-21 opened19:37arcrows18done19open/project25open; all36previous entire rows/criteria/dispositions retained.

2026-10-21 Slice19 current independent red review/cap gate:37arcrows18done19open/project25open; no row closed or silent scope change.

2026-10-21 Operator explicitly approved ONE exceptional sixth bounded F19-01/F19-02 correction; no budget reset/waiver/rowclosure,37arcrows18done19open/project25open.

2026-10-21 Slice19 independently closed after approved exceptional sixth pass:37arcrows19done18open/project25open. All15original and three repairparents remainopen.


## Public save-address foundation child — 2026-10-21 UTC

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| R06-S20 | Public original-JSON save-address decoder foundation independently accepted | Seven ADrows,45oracles/strictmatrix/publicdocs/orderedgates and independentCDC review | correctness-grade | ApprovedA / R06-R01 dependency | open | — | Required21 applicationatomicvalidation/semantic3→4/nativecomposition retained; no parentacceptance |

38arcrows19done19open/project25open; all37previousrows retained.
