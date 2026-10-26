# Slice05 CC candidate result

Status: candidate returned for independent CDC review; no acceptance is claimed. The assigned transport duplication is removed from the live MCP integration target, its existing public-in-test helper remains a timeout wrapper, and the protected main worktree has been restored exactly to HEAD without a commit.

## Change and preservation

`mcp_live_integration.rs` now uses `live_mcp::HttpResponse` and the existing `.text()` method. Its local `http_request` wrapper remains so it can preserve POST `Content-Type` and `Accept` headers, append session/other headers in the same order, and forward each call's exact timeout. The shared support request path now has an explicit-timeout form; the existing `http_request` signature delegates with `HTTP_TIMEOUT`. Connect/read/write deadlines and HTTP request framing are shared.

Response header parsing, representation, and chunk decoding now have one implementation. Shared chunk decoding accepts the chunk extensions already accepted by the removed integration decoder and continues to reject malformed-size and truncated-chunk controls. One literal parser test covers plain response parsing, a chunk extension, invalid size, and truncation. The live integration suite retains its own tolerant `http_json_messages` policy; the support module's stricter `json_messages` and both SSE stream/parser policies were left alone.

The mcp live integration source has 20 test attributes before and after; owned MCP remains 5 before and after; shared support goes from 3 to 4 attributes for the one added parser test. No test function, fixture matrix, expected literal, negative control, retry policy, SSE case, or cleanup case was removed or rewritten. The actual target totals were 24 tests and 9 tests respectively because each target includes the shared support module.

The main guard was verified and restored literally: main HEAD is `b616ab2cfa9c67dfb6361bb1355ca9e6680d18cc`; before removal the working-file blob was exactly `2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e`, with only the authorized nine-line `HttpChild::wait_for_stderr` addition. After removal, `git show HEAD:crates/nms-copilot/tests/mcp_live_integration.rs | cmp - <path>` succeeded, `git hash-object` returned `5536225183db9f36fd6ba79b045f08c52d9dddbf`, `git diff` was empty, and main `git status --porcelain` was empty. There is no main commit.

During the guard check I first compared `HEAD:path` to the expected working-file blob, which naturally did not match. I corrected the check by hashing the working file; that matched the authorized `2cd450…` blob exactly. No main mutation occurred until the correct guard and exact diff were verified.

## Commands and results

Candidate worktree `codex/workspace-coverage-contract`, baseline `8f10d4cbe04f7d781667cc4e37280c333a67ee8d`:

1. `make format` — passed.
2. `make lint` — passed, including Clippy and format check.
3. `cargo test --locked --offline -p nms-copilot --all-features --test mcp_live_integration` — **24 passed, 0 failed**, finished in 58.24 s. The new shared parser test passed with the existing native HTTP, stdio, SSE, cache, retry, and lifecycle cases.
4. `cargo test --locked --offline -p nms-copilot --all-features --test owned_mcp_integration` — **8 passed, 1 failed**. The failing test is `test_owned_live_loader_coordinator_and_mcp_preserve_placeholder_identity`; its isolated child panicked at `crates/nms-copilot/tests/owned_mcp_integration.rs:416:10` because `compare_cache_model` returned `complete graph node IDs/weights/edges or counts differ`, then the outer test failed at line 641. CDC traced the mismatch to `expected_unknown_lifecycle_model(Stage::Startup, false)`: its explicit startup oracle retains the old seed-only D1–E4 edge, while adopted Slice04 behavior rebuilds default edges after ensuring new U2. The other eight tests passed, including the new parser test and owned HTTP lifecycle case. This slice leaves that oracle unchanged and does not rerun it; the affected integration acceptance subclaim remains open for07 reconciliation.

Restored main worktree, HEAD `b616ab2cfa9c67dfb6361bb1355ca9e6680d18cc`:

1. `cargo fmt --all -- --check` — passed (read-only).
2. `make lint` — passed.
3. `cargo test --locked --offline -p nms-copilot --all-features --test mcp_live_integration` — **7 passed, 0 failed**.

No workspace replay, no-default package run, dependency/install action, or external effect was performed. Slice04 AT-03 and the Arc07 full-workspace/no-default gates remain open for07; this support-only change does not close them. The owned integration target's graph-oracle mismatch is separately carried to07 and is not claimed green.

Source commit: `8d572529445d80e404c1d45813c8b4a5b4b732fc` (`test: share MCP HTTP integration support`). Initial planning evidence commit: `38d55158d528c7edd1fc57eb990a903a43794591` (`docs: record slice05 candidate evidence`). CDC disposition remains independent.
