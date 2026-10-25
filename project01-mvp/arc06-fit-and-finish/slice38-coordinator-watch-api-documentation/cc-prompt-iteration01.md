# Slice38 first actual refinement — complete held documentation/oracle corrections

2026-10-25 UTC. FIRST ACTUAL refinement1/5/four remain; initial preserved. SameCDC6.1-solHigh/soleCCcc_mcp_docs6-lunaHigh/Arc06only. Actual88ba4800cd44402d9db567ed0bf5d0de format0/2cc7c8e65fcc4f568b9f9abd696bd736 lint0 fullraw/end/status/equalidentity354b229dba58dc625abc4461fec52a7994a77d9ffbe9e990d7d36e850816af60/diff93572cb5f766a17a593c9e5b3a1247a25fea965d6a58a71a531a54eb87684b47 directlyverified. Fullinitialtwo-file production/docblock/E1–E4 assertion walk closed, no runtime released. Exact initialheldpatch retainedDATA.

Only existing two allowlisted files' Rustdoc may change. Every non-doc/API/test/dependency/otherpath/currenttwo payloadlayouts FROZEN. E1 error-example code, E2 no_run code, E4 existing observer codefence byte-exact FROZEN. E3 remains same fixture/scenario/fullfour-note/keypopulation/metadata assertions; correct only independent expected projections and missing declared fields below. No new query/cache/native/test scenario or policy. No source repair or acceptance exception.

## F1 canonical public summaries/headings and actual panic

Rustdoc guide13 DC03/04/06 were read/applied; strengthMUST, not inapplicable headingboilerplate. LiveSaveCoordinator::start docblockcurrently has Errors→Panics→Examples. Reorder existing sections Examples→Errors→Panics, preserving COMPLETE E2code bytes and all useful failure/panic/startup teaching. Do not reorder function/body or invent newheading on infallible APIs.

Short single-source-line summaries BEFORE blankparagraphs, preserving detail after: LiveSaveCoordinator type `Owns one application watcher and serialized update worker.`; AppliedSaveUpdate.update field `Complete player state, additive delta, and source version from one parse.`; notifications field `Pre-apply movement and discovery notes in formatter order.` Current long firstsentences wrap; keep all underlying details in nextparagraph, do not delete semantics. names_changed/type/error/payload summaries already sound.

Add applicable `# Panics` on LiveSaveCoordinator TYPE: dropping it can panic if writing the joined-worker-panic warning to stderr fails (actual Drop eprintln). Do not claim successful Drop after workerpanic is universally infallible or add runtime repair. Existing blocking/join/in-flight/noqueuedflush/noharddeadline/modelguard-release caveats retained. Start's parser Panics section remains truthful; no extra ownedserver/runtime/subscriber panic invented.

## F2 exact subscriber ownership/no bounded-resource claim

Current subscribe says a disconnected RECEIVER is removed; registry actually owns SENDERS. Say: the sender for a subscriber whose receiver was dropped is removed when a later event is published. Do not imply coordinator removes/mutates caller-owned receiver. Add that stream carries applied events only, not watcher parse-error/worker-panic error messages; no promise of immediate disconnect on workerpanic. Source stores observer registry in coordinator as well asworker.

Qualify module/type/subscribe absolute `slow subscribers do not delay...` to actual channel property: publication does not wait for a receiver to consume; it clones each event and can accumulate an unbounded queue/memory cost for slow subscribers. No fixed latency/memory or bounded-backpressure guarantee. Preserve perworkerorder/afterregistration/inflight/noinitial/no replay/aftercacheattempt behavior. This is existing W1, not a new performance obligation or scenario.

## F3 complete already-issued watch semantics/navigation

notifications_for_delta docs: explicitly state scan name chooses FIRST exact-key matching incoming new_system before storedmodel, unlike movement's genuine-existing destination preference. This remains preapply text, not successful insertion receipt. Preserve every fallback/order/GG-qualified statement.

apply_and_notify docs: add actual existing model/session rules from W2: movement updates player only if playerstate already exists; physicalGG transition selects destination while sameGG keeps selectedview; session synchronization follows missing/player reference but preserves manual/base snapshots, biome filter and warp preference. No completeplayer/currencies/version/DiscoveryNames/ensure/cache guarantee.

drain_watch_events must link its actual teaching to `[apply_and_notify]` (E3 is on that function, not module), without inventing a module example. Original optional final legacydrain example may remain omitted; do NOT add newfile/output/cache scenario. Existing E4 is separate literal concatenation and staysbyteexact.

## F4 E3 independent full relevant address/anchor projections

Valid fixture, complete literal four-note vectors, system/planet/base key populations and actualname/biome membership assertions are sound; keep byte-exact unless below targetline necessitates address projection. No observed-derived expected values, droppedassertions or scenario substitution. Actual source core::PlayerState fields include `current_address.reality_index` SEPARATEfrom current_reality; SystemId fields private/gettersalreadycorrect. Galaxy.name is &'staticstr, GalaxyType Copy; existing E3 tuple name/type valid.

Before/after player projection MUST include BOTH addressGG and current_reality. Use exact shape:
```rust
let before = model.player_state.as_ref().map(|player| (
    player.current_address.packed(), player.current_address.reality_index, player.current_reality,
    player.previous_address, player.freighter_address,
    player.units, player.nanites, player.quicksilver,
));
assert_eq!(before, Some((0x2001_0000_0003, 0, 0, None, None, 7, 8, 9)));
```
After actualapply analogous projection and EXACT expected `Some((0x4001_0000_0003,9,9,None,None,7,8,9))`. Replace current `origin.packed()` / `destination.packed()` EXPECTATIONS with these independently pinned literals, not converter/fixture computations. Keep actual observation packed() methods.

Base existing lookup observation expands to `(base.name.as_str(),base.address.packed(),base.address.reality_index)` and expected `Some(("Outpost-9",0x4001_0000_0003,9))`; complete `["outpost-9"]` population stays.

Session projection MUST discriminate PositionContext variant and label, not addressonly. Exact observed optionalposition shape recommended:
```rust
session.position.as_ref().map(|position| {
    let kind = match position {
        PositionContext::Address(_) => "address",
        PositionContext::Base { .. } => "base",
        PositionContext::PlayerPosition(_) => "player",
    };
    (kind, position.label(), position.address().packed(), position.address().reality_index)
})
```
In existing full session tuple preserve selectedindex/name/type/counts/filter/warp observations, replace positionfield with shape above. FULL expectedtuple `(9,"Eissentam",nms_core::GalaxyType::Lush,2,2,Some(("address","0x200100000003".to_owned(),0x2001_0000_0003,0)),Some(Biome::Lush),Some(1200.0))`. This proves manualAddress snapshot GG0 despite physicalplayer GG9; no fixture-derived expected packedbits. All eight literal notes across pure/apply and exact keypopulations remain.

## Execute and hold

Coherent readback→exactdedicated-root+allowlist/currentbytes/count guards forEACHmutation/allreplacement counts in memory→scopedcorrection→rawfirstoffline format THENlint→IMMEDIATEsource/index/gates HOLD for targetedCDC review. No runtime/postpair edits untilrelease. Preserve all4currentCCrawvalues/initialpatch/history, no invented compiler failure (currentfinding is source-review only). Complete actualignored execution chronology. Originalninegroups/docsfirst/allfeature+no-default/currentactualzeroownedexact17residual/normalhooks exact2paths/uniqueaccuratemessage+parsedtrailers/completecanonical6row+3bubbleupreturn/fulloriginalowners remain. Initialprompt preserved, firstactual1/5/fourleft, no reset/model substitution/newrole/push/merge/install/network/release/otherarc/Project02.
