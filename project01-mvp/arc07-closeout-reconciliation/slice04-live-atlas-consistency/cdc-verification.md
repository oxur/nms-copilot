# Slice04 — independent CDC disposition

2026-10-25. CDC SolHigh reviewed source `8f10d4cbe04f7d781667cc4e37280c333a67ee8d`
and CC records `d1514db4aaff9492b97292b74be1ca1384779a42` /
`39cf2998efec4e725b016c26f9cc09ce15d0019c`. **AT-01/02 accepted for the scoped
behavior. AT-03 remains open.** This is a reviewed implementation candidate,
not successful whole-workspace, feature-matrix, Arc07 or Project01 acceptance.

## Source, oracles and reproduced checks

Inspected all seven changed paths against fb2ecebb, the pertinent complete
snapshot/render/terminal loop, production dispatch/capture, coordinator and
cache ensure paths, graph constructor/insert/update/edge/route contracts and
changed test bodies. No additional path, dependency, manifest, archive schema,
low-level graph API or routing algorithm changed. The private owned MapState
and address vector retain no model/lock reference and avoid cloning the graph.
The actual main dispatch uses the capture helper before entering the modal
runner. The public borrowed entry and renderer remain compatible wrappers.
The terminal guard and input/error precedence remain unchanged.

The default-edge refresh belongs to application boundaries and runs once on
system growth; additive updates do not delete systems. Supplied custom edges
are intentionally replaced by default KNN10 on growth, as the new public
coordinator docs disclose; no-node updates preserve their topology. Cold,
no-cache and compatible warm missing-player repair perform the same refresh.
Raw supported cache records and schema4/semantic6 are unchanged; edges were
already derived on warm rebuild. Staged zero-edge expectations were corrected
to literal edge endpoints/weights, preserving raw metadata, no-write/error,
foreign-galaxy, no-player and genuine-promotion controls.

The actual baseline topology opposition is CC-attested: after correcting an
identity-order oracle, route/reachability matched while live edges had only
A–B1600ly versus full rebuild A–B1600/A–C800/B–C800. **Reachability did not expose
a runtime difference**; the closing report's phrase grouping reachability with
topology as exposing one is inaccurate and is not accepted. The route fallback
and spatial reachability remain their documented behavior. The new literal
800ly route and identity-sorted reachable set, exact edge set and GG9 decoy
test are discriminating. Independently checked the cache witness distance:
sqrt(1017²+1352²+3²)×400 =676721.744884853ly.

CDC executed these exact commands on the unchanged candidate; each exited0:

```text
cargo test --locked --offline -p nms-copilot --all-features --lib map::
cargo test --locked --offline -p nms-copilot --all-features --lib coordinator::full_update_component_tests
cargo test --locked --offline -p nms-copilot --all-features --bin nms-copilot test_production_map_capture_releases_model_lock
cargo test --locked --offline -p nms-cache --test load_integration
```

Results respectively41/41,4/4,1/1,31/31. These include the existing injected
guard/input/render cases, owned mutation/render parity, actual production
capture lock release, literal live/rebuild/no-node graph cases, cache
predecessors/current warm repair, promotion and negative controls. CC's final
format→lint, scoped no-default library/cache and broken-link rustdoc/doctest
checks are attested in result.md; CDC does not mislabel them independently
reproduced or broader than their actual filters.

## Independent owned-PTY observation and real runner failure

One CDC direct Python stdlib PTY/CPR run used a copied fixture, isolated scratch
HOME/config/cache, enabled watcher, owned loopback port56183 and the CC-built
normal HTTP binary. Binary SHA256:
`30b5a5709635a774bdc1d24f8eec5406bdf7efc023a179299c73edc6bef30302`.
MCP initialize/initialized and actual where_am_i responses observed X100→150
after replacing the save while the modal remained open. The cache file hash
changed before map exit; this observes persistence progress, while the31 cache
tests establish decoded semantic parity. A forced100×32→90×28 redraw still
printed the entry cursor X=-1948, not the changed-entry X=-1898. No exact-cell
screen reconstruction is claimed. q left the alternate screen; ordinary
status printed the position label; the application printed **APP_EXIT=0**.
At the retained owned-shell boundary, full before/after termios arrays matched,
including lflag0x5cb. There was one alternate enter/leave and nine CPR replies.

The product observations above completed, **then the runner failed** waiting
five seconds for its retained shell to exit. Its cleanup killpg attempt raised
PermissionError. The tool/receipt success is false and is not promoted to a
green runner. Closing the PTY ended the shell; one subsequent native ps search
for the exact scratch nonce found no remaining process. No retry or driver
construction followed. Actual retained files:
`/private/tmp/arc07-cdc04-vku1zirv/{actual-pty.bin,receipt.json}`. Raw stream8929B,
SHA256 `fb0701febb371a41a9b425166b929b649eb4f2f83a6bdaab9e4c83e9874db4ce`.
This separates observed normal app exit/restoration from later harness cleanup.

CC's first native attempt observed X100→101/map-open/MCP and q→ordinary REPL,
then failed its incorrect status-coordinate predicate. Its corrected attempt
failed CPR before map entry. Both remain attested in native-smoke.json; that
file is a summary, not raw terminal bytes. Its occasional “shell returned”
wording means REPL return only. CDC's direct observations supply the missing
normal app exit and restoration proof without claiming either CC runner passed.

## Open proof and exact limits

CC reported make test247passed/7failed, classified as restricted-sandbox watcher
timeouts, then native coordinator13/13 and retained-watcher1/1 passes. The seven
individual failure identities and complete original command transcript are
**unavailable after CC compaction**; they are not inferred from likely names.
The targeted native populations do not establish whole-workspace success or
prove every original failure's identity. No forensic recovery or broad04 replay
is commissioned. **AT-03's whole native-candidate gate stays open for07**, which
must run its actual final explicit commands and retain that result independently.

The full no-default package command/transcript is likewise unavailable in the
CC context. Source inspection at fb2ecebb independently confirms the reported
pre-existing HTTP cfg blockers: main's unconditional background spawn and
Transport::Http, plus the integration test's gated ExitStatus import. Library
subset success cannot close that package criterion;07 retains it. No platform,
MSRV,95%/module, performance, genuine-game/nonzero input, packaging, higher
acceptance or external-action gate is waived here.

Old writer was interrupted before the serial LunaHigh continuation. No formal
returned source revision was issued; this does **not** mean zero actual initial
corrections: reachability order, no-node SSI/import readiness, cache fixture/
order, staged old-edge expectations and native predicate/CPR failures remain
disclosed. Serial context replacement did not reset the assignment or budget.
No further04 refinement or capture work is opened.

Source is clean at8f10d4c. The source/evidence commits have both required
consecutive trailers once. Main's protected whole-file blob remains2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e;
old49 remains untracked/unissued with SHA256d9600d9cddf3f9583fc1768cd67644b1e32ec6926b8f1f298b2982289de30986.
Proceed only with independently ready agreed05 work;04's validation carry is
explicit, not retroactively narrowed or called complete.

## Final07 carry disposition — 2026-10-25

The composed clean candidate2b7fc9eba87529181ba609e52ac6a04a8ba6b1b0 independently passes HOST lint and make test:1129runtime+119docs=1248,0failed/ignored; owned9/live24 included. CC true no-default package command exits0 (18 unused test-helper warnings disclosed; its aggregate is not verified). AT-03 now accepted; historical failures/limits above remain unchanged. See [07 CDC review](../slice07-final-candidate-validation/cdc-verification.md). No source repair in this follow-up.
