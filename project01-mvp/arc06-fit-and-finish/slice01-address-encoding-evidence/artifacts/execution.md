# CC execution record — address encoding evidence

## Scope and checkout state

- Assignment: `project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence/cc-prompt.md`, read in full (58 lines). Evidence-only. No production, dependency, configuration, fixture, game-save, independent-verification, arc, or project edits are authorized here.
- Source root `/Users/oubiwann/lab/oxur/nms-copilot`: `main`, HEAD `a800950ef3558fcf463517696c3c140061e7353d`, clean at intake and rechecked after build and probes.
- Planning root `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/planning`: branch `planning`, HEAD `3fcf3ceb476473738499dd1052395819d75056d`, clean at intake. The prepared packet is at this HEAD; it is two commits ahead of `origin/planning`. CC outputs are limited to this slice's `artifacts/`, `closing-report.md`, and supported evidence updates to this slice ledger.
- No source changes were made. Current workflow is Two-Contributor (CDC + CC); independent CDC review has not happened. All evidence here is `attested`, and row outcomes remain proposed until CDC reproduces them.

## Required reading and actual coverage

| Required input | Loaded extent / data query | Coverage and limit |
|---|---|---|
| Prompt, sibling slice plan and ledger, committed preparation packet | Prompt all 58 lines; slice plan all 33 lines; ledger all 14 lines; preparation commit `3fcf3ceb476473738499dd1052395819d75056d` touched 54 files. Read `project-plan.md` §11 lines 551–564 and §12 lines 565–568; `closure-preparation.md` Baseline findings lines 15–19, Ordered route lines 21–29, Gates and authority lines 39–43; baseline report Behavior probes lines 37–43 and Other source-confirmed/unresolved findings lines 45–58. | All eight AE rows and current sequencing/authority read. Historical acceptance population remains outside this slice. |
| Project management and ledger / evidence method | Project-management `guides/README.md` all 133 lines; `04-closing-slices.md` all 108 lines; plan-change discipline in `05-closing-arcs.md` lines 76–108. Work-verification `01-ledger-discipline.md` all 84 lines, `02-evidence-strength.md` all 61, `03-row-closure.md` all 67, `04-silent-drop-checks.md` all 59, and `05-independent-verification.md` all 103. | Used canonical artifact homes, row status/strength rules, slice close report and bubble-up requirements, independent-review gate, and plan-change authority. |
| Collaboration role and posture | Engineering methodology role/workflow section and posture, structural-pulls, and collaborative-rights guides loaded. | CC is doer; CDC independently reviews in the default workflow. No independent verdict is authored here. |
| Rust API guidance | `rust-guidelines/guides/02-api-design.md` API-37 lines 1154–1176; `11-anti-patterns.md` AP-33 lines 1036–1062. | API-37 SHOULD motivates distinct save/portal types; AP-33 AVOID cautions against collapsing parse/missing errors into valid zero values. Neither proves a game format. |
| Core address source | `crates/nms-core/src/address.rs` lines 1–220 (constants, GalacticAddress declaration/constructors/accessors, signal-booster parse/format), 278–550 (display/parse/PortalAddress conversions), and 878–954 (discovery-shaped unit assumptions). | Complete for prompt's named API boundaries. Other arithmetic/routing methods are not a full workspace semantic review. |
| Save model and conversion source | `crates/nms-save/src/model.rs` lines 80–165, 167–239, 279–322, and related tests 344–410; `crates/nms-save/src/convert.rs` lines 1–109 and related tests 111–209 and 211–317. | Expanded, packed, discovery, base, current/previous address definitions and conversions loaded. Source-test owner values were not copied into outputs. |
| Extract / snapshot / query source | `crates/nms-graph/src/extract.rs` all 288 lines including tests; `crates/nms-watch/src/snapshot.rs` all 149 lines including tests; `crates/nms-query/src/find.rs` all 289 lines including tests. | Complete required functions and related in-file fixtures/tests read. |
| CLI and matching interface conversion callers | `crates/nms-cli/src/convert.rs` parse path lines 1–85, related tests 250–305 and `roundtrip_from_actual_save_address` lines 426–446. `crates/nms-copilot/src/dispatch.rs` `dispatch_convert`/`parse_glyphs` lines 454–500. `crates/nms-copilot/src/mcp/tools.rs` full `handle_convert` lines 646–695. `crates/nms-graph/src/import.rs` `import_csv` lines 92–160. | All direct production callers returned by the exact searches below are represented in `caller-map.csv`; public input behavior is recorded by interface. |
| Pinned game schema/reference code | Local `workbench/MBINCompiler` at `04a76be59439911fd9db1ea588e03ac604112e71`: full `GcUniverseAddressData.cs`, `GcGalacticAddressData.cs`, and `GcPersistentBase.cs`. `workbench/libNom.io` at `652aabb0147888776a088909cdb13bccbd3d9588`: `libNOM.test/Container.cs` lines 192–240. Searched both clones for packed-address decoders and relevant schema symbols. | Schema confirms expanded player fields and a `ulong` persistent-base member; checked-in code/tests expose no decoder for discovery DD.UA or persistent-base packed values. No claim that a schema field declaration establishes bit layout. |
| Required synthetic fixture/control data | jq projection covered every DD.UA in both fixtures (2 records each), both expanded current/previous addresses, every base address (0 in each), owner-sanitization result, plus complete probe command/expected/observed fields from both JSON receipts. | Inputs are synthetic and sanitized; both contexts' observed addresses use RealityIndex 0. Fixtures do not supply a base witness or nonzero galaxy witness. |
| Conditional private snapshots | No private decoded snapshot was supplied or present in the repository/workbench file search. No private save path was opened. | Population denominator, bit-width histogram, invalid/absent counts, and paired base/location witnesses are unavailable. Leave base layout and nonzero-galaxy interpretation unresolved; do not infer missing counts as zero. |

Several batched terminal outputs were truncated by the tool. The required bounded sections above were then re-read in smaller calls; no truncated excerpt is being treated as complete. The broader repository search included fixtures, in-file tests, and generated mapping data; those were separated from production operations in `caller-map.csv` and in the counts below.

## Source-cited contract readback (before dependent investigation)

1. Core `GalacticAddress` is explicitly a 48-bit portal-frame value with `P-SSS-YY-ZZZ-XXX` layout, and galaxy identity is a separate `reality_index`; `from_packed` masks to 48 bits (`crates/nms-core/src/address.rs:34-76,106-117`). `PortalAddress` stores 12 glyph nibbles and converts directly to/from that layout (`address.rs:352-451,491-544`). Therefore keep this existing API contract stable while investigating other encodings.
2. Save parsing reuses `PackedGalacticAddress(u64)` for discovery `DD.UA` and base `GalacticAddress`, but expanded player `UniverseAddress` has its own object fields and reality index (`crates/nms-save/src/model.rs:80-165,197-239,279-322`). Discovery and base conversions currently pass through low-48 decoding with galaxy 0; expanded current/previous player values use the stored reality index (`crates/nms-save/src/convert.rs:35-108`). That is source evidence of a shared lossy boundary, not evidence that both packed fields share a game encoding.
3. The assignment preserves the synthetic Chili witness and portal-shaped control as separate observations. The former's expected portal is independently supplied by the recorded game visit; the latter is only a negative control for fixture shape. No unsupported layout, width heuristic, nonzero-galaxy default, or base/discovery equivalence is authorized.
4. A later repair must use distinct save-domain decoding types/functions if supported by independent evidence, preserve `nms-core` portal semantics, map all affected callers, and carry representation identity into graph, watch, query, and cache paths. Missing or invalid values must not become successful zero addresses without an explicit supported policy (API-37 SHOULD; AP-33 AVOID).

## Caller search and reconciliation

Searches were run on all Rust source under `crates/`, excluding external `tests/` directories for the production pass:

```text
rg -n '\bfrom_packed\s*\(' crates --glob '*.rs' --glob '!**/tests/**'
rg -n '\bto_galactic_address\s*\(' crates --glob '*.rs' --glob '!**/tests/**'
rg -n '\bPackedGalacticAddress\b' crates --glob '*.rs' --glob '!**/tests/**'
rg -n 'from_packed|to_galactic_address|PackedGalacticAddress|parse_galactic_address|PortalAddress::parse_mixed|portal_glyphs' crates --glob '*.rs' --glob '!**/tests/**'
```

Every actual production operation in the direct-call results is classified in `caller-map.csv`: 10 `from_packed` call expressions (2 CLI, 1 internal portal conversion, 2 discovery extraction, 1 community portal import, 1 save wrapper, 1 REPL, 2 MCP); six direct `to_galactic_address` calls (CLI, REPL, and four save conversions); plus expanded-object conversion definitions, wrapper/schema definitions, find formatting, watch snapshot propagation, and cache serialization. Method declarations, comments, in-file unit tests, and `tests/` fixtures are not counted as production callers. The second pattern search contains matches across nine source files; each returned file and matching route was inspected, and each production conversion path is either a row in `caller-map.csv` or explicitly a downstream identity/render/cache consumer.

Related assumptions found in source/tests include: `crates/nms-core/src/address.rs:878-954` models discovery values as portal-layout values; `crates/nms-save/src/model.rs:344-410` accepts zero, hex, and integer scalar values; `crates/nms-save/src/convert.rs:161-191` supplies a 56-bit base address but does not assert its decoded address; `crates/nms-cli/src/convert.rs:276-292,426-446` explicitly preserves truncation behavior; graph/query/cache/watch fixtures use portal-shaped 48-bit values. These test fixtures are not independent game-format evidence.

Downstream identity note: `SystemId::from_address` clears the planet nibble but uses only `packed()`; the `reality_index` is absent from the ID (`crates/nms-core/src/system.rs:7-21`). If corrected discovery decoding permits identical 48-bit addresses in different galaxies, `GalaxyModel.systems` keyed only by `SystemId` may collide. This is a source-derived risk to test during a later repair, not a collision reproduced by this slice.

## Commands, outcomes, denominators, and failures

| Command / query | Result |
|---|---|
| `git worktree list --porcelain` | Exit 0; confirmed the main and `.worktrees/planning` checkouts. |
| `git rev-parse HEAD && git status --short --branch`; same commands with `git -C .worktrees/planning` | Exit 0; main pinned/clean, planning at preparation commit/clean before CC artifacts. |
| `rg -n` over address symbols, save/model conversion symbols, conversion parsers, cache identity/version sites, and pinned local references (paths listed above) | Exit 0 for matches; outputs were inspected. Search over explicit nonzero `RealityIndex` fixtures returned no match (exit 1); this means no such workspace fixture witness was located, not that game values are absent. |
| `CARGO_NET_OFFLINE=true cargo build --workspace --all-features --locked` | Exit 0 in 0.65 s. Cargo reported a future-incompatibility warning for `proc-macro-error2 v2.0.1`; no hard build failure. Source checkout was pinned and clean at build. |
| `./target/debug/nms export --save .worktrees/planning/project01-mvp/artifacts/closeout-2026-10-03/synthetic-address-save.json --format json` | Exit 0. Exact stdout is `artifacts/address-export.stdout.json`; stderr is empty. Projected output: portal `2B0004B2446A`, Euclid, coordinates `(1130, 4, -1244)`, blank system/planet names. Expected portal: `212B04B2446A`. |
| `./target/debug/nms export --save .worktrees/planning/project01-mvp/artifacts/closeout-2026-10-03/synthetic-portal-control-save.json --format json` | Exit 0. Exact stdout is `artifacts/portal-control-export.stdout.json`; stderr is empty. Projected output: `212B04B2446A`, Euclid, same coordinates. This control does not support the substituted UA as a game format. |
| `jq -c '[.systems[]? | ...]'` against each CLI stdout | Failed twice because actual export JSON is a top-level array, not an object with `systems`. Corrected to `jq -c '[.[] | ...]'`, which exited 0 on both outputs. This was an inspection-query error, not a CLI failure; both raw outputs are retained. |
| `rg -n '\b[0-9]{15,}\b|765611[0-9]+' artifacts/` | Exit 1, no matches: no copied long numeric owner/account identifier was detected in CC-produced artifacts. |
| `git status --short --branch` on source after all probes | Exit 0; source remains clean. Planning worktree contains only this slice's new artifacts and later close documents/ledger evidence. |

Ancillary command failures during intake were recovered: the initial skill-path guess under `/Users/oubiwann/.codex/skills/knowledge/...` did not exist (exit 1); canonical installed paths under `/Users/oubiwann/.codex/skills/<skill>/...` were then used. An attempted `nl` with multiple input files was rejected by BSD `nl` (exit 1); each source was read separately. A non-existent `project-management/guides/04-project-closeout.md` path was also rejected; project close authority was read from the actual `project-management/guides/README.md`, `04-closing-slices.md`, and `05-closing-arcs.md` files. These failures did not block required reading.

No unit/integration test suite was rerun. The committed closeout baseline records the existing test/build/lint outcomes and known no-default-feature/strict-rustdoc failures. This slice adds no test or changes no source behavior.

## Probes and unsupported denominators

The two fixture projections cover four discovery UA entries (two per fixture), four expanded current/previous player objects, and zero persistent-base entries. Every observed fixture reality index is 0. Both fixtures are synthetic; no raw private snapshots were used. The private-snapshot population query denominator is unavailable because no snapshot was supplied: do not state a zero count for missing, invalid, or nonzero-galaxy values. No independent base-address/location witness was available. These limits keep DD.UA's generalized bit layout, the base layout, and packed-field galaxy sourcing unresolved.

## Produced-artifact inventory

- `artifacts/caller-map.csv` — reconciled production conversion callers and downstream consumers.
- `artifacts/format-contract.md` — source-cited format observations, witness/control results, uncertainty, and bounded next repair recommendation.
- `artifacts/address-export.stdout.json`, `artifacts/address-export.stderr.log` — fresh public-path discovery fixture replay.
- `artifacts/portal-control-export.stdout.json`, `artifacts/portal-control-export.stderr.log` — fresh portal-shaped control replay.
- `artifacts/execution.md` — this intake, readback, query coverage, command outcomes, limits, and artifact inventory.
- `closing-report.md` — CC's per-row proposed disposition and bubble-up.

No owner/account identifiers or private snapshots were copied. No source, dependency, config, test fixture, or game-save file changed. CDC verification is not produced or implied by this record.

## Iteration 01 — complete address-input caller evidence (2026-10-03)

**Assignment identity:** Project01 / arc06 / slice01, `arc06-fit-and-finish/slice01-address-encoding-evidence/cc-prompt-iteration01.md`; predecessor `cc-prompt.md`; initial packet `8a5352bc5a9da3b15a51835879b9742c7d4f96f9`. Returned to CDC as proposed-done. This section supplements the initial attempt above; it does not erase its incomplete caller/read-coverage claim or its recorded query failures.

### Current fences and required-input coverage

- Source: `/Users/oubiwann/lab/oxur/nms-copilot`, branch `main`, `a800950ef3558fcf463517696c3c140061e7353d`, clean at intake and after the build/probes.
- Planning: `.worktrees/planning`, branch `planning`, `8f56eba1ebf0c340f98e162d57af2081909c8c6e`, clean before these edits. This is the active slice correction; the initial CC packet at `8a5352b` and CDC review at `8f56eba` are retained.
- Read completely for this assignment: iteration01 prompt; slice plan; all eight ledger rows; CDC verification including R01 and all row dispositions; CDC JSON receipts including all prior source-hit lists and five probe records; original CC prompt §§ “Binding boundaries and applied guidance” and “Validation and return”; arc plan §§ “Workflow” and “Current corrective preparation”; prior `caller-map.csv`, `format-contract.md`, and `execution.md`; initial closing report as committed at `8a5352b`.
- Read the complete required source spans: `nms-cli/src/convert.rs:6-168`; `nms-copilot/src/dispatch.rs:454-535`; `nms-core/src/address.rs:34-209,278-328,352-544`; and `nms-save/src/model.rs:80-165`. Also read complete reached `resolve_galaxy`, display/formatting, `From`/`FromStr` conversion bodies, and relevant input call bodies in those spans. Bounded output was re-read in smaller calls where terminal output was truncated; no truncated excerpt is treated as complete.
- Read Rust API-37 (SHOULD) in `rust-guidelines/guides/02-api-design.md:1154-1181`, AP-33 (AVOID) in `rust-guidelines/guides/11-anti-patterns.md:1036-1062`, and `work-verification/guides/05-independent-verification.md` § “Evidence During Correction Review”. API-37 keeps distinct input/encoding roles visible; AP-33 keeps parse/range failure distinct from successful zero; correction review permits retained evidence only when source, criterion, inputs, method, and applicability remain unchanged.

### Source-cited contract readback before evidence edits

The CLI `run` requires `--ssi` and calls `parse_voxel`; that complete parser reads X/Z as `i16`, Y as `i8`, rejects X/Z outside `-2048..=2047`, rejects SSI above `0xFFE`, and then calls `GalacticAddress::new` (`crates/nms-cli/src/convert.rs:6-31,79-128`). The REPL `dispatch_convert` also requires `--ssi` and calls its complete `parse_voxel`, which reads X/Z as `i16`, Y as `i8`, then calls the same constructor without those CLI X/Z/SSI checks (`crates/nms-copilot/src/dispatch.rs:454-533`). The differing parser behavior is an observed fact, not policy approval or an instruction to normalize the paths.

`GalacticAddress::new` masks X/Z and SSI to 12 bits, Y to 8 bits, and planet to 4 bits; `from_packed` masks to 48 bits (`crates/nms-core/src/address.rs:34-117`). Core `GalacticAddress` still defines `P-SSS-YY-ZZZ-XXX` with separate `reality_index` (`address.rs:34-76`); `PortalAddress` and `From`/`FromStr` implementations convert only that portal-shaped core representation (`address.rs:278-328,352-544`). Allowed evidence writes are the existing caller map, format contract, execution record, closing report, and supported ledger notes only. No production behavior, save-packed DD.UA decoding, base layout, invalid-input policy, or galaxy sourcing decision is authorized; generalized packed DD.UA/base/galaxy semantics remain unresolved and block dependent repair.

### Expanded source search and reconciliation

Search population: all Rust files under `crates/`, with external `**/tests/**` directories excluded. The exact case-insensitive-free expressions queried were:

```text
rg -n 'GalacticAddress::new[[:space:]]*\(' crates --glob '*.rs' --glob '!**/tests/**'
rg -n 'parse_voxel[[:space:]]*\(' crates --glob '*.rs' --glob '!**/tests/**'
rg -n 'from_signal_booster[[:space:]]*\(' crates --glob '*.rs' --glob '!**/tests/**'
rg -n 'from_portal_string[[:space:]]*\(' crates --glob '*.rs' --glob '!**/tests/**'
rg -n 'from_galactic_address[[:space:]]*\(' crates --glob '*.rs' --glob '!**/tests/**'
rg -n 'to_portal_address[[:space:]]*\(' crates --glob '*.rs' --glob '!**/tests/**'
rg -n 'parse_galactic_address[[:space:]]*\(' crates --glob '*.rs' --glob '!**/tests/**'
rg -n 'impl FromStr for GalacticAddress|impl FromStr for PortalAddress|impl From<u64> for GalacticAddress|impl From<PortalAddress> for GalacticAddress|impl From<GalacticAddress> for PortalAddress|impl From<GalacticAddress> for u64' crates --glob '*.rs' --glob '!**/tests/**'
rg -n 'GalacticAddress::from\(|PortalAddress::from\(' crates --glob '*.rs' --glob '!**/tests/**'
rg -n '(GalacticAddress|PortalAddress).*parse|parse.*(GalacticAddress|PortalAddress)|\.parse::<(GalacticAddress|PortalAddress)>' crates --glob '*.rs' --glob '!**/tests/**'
```

The raw matched-line populations, which include in-file test-module calls and definitions and therefore are not production-call totals, were: `GalacticAddress::new` 115; `parse_voxel` 14; `from_signal_booster` 8; `from_portal_string` 1; `from_galactic_address` 1; `to_portal_address` 5; `parse_galactic_address` 7; each of the two `FromStr` impl patterns 1; each of the four `From` impl patterns 1; combined `GalacticAddress::from`/`PortalAddress::from` usages 8; typed parse search 17. The prior CDC-reconciled search remains 10 production `from_packed` expressions (plus its definition) and six direct production `to_galactic_address` expressions (plus two definitions), as itemized in `artifacts/cdc-review-receipts.json`.

Manual source-boundary classification found three production `GalacticAddress::new` call expressions: the CLI voxel parser, REPL voxel parser, and expanded save-object conversion (`crates/nms-save/src/model.rs:110-120`). `parse_voxel` has two production call expressions and two definitions, both now mapped; its remaining matches are in-file tests. `parse_galactic_address` has one production call and a definition in CLI; its other match is a test. `from_signal_booster` production routes are CLI, REPL, and MCP, plus the two core definitions/internal conversion route; its remaining hits are in-file tests. `from_portal_string` and `from_galactic_address` are definition-only in the production source. `to_portal_address` production uses are CLI/REPL formatters plus the core method/`From` conversion; test hits remain separate. Both `FromStr` implementations and the four address-related `From` implementations are fully read; the only production typed parse reached by the string helper is `PortalAddress` inside `GalacticAddress::from_portal_string`. Of eight explicit `GalacticAddress::from`/`PortalAddress::from` matches, the production conversions are within core conversion methods; the remaining occurrences are in-file tests. No typed production parse or external `From` call site for save-packed `GalacticAddress` was found. Methods, definitions, in-file test calls, and downstream formatting are classified separately in the updated CSV rather than counted as independent decoding operations.

Every matched production conversion/input function in these families was read completely before classification. The two new voxel rows record declared primitive widths, validation behavior, constructor narrowing, separate galaxy and planet inputs, and the absence of a live REPL receipt. The rest of the map retains the ten packed and six direct wrapper routes plus portal, expanded save, MCP, import, and downstream format/cache consumers. No external test directory, fixture, or generated data was used to inflate production caller counts.

### Repeated public CLI probes

Rebuilt first from the pinned source with `CARGO_NET_OFFLINE=true cargo build --workspace --all-features --locked`: exit 0 in 0.98 s. Cargo emitted the existing future-incompatibility warning for `proc-macro-error2 v2.0.1`; no tests were run. Then ran the three required commands against `target/debug/nms` from the source root:

| Command | Exit | Actual observed result |
|---|---:|---|
| `target/debug/nms convert --voxel 1130,4,-1244 --ssi 299 --planet 2` | 0 | Output table reports hex/galactic address `212B04B2446A`, coordinates `(1130, 4, -1244)`, SSI 299, planet 2, Euclid (0). |
| `target/debug/nms convert --voxel 3000,0,0 --ssi 0 --planet 2` | 1 | `Error: Voxel X out of range: 3000 (must be -2048..2047)` on stderr; stdout empty. |
| `target/debug/nms convert --voxel 0,0,0 --ssi 4095 --planet 2` | 1 | `Error: System index out of range: 4095 (must be 0..4094)` on stderr; stdout empty. |

The first result confirms the expected exact portal output. The latter two confirm the CLI X and SSI guards. Complete REPL source inspection establishes that corresponding `3000` and `4095` values fit its `i16`/`u16` parses and flow to the masking constructor, but no live-terminal REPL test or result is claimed. These observations do not decide whether the two public interfaces should converge.

### Row dispositions and retained evidence applicability

- **AE-01 open:** completed prompt/source/query coverage and the pre-edit source-cited readback are now recorded above; CDC must review the corrected intake evidence.
- **AE-02 open:** the caller map now includes both voxel input paths and their reached constructor behavior; the expanded raw hit populations and production/test/definition distinctions are recorded above for CDC reconciliation.
- **AE-03 retained done — reproduced:** CDC `cdc-verification.md` identifies its checked source fence, claim, method, and results. This iteration changes no format witnesses, fixtures, reference code, or unresolved family conclusions; its accepted criterion remains limited to applicable pinned witnesses or explicitly unresolved status. No changed input or source affects that evidence.
- **AE-04 retained done — reproduced:** CDC’s exact discovery/control CLI outputs and input fixture digests are in `cdc-verification.md` and `cdc-review-receipts.json`. This iteration changes no export code or fixture and does not replace those claims; the separate voxel CLI probes are recorded fresh above. The exact failure/control evidence remains applicable.
- **AE-05 open:** the corrected contract now records CLI/REPL primitive widths, explicit checks, constructor masks, zero, signed coordinates, planet narrowing, galaxy separation, and malformed/overflow outcomes. Policy remains unresolved.
- **AE-06 retained done — reproduced:** CDC names its inspected fixture/test and graph/watch/query/cache source population in `cdc-verification.md`. This iteration changes none of those inputs, implementations, or criteria and introduces no downstream production dependency; the independently reproduced impact observations remain applicable.
- **AE-07 open:** the recommendation now explicitly holds any CLI/REPL validation harmonization for owner review and keeps save, base, galaxy, zero/sentinel, `--ga`, query identity, and cache gates unresolved. CDC review is still required against the complete caller map.
- **AE-08 retained done — reproduced:** CDC’s source/commit fence, exact initial CC scope, and identifier scan result are recorded in `cdc-verification.md`. This iteration changes only allowlisted planning evidence and makes no source, fixture, dependency, or private-data change; current scope and sanitation are rechecked below. The earlier accepted check is retained for its recorded scope; the current five-file scope is also checked before commit.

Retention is not relabelled as a fresh CDC run. Each retained row's source fence, inputs, criterion, method, and result are explicitly identified in the CDC record; this iteration affects only caller/input evidence for the four open rows. If CDC finds a changed dependency or an undermined retained method, that row must be reopened and freshly checked.

### Current inventory and checks

The only edited paths for this iteration are the five prompt-allowlisted existing paths: `artifacts/caller-map.csv`, `artifacts/format-contract.md`, `artifacts/execution.md`, `closing-report.md`, and `ledger.md`. No new probe files or source/test/config/fixture/game data were created. The prior supporting artifact inventory remains unchanged. A live identifier scan and final diff/path checks are recorded after the edits below; source tests, no-default-feature build, strict documentation checks, and REPL execution were not rerun by this evidence-only correction. Their existing recorded failures/limits remain unresolved.
