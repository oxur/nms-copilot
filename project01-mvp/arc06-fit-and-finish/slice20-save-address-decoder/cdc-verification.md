# Slice20 CDC independent verification

2026-10-21 UTC. Two-contributor Autonomous Arc06-only arrangement: actual implementer `/root/cdc_arc06/cc_slice17`, selected GPT6Luna High; independent reviewer `/root/cdc_arc06`, selected GPT6.1Sol High. CDC authored no production or test repair. Source `0b37d8b41222b8e4a084c088246f777490575756`, parent `4cb745b7190e7463663a0ead29f169060e36dc05`, dedicated `codex/save-address-decoder`. Initial prompt and immutable01–03 describe implementation;04 is closing-record completion only. Actual requested/applied settings remain unchanged; no root transport code-review substitution.

## Independent current review

Initial independent disposition before record-only04: AD01–06 accepted; AD07 open for closing-text completion. Current final verdict: all seven rows accepted; see final closure below. All twelve independent execution groups pass at the exact clean committed source, with explicit before/after status, identity and raw stdout/stderr/exit/end. Evidence is [CDC raw receipts](artifacts/cdc-review-receipts.json); [all29 preserved CC attempts](artifacts/cc-captures.json) remain doer evidence with their original HEADs, working identities, failures and retry environments. First two entries in each capture collection are author-run wrapper controls, not product gates.

| Independent command | Receipt ID | Result |
|---|---|---|
| make format | 570fa749ea064d4db16867ee6e661aae | 0; clean source unchanged |
| make lint | fb1670967b9842178f1d6edb3b228ae0 | 0 |
| cargo test -p nms-save --test address_decode | 8bd508759e7247758ce91c461c478305 | 0;8 |
| cargo test -p nms-save | 29061889673a4be0a167a0d46073beac | 0;102unit+8integration+8docs |
| cargo test -p nms-core | 2bf88437d81a4706812b82d48a712ed2 | 0;87 |
| cargo test -p nms-copilot --lib --no-default-features | 251e7b14d7f04eb683a92491ad835e57 | 0;233 |
| make test | cd5339cecc0c4612b2e731d14b030d15 | 0;36harnesses/1007runtime+18docs |
| cargo build --all-features --workspace | 03e8041d68604877986f68a74c616578 | 0 |
| cargo test --doc --workspace --all-features | 77402bdef7db41aba0be5505765fba6d | 0;18docs |
| RUSTDOCFLAGS=-D warnings cargo doc -p nms-save --no-deps | d8d47155f4a14692a46602728b87bd7b | 0 |
| git diff --check | 801078f66f864e89889cea663d9333ae | 0 |
| git diff 4cb745b7190e7463663a0ead29f169060e36dc05 --check | bdf6617fb6c94918837ec886ed642562 | 0;all5committed paths included |

Known watcher/filesystem/loopback restrictions justify direct authorized-host no-default/workspace runs, retaining original TMPDIR `/var/folders/dt/kc8ndn6d3kx8c1jnxr05drsh0000gn/T/`. There were no failed current CDC product attempts. Existing proc-macro-error2 future-incompatibility warnings remain visible; no broader dependency/MSRV or whole-workspace strict-documentation closure is inferred.

## Seven-row walk before record-only04

| Row | Independent observation | Disposition |
|---|---|---|
| AD-01 | Actual public discovery/base consumers; complete named six-field/portal assertions for Chili1 and all44bases (31string/13integer), precise adjacent integers above2^53, P0SSI299 and true below2^48SSI43, zero/max/signed endpoints, lexical positives and same-physical gal0/9 in both orders. Original numericGG9 positive restored. | accepted |
| AD-02 | Both APIs reject missing/null/bool/array/object/fractions/exponent representation, negative exact integers, excess56bit/valid15digit width and malformed/non-ASCII hex. Exact variants, paths, numeric bounds and invalid-character-before-width priority are exercised. | accepted |
| AD-03 | Original JSON is checked before core construction; all13 current endpoint/SSI4094 cases have independent literal whole-address expectations, plus required missing/null/type/one-beyond/u64MAX and deterministic precedence. Current zero and extra-key success use full literal expectations. | accepted |
| AD-04 | Previous absence and fully validated all-six-zero are distinct from null/partial failures. Each of six single nonzero components plus P1/GG9 together preserves full expected address. Same present-invalid cases reach both exports with same-root literal typed errors. | accepted |
| AD-05 | Documented public module/four fallible APIs/owned nonexhaustive error with field docs/path accessor, Error+Send+Sync and exact Display. All consumer success/error routes use immutable-input snapshot comparison. Eight public runnable examples pass. Four actual generated function-page links and module destination IDs directly inspected. Fixture byte copy/provenance inspected in full. | accepted |
| AD-06 | Exact5path commit; only old-file delta is lib.rs opt-in docs/pubmod, existing function bodies unchanged. No dependency/core/parser/model/convert/app/watch/cache/source-fixture migration. Fresh workspace/no-default/build proof and core compatibility controls preserve prior behavior. | accepted for foundation only |
| AD-07 | All29CC and14CDC entries retain raw channels/identities; actual failures/retries, source/trailers/transfer and independent execution verified. Closing report still needs explicit assignment, named bubble-up and precommit chronology qualification. | open;iteration04 |

The field-specific decoder validates syntax/56bit bounds before bit extraction and record-galaxy retention; expanded integer range validation precedes narrowing. Sign extension is bounded to8/12bits. Previous sentinel tests cover planet/reality, not just XYZ/SSI. The independently supplied Chili/44base expected components distinguish the wrong legacy48bit partition; expectations are not generated from the decoder/core constructor. Existing core48bit behavior remains intentionally separate. These observations do not establish universal game validity or game-backed nonzero evidence.

## Review history and honest chronology

Initial author mistake: P0SSI299 `0x012B0004B2446A` is49bits, not below2^48. CDC caught it before the first compiler/test, preserved the valid original positive, and issued01 adding synthetic46bitSSI43 with actual threshold checks. Historical Slice02 prose is qualified, not rewritten.

Full pinned five-file review after initial green consumer/workspace checks found incomplete positive endpoint/sentinel/immutable-input/public documentation proof. Iteration02 supplied a single literal/helper batch; necessary inline-field doc formatting was clarified before affected gates. All467module/913test lines and helper callsites were then reviewed. Iteration03 restored an accidentally removed original numericGG9 case and corrected four function-page URLs to module-index anchors. The exact five-edit delta and actual generated links were independently inspected; all remaining executable production statements/types/signatures were unchanged.

CC's first lint and first whole-crate rustdoc failure, later successful repairs, restricted no-default/workspace watcher timeouts and identical authorized-host retries remain in their actual capture records. Those are not waived or converted to current green attempts. A helper `--help` IndexError launched no child; it is a supporting-wrapper incident in execution.md, not missing product evidence. All raw channels were rehashed; initial16/20 entry prefixes reconstruct to their original exact serialized file hashes.

CC final gates were executed at HEAD4cb745b7 plus complete dirty working identity454a5064…, then the same five file bytes were committed as0b37d8b4. The original closing-report phrase “on the committed source” is qualified by that actual chronology, not a rewritten historical HEAD. CDC's current executions freshly identify committed0b37. CC precommit diff checks covered tracked working changes; current CDC postcommit baseline check covers every committed new file too. Both source commit trailers are consecutive and parsed once each. Frozen CC execution/validation/report/raw payloads were hash-checked before byte-preserved canonical transfer; original report bytes are additionally retained in the existing receipt before04.

## Bubble-up to the arc

This foundation delivers the planned usable opt-in decoder library, with exported consumers and runnable documentation. It does not repair existing application extraction. Required21 remains CDC-owned: validate complete original JSON before typed defaults/extraction/model/cache/publication, migrate every actual caller and faithful raw fixtures, preserve atomic invalid-update/last-good full state and five-failure/recovery, perform semantic3→4 genuine predecessor rejection/source rebuild/warm restart, and prove CLI/normalPTY/stdio/HTTP/watch/file/EOF composition. Cache schema/semantic remains3/3 in20 because applications do not consume the new boundary.

The existing foundation/application split remains correct; no new structural scope or policy change arose. Arithmetic and proof/docs corrections require honest history/retrospective follow-through, not a new application architecture. R06-R01/C01/R03 and every original raw/history/nonzero-game/freshness-import/export/import/format/docs/distribution/coverage/performance/higher-independent/Operator/integration/release gate remain open. Arc06-only endpoint and protected main edit/held pins/external worktree remain. The source is locally committed; no merge/push/install/release is authorized or performed.

AD07 record completion is the only current child blocker. No source/gate rerun is needed for the record-only04 handoff. The mandatory CDC retrospective will live in `cdc-retrospective.md`; it does not replace this independent verdict.


## Final independent closure — 2026-10-21 UTC

CDC reviewed complete actual iteration04 closing/execution append and verified both retain the exact prior byte prefixes. Corrected reportSHA3ec5c1dcfa1cd0c6420769929de74766158a15fd9a2b8b09c7ad677b83664a3f and executionSHAceac6cd7dd9e26e5bea2194702aba0ea972085a2a29eea58d9710f8c0411b98d are byte-preserved in the canonical locations. Validation89e9c77d… and raw29capturea8563306… remain unchanged; no source/index/gate work occurred. The report now names every issued assignment, explicitly distinguishes real precommit identities from matching committed bytes, answers all three bubble-up questions and links the mandatory [CDC retrospective](cdc-retrospective.md). AD07 is accepted. All7AD/R06-S20done, with no deferral/no-op, missing-history exception or silent scope drop. Fourof5refinements were used; no reset or future exception.

The twelve current independent groups remain applicable without rerun for record-only edits. Source0b37 stays clean; mainb616 remains its ancestor, protected main workingblob2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e/indexempty and every held worktree pin/external03e8d04 are unchanged (ordinary metadata fence in existing receipt). Source and planning commits use exact namedpaths/proper consecutive parsed trailers. All durable supporting artifacts are the existing45-row projection, CDCraw/review/helper/preserved-candidate receipts, transferred CCexecution/validation/all29captures; prompts and close-set/retrospective are first-class root records. No additional native20proof is claimed or required by this pure opt-in unit.

Arc-plan1.86/project-plan1.75 record foundation closure and required21 before further planning. Arc38rows20done18open/project25open. No new structural change is needed beyond the already-accounted foundation/application split; no application/raw/history/finalsemantic/nonzero-game/coverage/docs/distribution/higher-independent/Operator/integration/release parent is closed. Continue authorized required21 source-grounded readiness under approvedA and the unchanged one-arc endpoint/models.
