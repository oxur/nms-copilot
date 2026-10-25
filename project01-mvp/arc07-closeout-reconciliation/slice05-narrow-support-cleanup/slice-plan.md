# Slice05 — Narrow support cleanup

Status: ready/issued. Dedicated source clean8f10d4cbe04f7d781667cc4e37280c333a67ee8d,
branch codex/workspace-coverage-contract. Same sole CC LunaHigh implements; CDC
SolHigh independently reviews. Ownership transfers at dispatch and returns at
handback. AT-03/R07-S04 remain open for07; proceeding with independent ready work
does not close them. Duncan adopted this cleanup and exact main restoration.

Reduce one concrete maintenance duplication: MCP integration suites should use
the existing shared HTTP request/response/chunk transport mechanics, while each
test retains its own cases, raw/source/cache oppositions and expectations.
Restore only the known unused nine-line wrong-checkout addition on main.

## Resolved implementation boundary

The dedicated candidate's `tests/mcp_live_integration.rs` HTTP request/parser/
chunk decoder at4182–4282 duplicates `tests/support/live_mcp.rs`894–988. Both
use connect/read/write timeouts, Connection:close, optional Content-Length,
lowercase header maps and chunk decoding. Two differences must be preserved:
the integration wrapper implicitly adds Content-Type/Accept on POST and accepts
an explicit phase timeout; shared callers supply headers and use HTTP_TIMEOUT.
The integration decoder also accepts chunk extensions. Reuse existing support:

- Add a shared explicit-timeout request helper; keep existing shared
  http_request as an HTTP_TIMEOUT wrapper. No socket/process framework.
- Keep the local integration request wrapper to supply its existing POST
  headers plus extra headers and forward its exact timeout. Share response
  representation/parser/decoder; use the existing shared text method instead
  of retaining a duplicate response type. Preserve chunk-extension support and
  malformed/truncated rejection. Header ordering/values and body framing stay
  equivalent; no silent timeout extension.
- Preserve HttpClient sessions, child ownership/teardown, startup/bind-retry
  matrices, all test-specific fixtures/literal assertions/negative controls,
  shared SSE cursor behavior and called candidate wait_for_stderr helpers.
  SSE JSON parsers intentionally remain distinct: one is strict/error-returning,
  the other tolerant with CRLF normalization and direct-JSON fallback. Do not
  conflate their policies or refactor additional mechanisms.
- One small existing-module literal parser test may cover plain/chunked success,
  extension and malformed/truncated controls; existing real integration cases
  prove transport. No new testing system, model oracle or file hierarchy.

Main is a separate exact restoration, not implementation. Immediately before
editing, require HEADb616ab2cfa9c67dfb6361bb1355ca9e6680d18cc, whole-file blob
2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e and the known one-hunk nine-line addition
in `crates/nms-copilot/tests/mcp_live_integration.rs`. Remove that literal helper
block only; assert resulting bytes equal that file at main HEAD and its diff is
empty. Preserve any drift and report a blocker instead of restoring broadly.
No git restore/checkout/reset and no main commit. The committed/called helper in
the dedicated candidate remains. Old49 bytes remain unissued/protected.

## Proof and exit

Allowed candidate source paths ONLY the two test files above. Main allowed only
the exact nine-line restoration. Record pre/post test-function/case population
from existing source/diff without a new index/checker artifact; fixture matrices
and expected values must not change. Run candidate make format→make lint, then
native/offline locked all-feature mcp_live_integration and
owned_mcp_integration targets, including existing paired-SSE/timeout/malformed/
cleanup controls. No production change: final07 owns full-workspace gate.
No-default target compile is already blocked at8f10d4c; retain that explicit
gate rather than repairing feature cfg here or pretending a subset closes it.
After main bytes equal HEAD, use read-only format check→lint→affected native
MCP integration target. Do not run a formatter that could write unrelated main
paths; this is restoration to existing HEAD, not authored Rust there.

If an affected test fails, retain exact command, failed identities/counts and
material error while available in the single result; do not reconstruct them
after compaction. Distinguish pre-existing04 graph-oracle/feature issues from
transport regressions using direct source comparison; no opportunistic oracle
rewrites or new campaign. Correct only cleanup-caused issues within five actual
refinements; otherwise return precise affected failure and complete ready work.
Actual failures stay failures. CDC independently reviews transport semantics,
all-case/oracle retention, guarded main restoration and scoped real behavior.

Artifacts: single artifacts/result.md plus ordinary closing-report/ledger and
CDC review. Exact named commits/two consecutive trailers. No source reverts,
deps/install/rebase/push/integration/publish, new context/reviewer, runtime
features, coverage/benchmark campaign or06 folder in this assignment.

CDC grounding: read accepted inventory/recommendation, complete duplicated
request/parser/decoder/response bodies, relevant callers and strict/tolerant
SSE bodies, existing support tests, exact main diff and called helper sites.
Existing Rust/verification guidance remains applicable; no new dependency API.

Assignment: initial cc-prompt.md,2026-10-25; five-refinement cap, no budget reset
within this slice. Slice04 failures and open proof remain separately recorded.
