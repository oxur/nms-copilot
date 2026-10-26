# CC assignment — Arc07 Slice09 glyph error preview

You are the sole implementer, GPT6 Luna High. CDC is existing GPT6.1 Sol High and independently reviews. Root transports status only. This one Operator-requested correction is authorized; do not resume other cleanup, the exhausted07 budget, old49 or new scope. Five actual corrective refinements maximum; initial assignment is not a reset of another slice.

## Intake / ownership

Read this prompt and slice-plan.md/ledger.md fully. Source cwd MUST be /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/workspace-coverage-contract, branch codex/workspace-coverage-contract, clean ae0ef360389f555e5ec0d39930005cbfc374faad. Recheck and report drift before writes. Source/index assigned to you; planning/index CDC-owned. No main/ref/rebase/merge/push/CI/install/network/dependency changes.

Required source sections at that baseline: glyph.rs parse_next_glyph and its complete tests; address.rs PortalAddress::parse_mixed, its FromStr impl, GalacticAddress::from_portal_string and immediate docs; nms-cli/src/convert.rs run/parse_glyphs and docs; nms-cli/src/main.rs Convert dispatch; Makefile format/lint/test targets. Query existing CLI test assertion style only as needed, not full historical intake. Required domain sections: `/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/rust-guidelines/guides/11-anti-patterns.md` AP-09 and AP-25; `03-error-handling.md` EH-01/02; `/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/testing/guides/01-testing-discipline.md` and `03-validation-gates.md` Proportionate Proof and execution-population sections. Recover any truncated required read. In closing-report.md retain a brief loaded-extents/contract readback before dependent edits (or start that file once intake actually performed). Historical arcs/reports are reference-only; don't reread49.

## Binding behavior and recipe

Runtime changes ONLY in crates/nms-core/src/glyph.rs: the unrecognized-prefix error preview and source-boundary mapping for successful lowercase name/abbreviation matches. Replace unsafe `&input[..input.len().min(20)]` with a min(len,20) byte boundary backed down until is_char_boundary, then slice. Keep prefix `unrecognized glyph at: ` and the existing GlyphParseError/public API. Do NOT use chars().take(20) (20 characters violates byte cap), lossy conversion, ellipsis, catch_unwind or new dependencies. Retain existing input.to_lowercase() predicates/order and alias policy. For a matched lowercase prefix, map its byte length back to an original-input character boundary: iterate original char_indices, accumulate each character.to_lowercase() UTF-8 byte length, return the original end boundary when the matched lowercase length is consumed. Reuse one small private helper for name and abbreviation branches; slice original input at that mapped end. All table spellings are ASCII and none ends in i; a successful current table prefix cannot end inside the expanding lowercase of İ. Preserve Kelvin-sign matching and consume its ORIGINAL bytes correctly, with no new public API or lowercase policy. The byte boundary loop terminates at0 and needs at most3 decrements on valid str.

Allowlisted source paths, exact commit set:
- crates/nms-core/src/glyph.rs: repair plus literal inline regressions and obsolete20-byte-panic docs.
- crates/nms-core/src/address.rs: remove/update ONLY three propagated20-byte diagnostic panic doc passages, no executable change.
- crates/nms-cli/src/convert.rs: remove ONLY inherited20-byte panic claim, preserve stdout-failure claim and all executable bytes.
- crates/nms-cli/tests/glyph_error_integration.rs: small direct real CLI convert regression using existing assert_cmd dev dependency, no save required.

The same-function case-folding/original-byte slicing concern is explicitly included: `rcKt` lowercases to rckt but originalbyte4 splits K; `RocKet` consumes the wrong suffix with fixed ASCII offsets. Correct these using the mapped original boundary, not ASCII-only matching. LZ4 and warp helper risks remain undispositioned and untouched.

## Literal tests / sequence

Add FOUR separate baseline-red unit tests before runtime repair:
1. unknown `xxxxxxxxxxxxxxxxxxx界suffix` (19 ASCII x then3-byte界): exact error payload `unrecognized glyph at: xxxxxxxxxxxxxxxxxxx` (19x). Byte20 splits界.
2. unknown `xxxxxxxxxxxxxxxxxx🧪suffix` (18 ASCII x then4-byte🧪): exact payload `unrecognized glyph at: xxxxxxxxxxxxxxxxxx` (18x). Byte20 splits🧪.
3. `rcKttail` -> glyph13, suffix `tail` (baseline panics).
4. `RocKettail` -> glyph13, suffix `tail` (baseline returns wrong suffix).
Use literal expected strings; no catch_unwind. Run format then lint before the baseline filtered tests; preserve both actual panic failures. Additional exact boundaries: unknown ASCII longer than20 -> first20; exactly20 -> unchanged; short unknown `xyz` -> whole; short Unicode `界` -> whole; empty/colon-only existing reason unchanged. Include colon-prefixed split case (preview applies after separator).

Positive literal `(index, suffix)` checks: `🌅tail` -> (0,tail); `🕊️tail` -> (1,tail); `Dragonflytail` -> (8,tail); `rCkTtail` -> (13,tail); `:rckttail` -> (13,tail); `Ftail` -> (15,tail). Preserve existing tests. These assert public behavior, not helper implementation.

After repair/doc corrections: `CARGO_NET_OFFLINE=true make format`, then `CARGO_NET_OFFLINE=true make lint`, then `CARGO_NET_OFFLINE=true cargo test --offline --locked -p nms-core`. Direct CLI regression invokes `nms convert --glyphs <3/4-byte unknown input>` through cargo_bin_cmd; expect ordinary error exit1, empty stdout, inherited diagnostic with bounded literal prefix, and NO panic text. Use isolated scratch HOME/config/cache if relevant; never modify actual user environment. Run `CARGO_NET_OFFLINE=true cargo test --offline --locked -p nms-cli --test glyph_error_integration`. Preserve actual commands/counts/exits and failures. If an unrelated gate fails report it; no opportunistic repair.

CDC owns ONE final native all-feature workspace make test after you stop all processes; do not duplicate that global run. No coverage measurement/hardening/new validation tooling. No concurrent test load.

## Commit and return

Self-review exact diff/scope. Commit ONLY the four named source paths (omit any path if truly unchanged and explain) with git commit --only; both required trailers ADJACENT in one footer paragraph:
Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>
Verify actual git interpret-trailers --parse recognizes EACH exactly once; check clean source and report full SHA. No amend/rebase of history.

Write concise closing-report.md in this issued slice root (planning path), not a new report hierarchy: intake/readback, exact changed behavior/paths, baseline reds, actual checked commands/exits/populations, GP-row proposals, case-folding correction and other held gates. Useful raw command logs may go only under this slice artifacts/; no fabricated metadata. Do not mutate/commit planning ledger/plan/index, CDC will disposition. Return full source SHA/report path and STOP all work/processes.
