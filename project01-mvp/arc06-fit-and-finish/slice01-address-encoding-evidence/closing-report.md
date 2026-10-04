# CC close report — address encoding evidence

**Assignment:** `arc06-fit-and-finish/slice01-address-encoding-evidence/cc-prompt.md` (initial assignment; preserved unchanged).

**Source state:** `/Users/oubiwann/lab/oxur/nms-copilot`, `main` at `a800950ef3558fcf463517696c3c140061e7353d`, clean; no source files changed.

**Planning state:** `.worktrees/planning`, `planning` at `3fcf3ceb476473738499dd1052395819d75056d` before CC outputs; exact close-packet commit: pending until this report is committed.

**Workflow / evidence:** Two-Contributor; CC findings are `attested` and proposed-done. CDC independent review and final row statuses remain pending. This report does not close the parent arc or project.

## AE-01 — required intake and source-cited readback

**CC disposition: proposed-done.** `artifacts/execution.md` records both checkout states, loaded sections and source query coverage, conditional snapshot availability, and the pre-investigation readback citing the core 48-bit contract separately from save wrapper behavior. The bounded Rust API-37/AP-33 guidance and close/ledger procedures are also recorded. A required base/nonzero-galaxy private witness was unavailable; the affected interpretations remain explicitly unresolved, as the prompt directs. Evidence strength: attested.

## AE-02 — production caller reconciliation

**CC disposition: proposed-done.** `artifacts/caller-map.csv` records all production packed/core conversion routes plus downstream extraction, snapshot, query, import, interface, and cache consumers. `artifacts/execution.md` retains the search expressions and reconciles direct `from_packed` and `to_galactic_address` calls, method definitions, in-file tests, and fixture-only matches. Notable paths include `nms-save` discovery/base/player conversions, both `nms-graph` extraction passes, CLI and REPL parsers, MCP conversion, community portal import, `execute_find`, snapshots, and rkyv cache data. Evidence strength: attested.

## AE-03 — field-family witnesses or unresolved findings

**CC disposition: proposed-done.** `artifacts/format-contract.md` keeps core portal, signal-booster, expanded player, discovery DD.UA, persistent-base, and interface input families separate. Pinned MBIN schema supports expanded player field structure and base scalar width only; no packed DD.UA/base decoder was found in the pinned local reference clones. The Chili discovery witness establishes a reproduced mismatch for that value only. General DD.UA bit layout, base encoding, and packed-field galaxy sourcing remain unresolved pending an independent decoder or sanitized private witnesses. Evidence strength: attested.

## AE-04 — CLI failure and portal-shaped control

**CC disposition: proposed-done.** After `CARGO_NET_OFFLINE=true cargo build --workspace --all-features --locked` succeeded at the pinned clean source revision, the CLI export command was run for both committed synthetic fixtures. Discovery DD.UA `0x212B0004B2446A` exited 0 but emitted `2B0004B2446A` where the independent game-visit oracle expects `212B04B2446A`. The control with portal-shaped DD.UA `0x212B04B2446A` exited 0 and emitted the expected glyph string. Exact outputs and empty stderr files are in `artifacts/`. The control is not treated as an accepted save format. Evidence strength: attested.

## AE-05 — galaxy, planet, SSI, signed coordinates, zero, and invalid-input assumptions

**CC disposition: proposed-done.** The contract and execution record enumerate currently observed behavior and unresolved interpretation for galaxy source, planet/SSI/X/Y/Z, zero, signed integer values, oversized inputs, and missing fields. Expanded current/previous player objects carry a separate `RealityIndex`; discovery and base conversions currently force zero. `PackedGalacticAddress` accepts zero and signed JSON integers by casting `i64` to `u64`; core and CLI raw conversion narrow to 48 bits. Missing packed fields error, while omitted expanded fields default to zero. No nonzero-galaxy fixture witness or private snapshot population was available; no zero/sentinel, negative-value, or truncation policy is recommended. Evidence strength: attested.

## AE-06 — misleading fixtures, tests, and downstream impacts

**CC disposition: proposed-done.** The contract identifies portal-shaped 48-bit tests/fixtures, the CLI test that explicitly accepts 56-bit truncation, conversion tests that do not assert decoded base addresses, and relevant graph/watch/query/cache paths. It also records two downstream identity concerns for the next assignment: `execute_find` formats the system address while returning a separate planet whose index may differ; and `SystemId` clears the planet bits but omits `reality_index`, so a future multi-galaxy decoder needs collision tests. Those are impact findings, not repairs made here. Evidence strength: attested.

## AE-07 — bounded next repair recommendation

**CC disposition: proposed-done.** `artifacts/format-contract.md` gives the concrete source symbols and propagation path, exact Chili oracle/control, test/validation impact, cache invalidation site, and owner decisions. It recommends keeping the core portal API stable, adding a distinct save-domain decoder only after the packed format is independently supported, and separating base interpretation until a base witness exists. A discovery decoder, base decoder, public `--ga` compatibility, invalid-input semantics, galaxy identity, query glyph planet selection, and cache version/invalidation require design resolution before implementation. Evidence strength: attested.

## AE-08 — sanitation, source boundary, and accounting

**CC disposition: proposed-done.** Source status is clean at the pinned revision after build and CLI replay. Planning outputs are confined to this slice's `artifacts/`, this report, and supported ledger evidence. A scan of CC artifacts found no 15+-digit numeric account/owner values; fixtures used in replay are synthetic and their projection reports owner identifiers sanitized. No private snapshots or raw saves were copied. The execution record lists the intentional query failure/correction and ancillary intake-path errors. Evidence strength: attested.

## Supporting artifact inventory

Produced durable supporting files under the default slice `artifacts/` home:

- `artifacts/execution.md`
- `artifacts/caller-map.csv`
- `artifacts/format-contract.md`
- `artifacts/address-export.stdout.json`
- `artifacts/address-export.stderr.log`
- `artifacts/portal-control-export.stdout.json`
- `artifacts/portal-control-export.stderr.log`

First-class slice documents remain at the slice root. No production files, test fixtures, dependencies, configs, or game files were edited. No `cdc-verification.md` is authored or represented as complete; CDC has not reviewed or reproduced these claims.

## Bubble-up to arc06

1. **Assigned capability:** The slice delivered a source-cited current contract for core portal/expanded/interface routes, a complete direct-caller map, and a reproduced DD.UA failure/control. The packed DD.UA and base layouts remain unresolved, so this result does not authorize or fully specify a repair. That matches the slice's explicit unresolved-evidence allowance while preserving the arc's dependent implementation gate.
2. **What the arc plan did not anticipate:** The shared `PackedGalacticAddress` wrapper hides at least two separate save field families, but neither pinned local reference clone supplies a packed decoder. A base-specific independent witness is absent; discovery's one Chili witness cannot establish base equivalence or generalized galaxy bits. Public raw-address meanings differ across CLI, REPL, and MCP. Graph identity and query rendering also need affected regression coverage. CDC should decide whether to revise the arc breakdown into separately evidenced discovery/base work and address the multi-galaxy identity dependency. Under Two-Contributor authority, any arc-plan amendment needs the recorded CDC/Operator decision; CC has not edited `arc-plan.md`. Do not plan the next slice against an unamended or unresolved contract.
3. **Silent-drop diff:** The assignment required intake/readback, all production callers, all field families or explicit unresolved rows, fresh CLI failure/control, assumption enumeration, fixture/test/cache/query impacts, a bounded repair recommendation, sanitation/source-state scan, and row-by-row closure. Each is addressed above and in the listed artifacts. Deliberately not done: source or fixture repair, private-snapshot population query, nonzero-galaxy/base witness, full workspace semantic review, CDC verification, arc close, and project close. These are not presented as delivered. The evidence gaps stay open for CDC/Operator routing and later bounded work.

## Commit record

Expedited Mode commit is restricted to the following exact planning paths:

```text
project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence/artifacts/execution.md
project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence/artifacts/caller-map.csv
project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence/artifacts/format-contract.md
project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence/artifacts/address-export.stdout.json
project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence/artifacts/address-export.stderr.log
project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence/artifacts/portal-control-export.stdout.json
project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence/artifacts/portal-control-export.stderr.log
project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence/closing-report.md
project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence/ledger.md
```

Required commit trailers, once each:

```text
Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>
```
