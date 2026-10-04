# Project01 closeout preparation

Date: 2026-10-03. Status: **prepared; project remains open**.

The user enabled Expedited Mode and requested preparation for closing the MVP without skipping steps. Source baseline: `a800950ef3558fcf463517696c3c140061e7353d`. Project02's design was committed first as `143a796296bc299fc7a8965c85caec77606bc36c` on planning. This packet changes planning/evidence only.

## Preserved scope and acceptance population

The [original project plan](project-plan.md), [summary](README.md), and numbered milestone documents remain the scope inputs. The six existing arcs contain **43 numbered documents**. The summary contains **61 milestone rows**. Four initial research documents and four later UI documents are tracked separately, giving **69 recovered leaf-level review rows**, plus six arc-composition rows. These are an initial reconstruction, not a claim that every detailed requirement has already been extracted. Review must add omitted distinct requirements before closing their owner.

The [historical document inventory](artifacts/closeout-2026-10-03/historical-document-inventory.json) captures original files before this recovery addendum; hashes identify that earlier planning state. The [milestone inventory](artifacts/closeout-2026-10-03/milestone-inventory.csv) retains exact summary descriptions/deliverables and source lines. Numbering drift must be resolved by comparing document meaning, not matching number strings.

No leaf/arc/project closing reports or independent acceptance records existed at intake. New recovery plans and ledgers describe fresh acceptance work. They do not reconstruct imaginary past reviews or convert old source commits into closed slices. A legacy leaf specification needs an attributable complete review and disposition; any new remediation uses canonical open/close records.

## Baseline findings

See the [baseline report](artifacts/closeout-2026-10-03/baseline-report.md) for commands, exact denominators, and limits. All-feature build/lint and **807 tests** pass. Whole-workspace measured line coverage is **11,548/14,228 = 81.16%**, below the recorded 95% target. Builds without default features and strict documentation checks fail. A real CLI export reproduces the discovery-address and name-loss defects; a second export comparison reproduces wrong system associations when filtering by biome.

The existing CI run passed default-feature tests on Linux, Windows, and macOS at the exact source revision. That evidence is retained with its original date and narrower commands. It does not establish no-default-feature builds, coverage, transport handshakes, live watch composition, or current registry installation.

## Ordered route to closure

1. **Establish address contracts.** Execute the prepared evidence slice below. Distinguish save-packed, portal, expanded, and signal-booster forms; map every decoding caller and related fixture. Its output must resolve or explicitly fence uncertainties before an address implementation assignment is issued.
2. **Repair existing MVP correctness.** Prepare separate bounded corrective slices for address decoding/planet glyphs, planet-to-system query associations, lost names, and supported feature combinations. Inspect impact through conversion, graph, watch, cache, CLI, REPL, and MCP. Invalidate derived caches if the corrected representation requires it. Keep first-hand failing evidence; add independent exact-value regressions.
3. **Reconcile all original review units.** For each arc, read every original document completely, compare source-as-delivered with scope-as-specified, expand the ledger for distinct omissions, and attach focused evidence. Carry proposed deferrals with authority, reason, owner, and re-entry condition. Community import persistence was explicitly deferred historically; import source tagging was promised and must be assessed separately.
4. **Complete documentation and validation work.** Repair documentation errors and missing CONTRIBUTING/license/package material where required; define a faithful workspace coverage command and enforce the original target through meaningful tests. Keep module-level gaps and any justified exclusions visible. Test supported feature combinations and declared MSRV, not just the installed newer compiler.
5. **Demonstrate composition.** Exercise parser → model → query → each interface with independent locations; route constraints; cache miss/hit/stale/corrupt behavior; watcher change and partial-write handling; REPL and MCP observing the same update; actual stdio and HTTP initialize/tool calls. Include interactive map/setup/history behavior and clean terminal restoration. Unit dispatch tests and help output cannot stand in for protocol or interactive demonstrations.
6. **Verify distribution separately.** Inspect package contents and verify packaged builds in dependency order. Establish historical 0.2.0 publication/install evidence or explicitly distinguish it from a new release candidate. Do not publish, push a release tag, or mutate registry state as a side effect of validation. Retain blocked registry attempts; network failures are not proof of absent packages.
7. **Close leaves, then arcs, then project.** Fresh CC reports per-row dispositions; the assigned reviewer reproduces affected evidence and checks source diffs. Each arc closing report walks every review unit, proves composition, and bubbles up findings. Project closing report walks all project rows, reconciles all findings, and receives independent composition review plus Operator acceptance. Create verdict records when the review actually occurs, not in advance.

## Active assignment

```text
arc06-fit-and-finish/slice03-decoded-location-witnesses/cc-prompt-iteration02.md
```

Current follow-up: Slice02 is [independently accepted](arc06-fit-and-finish/slice02-packed-format-authority/cdc-verification.md), with its encrypted sample limitations preserved. Slice03 iteration01 has been independently reviewed: six rows accepted, DW-07 open under [R04](arc06-fit-and-finish/slice03-decoded-location-witnesses/cdc-verification.md). Iteration02 is a narrative-only correction, prepared for fresh CC and not yet executed. Its [open set](arc06-fit-and-finish/slice03-decoded-location-witnesses/slice-plan.md) examines one successfully decoded known local save and classifies location relationships before a distinct production repair is designed. The original slice01 assignment and iteration are preserved; its eight-row caller investigation is now [independently accepted](arc06-fit-and-finish/slice01-address-encoding-evidence/cdc-verification.md). Neither that acceptance nor the next research prompt selects a generalized decoder or compatibility policy.

## Gates and authority

Expedited Mode changes cadence and commits, not scope, evidence, or independent review. This context's new measurements are **attested baseline evidence**. It cannot independently accept its own assembled closeout packet. The [project ledger](ledger.md) and all arc ledgers remain open. No production work or release is represented as complete.

Open acceptance issues are not automatically transferred into project02. The user-facing save/account redesign belongs there by the accepted design placement, while defects in already promised project01 behavior retain project01 ownership until repaired or an explicit scope amendment is accepted. Structural re-scope and coverage-target changes require the Operator's decision after a concrete evidence packet; do not infer them from passing tests or a later project's existence.
