# CC Iteration01 — repair galaxy completion and clarify current evidence

Execute this first refinement of Project01/Arc06/Slice10 and return proposed-done to Sofie/CDC through the Operator. Assignment: `project01-mvp/arc06-fit-and-finish/slice10-galaxy-scoped-repl/cc-prompt-iteration01.md`; predecessor `arc06-fit-and-finish/slice10-galaxy-scoped-repl/cc-prompt.md`. Match the explicit current assignment in slice-plan.md. Existing two-contributor workflow and Expedited Mode continue. This is bounded implementation/evidence correction of F01/F02, not an investigation or a new architecture/acceptance decision.

Five rows RG-01/02/03/04/06 are independently accepted at source0346b193. RG-05 and RG-07 remain open. Preserve all five accepted ledger rows byte-identical. The preceding report's proposed completion does not execute this assignment. Initial prompt and review remain immutable; this numbered prompt supersedes only the active assignment, not the original binding contract.

## Exact boundaries and intake

Use clean `.worktrees/galaxy-scoped-repl`, branch `codex/galaxy-scoped-repl`, exact source `0346b19318326073a06ae921390add50a7e83d98`, parent `e9cabd5fe63f19e9fe2e6da840845315d2ee2d18`. The new source commit must directly parent0346b193. Do not recreate/reset another checkout. Planning is `.worktrees/planning`, branch planning; resolve issuance from the commit introducing this immutable prompt, retain actual intake HEAD and this prompt's SHA256. Pre-review planning was87359b25e6e0578f55f2367a9aebf44495df1117; that is historical intake, not this issuance. Held source pins are in artifacts/cdc-review-receipts.json audit. Keep main6d183bde, cachedecd8d6, identityf57a38f, queries7118a486, routingf71d607 and sessione9cabd5 clean/pinned. Preserve Project02 and unrelated work.

Only source mutation allowed: `crates/nms-copilot/src/completer.rs`, specifically `complete_galaxy_context` and directly relevant galaxy-completion `cfg(test)` code. Preserve all other production bodies, existing other-context tests and every other source path byte-identical to0346b193. Do not change dispatch/query/session/main/commands/prompt/README/parser grammar, source fixtures, dependencies, manifest/lock/version, cache semantics or public APIs. The initial eight-path allowance is narrowed for this refinement. No source integration/merge/push/publish/install or acceptance/closure authority.

CC planning mutation only four existing paths in this slice:

- `artifacts/execution.md` — preserve complete previous bytes as an exact prefix; append intake, readback, repair, gates, limitations and commit receipts.
- `artifacts/validation.json` — preserve every existing JSON value and all53 invocation objects/order; append new iteration data/receipts without rewriting old values. Label any new environment schema separately.
- `closing-report.md` — preserve complete previous bytes as an exact prefix; append this assignment's row accounting/current inventory/correction/evidence/limits/bubble-up and receipts.
- `ledger.md` — only RG-05/RG-07 Evidence and Notes cells may change. Their statuses remain open; all other cells and five accepted rows stay byte-identical.

Immutable: both issued prompts, slice/parent plans and ledgers except those cells, author baseline, CDC verification/receipts/witnesses, all earlier slice records. Commit only changed named allowed paths. If a required fix exceeds this scope or actual behavior contradicts the binding expectation, stop affected work and report to CDC without weakening tests. No unilateral evidence waiver; new investigation requires prior Operator agreement on usefulness/applicability/budget.

Before editing record assignment/issuance/prompt digest/source branch/parent/planning/pins/clean state, compare all seven current source digests in the CDC receipt (including the permitted file), record four return-file identity hashes and five complete accepted ledger rows, and retain per-item loaded extents and source-cited readback. Original e9cabd5 author-baseline digests remain historical, not replacements for0346b193 observations. Readback explains how trailing space belongs to a partly typed galaxy value, how actual span reconstruction tests it, what prior evidence remains applicable and what environment values are unknown. Coherent intake permits execution without another confirmation.

## Required reading manifest

Skill root: `/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills` or verified installed equivalent. Record actual paths/version/extents. Required-full means complete text; recover truncation in bounded ranges rather than substituting hashes/headings/summaries.

1. Required-full: applicable main/candidate AGENTS.md; this prompt; immutable initial cc-prompt.md; current slice-plan.md, ledger.md and cdc-verification.md. Required-data: cdc-review-receipts.json source/parent/planning identities, audit pins/source pathset, protected_source_sha256, row_dispositions, findings, cc_evidence_inventory and probe source hashes; gate/probe raw channels are reference-only unless needed for the disputed claim. Required-section: parent arc-plan/project-plan latest Slice10 review/refinement entry and unchanged later-owner limits. Original M7.4 remains governed by the initial prompt; no new scope.
2. Required-section: collaboration-framework SKILL.md Notes for Codex/When to use; engineering-methods/guides/01-engineering-methodology.md Roles and Shared Invariants/Two-Contributor Workflow/Workflow Selection and Transitions. Required-full: project-management/guides/README.md. Required-section: project-management/guides/02-canonical-planning-worktree.md Slice iteration filenames and preservation, guides/03-planning-top-down.md Issuing and executing an iteration and Agree investigations before starting; engineering-methods/guides/07-implementation-prompt-authoring.md Corrective iterations. Required-full work-verification/guides/03-row-closure.md; required-section guides/05-independent-verification.md Separation Rule/Assigned Reviewer Slice Verification/Evidence During Correction Review.
3. Required-full before Rust edits: rust-guidelines/SKILL.md and guides/11-anti-patterns.md as repository requires. Required-full testing/guides/01-testing-discipline.md; required-section testing/guides/03-validation-gates.md Gate Selection/Proportionate Proof/Minimum Validation Set/Warning Lint Format/Gate Evidence. Apply native behavior proof, no new verification framework.
4. Required-full candidate completer.rs. Required-section commands.rs imports/Action::Galaxy/parse_line/shell_words and current galaxy parser tests; core galaxy.rs Galaxy::by_index and names used below; pinned reedline Suggestion/Span semantics as used by existing completer. Required-full `artifacts/cdc-completion-probe.rs`; required-data its compile/run output in CDC receipt. Required-section current Makefile format/lint/test and copilot Cargo.toml feature/test configuration. Other source/reference probe is retained independent proof, not a new implementation task.
5. Required-section existing execution/report last complete gate summary/attempt/inventory/commit sections; required-data validation.json root shape,53 original invocations' identity/order/environment fields, final_gate_summary and commits. Ordinary raw base64 fields are data, not mandatory narrative reading. Conditional: if currently retained evidence defines old effective_environment schema, read that precise definition; if none is available, disclose unknown and stop that recovery claim. Do not start historical trace archaeology or substitute a fresh rerun for original metadata.

## F01 / RG-05 — resolved cause and prescribed repair

The public completer currently resets partial to empty on any line-ending whitespace (`completer.rs:192` at pin). It then derives replacement length from that reset prefix. `galaxy Hilbert ` gets span15..15 and reconstructs `galaxy Hilbert "Hilbert Dimension"`; `galaxy "Hilbert ` gets span15..16 and reconstructs malformed quotes. Both fail parse_line. Bare `galaxy `, unfinished quoted `galaxy "Hilbert D` and `galaxy Ei` are valid controls. CDC observation harness exited0 to record outcomes; two of five endpoint observations failed the contract.

Inside the existing helper, keep the complete value tail after command whitespace with leading whitespace trimmed only. Its trailing spaces are part of prefix matching. Compute replacement start from that raw tail's full byte extent before stripping the optional opening quote: `pos.saturating_sub(value_tail.len())` fits existing defensive style. Remove at most the optional initial double quote to derive matching partial. Remove the unconditional line-ending-whitespace reset. Reuse unchanged `complete_names_at`, reference names0..255, case-insensitive prefix matching, owned/quoted suggestions and existing cap20. Bare command separators naturally yield an empty value tail and empty prefix. Do not redesign other completion contexts or parse syntax, completed-quote handling, mid-token grammar, tabs/Unicode tokenization or raw-address support.

Regression tests must invoke the actual public Completer, find the expected suggestion and assert its exact value and `Span::new(expected_start,line.len())`, reconstruct the entire line using `line[..span.start] + suggestion.value + line[span.end..]`, assert the literal reconstructed command, then call real parse_line on that line and assert exact Action::Galaxy selection. Parsing only `format!("galaxy {}",suggestion.value)` misses this bug and is insufficient. Retain existing useful prefix/other-context tests.

Required literal matrix (end is the actual byte length of input):

| Input | Start | Expected reconstructed command | Exact selection |
|---|---|---|---|
| `galaxy ` |7| `galaxy "Hilbert Dimension"` | Hilbert Dimension |
| `galaxy   ` |9| `galaxy   "Hilbert Dimension"` | Hilbert Dimension |
| `galaxy Hilbert ` |7| `galaxy "Hilbert Dimension"` | Hilbert Dimension |
| `galaxy "Hilbert ` |7| `galaxy "Hilbert Dimension"` | Hilbert Dimension |
| `galaxy Hilbert D` |7| `galaxy "Hilbert Dimension"` | Hilbert Dimension |
| `galaxy "Hilbert D` |7| `galaxy "Hilbert Dimension"` | Hilbert Dimension |
| `galaxy "hIlBeRt ` |7| `galaxy "Hilbert Dimension"` | Hilbert Dimension |
| `galaxy Ei` |7| `galaxy Eissentam` | Eissentam |

Negative/discrimination checks: `galaxy NotARealGalaxy ` returns no suggestions; known Hilbert-space prefix suggestions exclude Euclid and match that prefix; bare `galaxy ` preserves the existing20-result cap. The two reproduced failures must be rejected by tests at0346b193 and pass after the prescribed repair; record their causal baseline/final observation without mutating a held checkout or inventing a historical test invocation. Local table-test factoring/names are CC discretion. Existing reference data and actual full reconstruction supply the oracle; no implementation-mirroring assertion alone.

## F02 / RG-07 — current inventory and honest environment semantics

Append correction to retained execution/report: original current validation has53 invocation objects/106 channels; its last complete final ordered run is 1-based41–51 inclusive (11 subprocesses,10 gate categories).52–53 are later separate diff receipts. Earlier “last ten invocation entries” and51-item inventory do not identify the complete current run. Preserve historical text and all existing JSON values; name explicit indices or stable receipt identifiers for each new final gate, with fresh counts rather than predecessor targets.

From already retained provenance only, clarify whether original `effective_environment: {}` records observed-unset variables or merely no overrides/unobserved fields, and state the actual supporting source. Strict entries18/29/38/49 contain the RUSTDOCFLAGS override. If old scope cannot be established, preserve old fields as written, explicitly label the historical observation unknown and report that limit to CDC; no retroactive nulls or reconstruction from today's environment. No broad archaeology/new inquiry is authorized or required. CDC/Operator will disposition any residual gap; CC does not waive it.

For every new validation attempt, before launching the child capture exact argv/cwd and actual effective relevant environment: RUSTDOCFLAGS, CARGO_TARGET_DIR, CARGO_NET_OFFLINE and RUSTUP_TOOLCHAIN, with null explicitly meaning observed unset. Record any per-command overrides and actual offline config/flags affecting the command; do not describe inherited variables as unset without observing them. Keep complete separate stdout/stderr bytes/base64/length/SHA256, actual exit, failures/retries and pre-child sandbox denials with honest capture limits. Correct strict flag must actually reach the child. These records prove current invocations only.

## Fresh validation and evidence retention

After final edits run from candidate in this order (format then lint before tests):

1. `make format`
2. `make lint`
3. `cargo test --all-features -p nms-copilot --lib test_complete_galaxy` (use matching names so every new galaxy regression runs; record selected tests/count, no zero-selected success).
4. `cargo test --all-features -p nms-graph -p nms-watch -p nms-cache -p nms-query -p nms-copilot -p nms-cli`
5. `cargo test -p nms-copilot --no-default-features --lib`
6. `make test`
7. `cargo build --workspace --all-features`
8. `cargo test --workspace --all-features --doc`
9. `cargo doc --no-deps --all-features -p nms-core -p nms-graph -p nms-query -p nms-cache -p nms-watch -p nms-copilot`
10. `RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links' cargo doc --no-deps --all-features -p nms-cache`
11. `git diff --check` separately on candidate and planning.

This is the original ten-category gate contract plus a focused regression command, not a new maintained test tool. No need to rebuild CDC's40-case reference probe. Preserve accepted reference proof only with exact scoped diff, unchanged protected source and fresh native suites; CDC makes retention decision. Report all actual passed/failed/ignored counts/harnesses, including zero doctests. Prior724/215/920 are observations, not targets to manufacture. Existing ordinary docs/HTTP-disabled import/proc-macro-error2 warnings stay disclosed and parent strict combined docs stays open; do not suppress warnings or update dependencies. No broad coverage/benchmark/transport claim. Retain failed attempts before successful retry; use authorized unchanged Cargo escalation if required.

## Exact commits and return

Source commit only `crates/nms-copilot/src/completer.rs`, direct parent0346b193. Planning commit only the four return files named above, with receipt followups limited to them. Use `git commit --only` with exact named paths; stage only named newly created files if any (none required). Each assistant-authored commit must contain each trailer exactly once:

```
Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>
```

Append closing report explicitly naming both assignments/iteration, issuance and actual intake/source/parents/pathsets/trailers, F01 matrix results/F02 provenance or residual unknown, exact current gates/artifact inventory, commit receipts and RG-05/07 proposed dispositions. Five other rows remain accepted/unchanged, not re-proposed. All current source and held worktrees must be clean; preserve receipt attempts/history and source fences. No synthetic CDC/CRC review/retrospective artifact and no cross-chat messaging.

Bubble-up: usable completion corrected in a wired candidate, not main-integrated or installed. Slice10/arc/project remain open pending independent CDC; Slice11 actual watch/MCP transports/subscriptions/startup/cache/unknown-system/historical-state disposition, later raw-field repair and every original parent gate/owner remain. Return concise report and project-relative close report path for Operator relay. This is first refinement within the existing five-pass budget; no automatic extension.
