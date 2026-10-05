# CC Slice05 iteration01 — finish regression and compatibility documentation

Execute this **CC refinement implementation**, then return through the Operator to CDC/Sofie. Current assignment: `arc06-fit-and-finish/slice05-cache-semantic-compatibility/cc-prompt-iteration01.md`, relative to Project01. Predecessor: sibling cc-prompt.md, preserved. The slice remains open; this is not a readback-only assignment. Two contributors/Expedited Mode remain; existing scope approval is sufficient.

## Candidate and current contract

Reuse `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/cache-semantic-compatibility`, branch `codex/cache-semantic-compatibility`, clean at `8d2cf00fba41689f817cad35ec835315ba9aa415`. Do not reset, create another worktree or repin silently. Main remains clean at `a800950ef3558fcf463517696c3c140061e7353d`. Planning is `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/planning`, branch planning; CC packet baseline `4f6ca66acd39bba522c3684c4de9e498c9cbf186`. Record the actual reviewer handoff commit containing this iteration as intake HEAD; it supersedes that packet state for planning authority.

CS-01/02/05/07 are independently accepted at the current candidate. CS-03/04/06 remain open only for CDC R01–R03. Runtime rejection/rebuild, empty-input behavior and explicit fresh schema/semantic lifecycle all independently pass; improve the specified durable regression assertions and docs, keeping production behavior unchanged. The binding envelope/error/lifecycle contract in slice-plan.md is unchanged. Retain initial failed histories, accepted evidence and all issued prompts. Source mismatch or a needed runtime change goes to CDC before affected work.

## Required reading and preflight

Path aliases: S=this slice's absolute planning directory; P=its Project01 directory; R=the absolute source worktree above; K=`/Users/oubiwann/.codex/skills`. They are not shell variables to replace system configuration.

| Order | Exact path/state | Load extent and purpose |
| --- | --- | --- |
|1 | S/cc-prompt-iteration01.md, slice-plan.md, ledger.md, cdc-verification.md at reviewer handoff | Required-full; active assignment, binding contract, all seven dispositions and R01–R03 |
|2 | S/cc-prompt.md sections Implementation spine, Behavioral oracles, Exact permitted paths and commit/return contract | Required-section; initial obligations/validation, subject to the narrower iteration allowlists and preservation instructions here |
|3 | P/project-plan.md §13–14; P/arc06-fit-and-finish/arc-plan.md §Approved scope and current execution and §Slice05 initial CDC review and current refinement | Required-section; approved boundaries, remaining parent gates and ordering |
|4 | Repository AGENTS.md/user instructions; K/project-management/guides/README.md; K/work-verification/guides/05-independent-verification.md §Evidence During Correction Review; K/engineering-methods/guides/07-implementation-prompt-authoring.md §Required reading and CC intake | Required governing instructions, full README and complete named sections; cadence, retention and intake |
|5 | K/rust-guidelines/SKILL.md Overview/selection/documentation workflow; K/rust-guidelines/guides/13-documentation.md DC01–02; K/testing/guides/01-testing-discipline.md and 03-validation-gates.md §Proportionate Proof | Required-section except full testing discipline; public contract docs and meaningful preconditions. Preserve additional applicable governing obligations |
|6 | R/crates/nms-cache/src/lib.rs and serialize.rs; R/crates/nms-cache/tests/load_integration.rs at 8d2cf00 | Required-full; complete changed files including all tests and helper callers |
|7 | R/crates/nms-cache/src/error.rs; R/crates/nms-cache/src/freshness.rs load_or_rebuild/is_cache_fresh; R/Makefile format/lint/test recipes | Full error.rs, complete named bodies/recipes; document actual public errors/fallback and reuse gates |
|8 | S/artifacts/validation.json baseline at 4f6ca66 | Required-data: all 30 initial run objects, projection key/argv/cwd/exit/channel byte counts and summary counts; final_gate_keys/order. Inspect complete projection, record denominator/exits in execution.md. Preserve every pre-existing JSON value on append, including full streams; no historical run may be replaced |
|9 | S/artifacts/cdc-review-receipts.json, cdc-probe.rs; prior execution.md/closing-report.md | Reference-only for fresh proof/history; actual preservation checks below still apply to existing CC files |

Actually load required text and data-query results before dependent edits, with bounded contiguous reads and explicit recovery of every truncation. File existence/hashes/headings/log redirection are not reading. Append source states/loaded extents/query coverage and concise source-cited readback to execution.md: especially why malformed payload versus empty envelope differs, why cache freshness is a tested precondition, and how the new enum variant affects external callers. Record unavailable/conflicting inputs and stop affected work; no extra permission request once coherent. Revalidate state/contracts after context recovery.

## Exact repairs — binding

### R01 / CS-03: empty envelope, not empty payload

In serialize.rs::test_deserialize_compatibility_error_precedence, change the size population from `[7, 8, 15]` to `[0, 7, 8, 15]`. The existing same predicate must assert Incompatible(TruncatedHeader{actual:size}) for all four. Keep all current magic/schema/semantic/priority and current-header-bad-payload tests unchanged. The current-header-empty-payload case produces Deserialize and cannot substitute for length 0. No new production branch, test special case, framework or reusable validator.

### R02 / CS-04: assert freshness where it controls the path

In tests/load_integration.rs::set_fresh, after both existing deterministic set_mtime calls, add `assert!(is_cache_fresh(cache_path, save_path));` (a helpful failure message is local discretion). Do not alter times, paths, cache versions or data. All existing helper callers must then assert their precondition before the next load/action. This includes legacy/schema/semantic/corrupt/source-error/no-cache probes and the warm stage of assert_rebuilt_and_warm. Retain the stale test's false assertion and standalone freshness test. Inspect the complete caller set rather than patching just one test. No production freshness or fallback changes.

### R03 / CS-06: public compatibility and continuing process

Use public crate docs in lib.rs and module/function docs in serialize.rs; precise prose/local layout is discretionary, required content is binding:

- Header is 16 bytes: ASCII NMSCACHE at 0–7; schema u32 LE at 8–11; semantic u32 LE at 12–15; checked rkyv payload starts at 16.
- Both revisions must exactly equal the reader's current values; no accepting larger/newer revisions. Legacy headerless or mismatched caches are rejected; load_or_rebuild reparses the source and writes the current envelope. Direct deserialize/read_cache return the typed error rather than rebuilding by themselves. Do not imply read-only freshness checks perform semantic validation.
- Schema revision tracks archive layout/codec/configuration compatibility; semantic revision tracks the meaning of decoded/model data. Later address decoder/galaxy identity changes must assess and bump semantic revision when meaning changes even if archive shape is unchanged. Both revisions may need to change for one repair; game SaveRoot.version, package version and cached_at are different values.
- Initial schema 1/semantic 1 labels current behavior; it does not fix the known address/identity defects or prove game format authority. Later semantic repair must reject its predecessor, rebuild from source and witness the current warm hit.
- CacheError gained Incompatible; external callers exhaustively matching that public enum may need updates. The new nested CacheCompatibilityError is non_exhaustive. Public function signatures remain unchanged, but serialized bytes/error matching have compatibility consequences. No release/version bump or universal compatibility claim.

Replace the crate's misleading zero-copy/startup-speed implication with a factual description of the current checked owned/aligned decode and reconstructed indexes; no benchmark promise or performance redesign. Keep documentation proportionate. Crate-level crosslinks can point to the public cache/error APIs; verify them with the bounded rustdoc command below. Add appropriate Errors explanations to changed serialize/deserialize docs using the existing error variants, not new ones.

**No runtime function bodies, types, signatures, error enum, dependencies, CacheData, revision constants or header values may change.** Two guarded try_into().unwrap calls were not a CDC finding; leave them. Tests change only the zero case and freshness assertion described above. No decoder/SystemId/active-galaxy repair or extra feature enters this iteration.

## Applied guidance and tests

CDC read the actual candidate bodies/docs/helper caller set and fresh source-flow proof before issuing. DC01 MUST and DC02 MUST put the public byte/error/process contract in actual rustdoc rather than only a CC report. Testing's behavioral/precondition guidance maps the empty envelope to exact error data and observes the mtime branch before action. Proportionate Proof uses the existing loop/helper and native gates; no new tooling product. Public constants/semantics and all prior accepted runtime behavior stay fixed.

| Repair | Observable check | Wrong result rejected |
| --- | --- | --- |
| R01 | Existing loop includes0/7/8/15, exact truncated actual values; current 16-byte-header-empty-payload still Deserialize | Confusing envelope absence with payload corruption or omitting empty input |
| R02 | Existing helper asserts freshness after deterministic writes, and all its callers still pass; poisoned9999 returns source 4720 then warm true | Source fallback passing a claimed incompatibility test merely because cache timestamps are stale |
| R03 | Public docs cover all bullets and correctly attribute direct-reader vs startup behavior; crate rustdoc links build successfully | Incomplete process, overstated compatibility/zero-copy/performance or reinterpreting old decoded bytes |

After edits run, from R, make format then make lint before tests (CARGO_NET_OFFLINE=true for make targets), `cargo test --offline --locked -p nms-cache`, make test, `cargo build --offline --locked --workspace --all-features`, `cargo test --offline --locked -p nms-cache --doc`, plus `RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links' cargo doc --offline --locked -p nms-cache --no-deps`. This is crate-only documentation validation, not repair/acceptance of the known failing workspace strict-doc gate. No new doctest is required solely to inflate a zero target. Expected existing test counts remain 18 unit + 13 integration and 824 workspace if you follow the existing-loop/helper recipe. Derive/report actual counts; unexplained divergence is a finding, not permission to rewrite expectations. Capture all failed/blocked/unrun attempts and exact raw channels; fix only in-scope issues and rerun affected checks. No ignored/weakened assertions. Do not repeat other broad known failing gates without a changed concern; their original requirements remain open.

## Evidence preservation and exact paths

Source modifications/commit only:

1. crates/nms-cache/src/lib.rs — docs only
2. crates/nms-cache/src/serialize.rs — docs and R01 loop only
3. crates/nms-cache/tests/load_integration.rs — R02 helper assertion only

Planning modifications/commit only, under `project01-mvp/arc06-fit-and-finish/slice05-cache-semantic-compatibility/`:

1. artifacts/execution.md — append iteration01 intake/results, preserve existing byte prefix
2. artifacts/validation.json — add a new iteration01 object with fresh commands/streams/summaries/new source commit and preserve all pre-existing values, including the initial 30 runs; old top-level source_commit remains historical
3. closing-report.md — append iteration01 seven-row walk/results/bubble-up, preserve existing byte prefix
4. ledger.md — Evidence/Notes for CS-03/04/06 only; all criteria/statuses and four accepted rows unchanged

Do not modify reviewer records/probes/receipts, author inputs, slice plan, parent plans/ledgers, issued prompts, source consumers or other files. Temporary logs/scripts may live in /private/tmp; no user game/cache/config paths. No data re-decoding or new semantic policy. If source/contract drift or a need for a different API/body/path is found, report to CDC before affected edits; do independent authorized work meanwhile.

Retain the current 31/824 runtime/gate evidence and failures at 8d2cf00 as historical reproduced evidence. New docs/test changes require fresh affected checks; CDC will decide retention applicability on the actual diff. Never relabel old runs as fresh. Append-only report sections should clearly identify iteration01 and supersede the earlier proposed completion for these remaining rows, without deleting it.

Commit exact three source paths on the existing source branch and exact four planning paths on planning with explicit `git commit --only -- <paths>`, preserving unrelated staged work. Each assistant-authored commit has these contiguous trailers exactly once:

    Co-authored-by: Codex <noreply@openai.com>
    Co-authored-by: Billo AI <ai-engineering@billo.systems>

No merge, push, publication, amend or main movement. Validate whitespace/JSON, exact commit pathsets/trailers, unchanged initial streams/values/report prefixes/criteria and clean fences. Report actual two complete SHAs and changed paths. CC completion stays proposed-done; accepted rows remain attributed to their original CDC evidence, unresolved rows remain open pending correction review. Slice05/Arc06/Project01 remain open. No scope amendment or new Operator approval is needed for these original obligations.
