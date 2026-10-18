# Slice13 CC closing report — MCP watch settings

## Assignment and provenance

- Assignment: project01-mvp/arc06-fit-and-finish/slice13-mcp-watch-settings/cc-prompt.md, SHA-256 455a3e9c2586fa6826b66a8cde8615da2af3df3c85773d1c3669d483732bf454.
- Issuance: planning commit fad379d9692964db4ef0a9e922d4d7752fedd9f5, parent ae9bd828bb614f6348a0cba5fba237130fa275e4. Planning intake: b32da856757009e2c893873535f4b867551e4f76, receipt-only child of issuance.
- Accepted source: a101bd44b277f06c75ed7260b470b31cd5510130. Candidate: .worktrees/mcp-watch-settings, branch codex/mcp-watch-settings.
- Source commit: 16c078725492744db22bae837d87c89489a6add5, direct parent a101bd44b277f06c75ed7260b470b31cd5510130. It changes exactly README.md, crates/nms-copilot/src/main.rs, crates/nms-copilot/src/mcp/mod.rs, and crates/nms-copilot/tests/mcp_live_integration.rs; both required trailers occur once. The final planning close commit is recorded in the validation artifact's close receipt.

## Implemented behavior

Before choosing headless or normal mode, main resolves one optional owned WatchConfig from the already resolved save path. Disabled watching or a missing path produces None without filesystem probes, rediscovery, or path canonicalization. Enabled watching carries the exact path and configured debounce, including zero. Headless MCP receives that value; normal mode starts its existing REPL watcher from a clone and moves the remaining setting to the background MCP server. Existing announcements, blank line, warn-and-continue behavior, cache/session/drain code, and both normal-mode producers remain.

The MCP module retains the legacy public signatures and default behavior (Some(path) uses 500ms; None disables watching), and adds only run_headless_with_watch_config and spawn_mcp_background_with_watch_config. The configured helper forwards the caller's exact configuration and retains watcher ownership for the existing server lifetime. Recoverable watcher failures still warn and serve.

On Unix, native children receive isolated temporary HOME directories and explicit config files. The existing stdio and HTTP enabled scenarios pass with 137ms debounce. New disabled stdio and HTTP scenarios initialize the real server, atomically replace a valid Phase A save with Phase C, and repeatedly query the real tool and location resource across the 1500ms boundary; both continue to report full Phase A state. The user README now documents defaults, disabled behavior, startup-only application, transport stream guidance, and known follow-up boundaries.

## Validation and limitations

All fourteen prescribed gates passed after four unchanged authorized-host retries. The sandbox attempts are retained with their separate raw stdout/stderr, digests, environment observations, and harness counts. The retries were needed for HTTP loopback binds or native filesystem watcher events denied or timing out under sandbox execution. The native MCP gate passed all four tests on retry; the focused suite, no-default-features unit suite, and make test passed on their retries. All other gates passed on the first attempt. See artifacts/validation.json for every command and channel.

The candidate has exactly the assigned four source paths. All 26 protected source hashes and all three frozen MCP runtime regions match intake. The source and planning git diff --check gates pass. Validation ran on Unix only; non-Unix child launch behavior remains the existing path.

## Disposition and handoff

CC proposes all seven WS-01 through WS-07 rows done. Every row remains open pending independent CDC replay; no acceptance, parent closure, or integration is implied. Ledger status and criteria cells are unchanged.

Artifacts retained for review: artifacts/author-baseline.json, artifacts/execution.md, artifacts/validation.json, slice-plan.md, ledger.md, this closing report, and the immutable issued prompt. The source candidate remains isolated; no merge, integration, install, push, or release was performed.

The assigned increment is complete. Broader startup-cache/live-update lifecycle, unknown-system resolution, write-through and duplicate-producer composition remain with planned Slice14/CDC. No new bubble-up was found. Existing parent owners and the seven independent CDC dispositions remain open.
