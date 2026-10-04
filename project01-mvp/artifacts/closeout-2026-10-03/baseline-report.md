# Project01 baseline — attested preparation evidence

Local date: 2026-10-03; raw timestamps are UTC (some are 2026-10-04). Source: main `a800950ef3558fcf463517696c3c140061e7353d`. Toolchain and source cleanliness are in [baseline.json](baseline.json). No production files changed.

## Executed checks

All local Cargo checks used `CARGO_NET_OFFLINE=true` and `CARGO_TERM_COLOR=never`. Commands, exits and elapsed times are retained in [gates.json](gates.json) and [additional-gates.json](additional-gates.json).

| Check | Result and limit | Evidence |
|---|---|---|
| `make lint` | Pass: all-feature/all-target Clippy with warnings denied and format check | [lint.log](lint.log) |
| `cargo build --workspace --all-features --locked` | Pass; compile only | [build.log](build.log) |
| `make test` | Pass: 807 passed, 0 failed, 0 ignored, 0 measured, 0 filtered; 30 result groups including empty doctest groups | [test.log](test.log) |
| `cargo test --workspace --no-default-features --locked` | **Fail, compilation exit 101**: four invalid HTTP references in the binary; unused SocketAddr warning. Tests did not complete | [no-default-test.log](no-default-test.log) |
| `cargo doc --workspace --all-features --no-deps --locked` | Exit 0, but broken links/HTML and output filename collision warnings; not clean documentation acceptance | [rustdoc.log](rustdoc.log) |
| Same doc command with `RUSTDOCFLAGS='-D warnings -D missing_docs'` | **Fail, exit 101**: missing public docs and documentation errors | [strict-rustdoc.log](strict-rustdoc.log) |
| `make coverage` | Exit 0; selected REPL library tests without default features. Mixed dependency report total 3,908/7,883 lines = 49.58%; cannot establish workspace test coverage | [coverage-existing.log](coverage-existing.log) |
| `cargo llvm-cov --workspace --all-features --locked --summary-only` | All 807 exercised tests pass; **11,548/14,228 lines = 81.1639%**. 69 reported source files; below 95% | [coverage-workspace.log](coverage-workspace.log), [exporter summary](coverage-workspace-summary.json) |
| `cargo package --workspace --list --locked` | Pass; file-list inspection only. Does not prove archive builds, installation, or publication | [package-list.log](package-list.log) |

Coverage function/region totals: 1,353/1,593 functions = 84.93%; 18,836/23,477 regions = 80.23%. Branch/MC/DC counts are zero because those metrics were not collected; that is not 100% coverage. No custom ignore regex, doctest coverage option, or coverage hardening was added. Full segment/function JSON detail was moved to a temporary file to avoid committing a 68 MB dump; its digest/location is [recorded](coverage-detail-location.json). The official per-file summary and console totals remain durable; temporary detail is not a closure dependency.

Crate-level sums from the 69-file exporter summary (umbrella crate has no reported executable lines):

| Crate | Covered/total lines | Line coverage |
|---|---|---|
| nms-cache | 221/241 | 91.70% |
| nms-cli | 902/1,323 | 68.18% |
| nms-compat | 68/68 | 100.00% |
| nms-copilot | 3,467/5,398 | 64.23% |
| nms-core | 1,213/1,269 | 95.59% |
| nms-graph | 2,218/2,280 | 97.28% |
| nms-query | 1,516/1,576 | 96.19% |
| nms-save | 1,598/1,668 | 95.80% |
| nms-watch | 345/405 | 85.19% |

## Behavior probes

These are captured failures through the built CLI, not new committed Rust tests. Probe JSON retains commands, exits, expected/observed values, and the limited oracle. Fixtures contain synthetic identity values; no private save was committed.

1. **F01 — Wrong packed discovery address.** [Synthetic input](synthetic-address-save.json) carries UA `0x212B0004B2446A`. Expected portal `212B04B2446A` derives from the user's successful independent game visit. Export returns `2B0004B2446A`. A [portal-format control](portal-control-probe.json) substitutes a 48-bit portal encoding into the same UA field and returns the expected glyphs, exposing why portal-shaped fixtures conceal the game-packed decoding error. It does not establish support for that substituted save encoding. [Probe](address-probe.json), [output](address-export.stdout.json).
2. **F02 — Custom names discarded.** The same synthetic input gives system and planet `DM.CN` values. Both names are empty in export. The observed real save records motivated this field witness; source conversion/extraction explicitly supplies `None`. No conclusion about unknown name representations is implied.
3. **F03 — Biome filtering changes associations.** Export the tracked multi-system fixture with/without `--biome Lush`. Filtering should retain the unfiltered Lush planets' system coordinates. Expected X coordinates are 100, 100, 400 with distinct full coordinate/address tuples; the observed filtered result assigns all three to the same system. [Probe and exact tuples](biome-filter-probe.json). `execute_find` searches systems by planet index alone in its biome-only branch; planet indexes repeat across systems. This probe preserves the unfiltered association as its control and does not independently prove the unfiltered addresses themselves are correct.

## Other source-confirmed or unresolved findings

| ID | Finding and evidence strength | Required disposition |
|---|---|---|
| F04 | No-default-feature compile failure, reproduced above | Repair supported HTTP feature boundaries and feature-appropriate tests |
| F05 | Coverage falls short; the Make target selects a narrower test population | Meaningful workspace/module hardening and faithful coverage gate |
| F06 | Strict docs fail; CONTRIBUTING absent; no tracked LICENSE files found in root inspection | Repair public docs/examples/contributor guidance and assess package licensing material; metadata alone is not a package-content witness |
| F07 | Slot resolver indexes `accounts[0]`; import bypasses the slot resolver; list errors become empty-state messages. Source-confirmed, not dynamically exercised here | Safe selection/error behavior or explicit accepted amendment. Friendly redesign remains project02 |
| F08 | Community import `_source_name` is ignored; imported rows are ephemeral. Source-confirmed | Assess promised external-source tagging separately from historically explicit persistence deferral |
| F09 | Separate watcher startup paths in REPL and MCP. Source-confirmed; simultaneous runtime behavior not exercised | Demonstrate/update shared-view composition and reconcile duplicate ingestion risk |
| F10 | Packed base conversion also routes through the truncating wrapper and forces galaxy zero. Source-confirmed; field semantics not fully established | First evidence slice must audit base vs discovery encodings before a repair contract is issued |
| F11 | Original acceptance/closure records absent; summary numbering and detailed documents differ; some early examples conflict with 12-glyph parsing | Complete the historical scope reconciliation; preserve accepted amendments with authority |
| F12 | Public registry reads blocked; package archive verification and clean installation unrun; MSRV 1.91 not installed locally | Retain failures and run remaining checks in a suitable environment; do not infer publication absence or MSRV compatibility |

## External and environmental checks

[CI run 33693728927](https://github.com/oxur/nms-copilot/actions/runs/33693728927) passed on 2026-09-02 for the exact source hash. The Check job and default-feature tests on Ubuntu/macOS/Windows succeeded; [run receipt](ci-runs.json) and [job details](ci-jobs.json) preserve original commands/dates. Retrieval used GitHub's CLI after a restricted network attempt failed. This is retained external evidence, not a new cross-platform run on 2026-10-03.

Ten historical 0.2.0 registry probes were attempted. Python failed certificate verification; a retry with normal system TLS verification received HTTP 403 for every endpoint. Both attempts are retained: [Python](published-0.2.0-python-attempt.json), [curl](published-0.2.0-curl-attempt.json). A response denial establishes no package-publication verdict. No credentials were output, no verification was bypassed, and no package was published.

## Checks still unrun

Full semantic review of the 43 original numbered documents; independent per-row review; exact planet glyph generation for every result; nonzero-galaxy game-packed fixtures; complete route/live-watch/cache composition; real MCP stdio/HTTP handshakes and update propagation; interactive map/setup terminal behavior; declared-MSRV builds; package archive verification/clean registry installation; performance promises (<10 ms spatial queries and <100 ms cached startup). Their original requirements remain open in the ledgers. Documentation tests/builds and existing unit tests cannot substitute for these demonstrations.
