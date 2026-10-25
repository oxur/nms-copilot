# Slice41 — banner/setup and library navigation documentation

2026-10-25 UTC. Initial implementation, zero actual refinements. Operator-authorized Autonomous Project01 Arc06 ONLY: CDC `/root/cdc_arc06`, gpt-6.1-sol High, independently reviews one fresh CC gpt-6-luna High. Prior40 CC completed/frozen and recommends a fresh context because its retained history is dense. No extra reviewer, substitution, next arc, integration or release.

## Outcome and boundary

Deliver the next original0039 M7.6 documentation increment in EXACT FOUR files:

- `crates/nms-copilot/src/banner.rs`
- `crates/nms-copilot/src/setup.rs`
- `crates/nms-copilot/src/lib.rs`
- `crates/nms/src/lib.rs`

Dedicated source `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/banner-setup-api-documentation`, branch `codex/banner-setup-api-documentation`, clean baseline **fff40ed23c02184e70f175cc3fc2dc687862ec34**. All changes are whole-line `//!`/`///` plus adjacent blanks ONLY. Every ordered non-doc byte, API, attribute, signature, ordinary comment, body, test, layout, dependency and other path remains exact. No tuple-layout normalization is needed: document existing SetupError payload meaning in variant prose. No banner algorithm/ANSI/Unicode fix, wizard behavior/config migration, TTY/native scenario, host save/config access, new API/helper/dependency or installation.

CDC owns planning and acceptance; sole CC owns dedicated source/index/gates only after full coherent intake/readback and CDC acknowledgement. Every mutation must resolve the dedicated root AND exact allowlist, compare current bytes/counts, use absolute target and explicit source cwd; prepare transformations in memory before writing. Main HEADb616 with known pre-existing agent nine-line native-test addition OID2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e and all held/external pins remain untouched. The main leftover has already been assessed; no cleanup or further provenance inquiry is authorized here.

Full original0039 M7.6 is STILL OPEN: required42 CLI namespace/navigation and complete public-surface/original-criterion reconciliation, CDC-owned/further useful decomposition permitted. All accepted29–40, original raw/history/nonzero game-backed/finalsemantic/cache/freshness/coverage/themes/D0040–43/distribution/parent/Operator/integration/release owners retained. Authentic NomNom input remains unanswered. Documentation of a limitation does not accept or repair its owning product criterion.

## Required reading before dependent work

Skill root `S=/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills`; framework root `F=S/collaboration-framework/knowledge`. Expand these exact aliases to real paths. Required-full means complete text/tables/code/end, recovering truncation; required-data means complete named population, not sampling. Record actual loaded extents and source state in the allowed ignored execution record, then give a source-cited contract readback. Hashes/headings/another context's claim do not establish reading.

1. **Required-full:** this prompt, sibling slice-plan.md and ledger.md; current supplied AGENTS instructions and dedicated source `AGENTS.md`; `S/collaboration-framework/SKILL.md`; `S/rust-guidelines/SKILL.md`, full `S/rust-guidelines/guides/11-anti-patterns.md`; `S/agent-coordination/SKILL.md`, `S/testing/SKILL.md`, `S/work-verification/SKILL.md`. Load active routed applicable required guides; do not expand unrelated optional history.
2. **Required-full before planning/execution:** `F/project-management/guides/README.md`, guides02 canonical planning,03 top-down,04 closing slices; `F/engineering-methods/guides/07-implementation-prompt-authoring.md`. **Required-section:** `F/engineering-methods/guides/01-engineering-methodology.md` Notes for Codex, Roles and Shared Invariants, Two-Contributor Workflow, Autonomous Mode including mandatory model selection; `F/agent-coordination/guides/01-when-to-delegate.md` complete; `F/testing/guides/03-validation-gates.md` complete; `F/work-verification/guides/03-row-closure.md` and05 independent verification complete. These govern ownership, proof strength, preservation, gates, corrections and return. Required-full retrospective entrypoint `S/methodology-stewardship/SKILL.md`; CDC owns the closing analysis.
3. **Required-section:** `S/rust-guidelines/guides/13-documentation.md` DC01–12 inclusive. Apply DC04 Examples→Errors→Panics; DC06's contrary illustrative order is not the governing order. Actual Result examples use `?`; only actual external interaction is no_run. Link imported in-scope labels directly, avoiding Slice40's redundant explicit-link targets.
4. **Required-full source at baseline fff40ed2:** all four allowed files including tests; copilot Cargo.toml and umbrella Cargo.toml. **Required-section:** copilot main.rs actual banner callsites and complete resolve_save_path; nms-save locate.rs LocateError enum, nms_save_dir/platform impls, list_accounts, list_saves, group_into_slots and SaveSlot::most_recent; copilot paths.rs config_path/ensure_data_dir. These resolve real wizard caller/discovery/write effects, not an authorization to execute host discovery. Source content controls over an inaccurate old summary.
5. **Required-section planning:** original `0039-phase-7c-release-prep-docs-tests-publishing-themes.md` complete M7.6 plus coverage implementation note; arc-plan Capability/authority/workflow, Operator Autonomous authorization/model/endpoint and latest accepted40/current41 owner sections; project-plan current authority/current41 section; original D0040/D0041 are reference-only future product reconcilers, not permission to change behavior. Required source module targets for navigation are actual lib.rs exports; links must resolve to real modules.
6. **Required-full support:** both `/private/tmp/nms-slice41-{cc,cdc}-capture.py`, `/private/tmp/nms-slice41-scope-audit.py`. **Required-data:** all two CC support records in `/private/tmp/nms-slice41-cc-attempts.json` and four CDC support/readiness entries in `/private/tmp/nms-slice41-cdc-attempts.json`: inspect complete fields/raw channels/exits/end/status/identities, preserve those prefixes. Canonical snapshots/helpers are in sibling artifacts. Support controls are not product evidence. Original40 return is reference-only, not the current assignment.

Complete readback connects W1–W4 and E1–E5 below to actual source, exact freeze/gates and original remaining owners. Wait for CDC acknowledgement before source edits; this is direct authorized handoff, no new human permission gate.

## Applied documentation design and concrete recipe

Binding behavior is the existing source; local prose organization may vary while every fact/example below remains. No unresolved policy or API choice is delegated. DC01–03 supports concise summaries/module navigation; DC04–06 concrete ordered failure sections; DC08 SHOULD is applied through the key shared pure examples and enum example rather than a redundant example on every simple printer. DC09–11 gives fallible hidden mains, executing pure examples and one wizard no_run. DC12 links actual module/items. The four-file preservation auditor plus its opposed actual scalar-width mutation protects the entire non-doc surface; it does not prove documentation truth, which CDC reviews substantively.

### W1 — actual banner contracts

Complete module/function docs, retaining useful teaching while correcting claims that the scanner supports ALL ANSI or actual terminal display cells. `visible_width` counts Unicode scalar values, not byte length/graphemes/terminal columns. Both it and `strip_ansi` discard ESC itself; ESC followed by `[` consumes through the next `m` or end, without validating that it is SGR; other following characters remain ordinary input. This is a limited scanner, not general ANSI handling. No real terminal interaction is needed.

`substitute_placeholder_in_line` uses str::replace for ALL matching substrings, including ordinary empty-placeholder replacement semantics. Absent placeholder returns original. Width comparison uses the limited scalar scanner. Adjustment location priority is LAST ESC first, otherwise last `║`, otherwise last `│`, otherwise last `|`, otherwise last `]`, otherwise end; it is not globally the rightmost of all border classes. Shorter values insert spaces; longer values remove available trailing whitespace before that location, potentially leaving the result wider when insufficient. Trimming uses trim_end and BYTE lengths; slicing can panic when removal splits multibyte trailing whitespace. State that in `# Panics`, link it from forwarding substitution/resolution functions as appropriate. Do not promise universal visual alignment or silent safe Unicode truncation.

`substitute_placeholders` replaces only literal `{version}` line by line using str::lines/join: removes final line terminator and normalizes CRLF to LF, preserving other content except spacing adjustment. `resolve_banner`: show=false short circuits; Some("") disables; None selects compile-time embedded default, nonempty Some uses supplied text; substitute compile-time crate version then optionally apply limited strip when color=false. It does not read a banner file/path, auto-detect terminal support or alter input. `print_banner` prints resolved Some text with println newline, disabled prints nothing; `print_system_banner` show=true prints supplied counts/source and help hint, not model-derived counts or a terminal probe. Both printing APIs can panic on stdout write failure, expose no Result. No printer execution or screenshot is required by this docs unit.

### W2 — actual wizard/error contracts

Correct old module wording: actual main's caller invokes wizard for explicit `--setup` only when non-headless and after an explicit `--save` short circuit; failed auto-detection by itself does NOT invoke it. Link public wizard/error API; don't promise automatic fallback or automatic first-run execution. Wizard performs interactive stdout/dialoguer activity, host filesystem discovery and optional config write; its example MUST be no_run.

Flow: choose existing default root if nms_save_dir returns an existing path, otherwise trim literal prompted input into PathBuf and require exists. No tilde expansion, canonicalization or save-content parsing; the UI mentions a file but actual flow still treats supplied path through account/save discovery, so no guaranteed direct-file bypass. If recognized account dir with saves and found in parent listing, use it; otherwise enumerate account subdirectories via list_accounts (one auto-select, multiple menu; not Steam-only). Group accepted save filenames into numbered slots, one auto-select otherwise menu in slot order; chosen slot uses most_recent manual/auto, manual wins equal mtime. This is metadata/filename selection, no gameplay/save-content validation. No host input fixture or new scenario.

Optional Confirm default=true: accepted writes paths/format="auto" to crate paths::config_path after ensure_data_dir; preserve other TOML root tables but REPLACE entire [save] table with dir/file/format. Readable malformed TOML is replaced by a new table, not guaranteed preservation of corrupt config/comments. Direct fs::write is non-atomic and errors can occur after earlier interaction, do not claim rollback. Declined returns selected path without writing config. Function returns owned PathBuf only after optional write succeeds; does not parse/validate a save, start watch/model, or activate selected config in an existing object.

`SetupError::Cancelled` also covers ALL dialoguer interaction errors (not solely explicit human cancellation); NoInstallation is actual non-existing prompted path; Locate transparently carries discovery LocateError; Io transparently carries config/data-dir/read/serialize/write std::io::Error. Explain tuple payload meaning in variant prose, no payload/layout/attribute changes. `run_setup_wizard` has Examples→Errors→Panics, concrete variants/partial-effects, stdout write panic, unavailable-home panic through config_path/data_dir when saving, and internal assumptions of valid selection/nonempty located slot. Do not invent new fallible API or fix existing unwraps/private ordinary comments. Public callers should handle Result with `?`.

### W3 — copilot root navigation

Expand crate/module overview into a usable navigation map of all actual thirteen public modules: banner, commands, completer, config, coordinator, dispatch, map, mcp, paths, prompt, session, setup, watch. Group by startup/session, command/query dispatch, single live coordinator/watching, map/presentation, MCP. Link actual own modules, don't invent root reexports/functions or a standalone nms-mcp package. Explain library reuse vs nms-copilot binary startup; HTTP-specific APIs depend on existing http feature, stdio/common surfaces remain. Briefly preserve one-owner live/cache/session boundaries through links to their established modules, no architecture/new guarantee.

### W4 — umbrella navigation

This package has NO dependencies/reexports/API beyond crate docs. Say it is a workspace documentation/navigation landing page, not an umbrella reexport facade. Retain useful existing links; list actual workspace roles including core/save/compat/graph/query/watch/cache, nms-cli's `nms` binary, and copilot's REPL/MCP. External crate landing links are navigation only, not verified publication/release status; no network check or extra dependency to make intra-doc links work. Avoid a relative `nms/index.html` link that would conflate the nms-cli binary rustdoc namespace with this package. Required42 owns CLI namespace/full reconciliation; do not claim that this increment completes all M7.6 or D0040/D0041 product behavior.

## Concrete literal examples — actual APIs, not precompiled author code

E1–E4 EXECUTE, E5 alone no_run. Keep owners alive for borrowed projections; complete arrays/strings, not contains/count-only or implementation-derived expectations. Placement may combine one related block per key item/module without duplicating all examples at each forwarding function.

**E1** banner module or scanner docs, import actual `nms_copilot::banner::{visible_width,strip_ansi}`. Input array `["\x1b[31mA界\x1b[0m", "a\x1bx", "a\x1b[31"]`, map to `(visible_width(text),strip_ansi(text))`; ENTIRE expected array `[(2,"A界".to_owned()),(2,"ax".to_owned()),(1,"a".to_owned())]`. The CJK scalar distinguishes columns/bytes; unsupported/incomplete controls distinguish the actual limited scanner. Do not add runtime parser fixes.

**E2** substitution docs: exact full outputs for calls `(line,placeholder,value)`:

| Input | Placeholder | Value | Whole expected output |
|---|---|---|---|
| `no placeholder` | `{v}` | `x` | `no placeholder` |
| `{version} |` | `{version}` | `1.0` | `1.0       |` (SEVEN spaces before border) |
| `{v} |` | `{v}` | `ABCDE` | `ABCDE|` (insufficient removable space; wider than original) |
| `{v}:{v} |` | `{v}` | `x` | `x:x     |` (FIVE spaces) |

Also assert `substitute_placeholders("v{version}|\r\nkeep\n", "1.0")` equals EXACT `"v1.0      |\nkeep"` (SIX spaces, no final LF). These counts follow original placeholder9chars→value3chars and existing1space; do not reuse a different row's spacing. No forced multibyte-panic fixture or broader new scenario required.

**E3** resolve_banner full array of five calls and whole Option<String> expected values: `(Some("ignored"),false,true)`→None; `(Some(""),true,true)`→None; `(Some(" "),true,false)`→Some(" "); `(Some("\x1b[31mRed\x1b[0m\n"),true,false)`→Some("Red"); same last input/color=true→Some("\x1b[31mRed\x1b[0m") (finalLF removed). Use owned String expected values/direct complete array comparison; no borrow from consumed temporary. Defaults/compile-time version explained in prose rather than treating host env as a fixed version oracle.

**E4** SetupError docs: import `nms_copilot::setup::SetupError`, actual `nms_save::locate::LocateError`, std::io. Array `[SetupError::Cancelled,SetupError::NoInstallation,LocateError::NoHomeDir.into(),io::Error::other("fixture").into()]`; map each to owned display String. ENTIRE expected strings `["setup cancelled by user","no NMS installation found","could not determine home/data directory","fixture"]`. This verifies all four variant/display/from paths with NO filesystem/UI call; don't assume transparent source() returns the direct wrapper or add that assertion.

**E5** wizard item example `no_run`, hidden `fn main()->Result<(),nms_copilot::setup::SetupError>`; `let selected=nms_copilot::setup::run_setup_wizard()?; println!("Selected {}",selected.display()); Ok(())`. Real interaction/config side effects stated before example; never execute wizard in this unit. Key examples use no unwrap/expect/ignore/compile_fail. Closing count distinguishes executing versus compile-only and overlapping feature matrices.

## Captures, gates and hold

Both current raw-first kernels fully read by CDC before first support diagnostic. Actual CC controls2bf7764e(exit7)/dc77eb4f(exit0); CDCe2f34951/82708779 same; rawstdout10B/stderr5B with NUL/end/equalcleanidentity. CDCsupportcompileb8100031 and sourceauditor--self-testadbfb7a8 pass (four SYNTHETIC doc additions/non-doc preservation and actual scalar increment1→2 rejected); SUPPORTONLY, not product proof. Read/verify full current record fields before use.

Every actual compiler/gate/validator from FIRST attempt goes through `/private/tmp/nms-slice41-cc-capture.py`: immutable UUID/argv/cwd/inheritedenv+overrides/originalTMPDIR/start/preHEAD+status+diff+untracked identity; raw stdout/stderr base64/bytes/SHA/exit/end fsynced BEFORE postmetadata/summary. Preserve failures/retries/original prefixes. No direct duplicate compiler/validator, quiet-output exit inference or retrospective observations. Ordinary source/status/edit guards are incidental metadata, not product checks; no manufactured waiver. Sequential stable source/no writes during gates; all Make/Cargo offline.

Original NINE groups/nine commands (both selected packages have libraries; CLI binary's colliding nms rustdoc namespace is EXCLUDED and remains42):

1. `make format`, CARGO_NET_OFFLINE=true.
2. `make lint`, same. IMMEDIATE source/index/gate HOLD; send actual UUIDs/identity. CDC full W1–W4/E1–E5/helper walk before runtime release; no postpair edits.
3. `cargo test -p nms-copilot -p nms --doc --all-features --offline` FIRST after release.
4. Same packages `--doc --no-default-features --offline`.
5. `cargo test -p nms-copilot -p nms --lib --all-features --offline`, ACTUAL tools.exec_command require_escalated on established watcher host substrate, original TMPDIR inherited. Retain any accidentally restricted failure then IDENTICAL host retry; never rerun actual host red until green.
6. `cargo doc -p nms-copilot -p nms --lib --all-features --no-deps --offline`, overrides CARGO_NET_OFFLINE=true, RUSTDOCFLAGS EXACT `-D warnings -D missing_docs`. Actual zero rustdoc/missing-doc warnings required; no generated-field/residual exception transferred. Existing unrelated proc-macro-error2 Cargo future notice is distinct.
7. `python3 /private/tmp/nms-slice41-scope-audit.py` exact4path/ordered non-doc equality/no untracked and actual unique scalar-width increment1→2 mutation rejected.
8. `git diff --check`.
9. `git diff fff40ed23c02184e70f175cc3fc2dc687862ec34 --check`.

CDC independently runs all original nine groups on final clean commit. No fullworkspace/new native/hostwizard/save discovery/coverage claim or acceptance of original product carryovers. Unexpected diagnostics/mismatches keep affected rows open, no weakening/suppression.

## Commit, return and boundaries

After actual greens/released review, local normal-hook commit EXACT FOUR named paths via git commit --only; Git metadata require_escalated. Unique `/private/tmp/nms-slice41-cc-source-message.txt`, read complete body and positional `git interpret-trailers --parse FILE` BEFORE commit. Both trailers exactly ONCE CONSECUTIVE:

Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>

After commit compare RAW git cat-file body split ONCE at bytes([10,10]) to exact prepared bytes; no overescaped delimiter, strip or git-show formatting confusion. Verify exact paths/parent/tree/trailers/clean source/no owned process, explicitly return ownership. Do not silently amend history after a mechanical verification error.

Ignored override `workbench/slice41-return/{execution.md,validation.json,closing-report.md}`; no CC planning writes. Record actual required reading/readback/chronology/failures/raw full records/commit/source fences/executed-v-compiled denominators. Canonical CC closing-report itself walks SIX BS opening/closing rows and ALL three bubble-ups (assigned capability+revision+integration; unforeseen finding+owner; specified-v-delivered/no silent drop), actual artifact inventory and pending CDC planning close paths/hash only where known. No selfacceptance or invented earlier inventory/close hash. Preserve issued prompt; every correction immutable sibling iterationNN. Five ACTUAL refinements after initial, filename count is not budget; cap without convergence requires concrete Operator disposition, no automatic sixth. Independent acceptance/CDC retrospective/bottom-up closes each unit. Continue only within Arc06 standing authority; actual unresolved material/input/Operator gate stops affected work. No merge/push/install/publish/release/main cleanup/next arc/Project02.
