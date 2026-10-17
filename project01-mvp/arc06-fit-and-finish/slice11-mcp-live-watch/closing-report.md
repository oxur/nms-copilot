# Slice11 CC return — MCP live save tracking

## Disposition

**Incomplete; affected live filesystem work stopped for CDC.** The Slice11 source changes are committed and reviewable, but the prompt-required real watcher proof failed on both transports and in the retained-handle unit assertion, including unchanged authorized-host retries. The current source allowlist excludes `nms-watch` source; CDC must decide whether the next diagnostic or correction needs a scope amendment. No test was waived, ignored, or replaced. All seven ledger rows remain `open`; this is not CDC acceptance or release evidence.

## Identity and boundaries

- Assignment: initial `cc-prompt.md`, SHA-256 `e37e0ac70d3fdccd0d6c95df5952b490222d588855d7155f8914868f254f18b9`; issuance `3509057d2c0af7e0134a06777c16dc97267eea72` (parent `5fed0e023da725afcf312cb9bf8c0492f5d59cdf`); planning intake `0880cc5d9564f021c2d09385e360a4d13e086d83`.
- Source branch `codex/mcp-live-watch`, parent `e615f25419cc68e367c07eda6ef73d26c4e4839c`, commit `cdf1e20f46a2e1252f4c27d2dd228d318a179f61`.
- Exact source paths: `README.md`; `crates/nms-copilot/src/main.rs`; `crates/nms-copilot/src/mcp/mod.rs`; `crates/nms-copilot/tests/mcp_live_integration.rs`. Trailers: `Co-authored-by: Codex <noreply@openai.com>` and `Co-authored-by: Billo AI <ai-engineering@billo.systems>`.
- Source worktree is clean after commit; no main/dependency/Project02 worktree movement, integration, install, push, or release occurred.

## Proposed row walk

| Row | CC disposition and evidence |
|---|---|
| LW-01 | Wiring retains the WatchHandle for both entrypoints and guards the delta task. Private later-write test timed out, including on authorized host; runtime behavior is unproven. Open. |
| LW-02 | Real stdio child completed initialization, enumeration, phase-A reads and subscription. Atomic B produced no location update by 8 seconds; C/D and recovery were not reached. Open. |
| LW-03 | HTTP health, session initialize/initialized, tools/resources, phase-A reads, subscription and open SSE GET were reached. Atomic B produced no location update; later reads and `mcp-smoke` were not reached. Open. |
| LW-04 | Lexical model lock release and headless stderr logger override are in source. No successful post-update reread; static and lint evidence only. Open. |
| LW-05 | None/missing/malformed startup fallback checks passed. Retained-write and five-failure/recovery behavior were not demonstrated. Open. |
| LW-06 | README MCP section updated; workspace build and docs passed. Test gates remain red at the watcher assertion. Open. |
| LW-07 | Intake, source scope, attempts, environments, raw channels and commits are recorded. Independent CDC/Operator disposition is outstanding. Open. |

## Validation

Every attempted command is retained losslessly in `artifacts/validation.json` with pre-launch argv/cwd, the four environment variables (all unset except the strict-doc command override), offline Cargo config search, separate stdout/stderr base64, byte lengths, SHA-256 and exit. The strict gate recorded `RUSTDOCFLAGS=-D rustdoc::broken_intra_doc_links` as a command override. Failures and authorized-host retries are preserved as separate attempts.

- Passed: final `make format`, final `make lint`, `cargo build --workspace --all-features`, `cargo test --workspace --all-features --doc` (0 doc tests across listed crates), selected no-deps docs (warnings remain in existing unrelated docs), strict no-deps nms-cache docs, and pre-commit source/planning `git diff --check`.
- Failed: native all-features integration (2 failed/0 passed); package tests (nms-copilot 219 passed/1 failed; other recorded package suites passed); no-default copilot library (218 passed/1 failed); `make test` (same copilot 219/1). Failures are the absence of filesystem watcher deltas after valid writes/replacements, even on authorized-host retry. Ignored tests: 0.
- Sandbox attempts were retained before their required unchanged host retries. The host retries still failed, so the issue is not recorded as a mere sandbox denial.
- The phase fixtures assert A/B glyph equality with distinct Euclid/Eissentam reality indices, but actual update payloads for B/C/D were never observed. The existing system label was not durably recorded; no claim is made about unknown-system naming or promotion.

## CDC handoff and retained scope

The source change retains `WatchHandle`, moves only the receiver into the existing bridge, owns/cancels/joins the delta task, releases the model write guard before notifier awaits, routes headless logs to stderr, adds real process tests, and updates MCP docs. However, both transports observed no notification after save changes. The unchanged authorized-host retries reproduced this; the in-place private-handle write test also timed out. Diagnose within the still-open watcher consumer/source contract before accepting live behavior. The prompt explicitly prohibits changing `nms-watch/src/watcher.rs` in this slice, so this work stops for CDC disposition rather than widening source scope or weakening the recipe.

Original parent owners R06-R02/R03/M7.4/C04 remain open. Planned Slice12 retains startup/cache C04, old/current envelopes and warm/write-through composition, unknown-system placeholder/promotion, historical player-state diagnosis/disposition, and watch configuration/duplicate-producer composition before original parent acceptance. Slice10's waiver does not apply.
