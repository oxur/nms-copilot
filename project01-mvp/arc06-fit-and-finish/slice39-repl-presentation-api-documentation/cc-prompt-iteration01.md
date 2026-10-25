# Slice39 first actual refinement — closed initial documentation/example findings

2026-10-25 UTC. FIRST ACTUAL refinement1/5/four remain. Initial immutable prompt/plan/readiness/history preserved; same soleCC /root/cdc_arc06/cc_repl_docs gpt-6-luna High and CDC /root/cdc_arc06 gpt-6.1-sol High, standing one-arc Autonomous authority. No newmodel/reviewer/policy/exception/scenario/API/runtime behavior repair.

Actual initial format IDbc21f37ee186479cbeaf601cabc9e509 exit0; lint ID968dc8c42c3c489195eaae310f2b39c6 exit0. Complete raw/end/status/equalidentity4b0d65ce71dd2ec485974babd8fbb911b7e812334c42a5d7f228baef11b2dfa0/diff58728d7dc1c626fd887287f18ac81f69d723dca6df1cb1712d532b71af10b74e directlyverified. FULL3filepublicdoc/production/newE1–E3 assertion/oldhelpinput walk closed and truncated106-token prompt read recovered. No doctest/runtime/cargo-doc/scope/diff gate was released/run. These are STATIC review findings, not invented compiler failures. Exact initialpatch DATA in artifacts/initial-held-diff.json.

## Scope and frozen good controls

Only commands.rs/completer.rs existing Rustdoc/example fences may change in the precise groups below. ENTIRE prompt.rs FROZEN (sound W3/full literal snapshot/indicator/history/update example). All original non-doc/API/attributes/body/tests/deps/otherpaths/current8derived-doc insertions/four enum summaries/existingClaphelpinputs/otherhelpers/kernel/registry FROZEN. No new examples, query/native/TTY scenario, forcedpanic or normalization.

CurrentsourceHEAD3fc7c3ff7bdd65b6b13d9db087dbeb01a447007a/indexempty; commands25399bytes/SHAef5bff4819f11f2dd3e6423fe5748c750d560d66805cdf7ab903a45c305b7779; completer26487bytes/SHAd187f38c766ff56679424d38c92ffaa2a0835875f16d0822ac2f9d369bd0e685; prompt9403bytes/SHAf26a74d673fd928291baff1aa95751ee8dcba6cd2025fd8bbd77d31d3b751e71. Guard resolveddedicated-root AND exactallowlist/currentbytes/count forEVERYwrite, explicitcwd/absolute path. No planning mutation byCC.

## F1 command example correctness and canonical order

parse_line headings currently Errors→Panics→Examples violate the issued canonical Examples→Errors→Panics. Reorder COMPLETE existing sections, preserving all real error/panic teaching and every valid assertion. Do not alter derived section or sourcebody.

Current Find match arm consumes owned Strings and returns references from as_deref out of that arm; locals drop before the returned tuple can be used. Bind the parsed Action OUTSIDE its borrowing match. Exact safe shape (keep existing parse helper/ExampleError and full Find input/expected tuple):
```rust
let parsed_find = parse(FULL_EXISTING_FIND_LITERAL)?;
let find = match &parsed_find {
    Some(Action::Find { biome, infested, within, nearest, named, discoverer, from }) => (
        biome.as_deref(), *infested, *within, *nearest, *named,
        discoverer.as_deref(), from.as_deref(),
    ),
    other => return Err(Box::new(ExampleError(format!("expected find action, got {other:?}")))),
};
```
The FULL input literal and entire expected seven-field tuple stay exact; no dropped fields, no assertion inside a nonexecuted branch or fixture-derived expected. Local bound parsed_find outlives borrowed projection. No wrapper removal or newerror API needed.

Three expressions currently use `parse(...)??` for galaxy/list/help. The helper returns Result<Option<Action>,Box<dynError>>; ONE ? unwraps Result to Option for existing matches! patterns. Replace exactly those three ?? with SINGLE ?, preserving all selection/list-default/plain-help checks. Whitespace case already has one ? and remains valid.

FOUR wrapper_about EXPECTED strings incorrectly retain terminal periods; pinned clap_derive remove_period removes them. Exact expected whole tuple:
```rust
(
    Some("Top-level REPL command parser".to_owned()),
    Some("Set session context (position, biome filter, warp range)".to_owned()),
    Some("List reference data or model collections".to_owned()),
    Some("Show detailed information about a system or base".to_owned()),
)
```
Keep all actual get_about/find_subcommand/ToString observations, full FOUR direct_about expectations and four enum summaries EXACT. Existing original wrapper docs remain byte-preserved. Originalauthorplan's period-free oracle is sound, not an author retraction. Unknown-argument error fragment/full matching and all other existing E1 controls retained.

## F2 actual full Suggestion projection — six booleans, nine fields

Current projection return type/FOUR expected rows contain SEVEN bool slots, but actual item fields yield SIX: append_whitespace + display_override/description/style/extra/match_indices.is_none(). This is a type-shape mismatch and invented unobserved seventh expectation.

Keep the current safe OWNED String projection (consuming Vec<Suggestion> is fine; no forced borrowed rewrite). Exact return type is Vec<(String,usize,usize,bool,bool,bool,bool,bool,bool)> — nine fields, six bools. Actual tuple already emits exactly these six true/None observations and MUSTremaincomplete. For THREE base suggestions and ONE galaxy suggestion remove ONLY the extra seventh expectedtrue; retain six trues, exact quotedvalues/order/spans10..12 and7..12. No dropped optional metadata or weakened contains/count checks.

All15command values, exact5list targets, empty unknowncontext, all21capfixture names and independent literal20-value expected Base-00..Base-19 vector remain EXACT. Do not derive expected from fixture/generator, sort/dedup or substitute shorter checks.

## F3 complete issued W2 caller preconditions/context/quote/cap teaching

Public docs OMIT actual complete(line,pos) UTF8 BYTE slice precondition/#Panics; add applicable concrete #Panics on CopilotCompleter TYPE or its Completer::complete method: pos must be in range/on UTF8boundary, otherwise line[..pos] panics. Do NOT claim infallible new() panics or add forcedpanic example. Preserve canonical heading order wherever combined.

Complete existing module/type/constructor prose from original W2, no code/example scenario change: ordinary contexts split_whitespace and recognize trailing ASCIIspace, not a general quote-aware shell parser; case/prefix behavior with original-spelling find/route flag caveat retained. Matching names containing spaces are doublequoted; arbitrary embedded quote escaping is NOT provided. Galaxy path matches against256 reference names in index order, strips one leading doublequote for matching and replaces the ENTIRE value-tail span; same first20 named-match cap applies to GALAXY names as base/system names. Static command/flag/biome lists are prefixfiltered without that named-cap truncation.

Current phrase “fixed command and reference lists use their own complete populations” can imply uncapped256galaxy output; qualify underlying256candidates versus cappedmatching output as above. Existing snapshot/inputorder/case/duplicates/no-refresh/no-modelIO/namespace omissions stay. Type summary may say owned model-name snapshot to avoid inherited “dynamic” suggesting live updates. No API/body/cap/timing repair.

## F4 precise existing command syntax/default teaching

commands module currently says convert “accepts one of” and SSI “accompanies” voxel while later saying syntax/default descriptions. State parser's input group allows AT MOST ONE optional input and that required-input/voxel-SSI semantic handling is deferred; do not falsely imply this parser requires one input or enforces SSI. Existing declared attributes/defaults and derived help inputs remainfrozen, including older help text. Keep W1 all15actions/defaults/lexical limits and no model dispatch; no new parser behavior/test scenario.

## Execute and hold

Coherent exactreadback→CDCack→bounded2fileguarded corrections→rawfirst offline makeformat THENmakelint→IMMEDIATE source/index/gates HOLD for targeted whole corrected doc/example walk. No runtime/postpair source change until release. Current4CCvalues remainexactprefix; prefix and controls preserved, actual error attempts retained, ordinary guards not SUT. No direct validator/compiler outsidekernel.

On release originalninegroups/tencommands apply, including explicit cargo test --doc --all-features then --no-default-features BEFORE hostlibrary, ACTUAL tools.exec_command sandbox_permissions=require_escalated/originalTMPDIR/offline unchanged, actual CargoJSON+separate exact0owned4residualpredicate, original3file/8site/opposedcap+help auditor, bothdiffs. No same-restricted retry, no blanketwarningwaiver/fullcrateclaim. Exact3namedpathnormalhookcommit/uniquemessage+rawstoredbody/onceconsecutiveparsedtrailers/canonical6RP+3bubbleupreturn/independentacceptance+retro asinitial. Required40map/41banner-setup-root/fullM7.6/authenticNomNom/alloriginalhighergates remainOPEN. No source/selfacceptance/newrole/model/budgetreset/push/merge/install/network/release/otherarc/Project02.
