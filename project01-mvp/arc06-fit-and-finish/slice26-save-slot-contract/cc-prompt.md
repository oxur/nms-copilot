# Slice26 initial CC assignment — full original M7.3 save-slot contract

2026-10-22 UTC; initial0of5 ordinary refinements. Standing human-authorized Autonomous Project01 Arc06 ONLY. Same actual selected CDC gpt-6.1-sol High / sole existing CC gpt-6-luna High. CC qualitative headroom/no owned process confirmed after accepted25, no new model/role/CRC or inquiry. No install/remote/integration/release/otherarc/Project02.

## Intake / ownership / absolute source paths

Read this ENTIRE immutable prompt, slice-plan.md, ledger.md, source-allowlist.md, original0037 entire M7.3 and shared-load/stdout notes, source AGENTS.md, complete affected main.rs/locate.rs/saves.rs and existing slot tests, relevant frozen info/find/show/stats/route/list/export/import dispatch/load boundaries and save56_minimal fixture. Apply current framework3.0.1/PM README+operation guides/engineering implementation-prompt-authoring/verification/testing/agent-coordination and full rust-guidelines including FULL required anti-patterns/CLI/error/filesystem/testing guidance already read; recover any missing/truncated required reads before writing Rust, do not replace them with this prompt.

Send coherent readback BEFORE editing: actual HEAD/status/index/operations, five absolute paths, exact APIs and caller/policy boundaries, fixture literals/oracles/negative controls/error/setup lifetime, capture first and format→lint→IMMEDIATE HOLD, eleven groups/refinement cap/returns. That readback grants sole dedicated source/index/gates until frozen handback; CDC owns planning and independent review. No other worktree/planning mutation. Every patch uses absolute dedicated path; reject any wrong-root edit BEFORE applying.

Source `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-slot-contract`, branch codex/save-slot-contract, baseline03b6536636f00a74a52f2d783a9ef80a671d191e. EXACT FIVE allowed tracked paths:

1. /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-slot-contract/crates/nms-cli/src/main.rs
2. /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-slot-contract/crates/nms-cli/src/saves.rs
3. /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-slot-contract/crates/nms-save/src/locate.rs
4. /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-slot-contract/crates/nms-cli/tests/multi_save_integration.rs NEW
5. /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-slot-contract/README.md ONLY Multi-Save Support section.

All parser/address/model/query/cache/watch/REPL/MCP/Cargo/deps/other tests/source frozen. Preserve all25closures including full export24/completions25. Mainb616ab2cfa9c67dfb6361bb1355ca9e6680d18cc contains ONLY human9line crates/nms-copilot/tests/mcp_live_integration.rs edit/blob2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e/indexempty; all accepted/held/external03e8d04pins protected. New source clean at issue; no implementation/compiler/product gate exists yet.

## Usable delivery / exact routine engineering decisions

Original0037 M7.3: discover/list all15 numbered slots/manual-auto files, global --slot selects that slot's newest file, --save explicit path takes priority; nms saves displays accounts, slot/manual/auto/most-recent information. Original first-account choice is EXPLICIT and preserved: slot selection uses FIRST name-sorted account from list_accounts, never searches another account if missing/empty. Without slot, existing global-most-recent across accounts remains. No new account selector, internal save-name/nickname catalog, interactive UI, persistence/import repair, new file format or cache semantics.

Demonstrated gaps: Import dispatch skips resolver; filename parser accepts save31.hg and above despite documented1–15; save listing masks any I/O error as no files. Fix ONLY these and corresponding validation/proof/docs:
P1 main.rs: make Import arm resolve_save_with_slot(save,slot)? then pass Some(path) to existing import::run(file,...,source), exactly like other save-backed arms. Preserve --save precedence, all other argument/query APIs, existing checked-save route and source errors. Add Clap `value_parser = clap::value_parser!(u8).range(1..=15)` to existing global slot arg; out-of-range/noninteger usage is code2 before discovery. No public API/enum additions or broad shared-model refactor. Existing central resolver fulfills shared path-selection purpose; do not replace approved checked model constructors or legacy helper compatibility. Keep original first-account/nonempty list_accounts guarantee/no-account error path, no silent fallback.
P2 locate.rs: parse_save_filename accepts canonical save.hg or numeric file indices2..=30, rejects out-of-range31..255 and parse overflow. Preserve current leading-zero/extension/account-kind/symlink/duplicate/equal-global-mtime behavior (no new normalization policy). Same public APIs/errors, all valid files/slots/types/order unchanged; slot.most_recent tie remains Manual via >=. Update relevant bound docs and add meaningful portable tests below. No parser/cache semantic revision: no address/model/cache interpretation changed.
P3 saves.rs: preserve table formatting/account labels and existing friendly `No save files found.` for LocateError::NoSaveFiles ONLY. Other LocateError (including Io) propagates to existing main code1/stderr; never mislabel IO as empty. Use small private helper `slots_for_account(&AccountDir) -> Result<Option<Vec<SaveSlot>>, LocateError>`: None ONLY for NoSaveFiles; Some(group_into_slots(saves)) on success; other Err propagated. Run uses helper in existing account loop; no new public API/dependency or output redesign. Existing unreachable/empty-vector handling may retain its existing message. Unit-test helper real FS: obtain AccountDir through list_accounts on owned tempdir, replace that one captured directory path by a regular file, call helper => LocateError::Io with ErrorKind::NotADirectory (NOT None/NoSaveFiles). No chmod/root/permission-dependent test, race/thread/sleep/test hook. Also assert actual empty directory None and valid present slots Some.

## Closed proof worksheet — no implementation-generated expected values

Every actual CLI child owned TempDir BEFORE launch/assertions, timeout10s, argv only/no shell, current_dir within owned fixture, child HOME/XDG all isolated. NO process-global env mutation/unsafe env API. Whole owned tree snapshots after complete fixture setup and before/after EVERY child include directory entries, file bytes and actual modified times (ignore access time); no cache/config/temp/source changes allowed. Setup all files/sentinels before baseline; any intentional malformed-case edit occurs BEFORE snapshot, never compare against a pre-setup timestamp. Fixed FileTimes/set_times, no sleeps or write-order assumptions. No actual user saves or rc files opened/written.

NATIVE DISCOVERY TESTS only cfg macOS/Linux: pinned dirs6 Unix home honors child HOME; Windows dirs uses Known Folders instead of HOME/APPDATA, so NEVER attempt redirecting/discovery against real Windows user tree. Use literal owned roots Mac `$HOME/Library/Application Support/HelloGames/NMS`, Linux `$HOME/.local/share/Steam/steamapps/compatdata/275850/pfx/drive_c/users/steamuser/AppData/Roaming/HelloGames/NMS`. Library explicit-tempdir tests remain portable. Report current-host native proof and Windows runtime limit honestly; no allplatform/hostedCI claim or new installation/override flag. This does not narrow original APIs; higher platform/integration gates remain open.

O1 portable locate tests (existing old tests preserved): explicit literal filename/slot/type table for save.hg/manual1, save2.hg/auto1 through save29.hg/manual15/save30.hg/auto15; create all30 fake payload files, assign strictly ordered known mtimes, assert exact complete discovered basename+slot+type multiset/count30, grouping exact1..15/manual+auto paths/no empty slots. Most recent Auto when newer, Manual when newer, Manual on equal tie, one-sided manual/auto and sparse slots1/15 remain correct. Reject/ignore mf_save*.hg, save.hg.bak/save3.hg.backup/save.tmp/unrelated files, save0.hg/save1.hg/save31.hg/save255.hg/save256.hg; include invalid31 newer than valid30 to prove newest selection excludes it. Preserve existing leading-zero policy, do not add a rejection not required by bounds.
O2 portable accounts/errors: name-sorted exact GOG DefaultUser, Steam st_123, Unknown zz_empty classification/path; ignore root regular nonaccount files, NoAccountDirs for file-only/empty root; NoSaveFiles only actual empty account, real wrong-file path yields Io. Unknown DIRECTORY remains AccountKind::Unknown, never silently filtered as nonaccount. No game parser invoked on fake listing payloads. Private saves helper tests distinguish success/None/real Io as above.

O3 exact valid application fixture A/B — use serde_json clone of existing data/test/save56_minimal.json (NOT legacy minimal_save.json); do not call legacy core packed()/production decoder/query to construct BOTH source and expectations. Directly assert supplied raw source literals/shape after setup. Same existing base/owner/VP and other valid fields retained. Common SaveName `Save A`/`Save B`, Units101/202; current expanded address X=-1017,Y=3,Z=-1352,P=0,GG=0,SSI1280(A)/1281(B); previous wrapper remains valid existing literal.
A primary SolarSystem UA `0x05000003AB8C07`, DM.CN `System A`; Planet UA `0x15000003AB8C07`, CN `Planet A`; base Name `Base A` and GalacticAddress primaryP0. Add cloned SolarSystem record UA `0x05020003AB8C11`, CN `Destination A`.
B primary SolarSystem UA `0x05010003AB8C07`, CN `System B`; Planet1 UA `0x15010003AB8C07`, CN `Planet B`; add cloned Planet2 UA `0x25010003AB8C07`, CN `Planet B2`; base Name `Base B`, address primaryP0. Add cloned SolarSystem record UA `0x05030003AB8C1B`, CN `Destination B`.
These literals independently mean A primarySSI1280, B1281; destinationA1282/X-1007, B1283/X-997. Each has2 genuine systems/1base, A1planet/B2planets; current matches real primary (no incidental player placeholder). A records3/B4. Names explicit approved CN string semantics. Do not generate expectations with same transforms or use legacy portal-frame fixture. CSV bytes (one LF after each of these TWO lines):
```csv
System Name,Portal Glyphs,Galaxy
Imported A,050003AB8C07,0
```
This CSV refers genuine A primary system; in A duplicate stats0added/1duplicate/0skipped, in B1added/0duplicate/0skipped. 56-bit save versus48-bit portal distinction is intentional, not guessed by width.

O4 full real-child caller selection — fresh owned fixture per invocation:
First sorted account DefaultUser: save.hg=B/manual timeT+100; save2.hg=A/auto T+200; save3.hg=B/manual T+400; save4.hg=A/auto T+300; save29.hg=A/manual T+500; save30.hg=B/auto T+500. Second account st_123: save.hg=B T+900. Third zz_empty empty. Add metadata/backup/regular nonaccount noise; invalid save31.hg poisoned/newerT+5000 must be ignored. T=UNIX_EPOCH+4,000,000,000seconds (fixed future timestamps yield existing `just now` relative label); use actual post-set_times snapshots. Preserve unselected source bytes/times too.
Expected selection is LITERAL: default(auto across accounts)=>B from st_123; --slot1=>A auto from FIRST DefaultUser even though second account newer; --slot2=>B newer manual; --slot15=>A manual tie; --save explicit-A.json plus --slot2=>A explicit-file priority.
For EACH of TEN save-backed routes below execute three modes (defaultB, global --slot1A, global --slot2+explicit --save A). Thirty real children; same production query behavior, only selection is under test. Query target arguments follow the independently specified expected A/B row, never resolver output:
- info: exact parsed `Save Name` pair Save A/B; Units101/202.
- find --named: selected Planet A versus Planet B+Planet B2 exact named-planet marker population; opposite markers absent.
- show system `System A/B`: code0 selected primary system and its planet names, opposite absent.
- show base `Base A/B`: code0 selected base name, opposite absent.
- stats: exact key/value `Systems`2, `Planets`1(A)/2(B), `Bases`1; no broad substring alone.
- route --target `Destination A/B` --from `Base A/B`: code0 selected destination name/opposite absent, no query semantics change. The two destinations are physically distinct from base; do not use a zero-length same-system route or invent distance literals from a production formatter.
- list systems --all: exact selected system-name population System A+Destination A or B equivalents; opposite absent.
- list bases --all: selected Base A/B exact name field; opposite absent.
- export --format json: parse complete stdout JSON; exact planet_name/system_name/galaxy projection A[(Planet A,System A,0)] or B[(Planet B,System B,0),(Planet B2,System B,0)], preserving multiplicity. Full12field export contract remains accepted24, no new schema.
- import CSV: stdoutEMPTY, stderr EXACT `Imported 0 systems (1 duplicates, 0 skipped)` plus ONE trailing LF for A / `Imported 1 systems (0 duplicates, 0 skipped)` plus ONE trailing LF for B. No persistence promised/repaired here; allfiles unchanged.
Success other routes code0/stderrEMPTY. Helpers may remove only valid ANSI SGR before semantic table parsing (nms_theme uses ANSI); preserve Unicode and field boundaries, no production formatter to build expected output. Derive table fields using actual `│` boundaries/trim, literal selected population and opposite negatives, not arbitrary global Name substring as entire proof. Info/stats select exact key rows, no title/footer pollution. Add TWO actual cases for global-flag positions: `nms info --slot2`=>B and nested `nms show system "System A" --slot15`=>A. Existing --save precedence unit tests retained; explicit-file selection also works with no discovery root (one actual info child).

O5 actual listing and errors:
- `nms saves` on O4 three accounts: exact account header/order/kinds DefaultUser(GOG), st_123(Steam123), zz_empty(Unknownzz_empty); DefaultUser slot rows1(auto),2(manual),15(manualtie), all manual/auto yes; st_123 slot1 manualyes/auto-; zz_empty friendly nofiles, never hidden IO. Complete rows parsed with existing column order Number/Manual/Auto/Most Recent; fixed future label just now. No raw goldens of ANSI decoration/table geometry or new table format.
- separate all30 canonical fake-payload fixture: `nms saves` yields exact15slot rows, bothfilecolumns yes, newest type matches fixed per-pair mtimes; invalid backups/metadata/out-of-range absent; proves listing does not parse payload names/content.
- root missing=>code1/stdoutEMPTY/save-dir-not-found diagnostic; root empty/file-only=>code1/no-account diagnostic; root path regular file=>code1/real IO diagnostic (not friendly nofiles); first account empty but later valid=>slot1 code1/NoSaveFiles, NO cross-account fallback; absent slot3=>code1 `save slot 3 not found`.
- selected newer slot1 A payload intentionally malformed BEFORE baseline=>code1/error, no fallback to old manualB or latestB; explicit absent --save with valid slot=>code1, no fallback. No source mutation by actual child.
- --slot0,16,256,not-a-number each code2/stdoutEMPTY/argument diagnostic, no discovery root required; preserve explicit valid bounds1/15. One load-free `nms --slot15 list biomes` on missing root succeeds (static reference path bypasses save discovery).
- SAME actual-output semantic listing/selection predicates must reject meaningful copies: remove slot15 row, swap slot1 Manual/Auto recent type, wrong selected save-name/key/planet marker. Actual positives first, assert effective mutation and unchanged unrelated data; no fake observed fixtures or negative control that always returnsfalse. No discretionary extra matrix or shared test framework/dependency.

## Raw-first capture / strict first-pair hold / eleven groups

Inspected `/private/tmp/nms-slice26-cc-capture.py`, JSON `/private/tmp/nms-slice26-cc-attempts.json`; same proven raw-first code with ONLY source/record namespace substitutions, SHAedbdcd5d58d59e05f8d5af018ac4dcb25f598f805c4a8923baf94b0382436e89. Before first compiler, actual support756a0f2338cf436faccd70575892ed1b exit7 /f90155f2c7de4bf49b6de036a5eaaff9 exit0 each stdout10/stderr6, rawexit/end/channels/hash and before==after clean03b/status verified. Support controls no product credit. NEVER overwrite records/old IDs/values or reuse25 namespaces.
Every actual compiler/gate/diagnostic preregisters UUID/argv/cwd/start/HEAD/workingdiff/untracked composite/explicitstatus/env; saves full raw exit/endUTC/stdout/stderr FIRST before postprocessing, then afterHEAD/identity/explicitstatus. functions.exec store full outer tool result BEFORE text/summary; retain session IDs/complete results, no inferred exit/end/identity. Ordinary source/status/interface/inventory/helper reads are not SUTgate/waiver obligations, receive no product credit. No unsupported helper --help/uncaptured py_compile probes. No private runtime controller/new terminal harness is needed for this unit; actual Rust child tests supply the real CLI boundary.

Complete ALL source/helpers/fixtures/O1–O5 before first settled pair; run captured make format THEN make lint (initial compiler self-repair only within exact paths, retain every attempt). At passing pair IMMEDIATELY HOLD ALL source/helper/index/gates; no post-pair mutation or runtime until CDC full fivepath/helper/call/assertion/README review/release. Consolidate demonstrated defects into immutable numbered siblings. Five ordinary refinement cap, no inherited exception/reset. No weakening valid controls or changing production to appease wrong author fixtures; source-ground root cause and exact evidence first.

After explicit release, original ordered remaining9groups (pair+9=ELEVEN):
1 cargo test -p nms-save
2 cargo test -p nms-cli --test multi_save_integration
3 cargo test -p nms-cli
4 make test
5 cargo build --all-features --workspace
6 cargo test --doc --workspace --all-features
7 cargo doc -p nms-save -p nms-cli --no-deps with RUSTDOCFLAGS=-D warnings
8 git diff --check
9 git diff 03b6536636f00a74a52f2d783a9ef80a671d191e --check
Known sandbox nativeFS/loopback failures retained then identicalargv/env hostretry with ORIGINAL TMPDIR; directhost fullwatcher make test already established/authorized. No lowerconcurrency/timeouts/flags/testassertions or false partial denominators. No unrelated nativeMCP matrix repetition. Once passing, no unchanged broad rerun for metadata/return edits. Final baseline diff MUST include new tracked file (stage named newtest before finaldiff, or one captured identical postcommit baseline check); earlier untracked-excluding diff gets only actual scope credit.

## Concrete local delivery / frozen return

Assistant local named commits authorized/Gitmetadata escalation required, no external action. Stage only named new file, message-file EXACT consecutive block (no separate -m paragraphs):
```
Implement save slot contract and proof

<concise actual change>

Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>
```
Exact command shape: `git commit --only -F /private/tmp/nms-slice26-message.txt -- crates/nms-cli/src/main.rs crates/nms-cli/src/saves.rs crates/nms-save/src/locate.rs crates/nms-cli/tests/multi_save_integration.rs README.md`. Options BEFORE `--`; every mutation uses dedicated cwd. Verify actual Git-parsed coauthors once/consecutive, exact5paths/currenttree/clean index, precommit→commit content identity. If own unpublished message repair needed preserve old object/tree/mapping, no source/runtime rewrite.

Ignored return `workbench/slice26-return/{execution.md,validation.json,closing-report.md}`, actual new sections atEOF, original intake prefix/hash preserved; retain all rawattempts/failures/retries/supportlimits. Use actual id and before/after.working_identity_sha256 keys; verify existing real records/argv/exit/end before green claims. Finish ALL metadata checks before declaring final frozen handback, no late file edits without explicit owner transfer. Proposed-done only: all6MS/R06-S26/M7.3 OPEN pending CDC independenteleven groups/wholeleafreconciliation/mandatoryretro/bottomup. Preserve all original/repair/raw/history/finalsemantic/freshness-import/game/platform/coverage/docs/distribution/performance/higherindependent/Operator/integration/release gates, main9line/heldexternalpins/samemodels/onearcendpoint. No modelcomparison inference or new authority.
