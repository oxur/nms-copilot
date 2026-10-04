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
