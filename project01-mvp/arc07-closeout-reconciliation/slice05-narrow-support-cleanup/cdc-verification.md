# Slice05 — independent CDC disposition

2026-10-25. Reviewed source8d572529445d80e404c1d45813c8b4a5b4b732fc and CC
records38d55158d528c7edd1fc57eb990a903a43794591 /
667cf7d8301b3de946248c631fdb69c2fab5472b. **NS-02 accepted. Transport cleanup
is independently reviewed, but NS-01/03 remain open for07's owned-graph oracle
reconciliation.** No all-gates or complete-slice claim.

Inspected the entire two-file diff, complete old/new socket/request/response/
header/chunk bodies and wrappers, callers, preserved SSE policies and shared
parser control. Local POST headers/ordering and exact caller timeout still
reach the shared helper; the established helper uses HTTP_TIMEOUT. Connect,
read/write timeouts, Connection:close and body framing are equivalent. Shared
response normalization/text is equivalent; chunk extensions remain accepted,
bad-size/truncated frames rejected. Private transport diagnostic wording now
uses the shared helper; no test-specific expected value depends on the removed
wording. No application API, data/cache, dependency or production path changed.

Only the response text accessor changes at existing test call sites. Existing
test functions, phase/cache matrices, literal data/expected values, negative
controls, child cleanup/retry/session behavior and called candidate stderr
helpers are preserved. Source test attributes stay20 live/5 owned; shared
support3→4 adds one meaningful literal parser control, included in both targets.
Strict and tolerant JSON/SSE policies remain deliberately separate.

CDC ran on the unchanged committed candidate with native socket/watcher
permission and original TMPDIR:

```text
cargo test --locked --offline -p nms-copilot --all-features --test mcp_live_integration
```

Exit0,24/24,58.00s, including real HTTP/stdio, SSE, malformed/parse recovery,
cache/source oppositions, bind-retry and late-stdout controls. Then CDC ran:

```text
cargo test --locked --offline -p nms-copilot --all-features --test owned_mcp_integration test_owned_live_loader_coordinator_and_mcp_preserve_placeholder_identity -- --exact
```

Exit101,0/1 with8 filtered. Actual isolated child fails at owned_mcp_integration.rs416:10:
`complete graph node IDs/weights/edges or counts differ`; outer failure641:13.
This independently reproduces CC's8/9 whole owned-target result, without
pretending the other eight were freshly reproduced. Startup calls
expected_unknown_lifecycle_model(Startup,false), whose explicit implementation
keeps only old seed D1–E4. Adopted04 now rebuilds default edges after ensuring
missing U2. The source-grounded mismatch predates this transport cleanup; its
fixture/oracle was intentionally not rewritten in05. **07 must align that
literal graph oracle with the adopted behavior and rerun the actual owned
target**, preserving metadata/placeholder/promotion/foreign controls. This is
a concrete existing dependency, not new hardening scope or a green target.

NS-02: CDC independently checked main before cleanup had required HEADb616ab2
and working-file blob2cd450ce plus only the known nine-line addition. After
CC's literal removal, current file blob is
5536225183db9f36fd6ba79b045f08c52d9dddbf; CDC's `git show HEAD:<path> | cmp - <path>`
exited0 and `git status --porcelain` was empty. No main commit or other mutation.
The committed/called candidate helper is preserved. CC additionally attests
main's read-only format→lint→native MCP7/7; CDC's exact-byte check establishes
restoration without claiming that test command independently rerun.

CC candidate format→lint and full owned8/9 remain attested as such. Both source
and planning commits contain the two consecutive once-parsed trailers. Source
is clean; planning has only protected old49 untracked. No full workspace,
no-default, coverage or platform success is inferred.04 AT-03/full native gate
and these05 integration subclaims remain open for07; original numeric/input/
acceptance/external gates stay binding. No formal source refinement/replay or
further05 cleanup is opened. Proceed only with agreed ready06 work.

## Final07 carry disposition — 2026-10-25

The composed clean candidate2b7fc9eba87529181ba609e52ac6a04a8ba6b1b0 independently passes HOST lint and make test:1129runtime+119docs=1248,0failed/ignored; owned9/live24 included. CC true no-default package command exits0 (18 unused test-helper warnings disclosed; its aggregate is not verified). NS-01/NS-03 now accepted; historical failures/limits above remain unchanged. See [07 CDC review](../slice07-final-candidate-validation/cdc-verification.md). No source repair in this follow-up.
