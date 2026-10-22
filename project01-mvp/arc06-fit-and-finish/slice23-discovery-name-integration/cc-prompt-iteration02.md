# Slice23 iteration02 — finish the closed helper/oracle boundary

2026-10-22 UTC. Second ordinary correction, 2of5; three remain. Preserve initial and iteration01 exactly. Same actual CDC gpt-6.1-sol High / sole CC gpt-6-luna High, Autonomous Project01 Arc06 ONLY. No production defect or runtime result is established. No new policy, scenario population, role/model or dependency.

Source ONLY `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/discovery-name-integration`, HEAD753bb3a9a01451e57fff34262acd4ab194d99341/indexempty, frozen tracked diff9e33549cacaf69fac02f820a79d18fa347f2eaf4ca142d27c55faeee12c28d45/composite2e2885630da39c1fff76f048c625f145adfa4c37cf0454b5beb9a287fca25351. Actual final format ae7f0bb624e149c3b36d4a7760618aeb/lint2c03c00975af44e899e92e8745c8a1ea both0, complete raw/status records. Raw log18entries, prior failures retained. CC's initial report described 17 wiring errors; actual stderr reports17 probe errors plus repeated shared-helper compile errors across integration targets. Preserve original report and raw diagnostics, no invented denominator. Runtime/build/selfcheck/native have NEVER been released.

Revalidate pin/emptyindex/protected main9line blob2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e/heldpins/no processes. Read this entire batch and sibling plan/manifest, append attributable readback/currentassignment/start, then take dedicated source/index/gate ownership. This prompt authorizes start after coherent readback. CDC owns planning. EVERY write uses absolute prefixed manifest-checked targets; no main writes.

Only SEVEN tracked paths may change:
1. `crates/nms-watch/src/watcher.rs` TEST MODULE ONLY.
2. `crates/nms-copilot/tests/name_observations_integration.rs` literal live edges only.
3. `crates/nms-copilot/examples/repl_live_probe.rs` current new-name helpers/self-check corrections below, preserve original11 native cases/strong controls.
4. `crates/nms-copilot/tests/support/live_mcp.rs` new paired-SSE mock/self-check cleanup only, existing consumed predicates/legacy wrappers frozen.
5. `crates/nms-copilot/tests/mcp_live_integration.rs` new stdio names EOF/file boundary only; the reviewed HTTP loop/attempt/receipt implementation and full query-row oracles FROZEN.
6. `crates/nms-copilot/tests/owned_mcp_integration.rs` missing actual Stats subscription only.
7. `README.md` two explicit adopted-policy clarifications below.

Private CLI controller `/private/tmp/nms-slice23-cli-name-smoke.py` and corresponding private native command recipes/attributable return files allowed. ALL other current source bytes, ALL executable production bodies, public API/codec/constants, literal source/expected support, graph/cache tests, tools description, dependencies and genuine fixtures remain FROZEN. Overall23path manifest unchanged. Do not repair unrelated wording or broaden proof.

## S1 — complete already-required guard bounds

watcher.rs blocked owner-drop now has a bounded dropper/completion fallback, but idle/queued call drop(handle) synchronously. Apply a common bounded drop helper to these existing cases: keep an external clone of shutdown, move the real handle into a dropper thread, observe completion with recv_timeout(1s) before join; install failure cleanup BEFORE first assertion. On timeout set external shutdown Release and disconnect/drop the held receiver, then join and fail the bound. Idle dummy worker currently loops only on shutdown, so receiver disconnect alone cannot release it. Preserve all queued retained-map/version/disconnection assertions, and direct sender order/cancel checks. No production WatchOwner changes, no detached threads or timeout weakening. Clear/omission/recovery actual observation assertions should retain complete literal PlayerState/save_version/emptydelta/full maps, filling the remaining units-only/missing-field assertions from the same fixed source (current(0,0,0,SSI1,P0,GG0), previous/freighterNone, units7/nanites8/specials9; recoveryversion4721). Do not invent a different fixture.

live_mcp currently has TWO new SSE mock implementations; both block accept/read and the callable one can return via ? without joining. Consolidate them into ONE fixture owner used by the callable check and thin unit wrapper. Concrete shape: owner retains command sender, stop Arc<AtomicBool>, JoinHandle and completion receiver. Listener is nonblocking; accept loop checks stop/command close and a finite deadline, handles WouldBlock with short bounded polling. Accepted stream gets read/write timeouts before header I/O. Its Drop FIRST sets stop/sends close/shuts down an accepted socket when available, then observes worker completion before join; timeout cleanup must release accept/header/command waits, never join an unreachable blocking accept. The object/guard exists before connect or any assertion/? path. Keep diagnostic errors as failures, no broad suppression. Existing stream Drop remains unchanged.

The SAME callable paired check must cover initial location+Stats one-chunk consumption/history quiet, fresh quiet rejection for EACH of the two URIs, pending-update EOF rejection for EACH URI, and clean EOF acceptance. Iterate the literal two-URI list with fresh owned fixtures per EOF case; preserve original old cursor/biome tests. Do not count only a Stats EOF as both-URI proof. Unit test may call the callable helper rather than maintain a second divergent mock. Existing2s/20ms test bounds stay; no product server or extra native row.

## S2 — valid raw marker control and independent live edge literals

The new Recover-stage raw marker negative sets is_player_placeholder=true on a named row with metadata/planet, which checked write_cache correctly rejects BEFORE the consumed raw/rebuilt oracle. Replace that one marker case with a SERIALIZABLE Initial-stage control: fresh expected_cache(Initial,7), independently expected_model(Initial), flip only A0 marked true→false, write/read actual file, rebuild actual, require the complete predicate to reject. This models a genuine empty row versus marked unknown; it preserves all other literal fields. All remaining raw negatives retain Recover and actual checked disk readback. Do not treat a codec error as oracle rejection and do not weaken placeholder validation.

Add the omitted derived marker control to the existing model-negative walker: fresh observed expected_model(Initial), confirm A0 is marked, ordinary insert of the identical empty System::new(a0(),None,None,None,vec![]) clears marker, confirm unmarked; compare valid Initial raw data plus that observed model against independently expected Initial model and require rejection. Retain all11 current raw and11 current derived controls plus this missing marker case; counts/messages must report actual cases. Existing old self-check controls remain semantically intact.

name_observations_integration currently pins A0 graph index/weight/point but never pins the issued literal edges; both sides otherwise call build_edges. Add an assertion on the ACTUAL live READGUARD in assert_stage: enumerate actual edge endpoints using graph node weights, canonicalize each endpoint by (reality_index,system_address), sort the full multiset, compare exactly three independently literal rows A0–B0=400.0, A0–C0=800.0, B0–C0=400.0, with no GG9 edges or extras. Use f64::to_bits for exact weights and literal a0/b0/c0 address constructors. Preserve full validator and A0 index/spatial persistence. Do not compute expected distances via the distance/build_edges algorithm under test.

## S3 — restore the complete normal phase/EOF contract

Current normal warm command table includes recover but its match omits it; omit also calls a helper that ALWAYS waits for Stats. Restore the exact action table using Option<bool> or equivalent:

| command | stage | notification expectation |
|---|---|---|
| overwrite | Overwrite | location + exactly one Stats |
| promote | Promote | location + exactly one Stats (new discovery delta) |
| rename | Rename | location + exactly one Stats |
| clear | Clear | location + exactly one Stats |
| omit | Ignore | location ONLY; both-URI quiet afterward |
| recover | Recover | location + exactly one Stats |
| null/nonstring/nonobject/nonobject-dm | Ignore | both-URI quiet |
| invalid1..invalid5 | Ignore | both-URI quiet |

All accepted Some events refresh allowed cache stamp AFTER publication, including omission. Ignored/invalid/identical preserve cache bytes+mtime. Existing identical/finish branches remain separate. Keep the explicit nonobject CN and nonobject-DM phases; private controllers must include BOTH.

Current finish handling performs a live RPC boundary before waiting EOF, but controller enqueues finish then quits concurrently: that RPC can race the stopped server. Handle finish BEFORE the per-action live-RPC call. Its pre/post checks are PURE file/cache, then SSE EOF/join; previous acknowledged phase or initial ready already supplies the last live observation. No post-quit HTTP call and no weaker model/cache predicate. For disabled predecessor first monitor and BOTH compatible restart monitors, observe a full650ms both-URI quiet window while server alive BEFORE publishing ready, then finish/quit/EOF. This implements the original ready→quiet→finish sequence without a new handshake or terminal method.

verify_names_restart must explicitly require source bytes equal the fixed malformed literal br#"{"Version":slice23-restart-opposed"#, cachemtime>source_mtime and is_cache_fresh, actual3/5 header, disabled exact config/mode and whole expected profile. Checkpoint equality alone is not an independent older-malformed fixture assertion. Preserve row12 selected9/imported C0 versus row13 selected0/three systems/A9unnamed.

self_check must also exercise authentic224B3/4 typed rejection(found4 expected5) and actual current3/5 name golden headers; currently it only explicitly rejects the old752B3/3 predecessor. Preserve both genuine fixtures/old negative controls, no header patch/payload inference. Existing file-negative controls and all full raw/derived comparisons remain.

## S4 — actual subscription and stdio EOF files

owned_mcp_integration still calls subscribe_sse(location-only) before later waiting for Stats. Use subscribe_sse_uris with BOTH literals before any cursor/update. Keep the existing old location-only quiet checks and full unchanged read-only model assertion; legitimate Stats from earlier contradictory new-discovery events can be consumed by their location-only quiet wrapper as before, and names_cursor must be taken after that consumption. New name flag event still requires location then oneStats and both-URI quiet. No observer production edit.

New stdio lifecycle still lacks the required first-process post-EOF source/cache/config/mtime/temp assertions. Before close_stdin record literal recovery source/time, recovered cache bytes/time, exact enabled config/path; after complete NDJSON EOF compare those SAME observations plus source/cache temp absence and parent existence. Restart: checkpoint malformed source/time and exact disabled config before spawn; after EOF compare source bytes ANDmtime, cache bytes ANDmtime, config and temps. Keep actual full stage/query oracles, two actual children, stdout EOF and quiet checks. Existing HTTP successful EOF boundaries already contain these checks and stay FROZEN; do not add another HTTP rewrite or scenario.

## S5 — exact CLI literal/raw-first and two doc sentences

Private CLI source C0/P3 uses VP[1]=8 (Red) but asserts Frozen. Change ONLY that intended GG0 unnamed negative planet VP[1] to4 (Frozen), preserving the GG9 decoy, names/address/seed, unfiltered eligibility/show and named exclusion positives. No expected string relaxation.

In run(), completion dict currently hashes the fixture BEFORE persisting returned child channels. Write exit/endUTC/rawstdout+stderr completion FIRST; then add post-fixture hash/other metadata and persist again. Timeout facts likewise persist first, only actually observed facts; subprocess.run's bounded timeout cleanup stays. Keep prelaunch helper/binary/fixture inputs and retained failure folder. No historical receipt reconstruction and no uncaptured py_compile.

README explicitly add that exact-key source string ownership can overwrite imported labels, and that cache rebuild can lose CACHE-ONLY LABELS on otherwise retained source entities as well as whole absent-source imports. Also state accepted name-only changes refresh location and invalidate Stats once when labels/counts change, after shared model/cache update; omit/no-op does not imply false discovery/warp. These are the two affected policy areas, concise wording only, no claim export24/game-backed universality. Current set/emptyclear/ignored/exactGG/firstrow/planet-named wording stays.

## Sequence / evidence / stop

Full settled F1–F7 walk is complete; this batch lists demonstrated remaining defects/omissions in the original closed boundary. Valid cache primitive/schema controls, independent legacy literal, genuine nonobject source, unchanged-name currency discriminator, complete query rows/stats normalization and HTTP max-three Result structure are preserved. No source runtime failures are invented from this static prediction.

Read back exact seven paths/frozen production/S1–S5/current budget; then implement only this batch, complete whole affected-helper/assertion/callsite walk, captured make format→make lint, and HOLD source/index/gates/returns for CDC bounded review. Compiler repairs within exact active test/helper scope retain all actual attempts/renew pair. Do not launch tests/build/self-check/native yet. Existing raw-first UUID/status/source/originalTMPDIR capture kernel applies from first actual attempt. Ordinary inventory remains metadata. Preserve all prior claims/corrections/records and the separate unavailable historical private diagnostic facts, no waiver inferred. After CDC release, ORIGINAL ordered runtime/native13/full gates remain, no weakening or unnecessary repeats. No sourcecommit until authorized original return sequence; exact named paths/two consecutive parsed trailers/local-only delivery as before.
