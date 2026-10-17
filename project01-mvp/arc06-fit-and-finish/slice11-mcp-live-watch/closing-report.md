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


# Slice11 CC return — Iteration01 save-path identity and HTTP proof correction

## Disposition

**Proposed-done for independent CDC review; not accepted.** The path identity correction and HTTP test-oracle/subscription corrections are implemented within the iteration allowlist. Corrected-source filesystem and MCP transport behavior passed all required gates, including unchanged authorized-host retries under the original TMPDIR. All seven LW rows remain `open` pending independent review. The initial CC stop report above is retained verbatim as historical evidence; this new proof does not rewrite the prior failed observations or the CDC's changed-input control.

## Assignment and source identity

- Active assignment: `project01-mvp/arc06-fit-and-finish/slice11-mcp-live-watch/cc-prompt-iteration01.md`, SHA-256 `247a15b9d8220f8b18f4036f4a11b2b9af929cba37928e0f8f1dba92d3acc2ce`.
- Issuance: `c23e2800351d87a944458bb6b60d3cad9d5f8676`, parent `0e8218a7ad71c5aab6856e292fa6bba1b81347b5`; actual CC planning intake HEAD before edits: `af36306f8d9d988662e28cc1b2ffff6e318c8087`.
- Source branch `codex/mcp-live-watch`; new source commit `46ca153c4418fcbf859f76ef59d600985a315b60` directly parents `cdf1e20f46a2e1252f4c27d2dd228d318a179f61`, which remains the initial CC return. Exact current source paths: `crates/nms-copilot/src/mcp/mod.rs, crates/nms-copilot/tests/mcp_live_integration.rs, crates/nms-watch/src/watcher.rs.` Required Codex and Billo AI trailers each occur once. Source worktree is clean after commit.
- Planning return changes are limited to `artifacts/execution.md`, `artifacts/validation.json`, `closing-report.md`, and `ledger.md` Evidence/Notes cells. Prior prompt, plan, baseline, CDC records, and prior evidence prefixes remain unchanged.

## Corrected-source observations and validation

Both native transports reached A/B/C/D using the same real fixture and original TMPDIR. Every phase received the actual `nms://player/location` notification and both `where_am_i` and resource reads returned literal expected physical galaxy/coordinates/SSI/glyph. Stdio also observed five malformed-save parse failures, held B as the last good location during malformed input, recovered to C, returned D to A, kept stdout JSON-RPC-only and exited orderly. HTTP confirmed successful subscription before mutations, sessionful POST/SSE GET, `mcp-smoke`, and explicit child cleanup. Exact tool/resource payload observations and lossless stdout/stderr captures are in `artifacts/validation.json` attempt 48. Synthetic galaxy9 system label was `(unknown)` in both tool and resource responses; no unknown-system policy or production location semantics were changed.

All twelve required gates pass on corrected source. The initial sandbox watcher/loopback failures and unchanged authorized retries are separate receipts: 43→44, 45→46, 47→48, 49→50, 51→52, 53→54. These successful retries used the original `/var/folders/...` TMPDIR, not the canonical TMPDIR test control. Full raw channels/environment/overrides and per-harness test counts are append-only in `artifacts/validation.json`. The initial nms-watch test fixture dependency compile error is retained at attempt40; the helper was changed to the existing raw JSON style, then format/lint and all downstream gates passed. Strict cache docs pass only at attempt59 with `RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links'`; attempt58 without the flag is separately identified as non-strict.

## Proposed row walk

| Row | Current CC evidence | CC disposition |
|---|---|---|
| LW-01 | Retained WatchHandle native write and owner-drop disconnect, corrected path identity, full lifecycle pass | open for CDC |
| LW-02 | Stdio real A/B/C/D notifications/rereads, malformed recovery, same-glyph galaxy negative control, stdout/EOF checks | open for CDC |
| LW-03 | HTTP real session/SSE updates and A/B/C/D reads, successful subscription result, mcp-smoke | open for CDC |
| LW-04 | Existing owner/task/lock/logger implementation unchanged from cdf1e20; current endpoint rereads and protocol stdout exercised | open for CDC |
| LW-05 | None/missing/malformed fallback; live writes; five-failure barrier, retained good state and valid recovery; bounded teardown | open for CDC |
| LW-06 | Protected README/main hashes unchanged; focused package/workspace suites, build and doc gates pass | open for CDC |
| LW-07 | Complete iteration intake, all original 38 attempts preserved, 24 new validation attempts (39-62), exact commits/pathsets/trailers and held fences | open for CDC |

No CC row is accepted or closed. The candidate remains unintegrated and uninstalled. Slice12 and all original parent/raw-repair/release gates remain open; no waiver, scope reduction, integration or Slice/Arc/Project acceptance is claimed.

## Artifacts and handoff

Artifacts preserved in the named slice home: `artifacts/author-baseline.json` (immutable), `artifacts/cdc-review-receipts.json` (immutable), `artifacts/execution.md` (appended), `artifacts/validation.json` (all38 original attempts preserved plus iteration01 attempts), this report (initial stop report retained as exact prefix plus this return), and `ledger.md` (Evidence/Notes only). No separate diagnostics framework or fabricated event trace was added.

Operator handoff: Iteration01's prescribed path correction, HTTP test oracle and subscription proof are implemented; native behavior and all current gates pass on the unchanged authorized-host retries. Please relay the candidate and evidence to CDC for independent review. All seven rows remain open.
