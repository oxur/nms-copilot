# Slice45 iteration01 — first ACTUAL refinement

2026-10-25UTC. Initial implementation exists; actualformat071abb9e1b724bf0a7a9f1ed332b50aa0 and lint38d7f4b60611462bb87cd4e474e1de25exit2 retained, stableworkingidentity43eef45377cb9c033c0e74828a46a05cb4d831946f267a8c7729844c8898e4c7. No runtime occurred. Full CDC three-path U1–U4/source/assertion/doc walk CLOSED: production scalar implementation and full literal matrix/public formatter witnesses sound; two newtest helper compile defects remain. This is firstACTUAL refinement1/5, no reset/exception/model change. Original cc-prompt.md preserved; it governs all unchanged authority/proof.

Dedicated source/branch unchanged cleanbase9b plus heldthree-pathcandidate; CDCplanning/CCsource ownership serialized. ONLY mutable path now `crates/nms-cli/tests/unicode_display_integration.rs`, ONLY the TWO function shapes below. ALL other bytes in this test file (fixtures/literal assertions/commands/environment/read-only checks), both already-reviewed querydisplay and dispatcher files, every other source/dependency remain FROZEN. Exact before01 full newtesttext+three sourcehashes in artifacts/before01-source.json. Existing querytests/fullfences and every non-helper production byte were independently compared to accepted9b; originals unchanged. No new scenario/API/assertion weakening.

F1: missing lifetime on `assert_child_succeeded` at44: tie returned &str ONLY to Output, not label. Exact signature:
```rust
fn assert_child_succeeded<'a>(output: &'a std::process::Output, label: &str) -> &'a str {
```
Preserve full functionbody/rawstdout+stderr failure diagnostics and UTF8 validation. Do not clone/output-normalize or remove assertions.

F2: `command_with_isolated_home` currently returns chained setter result (&mutCommand) where an ownedCommand is required. Exact shape:
```rust
fn command_with_isolated_home(temp: &TempDir) -> assert_cmd::Command {
    let mut command = cargo_bin_cmd!("nms");
    command
        .current_dir(temp.path())
        .env("HOME", temp.path().join("home"))
        .env("XDG_CONFIG_HOME", temp.path().join("config"))
        .env("XDG_CACHE_HOME", temp.path().join("cache"));
    command
}
```
No changed command/currentdir/environment/source seed/fixture expectations. Existing manual assertion already preserves complete actual child stdout/stderr; no additional assert_cmd machinery or helper redesign is needed.

Read this sibling/current plan+ledger+snapshot, actual two diagnostics/full newtest, current original guidance as applicable. Append finite source-cited readback acknowledging both exact shapes/allfreezes and WAIT for CDC ACK. No full unchanged guide/test reread ceremony. Every write resolves dedicatedroot+exactonepathallowlist/currentwholeoldtext/counts; buildbothreplacementsinmemory before writing, explicitcwd/absolutepath. Mainknownagent9line2cd450ce/indexempty/heldpins protected; no cleanup/integration/release.

After ACK, correct exactlyF1/F2 then original rawfirst captured makeformat→makelint→IMMEDIATE HOLD. Send actualIDs/identity and exactfreezeshashes. No runtime or postpair edits until CDC targeted wholefile/actualreplacement review releases original groups3–9. Capture all compiler/gate attempts from FIRST actual invocation; failures remain. Nine original groups/hostfullworkspace/originalTMPDIR/affectedstrictdocs/diffs/metadata+normalhooknamedthreepathcommit/finite exact returns/freshindependentproof/retro remain binding unchanged. No new reusablevalidator/mutation suite needed. Fiveactualrefinementcap:1usedafterthis,4remain. Original M7.7coverage/CI/allcommands/D0043/themes/otherUnicode/input/parent/Operator gates remainOPEN. No newarc/model/reviewer/dependency/install/push/merge/publish/release.
