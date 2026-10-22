# Slice23 CDC current review and Operator decisions

## Independent current review — 2026-10-22 UTC

CDC gpt-6.1-sol High independently reviewed and reproduced the frozen CC gpt-6-luna High candidate. Source `133bc762c19b91222f7aeea4eb514696a22bede5`, parent accepted22 `753bb3a9a01451e57fff34262acd4ab194d99341`, tree `a34dd237734a293d06a9744147616ea7797dea51`, exactly23 allowlisted paths and once-only consecutive parsed trailers; worktree/index clean. No integration or release.

**Disposition:** NI-01 through NI-06 accepted within their issued boundaries. NI-07 remains open for the reproduced strict-doc gate and the separate original historical capture limitation. NI-08 remains open for final acceptance, closing retrospective and bottom-up closure. The slice/R06-S23 is unaccepted; required24 original export has not started. This is a reviewable stop at an actual Operator gate, not a waiver of red current proof.

All22 required gate groups ran independently:21 pass, strict docs red. Native13 pass independently, including both actual processes for rows10/12/13. Full workspace passed **1,062 runtime tests +31 doctests**, with separate31 doctests also passing. The two capture controls and two support syntax checks are not product acceptance groups. No production defect was established by the observed failures; prior source/test/controller errors and red/retried commands remain preserved.

### Current gate receipts

Every row below points to the actual raw entry in `cdc/attempts.json` in the review archive. All55 CDC records are complete with verified raw lengths/SHA256, actual exit/endUTC and explicit before/after HEAD/status/diff/untracked/working identity. All have clean committed133bc762 and stable identity `82efbe028156ccd1d56a8881cc1f34c0472aaf08297962488a9529fedbd9621e`. Raw record SHA256 `9216df68694e7ef77abe56284d5565528006030c9e0cfc4bd783cc2012c64872`.

| Required group | Actual UUID | Exit | Observed result |
|---|---|---:|---|
| make format | `a1eb834752cd4005ab3d2b1a1c6f1b1b` | 0 | completed |
| make lint | `b260f0c606ff43e9983c9d142b6c8c6b` | 0 | completed |
| discovery names | `cddb9695425d42478f8624086c54b0aa` | 0 | 7 harness passes |
| graph | `043579df75c64efda13d1f2bacdb991e` | 0 | 175; 7; 3; 7 harness passes |
| watch | `c6aa7982dfa147babed3441961075f2b` | 0 | 38; 4; 3 harness passes |
| cache load | `8b2054b9f5d2477a8e184882fd8206be` | 0 | 29 harness passes |
| CLI | `ae69552fb3d04418b8afbdd896139e72` | 0 | 68; 3; 34; 7; 1; 29; 5; 5; 3 harness passes |
| copilot all-feature library | `326954e4c05e4c9db93199ac8fce1963` | 0 | 235 harness passes |
| name observations | `18fc912b4ff44c91be1f4074066f7fda` | 0 | 7 harness passes |
| query and route cache | `403549d9ba8f4aa687b5dd739fad4226` | 0 | 2; 4 harness passes |
| headless MCP | `5d309c4bb2e048338fe4d09d34a817b7` | 0 | 23 harness passes |
| owned MCP | `b9ba1613bb504bc9a7d1f8593ee694f3` | 0 | 8 harness passes |
| copilot no-default library | `20d321f7a7794152837dbd1c721db9ff` | 0 | 234 harness passes |
| binary and probe build | `4e082e682d4443deb9f476e42f4593a3` | 0 | completed |
| probe self-check | `103c0d1ce523446db77e8cd3ba69f08f` | 0 | completed |
| real CLI name smoke | `5c0b3d7533b54dbca016391d72c1d73f` | 0 | four actual children; full nested receipts |
| full make test | `327bc726b85e460ab2b8f8612629fbb1` | 0 | 1,062 runtime +31 docs; 42 harnesses |
| workspace all-feature build | `de907c29a67e4fbe9ee1b6f7df62a173` | 0 | completed |
| workspace all-feature doctests | `ff971d668e2e488f980c6ed1f215cf34` | 0 | 0; 2; 0; 4; 1; 7; 0; 14; 3 harness passes |
| strict docs current failure reproduction | `4c6ce8977c144048a33bf4e06a593f7b` | 101 | RED: inherited config.rs:288 link |
| working diff check | `90d92c8db5484481b8039add6c6fe4a0` | 0 | completed |
| baseline diff check | `652ccd8d7c364a36aae9a4e50310c246` | 0 | completed |

Capture negative control `c6040dffe66b4ce3b1c322245ebd0ee2` actually exits7 with10-byte stdout/10-byte stderr; positive `21d6bce7e55e4b7c9950cb4a27f0089a` exits0 with9/0 bytes. Native support syntax checks `82068d36b89041f7a9089786b8fa37fa`/`d67bd62cef544789b7ef08fba69f02b3` both0, before first native launch. No product/native credit comes from these controls. Established host filesystem/loopback authority and the original TMPDIR were used; source remained stable.

### Actual independent native13

| Row | Profile | Case UUID | Full launch/raw/post triplets | Result |
|---|---|---|---:|---|
| 1 | warm/enabled | `8964745d01544ce0ba3e0f6bd21c4279` | 27 | pass |
| 2 | warm/enabled | `7c277808e4f3480bb854619628f3f9bf` | 10 | pass |
| 3 | warm/enabled | `e5d0b5704cb94000b4c1e66545964478` | 10 | pass |
| 4 | cold/enabled | `649f5ce55ea34477b558d12aad329b25` | 10 | pass |
| 5 | sentinel/enabled | `313ba6c98b4b41998ee699d200c3959e` | 10 | pass |
| 6 | bypass-missing/enabled | `7e9c58ad25d4496bb718834905ea279a` | 10 | pass |
| 7 | warm/disabled | `25f477f706d94e7b8cf9ea4169deac4d` | 10 | pass |
| 8 | blocker/enabled | `cb0fb5346d334d22bbb7b5420ea49a11` | 10 | pass |
| 9 | warm/enabled | `7575ae03fae645b1849bca591047954c` | 7 | pass |
| 10 | unknown/enabled | `0e0b9449616944618f58d0e673b9c5bb` | 33 | pass |
| 11 | predecessor/disabled | `bd1e680f3aa84b958a0a3abdfcf375f6` | 11 | pass |
| 12 | warm/True | `08c49f1cf79646d496f224cfdce72cbf` | 46 | pass |
| 13 | predecessor/False | `f0ccc5c97d114993a882b090cb7774dc` | 22 | pass |

There are216 complete native tool triplets and916 native receipt files, including13 immediately persisted prepare tool objects. Raw terminal streams are explicitly multiplexed native PTY observations; no separate native stderr is invented. Rows1–3 preserve quit/Ctrl-D/Ctrl-C. Row1 preserves phase5→five separately acknowledged bad writes→phase6. Row9 observes expected actual bind exit1 and listener exit0. Rows10/12/13 preserve both process exits0, observer EOF and pure file/cache boundaries. Row12 sets Anchor, tests actual named find, temporarily selects0 for GG0 show, restores9, and verifies the manual/selected anchor before subsequent writes. Row13 observes genuine3/4 rejection/rebuild3/5 then a second compatible restart against older malformed source with watching disabled. Live typed graph state is separately proved by the in-process coordinator readguard tests; cache-rebuilt topology is identified as cache-derived.

Actual prelaunch hashes are unchanged for `nms` `034dc2e84faba934703309b1ef522efffcd57e43701952325ee812d7a2c31997`, copilot `d36ed4a5aced887640e59b662c67402a3a68c62c408e3aa41599c0e26a121e5d`, probe `b858335675490d6e4f0a5f7f0aa82e82c35a0c87efd95784815812c088ed4a92`. The CDCv4 helpers preserve the CC scenario/assertion/timing/order bytes apart from explicit namespace/path changes and the required initial prepare-result persistence. Originals are retained; actual support bytes/digests are archived.

### Row walk

| Row | Disposition | Independent boundary |
|---|---|---|
| NI-01 | done | Checked cold names, exact literal source keys and legacy unnamed compatibility; graph names7/fullgraph and actual four-child CLI smoke. |
| NI-02 | done | Same checked parse carries whole state/version/name maps, rename/clear/omit/ignored controls, legacy quiet, ordering and bounded idle/queued/blocked cancellation; watch42 runtime+3docs. |
| NI-03 | done | Actual live readguards, whole model/raw cache/rebuilt model/index/marker/edge/base/player/selection invariants and discriminating negatives; names7,235library and selfcheck. |
| NI-04 | done | Actual stdio/HTTP whole rows/resources/stats, no false movement, five invalid/recovery/identical, whole retry scenarios, both restart/EOF/file predicates; headless23/owned8 plus native named profile. |
| NI-05 | done | Schema3/semantic5, authentic224-byte old3/4 and earlier predecessor controls, strict freshness and actual native13 including two-process named/rebuild cases. |
| NI-06 | done | Issued API compatibility and complete changed-code/helper/README/public-doc content review, existing regressions and literal policy. This does not accept the separately required strict-doc gate, tracked in NI-07. |
| NI-07 | open | All current evidence is attributable, but strict docs remains red and one original support diagnostic missed the issued first-attempt capture contract. Decisions below. |
| NI-08 | open | Final slice acceptance/closing retrospective/bubble-up wait for NI-07. Interim retrospective observations are recorded separately. |

## Decision1 — one additional doc-only pass

Independent `4c6ce8977c144048a33bf4e06a593f7b` reproduces CC `5a22629a1c3442a4901a88b46c69cb6d`: exit101, unresolved link to `effective_save_file`, at `crates/nms-copilot/src/config.rs:288:24`. The original exact command is:

```bash
RUSTDOCFLAGS='-D warnings' cargo doc -p nms-graph -p nms-watch -p nms-cache -p nms-copilot --no-deps
```

The unchanged file is byte-identical to accepted22 and mainb616: Git blob `4798006d32c5fcce41635f05cb40ddf8489b83a1`,19867 bytes/SHA256 `6d3046d3e4e2cc6984a5f3db50e2a17f40c8764d3aaab6612c7c7c8aba98d391`. The target method exists in the same Config impl at line239. The proposed **entire** correction is:

```diff
--- a/crates/nms-copilot/src/config.rs
+++ b/crates/nms-copilot/src/config.rs
@@ -285,7 +285,7 @@
 
     /// Resolve the effective save path (if configured).
     ///
-    /// Delegates to [`effective_save_file`]. For backward compatibility,
+    /// Delegates to [`Self::effective_save_file`]. For backward compatibility,
     /// also checks the legacy `save.path` field.
     pub fn save_path(&self) -> Option<PathBuf> {
         self.effective_save_file()

```

**Recommendation:** approve ONE exceptional sixth pass, only this comment qualification in this one additional path. All executable bodies, other23 paths, fixtures, controllers, tests, assertions, flags and timebounds stay frozen. No warning suppression. This has not been applied; no iteration06 is issued.

After explicit approval: record it before execution, issue immutable06 and amend the allowlist only for this line/path; same selected CC readback/implementation; captured format→lint→exact strict-docs command→working/baseline diff checks; named-path local commit and freeze. CDC verifies the exact one-comment delta and independently repeats format/lint/strict-docs/diff verification, including the resolved method link. Current full runtime/native evidence remains applicable because executable/test/controller bytes are identical. Do not repeat unchanged broad tests/native or waive the red; repair and prove the actual gate. Any further substantive correction stops again; this override does not reset the five-pass budget.

**Hold alternative:** keep source133bc762 and NI-07/08/R06-S23 open, with no dependent24 dispatch. Reworking scope/context requires an explicit choice; it is not used to bypass the cap.

### Exact rule causing the stop

The applied [work-verification skill](</Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/work-verification/SKILL.md>) routes ledger protocol through [guide01](</Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/work-verification/guides/01-ledger-discipline.md>) to the retained [full protocol](</Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/work-verification/templates/LEDGER-DISCIPLINE.md:252>):

> Reaching 5 without convergence: stop. Do not iterate a sixth time on the same ledger in the same session.

The same protocol explicitly says, “An operator may override; keep an open mind, but give feedback if the override doesn't seem justified.” Five immutable corrections01–05 were used. A one-line inherited documentation correction is a proportionate override because all current functional proof passes and no production repair is proposed. No sixth pass is inferred from standing Autonomous authority.

## Decision2 — separate first support-diagnostic capture exception

This is ONLY the first private CLI-controller syntax check, before the first captured compiler pair. The actual known command is `python3 -m py_compile /private/tmp/nms-slice23-cli-name-smoke.py`, in the dedicated source checkout; its direct shell first generated/chmodded the helper. Known tool settings were yield10000/maxoutput3000. The outer JavaScript printed only `r.output`; the surfaced outer result is Script completed with an empty display string. This does **not** establish nested exit0 or separate empty raw channels. The original CC record retains the reported initial helper SHA447daaf9… and prefix/suffix; the full original embedded wrapper body/call ID/nested object are not retained. Current archived helper bytes are the later actually exercised version, not that original version.

The binding issued [initial prompt, capture section](cc-prompt.md#capture-and-ordered-holdrelease) says: “UsefromFIRST actualcompiler/gate/diagnostic, eachUUIDimmutable, explicitargv/cwd/env/originalTMPDIR/beforeafterHEAD+status+completeidentity/rawstdoutstderr/exittimes.” A private py_compile is an actual syntax diagnostic; the ordinary inventory exception does not exempt it.

| Required historical observation | Exact remaining limitation |
|---|---|
| UUID/preregistration | No contemporaneous kernel UUID/preregistration was created. The descriptive first-syntax-check label is not a fabricated historical UUID. |
| Environment and start | No recorded inherited environment/originalTMPDIR or startUTC for this attempt. Known cwd/argv/settings above remain retained. |
| Before source | No contemporaneous beforeHEAD/status/complete working identity (diff/untracked inputs) was captured for this diagnostic. Earlier/later workspace observations do not replace it. |
| Raw completion | Nested exit, separate raw stdout and stderr, and endUTC remain unknown. Empty outer display is retained as an outer display only. |
| After source | No contemporaneous afterHEAD/status/complete working identity was captured. No later identity is relabelled as historical. |

Missing chunk/session identifiers, full original wrapper text and nested wall time remain supporting limits; no additional blanket field requirement is invented for them. The current real CLI smoke and independent four-child run prove current behavior, not the original py_compile result.

**Recommendation:** explicitly accept a bounded historical-capture exception for ONLY the absent issued observations above, retaining unknowns and giving the original diagnostic no acceptance credit. This is independent of Decision1: approving the doc-only pass does not approve this exception, and approving this exception does not accept the red strict-doc gate or any future missing record. Prior15–18 exceptions do not cover23.

The work-verification guide requires every row to reach a final status before advancement and to obtain an authorized amendment rather than select evidence to fit a changed criterion. NI-07 therefore remains open until an explicit disposition. No product/native observation is missing: all83 archived CC entries and55 CDC entries retain their actual complete fields.

## Return reconciliation and preserved history

The immutable initial/01–05 prompts and the byte-exact CC return remain unchanged. The CC raw83 record SHA256 is `843c98a03132d36a58a9e2f714f4f8a692628c0cf507cc0ffeb9be16e2d44165`; every archived entry's channels/exit/end/status fields verify. CC summary before_identity/after_identity are null on all83 because the summary looked up the wrong key. Actual raw `attempts[i].before.working_identity_sha256` and `.after.working_identity_sha256`, plus status/HEAD/diff/untracked inputs, are retained. For example the two author controls have actual0b70c172 identities; final precommit entries have990eab17. This is a summary representation defect, not missing historical observations. No waiver is requested for these null summaries.

A narrow lookup of the supplied new-native syntax-history path found an auto-review mirror containing the already captured fafb73a0/17c4c460 calls, not separate earlier direct calls. The unsupported earlier “passed” message/subsequent summary claim is preserved and corrected; no additional attempts or missing-field exceptions are manufactured from it. Current namespace-v1/v2 syntax checks remain separate complete actual records. The confirmed first privateCLI diagnostic above remains the only historical capture exception proposed.

Preserved product failures include compiler/Clippy repairs, restricted watcher29pass/9timeouts then identical host42runtime+3docs, initial headless21pass/2 literal-oracle failures, restricted corrected headless6pass/17substrate failures then host23pass, the real CLI child exit0 followed by its private table-width assertion, and both current strict-doc failures. Search expected P0 portal was corrected while player P1 stayed unchanged; the short private negative-control label preserves its exact keys/unnamedFrozen/GG9 decoy. Original prefixes, JSON values and attempts are not rewritten. Current code was independently reviewed at the settled pins; no production issue was found.

The committed19 tracked-path patch exactly reproduces precommitc50e2934; all four formerly untracked committed files reproduce their recorded byte lengths/SHA256. Commit identity is not relabelled onto earlier working-tree receipts. All216 CC native tool triplets/916 receipt files and216 CDC triplets/916 files remain attributable. The CC archive contains receipts/controllers/capture record; its closing-report phrase about ten fixture trees is not used as an archive-content claim. The new review archive separately inventories and preserves all13 actually available CC fixture snapshots and13 CDC snapshots, plus the failed/successful CLI fixtures. They are current archival snapshots, not recreated earlier file observations.

## Evidence and retained boundaries

- Byte-preserved `cc-execution.md`, `cc-validation.json`, `cc-closing-report.md` and `artifacts/cc-native-evidence.tar.gz` (366592 bytes, SHA256 `e02b8493020a15580667945bf2db307641cc96e5fb18f5493bf597adf41351ca`).
- `artifacts/cdc-review-evidence.tar.gz`: 383874 bytes, SHA256 `e95ccca92992b44a6047f6d8946e6d4ff9cd013e1075b741978965d6874bd651`;1350 actual input/evidence files and103 directories, full per-file hashes in `inventory.json`, fresh source/main/held-pin observations in `source-state.json`. Includes current CDC55raw records/native916 files/actual support inputs,26 native fixture snapshots and three real CLI fixture trees. Inventory excludes itself; current snapshots are labelled as such.
- `cdc-retrospective.md` records the current review's observations; closing disposition remains pending. No causal model comparison or new inquiry is claimed.

Arc41 rows/22done/19open and project25open remain unchanged. All15 original capability rows/three repair parents and original export24, raw/history/final-semantic/freshness-identical-rewrite-stale-import/import/format/UI/docs/distribution/coverage/performance/game-backed nonzero/higher independent/Operator/integration/release gates remain. Mainb616 has ONLY the protected nine-line human native-test edit/blob2cd450ce, empty index. Accepted/held worktrees and external03e8d04 remain protected. Same CDC/CC models/one-arc endpoint; no source or helper changes after this proof, no06/24 dispatch, no merge/push/install/publish/release or next arc/Project02.
