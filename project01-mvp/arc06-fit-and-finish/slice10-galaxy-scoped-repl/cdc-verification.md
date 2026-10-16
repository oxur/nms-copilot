# Slice10 independent CDC review — 2026-10-16

## Scope, role and verdict

Sofie/CDC independently reviewed CC source `0346b19318326073a06ae921390add50a7e83d98`, direct parent accepted09 `e9cabd5fe63f19e9fe2e6da840845315d2ee2d18`, against the immutable initial `cc-prompt.md`. Planning intake `87359b25e6e0578f55f2367a9aebf44495df1117`; CC return95c4913 plus receipt87359b2. No implementation was authored or changed by this reviewer. Framework2.7.2, existing two contributors and Expedited Mode apply; no three-contributor retrospective or new investigation was commissioned.

Accept RG-01/02/03/04/06. RG-05 and RG-07 remain open. A reproduced completion defect requires one bounded correction; passing all suite gates does not establish its missing case. Slice10, Arc06 and Project01 remain open. This is refinement01 of the existing five-refinement budget.

## Independently reproduced evidence

All ten ordered gate categories (eleven subprocesses including separate diff checks) passed at the exact source: make format; make lint; focused all-feature graph/watch/cache/query/copilot/CLI tests; copilot no-default library tests; make test; workspace all-feature build; workspace doctests; six-package docs; strict cache docs; source and planning diff checks. Focused724/0failed/0ignored across25 harnesses; no-default215/0/0 in1; workspace920/0/0 across33; doctests9 harnesses/zero doctests. Strict cache docs used actual `RUSTDOCFLAGS=-D rustdoc::broken_intra_doc_links`. Full separate channel bytes, SHA256, argv/cwd, relevant environment including unset nulls and the runner source are retained in `artifacts/cdc-review-receipts.json`. Existing ordinary documentation warnings, HTTP-disabled unused import and proc-macro-error2 advisory persist; the parent strict combined-docs gate is still open.

Source inspection covers all changed production bodies, native new tests and retained caller paths. Actual main caller holds a bounded shared write guard only around dispatch. Selection validates before mutation; explicit Base/Address versus follow-player mapping agrees with the query layer. Source has exactly seven initial allowed paths, correct parent and both trailers once. Eight author-baseline digests, immutable prompt/plan/author input and CC planning pathsets/parents/trailers checked. Accepted session synchronization/movement tests and runtime prefix remain unchanged. README changes stay inside Interactive REPL. All held source/dependency worktrees are clean and pinned; audit identities are in the receipt.

The compiled public reference probe supplies stronger equivalent independent proof for the reference/error/state cases not all explicitly enumerated in CC's native test bodies:40/40 checks, reverse insertion on/off and nn/2opt. It executes actual parser/dispatcher calls for explicit Address, --from override, deliberately stale PlayerPosition, unknown base/target, no-result and absent-player cases; complete player/systems/planets/bases/active selection/session values are checked unchanged after every query. Literal400/800ly and Leaf-0/Leaf-9 outcomes discriminate galaxy collisions. Its source and compile/run channels are preserved. This is bounded endpoint verification, not a new reusable testing framework. Probe inherited environment was not separately snapshotted; that limitation is explicit and does not replace captured native gates.

## F01 — RG-05 completion loses a multi-word prefix ending in space

At `crates/nms-copilot/src/completer.rs:192`, `complete_galaxy_context` resets the prefix to empty when the current line ends with whitespace; deriving span length from that empty prefix loses the whole typed value. Applying the actual public reedline suggestion span reproduces:

| Input at cursor | Actual span | Actual reconstructed command | Parser result |
|---|---|---|---|
| `galaxy ` |7..7| `galaxy "Hilbert Dimension"` | correct |
| `galaxy Hilbert ` |15..15| `galaxy Hilbert "Hilbert Dimension"` | error: extra arguments |
| `galaxy "Hilbert ` |15..16| `galaxy "Hilbert"Hilbert Dimension"` | error: malformed quotes |
| `galaxy "Hilbert D` |7..17| `galaxy "Hilbert Dimension"` | correct |
| `galaxy Ei` |7..9| `galaxy Eissentam` | correct |

The observation probe compiled and exited0, but two of five endpoint observations violate the contract. Existing parser test constructs `format!("galaxy {}", suggestion.value)` instead of applying the suggestion span, so it cannot reject the broken reconstruction. This is a production completion defect, not a query/identity/reference bug. Preserve complete leading-trimmed raw value tail including trailing spaces; compute start from its byte length before removing optional opening quote for matching. Reuse unchanged name suggestion helper/cap/quoting. Regression tests must apply actual span and parse the reconstructed command. Exact repair/matrix is in iteration01.

## F02 — RG-07 current inventory/environment clarity and corrected-source evidence

All106 original CC channel objects across53 invocations validate their retained base64 bytes/length/SHA256. The actual final complete ordered run occupies 1-based invocations41–51 (11 subprocesses/10 categories);52–53 are later diff receipts. The retained execution narrative's “last ten” pointer and closing inventory51 need appended corrections to current indices/counts. Original early failures and all values must remain intact.

Non-strict `effective_environment` values are `{}`; strict entries include the RUSTDOCFLAGS override. This alone cannot distinguish a schema recording known-unset values from overrides-only/unobserved fields. Clarify its meaning from already retained provenance, if available. If unavailable, explicitly preserve historical uncertainty; neither fresh successful commands nor today's environment recovers past values. No extended archaeology, fabricated capture or unilateral evidence exception is authorized. Fresh correction commands must snapshot actual relevant environment with null for unset and label capture scope. RG-07 stays open for this clarification and exact corrected-source gates/intake/return evidence. CDC will disposition any residual historical gap on its actual facts.

## Row disposition and evidence retention

| Row | CDC disposition | Independent basis |
|---|---|---|
|RG-01|done| Native optional parser/list/select/bounds/names/empty model and atomic invalid input; reviewed resolver-before-mutation. |
|RG-02|done| Native full player/anchor/filter/warp and cross/same movement proof, retained accepted09 synchronization contract and current suites. |
|RG-03|done| Native find semantics plus40-case public reference/error/state probe including typed Address/stale follow/absent player/unknown base/no-result/reverse insertion. |
|RG-04|done| Native route mismatch/origin/distance and40-case public probe, both algorithms/insertion orders and error/override/full-state controls. |
|RG-05|open| Caller/status/prompt/help/docs proof retained, but F01 violates full value-span completion contract. |
|RG-06|done| Existing useful commands/assertions preserved; current native gates, feature configurations and seven-path source fence. |
|RG-07|open| Current-source scope/gates largely reproduced; F02 and corrected-source return evidence outstanding. |

Five accepted rows remain byte-identical during CC correction. Their proof applies at0346b193 and is retained only if the next diff remains confined to the specified completion body and directly relevant tests, all protected source bodies stay unchanged and required fresh suites pass. Review uncertain consequences rather than inferring acceptance from CC's claim. Fixing completion does not require repeating the reference investigation.

## Bubble-up, assignment and artifact inventory

Current CC assignment: `arc06-fit-and-finish/slice10-galaxy-scoped-repl/cc-prompt-iteration01.md`; initial prompt returned/reviewed and superseded as active, preserved unchanged. This routine refinement changes neither scope, acceptance, architecture nor sequence. Arc childR06-S10 remains open:28 arc rows/nine done/nineteen open; all25 project rows open. Candidate remains0346b193 and main6d183bde; no integration/release/installation claim. CDC/Operator own later authorized integration/final-baseline drift/composition. Slice11 live watcher/transports/subscriptions/startup/cache/unknown-state, later raw-field repair and every original parent gate/owner remain.

Review artifacts: this verification; ledger and updated plans; `artifacts/cdc-review-receipts.json` with all native/probe raw channels and audit; `artifacts/cdc-completion-probe.rs` and `artifacts/cdc-reference-probe.rs` as bounded reproducible witness sources; immutable initial prompt/author baseline and preserved CC execution/validation/report. No CC history was rewritten. No slice close-set or synthetic retrospective is created.

Planning-check disclosure: the first incidental structural check incorrectly required standalone RG ID tokens despite the prompt listing accepted rows in a grouped form. Its corrected token check passed; no prompt, oracle or acceptance changed. Attempt/exit/capture scope is retained in the CDC receipt. All required Rust gate channels remain complete.
