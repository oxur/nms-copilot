# Slice37 third actual refinement — stable borrowed tool names

2026-10-25 UTC. Third ACTUAL refinement3/5/two remain; same selected CDC6.1-solHigh/CCcc_mcp_docs6-lunaHigh, Arc06 only. Preserve initial/01/02 and all actual failures. Source frozen at d553037 plus current four-file docs diff e52a68362a4144e104a72cc7e18faee2cf96526041fade7ef32575eccc73597e, identityc9dabb3a1f1a713e309dc30203c30611b2e1d03455d47cfdc3fad7237d85519f0.

Actual cf2fbff407374343885f1e157858012a restricted library228passed/7watcher-timeout failures retained; identical authorized-host retryc586ada8600040b3b705df25b2c0cd60 all235pass, sameargv/override/originalTMPDIR/source. Allfeatures doca9c4295d069d40ddb8e7c84a9e82535e exit101/12passed1failed/fullraw/status/equalidentity independently inspected. E1 tool.name.as_str resolves through Cow deref to unstable str_as_str (E0658). No runtime product defect established. CDC initial/twice-held source review missed this concrete name-field API despite preserving the valid population; record this review miss, not an author-literal retraction.

## ONE exact example-line change

Only `crates/nms-copilot/src/mcp/tools.rs` E1 module codefence line76. Replace exactly:
```rust
//! let names: Vec<_> = listed_tools.iter().map(|tool| tool.name.as_str()).collect();
```
with:
```rust
//! let names: Vec<&str> = listed_tools.iter().map(|tool| tool.name.as_ref()).collect();
```
Actual pinned primarysource rmcp1.1.1 src/model/tool.rs19: Tool.name is Cow<'static,str>; fabryk-mcp-core0.4.2 registry.rs45 returns Vec<Tool>. Existing source tools.rs890 uses precisely Vec<&str>/t.name.as_ref(). Explicit type avoids ambiguous AsRef inference. Keep named owned listed_tools alive; preserve ALL eight literal ordered names and every wrapper/typederror/fullJSON assertion. No other source/doc/example/layout/test/non-doc change. config/mod/resources wholefiles FROZEN; remaining tools bytes FROZEN. No new functional scenario, dependency or policy.

## Execute/hold/release

Read current assignment/ledger and coherent readback; validate resolved dedicated-root prefix+exactallowlist/currentexactline/count before mutation. Capture offline format THENlint rawfirst through original kernel, immediate source/index/gates HOLD; no postpair edit or runtime until CDC exact-line review. Preserve all earlier IDs/denominators. CDC then releases affected allfeatures/no-default docs and remaining original CargoJSON+separate predicate/scope/diffs. Prior library235pass remains applicable by byte identity of ALL executable and test regions; no gratuitous library repeat for doc-example-only change. Final independent original ninegroups still required on frozen source. Original record/normalhook/exactfourpathcommit/uniqueCCmessage/parsedtrailers/ownershipreturn/6MP/retro/bubbleup/fullparent+required38 owners remain. No implicit extra authority beyond five actual refinements, no push/merge/install/network/release/otherarc/Project02.
