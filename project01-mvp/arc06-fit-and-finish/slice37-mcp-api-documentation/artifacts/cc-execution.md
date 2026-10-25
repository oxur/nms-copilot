# Slice37 CC execution return

Source worktree: `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/mcp-api-documentation`  
Branch: `codex/mcp-api-documentation`  
Source base: `d553037a94055457995d064e02192e9eed0ecb00`  
Scope: only the four allowlisted MCP Rust source files; no production logic, tests, dependencies, or other paths changed. Four actual refinements were used (initial implementation plus refinement iterations 01–04; one of five refinement slots remains). CDC released the final frozen-source checks and commit step. This is CC proposed-done evidence, not independent acceptance.

## Actual execution chronology

The complete append-only raw-first capture is `validation.json` (23 records, including support controls, all failed attempts, retries, complete raw byte counts/hashes/base64, statuses, and before/after identities). The original `/private/tmp/nms-slice37-cc-attempts.json` was retained.

- Support controls (no product credit): `403b6d23285848dd8ff7625ef159a36b` exited 7; `d0cefd7bfb7042c699afb16d20378843` exited 0.
- Initial `make format` `552569b22c4b48f1af348b18fad172e3` and `make lint` `edd82b1b77a14ffc831b21a5eb7737fd`: exit 0.
- Refinement01 format `d0054c6f0aaa4b5496093a4f5c41e080`: exit 0; lint `647a99997ec84204ab0359fe359e6731`: exit 2. The actual Clippy diagnostic and raw record are retained; subsequent refinement corrected it.
- Refinement02 format `23fa41e2fe604d66981f418fa550eabf` and lint `d0deea962500462cbb08813ff4e9ae54`: exit 0.
- Group 3 all-feature library tests: `cf2fbff407374343885f1e157858012a` recorded 228 passed and 7 watcher publication timeouts (exit 101). The authorized identical host retry `c586ada8600040b3b705df25b2c0cd60` passed 235, failed 0, ignored 0 (exit 0). Both attempts remain in the raw log.
- Initial Group 4 all-feature doctests `a9c4295d069d40ddb8e7c84a9e82535e`: exit 101, 12 passed/1 failed on the Cow name example. Refinement03 format `378ba871c87049a09873154080a45821` and lint `87b8cc0a3fd546188935b608092ae721`: exit 0. Group 4 retry `8779c1c467e84d0bbe46649398d5420b`: 13 passed (exit 0). Group 5 no-default-features doctests `a7f95357fb784a4c94f196844fc4258e`: 9 passed (exit 0).
- Initial Group 6 strict all-feature Cargo JSON `4c18c38a13b44430afdfa732b992354a`: exit 101 with two `rustdoc::redundant_explicit_links` diagnostics. Refinement04 format `84c12b56a90c40d8a5de908b8a5c9c2e` and lint `d203f6b5eb9847aa8e1d1686d9acaa98`: exit 0. The corrected Cargo JSON build `23b61fca8b05484a88e36d5cea70120b` exited 0; the separately captured scoped predicate `a3f0b50d4bea4b24b3ec4ba7db2fb228` exited 0 and reported 0 owned missing-doc warnings and exactly 19 expected residual warnings. This is scoped evidence, not strict/full-crate acceptance.
- Group 7 ordered source-scope/opposed-mutation audit `b249c6c616f748f4958d31deab8c3fe9`: exit 0. Group 8 working diff check `e45ee5bcac274d6c89c722be7fd4519d`: exit 0. Group 9 baseline diff check `a5a8a440d52042f3976b0a41b367f7c2`: exit 0.

CDC independently reviewed and released each refinement hold, including reconstruction of the held source, the E1 line, and the final two-link correction. No post-pair source edit or validator was run outside the issued released sequence.

## Six return proposals

- MP-01: proposed done; all four public MCP source modules received the scoped navigation/payload docs. Independent CDC acceptance remains open.
- MP-02: proposed done; docs distinguish host/owned versus compatibility lifecycle, configuration fallback, logging conversion, and registry/model behavior. Independent CDC acceptance remains open.
- MP-03: proposed done; literal examples and typed negatives compile under all-feature and no-default doctests; scoped missing-doc predicate is 0 owned/19 expected residual. Independent CDC acceptance remains open.
- MP-04: proposed done; four-path scope audit, unchanged executable/example/test regions and named tuple layouts, plus opposed mutation control passed. Independent CDC acceptance remains open.
- MP-05: proposed done; required captured format/lint and nine-group checks are recorded above and in the raw log; exact-path normal-hook commit and clean freeze are recorded in `closing-report.md`. Independent CDC acceptance remains open.
- MP-06: open; independent acceptance, retrospective, full M7.6 and the 38 remaining API owners are not claimed by this CC return.

## Bubble-up and artifact inventory

1. No new product defect or API/lifecycle repair was introduced; the two red documentation checks exposed examples/link syntax that were corrected within the authorized doc-only scope.
2. The sandbox library attempt retained seven watcher-publication timeouts; the same authorized host retry passed, and the restricted 228/7 denominator is preserved.
3. The exact scoped predicate is green while original full strict missing-doc reconciliation and the separately required 38 API owners remain open.

Artifacts: the three ignored return files here (`execution.md`, `validation.json`, `closing-report.md`); original attempt log `/private/tmp/nms-slice37-cc-attempts.json`; prepared source message `/private/tmp/nms-slice37-cc-source-message.txt`. No planning-worktree file was modified by CC.

## Readback and edit attribution

The initial contract was read back against the full immutable prompt, plan, ledger, source allowlist, and provided fixtures before edits. It fixed exactly four MCP source files, whole-line Rustdoc additions plus three named tuple-payload layouts, with all executable/test/non-doc/dependency/other-path content frozen. The source readback distinguished owned host readiness/shutdown and structured errors from compatibility wrappers, configuration-only logging defaults and fallbacks, registry/current-model observations, and the exact documented tool/resource literals. Serving examples remained `no_run`; pure registry/logging examples could execute. CDC acknowledged the source-cited readback and released the first edit group.

The initial docs were followed by these CDC-authored, immutable actual refinements; each response was read back before its scoped edit and held for CDC review:

1. Iteration01 F1–F5: correct registry vector lifetime and promised module example placement; complete logging boolean/Simple timestamp and home fallback behavior; distinguish background from headless exit, error, panic, and notification behavior; describe supplied tool boundaries and actual route-overflow behavior without an unreachable glyph-parser panic; remove inapplicable safe-API boilerplate. Documentation-only changes across the four allowlisted files.
2. Iteration02 R1–R3: prose-only clarifications in `config.rs`, `mod.rs`, and `tools.rs`; `resources.rs`, all code examples in every file, tuple layouts, and all non-doc bytes were frozen. CDC independently reconstructed the held bytes and reviewed all example populations before releasing affected validation.
3. Iteration03 E1: exactly one `tools.rs` example line changed to explicitly retain the returned vector and borrow the Cow tool name: `let names: Vec<&str> = listed_tools.iter().map(|tool| tool.name.as_ref()).collect();`. CDC inverted this line and verified the complete pre-iteration diff identity before release.
4. Iteration04: exactly two prose links in `tools.rs` changed from `[execute_route](nms_query::route::execute_route)` to compiler-supported `[execute_route]`, retaining the same imported target. CDC verified the exact two-link inverse and all other bytes before release.

Across holds, source/index changes were frozen until CDC released the specified next gates. All code-example fence population hashes and the whole `resources.rs` hash were retained during mutation guards. The final source then passed the released remaining groups and was committed as documented in `closing-report.md`.


## Current05 additive wording qualification and correction record — 2026-10-25 UTC

The earlier closing-report sentence saying that no validators were performed after the “final released validation pair” was too broad. After the refinement04 format/lint pair, the original released downstream checks did run: actual Cargo JSON plus the separately captured scoped predicate, followed by scope/opposed-mutation audit and working/baseline diff checks. The precise statement is: no source edit occurred after the final pair, and no validator was run outside the originally authorized released sequence.

The earlier MP-04 phrasing that grouped executable/example/test regions as unchanged from the baseline also needs qualification. Rustdoc prose and examples were authorized additions/changes; they were not asserted unchanged from baseline. The preservation evidence covers the four ordered non-doc comparisons, the three exact existing tuple-payload layouts, tests, dependencies, APIs, and other paths. Example-population freezes apply only to the explicitly held-to-held intervals. Iteration03's one authorized E1 line change is the stated exception for that interval.

A record-writing call immediately before this correction preserved the original prefixes but appended sections with literal backslash-n separators. Its surfaced output was:

```text
closing-report.md prefix 1827 c4305e8556ca89bbf52eea60dacfb05ba9d829ac608da7ddb503032cfa999ef0 total 10035 sha256 a9be02c0018ca05548432413fde91b1dc93effc109500711bb7cd0516853e795
execution.md prefix 7810 c88b96c4e5bce9521ea22c059ca50d44e05b6f4b810e0aaa529371628c439873 total 9189 sha256 0523d1fb592bd2a7af907732df2cb508c3234667c7e8c33f151169171f4331c3
```

The failed append used raw Python strings with escaped `\n` at the separators, which encoded backslash characters instead of Markdown newlines. I verified the exact original prefixes again, replaced only the malformed newly appended suffixes, and re-appended the content with real line breaks. The original prefixes remain byte-exact. This was a return-document repair only; no source, validation JSON, gate, or planning file changed.

Current05 completes the fifth and final actual refinement as record-only. All CC-owned tool calls are complete and source ownership remains frozen with CDC.
