# Slice25 CC implementation assignment — original M7.2 shell completions

Issued 2026-10-22 UTC; initial immutable assignment, ordinary refinement0of5. Standing human-authorized Autonomous Project01 Arc06 only. Actual selected CDC gpt-6.1-sol High / sole existing CC gpt-6-luna High, unchanged. CC confirmed qualitative headroom for this self-contained unit and no owned process/gate after accepted24. No extra role, model substitution, inquiry, dependency, installation or remote action authorized.

## Readback, ownership and exact source boundary

Read this entire prompt, slice-plan.md, ledger.md, source-allowlist.md, original0037 M7.2 (and relevant CLI definitions), source AGENTS.md, current completions.rs/main.rs registration/README section and existing completions integration/unit tests. Apply already loaded current framework3.0.1/PM operation guides, implementation-prompt-authoring, agent-coordination, verification/testing, full rust-guidelines including required anti-patterns/CLI testing and documentation. Recover any truncated required reads; do not replace full guides with this assignment.

BEFORE editing send coherent readback: baseline/absolute source paths, two-path freeze, acceptance populations/independent oracles/negative controls/shell limits, capture implementation and support controls, first format→lint→IMMEDIATE HOLD, ordered gates/refinement cap/returns. CDC owns planning; after that readback CC owns ONLY this dedicated source/index/gates until explicit frozen handback. CDC independently reviews implementation it did not write. Do not mutate planning or another worktree.

Absolute source: /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/shell-completions, branch codex/shell-completions, baseline cd38d7e1603b957f862b75c23b60ed5bc713bcf0. Revalidate HEAD/status/index/operations before work. EXACT TWO allowed tracked paths, use absolute paths for every patch:
- /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/shell-completions/crates/nms-cli/tests/completions_integration.rs (NEW)
- /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/shell-completions/README.md (Shell Completions section ONLY)

All production generator/CLI/query/parser/cache/model/Cargo/dependencies and old tests remain byte-frozen. Preserve accepted24 and every prior closure. Main b616ab2cfa9c67dfb6361bb1355ca9e6680d18cc has ONLY the human nine-line crates/nms-copilot/tests/mcp_live_integration.rs edit (working Git blob2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e); main/index/all held and external pins protected. No merge/push/install/publish/release/otherarc/Project02. Report a demonstrated product defect rather than silently widening scope.

## Usable delivery / preserved original contract

Original0037 M7.2 requires clap_complete generation for Bash/Zsh/Fish through `nms completions <shell>` and README install instructions. Existing generator already uses Cli::command(), actual binary dispatch bypasses save resolution, and existing tests only check name/nonempty. Deliver discriminating real-child proof and usable activation docs; no production repair is presently established. Preserve all five existing Shell values (bash,zsh,fish,powershell,elvish), positional parsing, help/errors/stdout and no-save behavior; no dynamic save-derived name/path completion promise is introduced. M7.3 slots and every other original capability/repair/higher gate remain separately open.

IMPORTANT actual tool inventory: /opt/homebrew/bin/bash is GNU Bash5.3.9; /bin/bash and /bin/zsh are actual system shells. `/opt/homebrew/bin/fish` is the BSD Go Fish GAME (bsdgames-osx), NOT Fish shell. NEVER execute that program or install/download a Fish shell. Fish generation/output/manifest proof is mandatory; Fish shell syntax/runtime/interactive completion is NOT claimed. Original generation acceptance does not require a new installed target shell. Available Bash/Zsh syntax/registration and actual Bash candidates below provide additional bounded proof, with explicit platform limits. Do not turn unavailable Fish runtime into a new waiver or generic skip/green claim.

## Implementation / exact closed oracle worksheet

New test file uses existing assert_cmd/tempfile/std dependencies and AGENTS test names `test_<fn>_<scenario>_<expectation>`. Every actual binary child is bounded10s, owns TempDir before launch, isolated empty HOME/config/data roots where existing project conventions need them, no save/config seed. All successful generation children: exact code0, stderr EMPTY, stdout complete UTF8/nonempty, correct command/namespace/registration, no banner/ANSI or save-load diagnostic. Assert temp HOME remains unchanged/no cache/config creation through complete recursive file/directory snapshots, excluding only explicitly test-authored script artifacts where relevant. Do not assert substring presence as the entire proof; parse shell-context scoped records and compare literal sets. Expected scripts/flags must NOT be generated from production Cli::command() or clap_complete APIs in the test.

O1 — For EACH bash/zsh/fish actual child, independently literal ROOT COMMAND SET:
`info find show stats route list convert export import completions saves help`.
ROOT LONG FLAG SET `slot help version`.
EXPORT LONG FLAG SET `save biome infested within nearest named discoverer from format slot help`.
FIND LONG FLAG SET identical except no `format`.
Check actual command-specific scope, rather than same flag elsewhere in script. Assert show system/base nested command entries in each generated shell's actual representation. Keep short options/positional metavariables distinct from long flags/commands. English descriptions need not be snapshotted; no universal lexical formatting promise.

Pinned primary implementation is clap_complete4.5.66 in Cargo.lock; local sources /Users/oubiwann/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/clap_complete-4.5.66/src/aot/shells/{bash,zsh,fish}.rs were inspected. Use these exact shapes, not guessed parser syntax:
- Bash: `_nms` function, context case `nms)` / `nms__export)` / `nms__find)` / `nms__show)` and their immediate `opts="..."` words; root plain non-option words are commands; long options start `--`. Parse ONLY matching case's opts, stop context rather than consume later branch. Registration is `complete -F _nms ... nms`. Do not require a fixed Bash-version-specific option string.
- Zsh: `#compdef nms`, `_nms()`; `_nms_commands()` root helper holds quoted `command:description` entries before `_describe`. Context `(export)` owns its `_arguments` specs through its first `;;`, `(find)` similarly. Root specs precede first `case $state in`. Long value specs use `'--nearest=[description]:...'` (EQUALS, not plus); boolean uses `--named[...]`; conflict prefixes may precede `)--help[...]`. Extract spec head before first `[` and after last `)`, then strip `--` and trailing `=`. Do not collect unrelated context flags or prose. Nested show helper owns system/base entries. Preserve quoted specification grammar, avoid global arbitrary-line substring matching.
- Fish: `complete -c nms -n "__fish_nms_needs_command" ... -a "export"` describes ROOT commands; root long opts in that exact context use `-l NAME`. Export/find directives are `-n "__fish_nms_using_subcommand export"` / find with `-l NAME`. Descendant predicates can append `; and not ...` for nested commands; inspect actual show directives. Helpers `__fish_nms_needs_command` / `__fish_nms_using_subcommand` and `-c nms` namespace required. Generator covers named opts and subcommands; do NOT invent Fish arbitrary positional-value completion.

O2 — Same actual-output validators must demonstrate meaningful NEGATIVE controls per shell: remove ONLY root `export` entry while export subtree still exists => reject; remove/rename ONLY export `named` while find still has named => reject; wrong command namespace/registration => reject. Exact option sets reject unexpected replacement, preserving duplicates/discrimination as appropriate. Mutations must actually change intended actual text (assert mutation occurrence/count); no synthetic fake observed output or validators accepting missing sections. Feed actual whole output positive, then negative copies to SAME predicate. Preserve stdout through actual child completion, no pending-only observation.

O3 — Actual CLI composition: `nms completions --help` code0 stdout documents positional shell and allfive accepted values; stderr empty. Missing shell and invalid `not-a-shell` each usage code2, stdout empty, stderr identifies correct argument/error and accepted alternatives; no save-load attempt or filesystem effects. Preserve legacy existing tests, no test weakening. Additional discretionary scenarios are not needed.

O4 — Available-shell proof uses one bounded private stdlib-Python controller `/private/tmp/nms-slice25-cc-shell-proof.py` (CDC separate namespace). Its exact source/hash is retained with raw gate; no dependency/terminal adapter/new public API. Read actual scripts generated by current built `target/debug/nms` under temporary HOME; do not feed expected script as observed script. Each native subprocess timeout10s, raw output/status retained by containing captured controller gate, fail diagnostics identify stage. All temp dirs created before child and cleaned by owner, no actual HOME or rc/global writes. Generate scripts into a temp directory containing a space to exercise argv quoting. Execute available `/opt/homebrew/bin/bash` and `/bin/zsh` only; no Fish invocation.
Bash: `--noprofile --norc -n SCRIPT` syntax then a separate clean child sources quoted positional `$1`; `complete -p nms` must register `_nms` for nms. For candidate tests use static shell source and positional argv, never interpolate path into source:
`source "$1"; shift; COMP_WORDS=( "$@" ); COMP_CWORD=$((${#COMP_WORDS[@]} - 1)); current="${COMP_WORDS[COMP_CWORD]}"; previous="${COMP_WORDS[COMP_CWORD-1]}"; _nms nms "$current" "$previous"; for candidate in "${COMPREPLY[@]}"; do printf '%s\n' "$candidate"; done`.
Independent exact unique candidate vectors:
- [nms,ex] -> [export]
- [nms,--s] -> [--slot]
- [nms,export,--na] -> [--named]
- [nms,export,--nea] -> [--nearest]
- [nms,find,--bi] -> [--biome]
- [nms,show,sy] -> [system]
- [nms,does-not-exist] -> [] (function result, NOT a promise that readline filesystem fallback is disabled).
All statuses0/stderr empty; results are complete stdout line vectors, not contains.
Zsh: `-f -n SCRIPT` syntax. Put actual generated script at owned temporary function directory `.../_nms`; actual startup/registration child `/bin/zsh -f -c 'fpath=("$1" $fpath); autoload -Uz compinit; compinit -D; print -r -- "${_comps[nms]}"; whence -w _nms' zsh FUNCTION_DIR` must produce exact `_nms` and `_nms: function` lines, code0/stderr empty. `compinit -D` disables dump writing (installed zshcompsys manual); do not use insecure `-u` or silently-ignore `-i`. No interactive ZLE candidate invocation/Fish runtime claim. If a real prerequisite/security failure occurs, preserve and diagnose; no weakening or dependency adoption.
Private controller performs syntax/registration/candidates within its single gate. Capture any actual compiler/diagnostic of it from the first invocation. Do not run uncaptured py_compile or invoke capture helper with unsupported `--help`.

O5 — README ONLY Shell Completions section: usable Bash mkdir/generate/source for current session, quoted paths; explain persistent source entry in ~/.bashrc without editing real rc files or requiring a new completion package. Zsh mkdir/generate `_nms`, `fpath=("$HOME/.zfunc" $fpath)` BEFORE existing compinit; show autoload/compinit if not yet configured, avoid blindly duplicating existing setup/security bypass. Fish mkdir/generate standard ~/.config/fish/completions/nms.fish with Fish shell autoload, no installation/package commands. Generation requires only nms; using a script requires its shell. No private proof/model/process jargon in product docs, no claim dynamic names or allplatform interactive verification. Other README text unchanged, including accepted24 stored-system-address qualification/other5shell support/multisave docs.

## Capture and ordered hold/release

Use inspected `/private/tmp/nms-slice25-cc-capture.py`, JSON `/private/tmp/nms-slice25-cc-attempts.json`; NEVER overwrite existing attempts. Already verified support-only controls:777b20934efb4676908b0a617252e608 exit7 and dccddfba2dd2497f8efc61caffa196f2 exit0, stdout10/stderr6, clean cd38 before==after. They prove capture plumbing, NOT product acceptance.
Capture kernel interface `python3 HELPER LABEL '{"argv":[...],"overrides":{...}}'`; import definitions safely if needed; no unsupported helper CLI probes. EVERY actual compiler/gate/diagnostic preregisters UUID/argv/cwd/start/source HEAD+workingdiff+untracked identity/explicit status/env, persists raw exit/endUTC/fullstdout/fullstderr FIRST before fragile postprocessing, then post HEAD/identity/explicit status. Both channels byte/base64/hash retained, all failed/retried attempts and exact prefixes/JSON values unchanged. functions.exec full outer object stored BEFORE text/summary; preserve running session_id and raw completion. No inferred success/exit/end/identity. Ordinary source/status/inventory/interface lookups are not SUTgates or acceptance evidence; no invented incidental-metadata waiver.

After edits run captured `make format` THEN `make lint`; compiler self-repairs stay in exact allowlist/contract with every attempt retained and renewed format→lint. On passing pair IMMEDIATELY HOLD source/index/gates: send actual IDs/argv/exits/rawfield state/before-after identity and diff/path list. NO post-pair helper/source edits and NO runtime until CDC completes full two-path/private-helper O1–O5 worksheet review and explicitly releases. Consolidate demonstrated omissions into immutable numbered refinement; initial prompt never edited. Five ordinary refinements, no sixth automatic authority or evidence rewrite/weakening.

After release execute required ordered real gates, captured each separately; initial pair plus these TEN =12 total groups:
1. cargo test -p nms-cli --test completions_integration
2. cargo test -p nms-cli
3. cargo build -p nms-cli --bin nms
4. python3 /private/tmp/nms-slice25-cc-shell-proof.py (above complete bounded actual-shell proof)
5. make test
6. cargo build --all-features --workspace
7. cargo test --doc --workspace --all-features
8. cargo doc -p nms-cli --no-deps with RUSTDOCFLAGS=-D warnings
9. git diff --check
10. git diff cd38d7e1603b957f862b75c23b60ed5bc713bcf0 --check

Known sandbox native filesystem/loopback failures: preserve failed attempt, identical argv/env with ORIGINAL TMPDIR on authorized host retry; direct host established fullworkspace watcher gate permitted, clearly attributed. No reduced concurrency/flags/timeout/assertions or selective denominator credit. Once settled required proof passes, do not repeat unchanged broad groups absent edits/failure/unresolved concern. Evidence-only edits/closure do not trigger runtime rerun. CDC independently executes applicable12groups/actual-shell proof on committed frozen source, not selfacceptance from CC green summaries.

## Local delivery and return

Assistant local commits already authorized, Gitmetadata escalation required; exact two named paths `git commit --only` with once-only CONSECUTIVE parsed trailers Codex <noreply@openai.com> and Billo AI <ai-engineering@billo.systems>. Preserve all precommit identities, commit mapping/tree applicability and failed attempts. No commit claims acceptance before CDC disposition. Freeze source/index/gates after final commit and report no owned process.

Use ignored source `workbench/slice25-return/{execution.md,validation.json,closing-report.md}` plus raw channels/controller source. Append actual new sections AT EOF, do not reorder historical intake or overwrite prior values. Capture original intake hash/prefix and final returns. validation cites existing actual attempt IDs/argv/source/raw completion, correct `before/after.working_identity_sha256` fields, counts/limits/Fish output-only boundary. Proposed-done remains proposed until independent CDC review; all5SC/R06-S25/M7.2 open at issue. Record guideline application, failures/retries, remaining limits, compliance/structural change/bubble-up and retrospective observations; mandatory CDC retrospective follows acceptance. No unsupported allplatform/game/parent acceptance, no cost/model comparison inference.
