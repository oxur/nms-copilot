# CC assignment — Slice14 native headless cache startup

Execute `project01-mvp/arc06-fit-and-finish/slice14-headless-cache-startup/cc-prompt.md`. Deliver native startup-composition regression tests and accurate user documentation for the existing cache loader, returning proposed-done to Sofie/CDC through Operator relay. Two contributors and Expedited Mode; seven HC rows open. This is approved integration-test delivery, not an investigation or production policy repair.

## Exact state and scope

Start exactly at accepted `16c078725492744db22bae837d87c89489a6add5`, direct parenta101bd44; Slice13 closure `320de77a5ff3fab85ca0649cf5f905090f4b45ba`. Resolve this prompt's actual introducing commit and planning intake HEAD separately. Inspect git worktree list and create/reuse `.worktrees/headless-cache-startup`, branch `codex/headless-cache-startup`, only at that accepted source. No resetting dirty/mismatched worktrees. All main/dependency/accepted13/Project02 pins in artifacts/author-baseline.json remain clean/held. Candidate source commit directly parents16c0787; isolated/unintegrated/uninstalled, no push/release.

Only two source paths may change:

- `crates/nms-copilot/tests/mcp_live_integration.rs`: additive real cached-startup matrix, small directly needed existing private child/config/initialization helpers. Preserve every old enabled137/disabled1500ms movement/recovery/session/subscription/stdout/EOF/kill assertion, fixtures and timeout. Existing constructors continue defaulting to --no-cache; explicit new cached-startup construction can omit that flag. No broad harness rewrite.
- `README.md`: describe actual existing headless cache startup/recovery/bypass and accurate remaining composition limits, preserving useful transport/watch teaching/examples.

All production is frozen byte-for-byte, including main/mcp/config/paths/watch, cache encode/decode/freshness/error/API/layout/revisions2/2, graph/save/delta/query/notify; manifests/dependencies/lock, sealed fixtures, other tests/docs remain unchanged. Do not add rkyv to copilot (it is not a direct dependency), new cache hooks/public APIs/flags/settings or production special cases. If a prescribed expected behavior fails, retain failure and stop affected work for CDC; do not fix runtime or soften oracle. No new inquiry without prior Operator agreement on usefulness/applicability/budget.

CC planning writes only `artifacts/execution.md`, `artifacts/validation.json`, `closing-report.md`, `ledger.md` Evidence/Notes in this slice. Intake/readback precedes Rust edits. All seven statuses stay open; criteria/verification/significance/origin immutable. Plans/prompts/author baseline/CDC/parent/earlier-slice records immutable to CC. Preserve failed/blocked/retried commands, append corrections and history rather than overwrite. Artifacts stay artifacts/; no additional report framework.

## Source-grounded method and implementation spine

Current main resolves cache path, runs public load_or_rebuild before the headless/normal branch, then calls ensure_player_system. Headless output does not reveal was_cached, so infer neither a hit from file presence nor a rebuild from a banner. Use opposed source/cache identities and actual tool/resource reads. Existing native tests always force --no-cache and cannot establish this composition.

Reuse the native phase fixtures and real production binary/RPC clients. Phase A = reality0/X0, SSI1, planet0, glyph000100000000; phase B = reality9/X0, same glyph; phase C = reality9/X1, glyph000100000001. Existing source JSON has version4720, currencies7/8/9, no bases/discovery records and no previous/freighter address. Canonical source model therefore has empty systems/planets/bases, selected/source reality0 or9 and literal PlayerState(address,current_reality,None,None,7,8,9). main adds an unnamed physical placeholder **after** initial cache write; do not expect that placeholder in the persisted cold/rebuilt file or claim promotion correctness.

### A. Reuse child ownership, make cache policy explicit

Extend existing private launch plumbing so a new cached-startup constructor can supply (save path, explicit cache path, CLI bypass true/false) for either StdioProcess or HttpChild, with watch=false in a disposable Unix child HOME. Existing spawn/spawn_with_watch_settings behavior remains --headless --no-cache --save and enabled137 or disabled test settings. A private common helper with no_cache bool is acceptable; no new public type or configuration framework. Both variants use actual `assert_cmd::cargo::cargo_bin!("nms-copilot")` and the actual --headless --save flags, plus --http for HTTP. Only cached cases omit --no-cache; bypass cases deliberately include it.

Use existing isolated_watch_home(false,137), then add `[cache] enabled = true` and an absolute `path` pointing inside that case's temporary directory. Quote the fixture UTF-8 path with serde_json::to_string, and verify the completed TOML via existing toml::Value parsing before child launch: exact cache.path, enabled=true, watch.enabled=false. JSON string escaping for these generated fixture paths is valid TOML; the parse/exact-value assertion catches a bad encoding. Do not change the parent's HOME/TMPDIR or user config, and keep child HOME TempDir alive through termination. Watch=false isolates startup; it must not be replaced by --no-cache in non-bypass cases. No read of the user's cache/save needed.

Reuse bounded initialize/initialized/list/location request helpers and real HTTP health/session readiness. HTTP retains only actual bind-collision retries. Small factoring of existing initialization is allowed if every prior assertion/sequence survives. For each matrix case use a new save/cache/home and child; no fixture mutation/state leak between cases. Stop/reap each child with existing EOF/kill/drop semantics and retain stdio JSON-only / HTTP empty stdout checks. No new timeouts, longer deadlines or unbounded output() call.

### B. Construct independent literal source and cache witnesses

Add a local source expectation helper using `GalacticAddress::new(phase.voxel_x,0,0,1,0,phase.reality_index)` and `PlayerState::new(address,reality,None,None,7,8,9)`. Check parser-derived source model from original save_bytes against this complete literal state, selected reality and three empty maps before using it as the full-model comparand. Never derive expected physical coordinates/reality/funds from a served/cache result. Do not use raw discovery-packed fields to infer galaxy semantics.

Build a healthy typed sentinel model independently: GalaxyModel::new, insert `System::new(B_address,Some("Cached-9".into()),None,None,vec![])`, player literal B/current9/None/None/currencies700/800/900, selected galaxy7. Use existing extract_cache_data(&model,9999) and write_cache. Preassert header exactly `b"NMSCACHE\x02\0\0\0\x02\0\0\0"`, checked read/rebuild full systems/planets/bases/player equality against this original sentinel, selected7/version9999, and exact cached name. Keep original expected model untouched. This sentinel has the same coordinates but different reality from sourceA, so ignoring galaxy or bypassing every cache cannot pass the warm case. This is a test-built valid current cache, not a claim about old game files.

Use standard File::set_times/FileTimes already used by cache integration tests. Source mtime = UNIX_EPOCH+1700000000 seconds; fresh cache = source+100s, stale = source-100s. Assert actual is_cache_fresh true/false immediately before launch. No sleep-based freshness, no setting timestamps after the child loads. Capture original source bytes and original cache bytes/absence before launch; compare source bytes after every case.

### C. Eight cases on each real headless transport

Implement one stdio and one HTTP matrix test, both cfg(unix), HTTP additionally cfg(feature="http"), with case labels visible in assertions/captured observations. Eight cases per transport =16 successful startup witnesses on this Unix/http host. Derive actual counts from execution; a missing case is not equivalent to a green smaller matrix.

| Case | Source | Cache before startup / precondition | Actual tool/resource oracle | Cache file effect |
|---|---|---|---|---|
| cold_missing | C9/X1 | destination absent | full PHASE_C, system label "-" | checked2/2 source4720 cache created |
| fresh_current | A0/X0 | healthy B9/Cached-9 sentinel, actually fresh | full PHASE_B and exact system label Cached-9 | exact original bytes preserved |
| stale_current | A0/X0 | healthy B sentinel, actually stale | full PHASE_A, label "-" | source4720/0 rewritten |
| fresh_schema1 | C9/X1 | exact sealed216-byte schema1/semantic1 predecessor, actually fresh | full PHASE_C, label "-" | source4720/9 rewritten |
| fresh_semantic1 | A0/X0 | healthy current sentinel bytes with only bytes12..16 patched to little-endian1, actually fresh | full PHASE_A, label "-" | source4720/0 rewritten |
| fresh_corrupt | C9/X1 | literal current16-byte2/2 header with empty payload, actually fresh | full PHASE_C, label "-" | source4720/9 rewritten |
| bypass_existing | A0/X0 | healthy fresh B sentinel, actual CLI --no-cache | full PHASE_A, label "-" | exact original bytes preserved |
| bypass_missing | C9/X1 | destination absent, actual CLI --no-cache | full PHASE_C, label "-" | destination remains absent |

Before launching rejected cases, use actual nms_cache::read_cache and match typed category: schema1 gets Incompatible(SchemaMismatch{found:1,expected:2}); semantic1 gets SemanticMismatch{found:1,expected:2}; current empty payload gets Deserialize. The semantic1 header patch is a synthetic compatibility witness, not a genuine historical semantic1 archive claim. Sealed predecessor path from this test file: `../../nms-cache/tests/fixtures/predecessor-schema1-semantic1.rkyv`; include it unchanged, independently check216bytes/header1/1/digest against baseline. Do not decode historical payload as current or regenerate fixture. Preserve the separate360-byte consistent2/2 route fixture and its prior parity coverage.

After server initialization, reuse call_where_am_i_stdio/http for actual full PHASE values, tool/resource system-label equality, returned expected label and protocol hygiene. Emit a concise per-case actual transport/case/source/served/cache-effect observation in captured test stderr, in addition to existing actual tool/resource observations. Serving observation must precede cleanup and come from this child, not read_cache or a new local model. Headless configured watch=false means no later save-change event should contaminate a startup check; do not test live write-through in this matrix.

For cold/stale/schema1/semantic1/corrupt outputs, checked-read the resulting file, verify exact current header2/2, version4720, selected=source reality, complete original literal PlayerState, empty full source systems/planets/bases; rebuild_model maps/state/selected equal original parser-derived source model already checked against literals. Do not compare two rewritten outputs as the sole oracle. Ignore cached_at's changing timestamp rather than pinning wall clock. Original source fixture remains byte-identical. Validate file effect after successful initial RPC and again after child cleanup. For fresh_current/bypass_existing assert exact before/after bytes (including sentinel selected7/version9999/currencies700/800/900), not only successful read. For bypass_missing assert absence before/after, and no missing cache parent created if the selected fixture path has one. The served model's physical name/reality distinguishes warm from source; the independently selected view7 is a cache preservation check, not a claim that where_am_i reports selected view.

No new source-failure/historical-player survey, raw decoder, cache migration or runtime repair is commissioned. Existing malformed-source/contradictory-player lifecycle proof remains covered by unchanged cache suite; full native source-failure/normal/live composition remains15 as applicable. If existing runtime contradicts an oracle, keep original expected value and stop affected work for CDC.

### D. Documentation and proof method

README explains actual startup cache use for headless stdio/HTTP, default enabled per-save cache, explicit [cache].path override and --no-cache bypass of both reads/writes, fresh cache meaning, source rebuild for stale/rejected/corrupt archives, and independent selected/physical galaxy as already implemented. Keep precise watcher startup-only settings and live phase teaching. Explicitly retain normal-mode/full live-cache write-through, unknown-system/promotion and duplicate-producer composition limits; never claim full C04 or a performance gain. No new README section pretending an unimplemented feature exists; scope this as tested existing behavior.

Applied guides: testing Behavior/Contracts + Proportionate Proof choose existing production child/RPC plus checked cache file effects and opposed witnesses; no reusable validator/new test-only public API. Rust AP-09 AVOID permits test invariant expects while production remains frozen; EH-01 MUST preserves loader's source-before-rewrite behavior; OB-17 SHOULD preserves child/TempDir ownership and RAII cleanup; DC-01/02 preserve truthful source/user docs. Concrete constructors/mtime/path APIs already exist in pinned source. Author did not execute the illustrative matrix; CC must implement and run it. No precise OS timing/zero-copy/startup-speed/comparative study claim.

## Required reading and coherent preflight

Installed skills root `/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills` or verified equivalent. Read required text completely in bounded outputs and recover truncation. Record visible text extents and structured query coverage separately. Before dependent edits, record source/planning/issuance/clean pins, prompt/baseline digests, full reading/data results and a source-cited contract readback in allowed execution/validation files. Explain eight case distinctions, source/cache literal comparands, cache-write-before-placeholder boundary, CLI bypass/config/home/lifetime, source immutability and all remaining owners. Coherent preflight permits execution under existing authority without another approval.

1. **Required-full:** applicable AGENTS.md, this prompt, current slice-plan.md and ledger.md. **Required-section:** latest Slice13 cdc-verification.md full seven-row final review/limits, parent plans' Slice13 closure and Slice14/15 current ownership; parent ledger R06-R02/R03/S13/S14. Earlier slice prompts/raw archives reference-only; no recursive evidence re-read.
2. **Required-full:** PM guides/README.md, work-verification/guides/03-row-closure.md, testing/guides/01-testing-discipline.md. **Required-section:** collaboration-framework Notes/When to use; engineering-methods methodology roles/shared invariants/two contributors; implementation-prompt-authoring Required reading/intake; PM top-down useful increments/investigation agreement; WV independent verification assigned reviewer/retention; testing validation gates Gate Selection/Proportionate Proof/Minimum/Gate Evidence. Any future iteration conditionally loads PM preserved-prompt/iteration workflow before issuance; current prompt never edited.
3. **Required-full before Rust edits:** rust-guidelines/SKILL.md, guides/11-anti-patterns.md per repo. **Required-section:** EH-01 in error-handling, OB-17 in ownership-borrowing, DC-01/02 documentation. No unrelated runtime/dependency/style migration.
4. **Required-full at16c0787:** native mcp_live_integration.rs, cache serialize.rs/freshness.rs/data.rs/error.rs, copilot Cargo.toml. **Required-section:** main load_model call and mode split/configured paths/ensure order; config Config::load/cache_enabled/cache_path_for plus paths home; graph from_save/ensure_player_system/new/insert_system; core PlayerState constructor and System::new; save convert::to_core_player_state; load_integration.rs literal FileTimes/freshness/current header/revision fixture constructs; README watcher/MCP/cache descriptions. Source references named in baseline. No current production mutation authorized by these reads.
5. **Required-data:** author-baseline.json source/parent/closure, all allowed/protected hashes and held pins, whole-population comparison at intake/exit. Check actual immutable fixture bytes/hash and recorded216/360 identities. Query source/planning commit parents/pathsets/trailer counts and clean worktrees. Actual seven ledger criteria/statuses unchanged by CC, case16 witness and gate/output populations from current arrays; no predetermined invented invocation counts.
6. **Conditional:** sandbox loopback/notify failure retains command/raw channels, followed by unchanged authorized-host retry with original TMPDIR. Missing required file or conflicting source blocks affected work; report for CDC. Runtime mismatch outside frozen boundary stops affected work, no widened inquiry or test relaxation.

## Gates, evidence and return

Run from candidate in this order (gate14 from planning); format then lint before tests. Each attempted command is recorded before launch with argv/cwd/time, inherited RUSTDOCFLAGS/CARGO_TARGET_DIR/CARGO_NET_OFFLINE/RUSTUP_TOOLCHAIN/TMPDIR (observed unset=null), explicit overrides/effective flags/offline config, separate full stdout/stderr base64/byte counts/SHA256/exit and harness selected/passed/failed/ignored/filtered. Retain failures/retries. No canonical TMPDIR replacement, ignored/weakened tests, timeout increase, skipped gate or metadata waiver. Child-only test HOME override remains distinct from command inherited environment.

1. `make format`
2. `make lint`
3. `cargo test -p nms-cache`
4. `cargo test -p nms-copilot --all-features --test route_cache_integration`
5. `cargo test -p nms-copilot --all-features --test mcp_live_integration -- --test-threads=1 --nocapture`
6. `cargo test --all-features -p nms-graph -p nms-watch -p nms-cache -p nms-query -p nms-copilot -p nms-cli`
7. `cargo test -p nms-copilot --no-default-features --lib`
8. `make test`
9. `cargo build --workspace --all-features`
10. `cargo test --workspace --all-features --doc`
11. `cargo doc --no-deps --all-features -p nms-core -p nms-graph -p nms-query -p nms-cache -p nms-watch -p nms-copilot`
12. Actual `RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links' cargo doc --no-deps --all-features -p nms-cache`
13. Candidate `git diff --check`
14. Planning separate `git diff --check`

Accepted dependency observations38 cache/4 route/4 native/744 focused/220 no-default/940 workspace are historical, not targets. Current totals derive actual harnesses and new matrix; existing documentation/dependency warnings retained, no new warning hidden. Observe all sixteen distinct successful startup cases on Unix/http; raw native case receipts and tool/resource observations must identify actual source/output, precondition and file effects. Source fixture and cache-only state inputs remain synthetic, not discovered raw-galaxy evidence.

Commit source with `git commit --only` the two exact named paths directly parent16c0787. Separate planning commit only four return paths; stage new named files first if needed. Both required trailers once on every assistant commit:

Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>

Capture actual parents/pathsets/trailers and all held clean fences. Preserve failed attempts and append superseding counts derived from arrays, not repeated guesses. Record actual close commit in a small follow-up if necessary; never invent its future hash.

Closing report names this assignment/source/issuance/intake/commits, exact sixteen-case inputs/oracles/actual outputs and cache effects, all seven HC rows proposed/open, gate/platform limits, artifact inventory and bubble-up. Capability is real headless cache startup composition proof plus docs; all production frozen. Planned15/CDC retains normal-mode startup and full both-mode live-cache C04/write-through, unknown placeholders/promotion, duplicate producer ownership/settings, broader historical-state questions and final-baseline semantic transitions when raw-field repair lands. Those and all original export/import/format/docs/distribution/coverage/R06-R02/R03/M7.4/C04/parent independent/Operator/integration/release gates remain required; no parent closure from this slice. Concise Operator handoff with project-relative closing report path, source isolated/unintegrated/uninstalled and seven statuses open for independent CDC.
