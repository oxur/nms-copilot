# CC assignment — establish the MVP address format contract

This is the initial evidence-only assignment for `project01-mvp/arc06-fit-and-finish/slice01-address-encoding-evidence`. The user authorized closeout preparation and Expedited Mode. It is prepared for a fresh CC, with independent CDC review afterward. Do not repair production code in this assignment.

Source checkout: `/Users/oubiwann/lab/oxur/nms-copilot`, branch main, expected HEAD `a800950ef3558fcf463517696c3c140061e7353d`. Planning checkout: its `.worktrees/planning`, branch planning. Recheck both states. If source drift affects this investigation, report the delta before using old receipts as current evidence. Do not alter unrelated work.

## Required reading and preflight

Paths below are relative to their named source/planning roots. Required text must be loaded completely within the stated bounds; recover truncated output. Record scope/state, actual loaded ranges and data-query coverage in `artifacts/execution.md`, then write a brief source-cited contract readback before dependent work. An unavailable required input blocks its dependent step; this is not a new permission question once intake is coherent.

| Order | Path/state | Scope | Purpose |
|---|---|---|---|
| 1 | This prompt, sibling slice-plan.md and ledger.md; committed preparation packet | Required-full | Active evidence-only contract and all eight acceptance rows |
| 2 | project01-mvp/project-plan.md §§11–12; closure-preparation.md §§Baseline findings, Ordered route to closure, Gates and authority | Required-section | Current closeout authority, sequencing and limits |
| 3 | project01-mvp/artifacts/closeout-2026-10-03/baseline-report.md §§Behavior probes, Other source-confirmed or unresolved findings | Required-section | Existing failure/control and unresolved base semantics |
| 4 | Source crates/nms-core/src/address.rs: encoding constants and GalacticAddress declaration/constructors/accessors; complete parse/format functions reached by caller search | Required-section | Intentional 48-bit portal representation and expanded/signal conversions |
| 5 | Source crates/nms-save/src/model.rs: GalacticAddressObject, PackedGalacticAddress including Deserialize and conversion implementations, RawDiscoveryRecord/DiscoveryData/PersistentPlayerBase address-bearing definitions; crates/nms-save/src/convert.rs all non-test conversion implementations and related tests | Required-section | Distinct source fields sharing a misleading wrapper and zero-galaxy conversions |
| 6 | Source crates/nms-graph/src/extract.rs complete extract_systems and tests; nms-watch/src/snapshot.rs complete SaveSnapshot::from_save; nms-query/src/find.rs complete execute_find | Required-section | Actual consumers and query glyph generation, not just integer math |
| 7 | Source crates/nms-cli/src/convert.rs complete parse_galactic_address and roundtrip_from_actual_save_address; complete matched MCP/REPL conversion callers from workspace query | Required-section | Public input compatibility and existing truncation expectation |
| 8 | /Users/oubiwann/.codex/skills/rust-guidelines/guides/02-api-design.md API-37; guides/11-anti-patterns.md AP-33, with necessary definitions | Required-section | Distinguish encoding roles and preserve unresolved/error evidence |
| 9 | Baseline synthetic-address-save.json and synthetic-portal-control-save.json, address-probe.json and portal-control-probe.json | Required-data: all DD.UA, expanded UniverseAddress, owner identity sanitation, exact command/expected/observed fields | Whole supplied fixture/control projection; replay without silently correcting inputs |
| 10 | Available pinned game-schema/reference implementations | Conditional: load full decoding functions and necessary constants for each claimed field family | Support semantics with actual reference code. Local clones: workbench/MBINCompiler at 04a76be59439911fd9db1ea588e03ac604112e71; workbench/libNom.io at 652aabb0147888776a088909cdb13bccbd3d9588. Record missing clones/source and use primary upstream sources if needed |
| 11 | Private decoded snapshots if base/galaxy semantics remain unresolved | Conditional required-data: histogram types/widths for every DD.UA and both contexts' base addresses; select explicit location witnesses paired with expanded/teleport coordinates | Inspect locally only. Query denominator and exclusions must be retained; no raw snapshots, owner IDs or account paths committed. If unavailable, leave that interpretation unresolved |

The source sections are pinned to the stated main revision; planning inputs are the committed closeout-preparation packet. Existing repository instructions and workflow/verification requirements remain binding. The prompt does not authorize full-workspace semantic review or silently narrow universally required skill inputs.

## Binding boundaries and applied guidance

- Inspect and report. Allowed writes: this slice's `artifacts/` and its eventual `closing-report.md`; row evidence/status updates in this slice ledger when supported. No source/dependency/configuration/game changes, and no independently signed verification record authored by CC.
- Rust API-37 (SHOULD) motivates distinct names for packed-save versus portal values in the recommended future API. It does not settle an uncertain source encoding. AP-33 (AVOID) requires unsupported/malformed evidence to stay visible rather than become a successful default galaxy/coordinate.
- Preserve the current core 48-bit contract as an observed intentional API. Determine whether save wrappers need a separate decoding boundary; do not globally redefine `from_packed` or infer frame from numeric length.
- Zero, negative integer values, out-of-width inputs and missing fields need explicit observation/interpretation. Do not choose silent truncation, rejection, or fallback semantics for the later implementation without evidence and owner review.

## Investigation recipe

1. Search all production and test uses of `from_packed`, `to_galactic_address`, `PackedGalacticAddress`, parser address inputs, and glyph/result construction. Read every production matched caller completely. Write `artifacts/caller-map.csv` with path/symbol, field/input family, producer, consumer, current operation, galaxy source, and contract evidence. Separate tests and fixtures from production callers. Reconcile search counts; no unaccounted hit.
2. For each family—discovery DD.UA, persistent-base address, expanded player coordinates, portal glyphs, signal booster, CLI/MCP raw packed input—state documented layout, observed representation, conversion, supporting witness, and confidence/unresolved questions in `artifacts/format-contract.md`. Source body comments are not independent game-format authority. Inspect reference decoder constants and applicable schema, retaining commit/URL/path/range.
3. Replay the supplied two synthetic fixtures through `target/debug/nms export --save <fixture> --format json`. If the binary is absent, use `CARGO_NET_OFFLINE=true cargo build --workspace --all-features --locked` from source; record the gate and exact binary revision. The discovery input is `0x212B0004B2446A`, independently expected portal `212B04B2446A`, galaxy 0, planet 2, SSI 299, X 1130/Y 4/Z -1244. Baseline output is `2B0004B2446A`. The portal-shaped control substitutes `0x212B04B2446A` into UA and currently prints the expected hex. Retain both outcomes; neither fixture alone proves all game encodings.
4. Seek a nonzero-galaxy witness and independently matched base/expanded-coordinate witness. On private snapshots, report type/bit-width counts for the complete selected populations, then only the sanitized witnesses required to interpret the fields. Retain absent/invalid counts. If a source-packed value numerically fits in 48 bits, demonstrate why width-based detection is ambiguous. Derive expectations from reference/expanded evidence, not a round trip through the same candidate algorithm.
5. Inspect related tests and fixtures, including the CLI test expecting 56-bit truncation. Identify all assumptions a later repair must change, plus public API/CLI compatibility impacts and cache version/key/invalidation sites. Include the possibility that query output needs the result planet's index even when the system address has a different planet component. Do not implement any fix.
6. Recommend a bounded next repair: actual symbols/files, type boundary, propagation path, exact regression oracles, affected validation matrix, and unresolved owner decisions. Separate source-confirmed defects from hypotheses. Names/biome association/HTTP/coverage issues are bubble-up references, not repair scope.

## Validation and return

AE-02 requires a complete caller-search reconciliation, not a few examples. AE-03 requires one applicable witness per field-family interpretation or a clearly unresolved row. AE-04 requires the real CLI failure and control at the pinned source. AE-05 rejects a round-trip-only or hardcoded-galaxy argument. AE-06 covers misleading fixtures and downstream representation identity. AE-08 requires clean source status and a scan for copied numeric account/owner identifiers in produced outputs.

Use existing checks and small disposable queries; do not build a new reusable validator product. Baseline all-feature tests passed, while no-default-feature compilation and strict docs failed; preserve these known failures rather than implying this evidence slice repairs them. New relevant unexpected failures must be retained and bubbled up.

Record all actual commands, exits, denominators and exclusions in `artifacts/execution.md`; cite full source evidence in the format contract. At completion write `closing-report.md` walking AE-01 through AE-08 exactly once, artifact inventory, source state, changes, findings and unresolved dependent gates. CC reports are proposed-done until CDC independently accepts the evidence. Do not close the parent arc/project.

Expedited Mode: commit the exact files you produced/updated on planning using `git commit --only`; list those paths explicitly. Required trailers exactly once each:

```text
Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>
```

No main commit, publication, tag, or external message is authorized by this assignment. Return the evidence and closure paths, commit hash, unresolved field decisions, and the recommended next repair scope.
