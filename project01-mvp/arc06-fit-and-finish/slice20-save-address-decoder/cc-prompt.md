# CC — Slice20 public save-address decoder foundation

Implement this bounded usable public API, not the later application migration. Initial assignment, 2026-10-21 UTC,0of5refinements. Operator approved exact optionA (“Yup, I approve option A”); governing approvala5fed49/arc-plan1.83. CDC `/root/cdc_arc06` gpt-6.1-sol High designs/reviews; YOU existing `/root/cdc_arc06/cc_slice17` gpt-6-luna High implement. Settings unchanged; no substitutions/newroles. Autonomous endpoint Project01 Arc06 ONLY. Keep main's protected9line edit/allheldpins/external worktree/Project02 untouched. No merge/push/install/publish/release/newinquiry/network work.

## Baseline, outcome and source ownership

Dedicated source `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-address-decoder`, branch `codex/save-address-decoder`, HEAD4cb745b7190e7463663a0ead29f169060e36dc05, clean/indexempty atissue. Planning is separate `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/planning`; your planning/index mutation is NOT authorized. Canonical slice home there: `project01-mvp/arc06-fit-and-finish/slice20-save-address-decoder/`.

After complete coherent intake below, you own ONLY this dedicated source/index/gates and may begin immediately under this issued handoff—no additional approval-shaped return needed. Record readback and actualstart in ignored `workbench/slice20-return/execution.md`, send concise acknowledgment, then implement. CDC retains planning/index/independent acceptance. Revalidate identity/status before edits; unexpected changes hold affected work. Use absolute paths for patches (the prior wrong-root incident must not recur); every shell edit has explicit cwd. Never mutate accepted19/.worktrees/unknown-system-lifecycle or root main.

Outcome: library users can call four documented field-specific functions on ORIGINAL JSON and receive correctly decoded core addresses/typed path errors, including absent/null and exact integer distinctions. The public route, examples and external integration tests are delivered now. Existing `parse_save/file`, raw SaveRoot defaults/conversion APIs, graph/watch/cache/application code stay byte-frozen. Required21, CDC owner, will integrate whole-source atomic validation BEFORE defaults/model/cache/publication and perform semantic3→4 plus actual interfaces/restart. Do not claim20 fixes current applications or closes R06-R01/C01/R03/game-backed nonzero. No cachebump or partial-atlas architecture in20.

## Allowed source changes — exactly these five paths

1. `crates/nms-save/src/address.rs` new public decoder/error module.
2. `crates/nms-save/src/lib.rs` add documented `pub mod address;` and short opt-in crate documentation/example; existing function bodies unchanged.
3. `crates/nms-save/tests/address_decode.rs` new external public consumer tests.
4. `crates/nms-save/tests/fixtures/address-oracles.json` BYTE-COPY the supplied canonical projection.
5. `crates/nms-save/tests/fixtures/address-oracles.md` explain provenance, independent expected values and bounded applicability.

No Cargo/dependency/features/core/legacybody/otherfiles changes; ask CDC on a genuine out-of-contract need before editing it. No arbitrary test simplification/ignores/count-only replacement. Read full recipe and closed worksheet in slice-plan before coding; it is the detailed binding definition. This prompt supplies the implementation spine and does not override that recipe. Surface contradictions, don't choose a convenient version.

## Binding API and algorithm spine

Expose in `nms_save::address`:
```
pub fn decode_discovery_address(value: Option<&serde_json::Value>, path: &str) -> Result<nms_core::GalacticAddress, AddressDecodeError>;
pub fn decode_base_address(value: Option<&serde_json::Value>, path: &str) -> Result<nms_core::GalacticAddress, AddressDecodeError>;
pub fn decode_current_address(value: Option<&serde_json::Value>, path: &str) -> Result<nms_core::GalacticAddress, AddressDecodeError>;
pub fn decode_previous_address(value: Option<&serde_json::Value>, path: &str) -> Result<Option<nms_core::GalacticAddress>, AddressDecodeError>;
```
These are uncompiled signature sketches, binding API shapes. Option means actual JSONmember absence; `path` is a diagnostic pointer-style label, not a selector/path parser. Borrow input/path, return owned checked address. No I/O/global state/logging/concurrency. serde_json is the intentional existing raw JSON public boundary, not an excuse to invent a parallel AST/feature/dependency.

Typed public nonexhaustive `AddressDecodeError` (Debug/Clone/PartialEq/Eq/thiserror::Error) with exact variants/fields in plan: Missing{path},InvalidType{path,expected:&'static str},InvalidHex{path},PackedWidth{path},OutOfRange{path,value:i128,minimum:i128,maximum:i128}; `path(&self)->&str`. Actual path/reason Display, no raw document dump. Document all public items/fields, Errors and runnable examples (use? in fallible examples); public error match example and useful path example. User input never panics. Private helper factoring local discretion.

Packed entrypoints stay separately named but share validated mechanics. Accept unsigned JSONinteger through56bits or ASCIIhex1–14digits with optional0x/0X, bare strings explicitlyhex. Do NOT usef64/trim/sign/decimal fallback/negativecast/maskoversize. Deterministic errorpriority and Numberclassification in plan. P52–55/SSI40–51/GG32–39/Y24–31/Z12–23/X0–11; sign-extendX/Z12,Y8; construct existing coreaddress only after range/width validation with RECORDGG. Width cannot select format: P0below2^48 still save56. Zero remains representable.

Expanded current requires explicit object+RealityIndex+GalacticAddress and all5integer fields. Presence and Numberkind must be checked BEFORE defaults/narrowing/coremask. Validate named encoded ranges in deterministic planorder. Previous absent→None; present validates identically; only all6zero→None, P-only/GG-only differences retained. Unrelated keys ignored. This API accepts an address wrapper, not an entire save; activecontextchoice/whole-save validation and lifetime are21.

## Concrete implementation/proof order

1. Implement module/error/export/docs under the above signatures. Walk each range and errorpriority. Keep liboldbody fence; no unrelated refactor.
2. Copy supplied fixture EXACTLY from canonical `artifacts/address-oracles.json`, SHA2a48d77a745002227e6bdd5957189a35d415d69f4023d4c93e616236a9d4aeaa, into source fixture. All44base IDs (31string/13integer), duplicates and all independent expanded expected fields retained; integerinputs already JSONnumbers. Chili1separate discovery. No name/position/user payload or candidate-derived expectations added. Explain provenance/limits in fixtureMD, and do not call syntheticgalaxy9 game-confirmed.
3. Implement public external tests for ALL seven numbered worksheet categories in slice-plan. Named expected fields avoid tuple-position errors. Compare all six components AND12hex portal, not success/counts alone. Include Chili+44actualpublicdecodes, P0below2^48/GG9, exact9007199254740993vs...992, signededges/max56, complete strict syntaxerror matrix/BOTHpackedAPIpaths, every expanded member's bounds/missing/null/type and exact childpath, previousall6sentinel, JSONpurity/stdError+Send+Sync/docs. Errorvariants/paths/bounds must be checked. Expected values from supplied independent oracles/literals; no new production decoder/core roundtrip manufactures expectations. Plan contains exact positive/literal/error shapes; don't replace them with a generic fuzz/count suite.
4. Run captured format→lint BEFORE any test. First focused external consumer test and whole nms-save prove the actual difficult boundary early, then original ordered gates. Compile/test self-repairs inside this initial five-path contract are allowed with failedattempt retention and renewedformat→lint; stop broadsequence on red, explain actual cause/scope, fix at source within authority. A new consequential behavior/scope need goes to CDC before edits. No invented green IDs or summary-only counts: verify real record/argv/source/rawcompletion FIRST.
5. Self-review all5paths/full helpers/assertion sequences/rustdoc/provenance, then complete exact namedpath sourcecommit and return frozen handoff. CDC independently accepts; your green checks are proposed-done only. Do not update ledgers or create CDCverdict/retrospective.

## Required reading / intake manifest

`SKILLS=/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills` (literal root; shared on thishost). Required text must be loaded completely as scoped, recover truncated output contiguously. Prior summary/hash/heading/othercontext reading is not this intake. Record path/state/actual read extents and structured query coverage separately in execution.md, not a new reportforest. Before dependent edits give source-cited readback connecting API/errorpriority/presence/width/sentinel/frozenapplication/strongoracles/gates; receipts are attestation, not comprehension proof. Missing/conflicting normative inputs hold dependent work. Once coherent, proceed under existing handoff.

Required-full, in order:
- This prompt, sibling slice-plan.md and ledger.md at issuecommit; source AGENTS.md.
- `$SKILLS/collaboration-framework/SKILL.md` and `$SKILLS/project-management/guides/README.md` (current3.0.1 and applicable operation guidance). Existing two-contributor/model authorization stands; don't ask models again.
- `$SKILLS/rust-guidelines/SKILL.md`, THEN full `guides/11-anti-patterns.md` BEFORE Rust edits, then `guides/01-core-idioms.md`. Keep project edition/MSRV/deps/logging rather than substituting skill defaults.
- `$SKILLS/testing/guides/01-testing-discipline.md`, `03-validation-gates.md`; `$SKILLS/work-verification/guides/05-independent-verification.md` (failure/retry/current-vs-historical/evidence boundaries).
- Baseline source `crates/nms-save/src/lib.rs`, `error.rs`, `Cargo.toml`, `model.rs` and `convert.rs` (small typed source/legacy fences); unchanged coreaddress definitions/masks/getters in nextsection.

Required-section:
- Canonical arc-plan sections “Next required save-address repair — Operator decision” and “Operator adopts the save-field contract A”; source state planninga5fed49. Historical preapprovalhold superseded ONLY by explicitadoption. Source19finalreview and otherarc history reference-only.
- Framework `knowledge/engineering-methods/guides/01-engineering-methodology.md`: NotesforCodex, Rolesandsharedinvariants, Two-Contributorworkflow, AutonomousMode/selection and transitions. Framework PM `guides/02-canonical-planning-worktree.md`: existing-record/supporting artifacts, open/close sets, preservediterationfilenames; `03-planning-top-down.md`: issuing/executingiteration. No first-layout confirmation gate here; established layout/authority unchanged.
- Rust `03-error-handling.md`: EH01/04/06/07/13/14; `02-api-design.md`: API02/08/20/23/39/46; `05-type-design.md`: TD04/05; `13-documentation.md`: DC01/02 and runnable example/Errors conventions. Read complete sections with definitions, preserve MUST/SHOULD. Application decisions/rationale in plan apply, not every chapter recommendation promoted to MUST.
- Core `crates/nms-core/src/address.rs`: all bit constants, GalacticAddress struct/impl constructors/packed/getters through conversion-to-string methods necessary to compare; leave unrelated routing/docs history reference-only. Pin Number1.0.149 `as_i64`/`as_u64` actual dependency source or bundled localdocs before depending on its integer-vs-float behavior; author checked exact source.
- `$SKILLS/engineering-methods/guides/07-implementation-prompt-authoring.md`: Requiredreading/CCintake and Correctiveiterations; complete current recipe is in thisprompt/plan, not an exercise to redesign it.

Required-data:
- Canonical20 `artifacts/address-oracles.json` SHAabove. Query COMPLETE discovery and base arrays, print everyid/kind/input/expected6fields/portal in bounded lossless rows; assert counts1+44 and kind31string13integer, no dedup/sampling. Inspect all output. Verify sourcefixture byteequality and allrecords actually consumed by external tests. This is existing accepted projection, not a new empirical inquiry. Original Slice02/03 fullraw/history reference-only because current sourcefixture includes exact bounded needed oracles/provenance.
- Existing capturehelper exactsource `/private/tmp/nms-slice20-cc-capture.py` and current2author-run support records `/private/tmp/nms-slice20-cc-attempts.json`: inspect fullhelper, query actual controls'argv/status/rawchannels/exit/end/before+afterstatuses+identities. Canonical20artifacts/cdc-review-receipts.json stores originalhelper source/controls. Kernel is inspected before firstproduct; do not silently replace it or pretend these author controls were your productgates.

Conditional: if context recovery or unavailable guide/source, reload actual required contract extents before dependentwork; state limitations honestly. No new role/model/inquiry needed for routine APIimplementation. New fieldpolicy/scope conflict stops affected work to CDC.

## Captured validation and return contract

Capture EVERY actual compiler/gate/diagnostic from FIRST command using `/private/tmp/nms-slice20-cc-capture.py LABEL '{"argv":[...],"overrides":{...}}'`; launch from correct sourcecwd. Rawstdout/stderr/base64/bytes/hash/exit/end persisted before fragilemetadata/postprocessing; explicit before/afterHEAD/status/tracked+untrackedcompositeidentity and originalTMPDIR/env retained. NewUUID perattempt, append allfailures/retries, preserve originalvalues. Ordinary rg/sed/status/inventory/helper reads aren't productgates or acceptance claims; don't manufacture waiver obligations for incidental metadata. Requiredfinaldiffgates AREcaptured. Do not print onlyr.output for actualgate and lose the object. Outer exec wrapper receipt is not proof of childsuccess: use rawrecord. Never infermissingexit from emptyoutput.

Commands in source, ordered:
```
make format
make lint
cargo test -p nms-save --test address_decode
cargo test -p nms-save
cargo test -p nms-core
cargo test -p nms-copilot --lib --no-default-features
make test
cargo build --all-features --workspace
cargo test --doc --workspace --all-features
RUSTDOCFLAGS='-D warnings' cargo doc -p nms-save --no-deps
git diff --check
git diff 4cb745b7190e7463663a0ead29f169060e36dc05 --check
```
Use captureoverrides for RUSTDOCFLAGS, not discarded outer-shellstate. Established filesystem/loopback restrictions permit directhost for known affected workspace tests, originalTMPDIR unchanged; otherwise preserve restrictedfailure then IDENTICALargv/cwd/env/source hostretry. Testsownedstablethisworktree; don'tborrow movingtarget/process or change testconcurrency/timeouts. No makecheck/deps/build cleanup/install/publish. No newnative20matrix: pureopt-inmodule/legacybodiesfrozen; required21 owns actual corrected applications/native/caches. Wholeworkspace current tests stillrequired. Any newrequiredfailure remainsopen, no skippedtest/weakenednegative. No broad reruns after record-only edits absent failure/sourceconcern.

Commit ONLY the five source paths listed above (`git add -- <exactnames>`, `git commit --only ... -- <exactnames>`), Gitmetadata via require_escalated. Messagefile with BOTH consecutive once-only parsedtrailers:
```
Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>
```
Parse trailers mechanically BEFORE return; preserveoldobjects/mapping if an authorized metadataonlyfix is needed. No heldhistory rewrite.

Return three ignored files under `workbench/slice20-return/`: execution.md (intake/readback/sourceattempts/failures), validation.json (realcommands/UUID/counts/source/env/disposition and complete rawrecord location), closing-report.md (all7ADrow proposeddispositions, exact sourcecommit/paths/parent/trailers/fences, artifactinventory/provenance/limits/required21+originalparents). No fabricated independentacceptance. CDC will transfer exactbytes/rawattempts to canonical20/artifacts and append independentverdict/retrospective only after reviewing/reproducing. Handback says source/index/gates FROZEN and no activeownedprocess. Every subsequent CDCcorrection is a preserved siblingiterationNN; fivecap independent of19's explicitly exceptional sixthpass. Currentinitialprompt must never be edited afterissue.
