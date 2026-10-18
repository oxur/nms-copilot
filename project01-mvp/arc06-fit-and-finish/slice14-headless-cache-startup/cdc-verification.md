# Slice14 initial independent CDC review — 2026-10-17

## Current verdict

At88b184f999d4fde13e59b736dbd3a32896afd425 (direct parent16c0787), accept HC-01/02/03/06. HC-04/05/07 remain open for a bounded test/evidence correction. Slice14, Arc06 and Project01 stay open. Reviewed immutable initial cc-prompt.md SHA2561ab0abba4768262aecdb94dbaac1088baea48366244a4c7f53e755146bdbb61b, issuance d333d55/intake3c91b9b and CC closead71c79/receiptc3d1f87. Separate CC implemented; CDC authored no Rust. No waiver or production defect claimed.

## Reproduced behavior and current gates

All14 ordered gates independently pass after one retained restricted native failure and unchanged original-TMPDIR host retry. Fifteen reviewer invocations/30 complete raw channel objects, exact argv/cwd/environment/explicit offline and strict flags in artifacts/cdc-review-receipts.json. Cache38, route4, native6, focused746 across26 harnesses, no-default220, workspace942 across34 harnesses; zero failures/ignored on successful gates. Initial sandbox native2 passed/4 failed for loopback/notify restrictions; unchanged host6/0. Format/lint/build/doctests/ordinary docs/actual strict cache docs/separate source/planning diffs pass; existing docs/dependency warnings retained.

All eight nominal cases over each real stdio/HTTP transport reproduce,16 total with actual full tool/resource equality and case/effect receipts. Opposed source0/cache9 and Cached-9 label distinguish fresh current hit from source parsing. Missing/stale/old-schema/old-semantic/corrupt inputs rebuild complete literal source state and checked2/2 archives; explicit bypass serves source while preserving old bytes/absence. No new discovery-field semantics, normal REPL, cache live-write or full C04 claim. This green nominal path does not exercise the bind-collision retry branch described below.

## Required corrections

**HS-F01 / HC-04:** assert_startup_freshness_precondition at native test line1710 only handles initial_cache Some. The unique newly created directory makes missing paths absent by construction, but the explicitly assigned prelaunch absence assertions are omitted. Add destination absence for ColdMissing and BypassMissing and missing parent absence for BypassMissing before each launch. Existing after-RPC/after-cleanup checks stay intact. This is a required oracle correction, not evidence that production creates files during bypass.

**HS-F02 / HC-05/07:** run_http_cache_case lines1817–1822 prepares case files outside its retry loop. Production main loads/rebuilds the cache before MCP listener bind. A first bind collision may therefore rewrite a stale/schema1/semantic1/corrupt input into a fresh/current source cache; retry reuses mutated input. stale_current then fails its asserted stale precondition, while rejected cases may no longer exercise rejection. This is a source-inspected test-harness flaw; no actual collision was forced during CDC's green nominal replay. Prepare new independent original case files per launch attempt, carry the successful fixture/child/client owners out as one tuple, and prove the transition with one controlled real first-bind-collision stale_current regression. Observe original stale input, expected first-child source rewrite, actual bind diagnostic and pristine stale input on retry, then normal full source/RPC/cache effects. Production bind-error exit status can currently be0 because run_headless logs/returns; do not require a nonzero status or repair exit policy here. Reap the failed child safely.

HC-07 must append current correction/evidence/receipt scope and clearly distinguish16 nominal successful cases from the extra controlled collision scenario. Original evidence is complete and retained; no retrospective channel reconstruction or exception required.

## Source, records and row walk

CC archive has39 primary gate attempts/78 valid raw stdout/stderr channels, including initial lint compile failure, two native/broad passes and eight unchanged restricted/host retry pairs. Metadata/global+workspace offline configs and actual strict flag match; raw final harness totals38/4/6/746/220/942. Final case_observations array matches16 actual raw per-case tool/resource outputs;59 total native observation lines include existing phase/disabled samples. All28 protected hashes/two sealed fixture identities/ten held pins match. Source88b184f exact two allowed paths/direct parent/trailers once. Planningad71c79 directly parents3c91b9b, c3d1f87 parentsad71c79; four return paths only and trailers once. Prompt/plan/author baseline and all seven criteria/statuses preserved by CC. Reading/readback attestation checked against sources; it did not catch the retry/input interaction. No missing-current-evidence claim introduced.

| Row | CDC disposition |
|---|---|
|HC-01|done; actual binary/config/headless stdio/HTTP initialization/readiness/session/tool/resource composition reproduced for all16 nominal inputs.|
|HC-02|done; cold C/source9 rewrite and opposed fresh sourceA/cacheB9/Cached-9 exact-byte preservation reproduced with literal full state/maps.|
|HC-03|done; nominal stale/schema1/semantic1/current-empty-payload inputs, real freshness/error categories and literal source0/9 full checked rewrite reproduced. Retry preservation separately open under HC-05.|
|HC-04|open; actual bypass paths pass, required explicit missing-input prelaunch destination/parent assertions due HS-F01.|
|HC-05|open; all frozen runtime/prior native/required gates pass, HTTP test retry input isolation due HS-F02.|
|HC-06|done; README accurately describes tested startup cache defaults/path/bypass/recovery/physical-selection distinction and original follow-up limits.|
|HC-07|open; original scope/receipts/history/channels valid, correction's current proof and case/attempt accounting must be appended and independently reviewed.|

## Assignment, artifacts and bubble-up

Initial prompt returned/reviewed/preserved; active cc-prompt-iteration01.md, first refinement of five, only existing native test/prelaunch/retry helpers plus one focused controlled collision regression and four current return paths. README and all production frozen. Current full CC gates still required; four accepted rows may retain this independent proof only after exact delta/applicability review, with affected native/evidence checks fresh. CDC has not implemented the test repair or forced-collision sketch.

Supporting artifacts: author-baseline.json, execution.md, validation.json, cdc-review-receipts.json. Slice plan/ledger/initial+iteration prompts/CC close/this reviewer record are first-class files. No structural/scope/architecture/criterion amendment or inquiry; correction is within the same startup composition contract. R06-S14 remains open; arc32/13 done/19 open, project25 open. Planned15/CDC retains normal startup and full both-mode live-cache C04/write-through, unknown placeholders/promotion, duplicate producer/settings composition, broader historical questions and future final-baseline semantic transitions when raw repair lands. Original export/import/format/docs/distribution/coverage/R06-R02/R03/M7.4/C04/parent independent/Operator gates retained; main/dependency/Project02 held, candidate unintegrated/uninstalled, later authorized integration/composition/release CDC/Operator-owned.
