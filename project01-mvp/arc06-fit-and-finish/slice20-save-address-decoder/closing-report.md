# Slice20 CC proposed-done return

The five-path Slice20 library increment is implemented and committed locally as `0b37d8b41222b8e4a084c088246f777490575756` (`Add public save address decoder`). It adds an opt-in `nms_save::address` module with four typed, field-specific decoders; independent fixture data; external consumer coverage; and public documentation. It does not migrate save parsing, application callers, graph construction, or cache behavior. Those remain in required21.

The commit contains exactly these five paths:

- `crates/nms-save/src/address.rs`
- `crates/nms-save/src/lib.rs`
- `crates/nms-save/tests/address_decode.rs`
- `crates/nms-save/tests/fixtures/address-oracles.json`
- `crates/nms-save/tests/fixtures/address-oracles.md`

The source/index worktree is clean at the commit. Its message has the two required consecutive, once-only trailers, parsed in order: `Co-authored-by: Codex <noreply@openai.com>` and `Co-authored-by: Billo AI <ai-engineering@billo.systems>`. No source changes occurred during the final released gates.

## Validation proposed by CC

Captured final gates on the committed source passed:

- `make format` `bce1e356b6c946d99885843bbef183e6`: exit 0.
- `make lint` `60b5d6b6c4a2481a91927a965ef1e2b0`: exit 0.
- Focused external consumer test `81c61a980a8e441bb18c324e90fc08fc`: exit 0, 8/8.
- Strict save-crate docs `c0a7c86109cf4c268aa23fca35060e42` with `RUSTDOCFLAGS=-D warnings`: exit 0. Generated HTML links and destination fragment IDs were separately inspected and all four targets resolve.
- Current-source authorized-host `make test` `c7dbab9d09cb48cbb69694957c03bad5`: exit 0; 36 result groups, 1,025 passing tests including 18 doctests across nine groups.
- All-feature workspace build `2905379aad404460ac9e380bedd374ef`: exit 0. The captured output contains the existing future-incompatibility warning for `proc-macro-error2 v2.0.1`.
- All-feature workspace doctests `4cb4f186aca14661b29f06ff627ba825`: exit 0; nine groups, 18 passed.
- Working and baseline diff checks `b2b15ff6a26b47c4a92a7aad7c3e1dd4` and `2b848d7cd55d4656a1753a1e08db1b64`: both exit 0.

The current raw-first capture record is `/private/tmp/nms-slice20-cc-attempts.json`, SHA-256 `a856330640598c981bd635be548f128d2e77806e0631136b2e5cb43c53c4c4d6`, with 29 append-only entries: two CDC wrapper controls and 27 CC product attempts. It retains actual argv, source/status metadata, raw channels, exit codes, and end times. Earlier failures and retries are preserved rather than replaced by later green runs. In particular, the initial lint and doctest failures and the restricted watcher/coordinator failures remain separately recorded beside their repairs or identical authorized-host retries. The first whole-workspace host run was on the pre-iteration02 candidate; the final `make test` above is on this final source.

## AD rows — CC proposal only

| Row | CC proposed disposition | CDC disposition |
|---|---|---|
| AD-01 — Packed coordinate/reality decoding | Proposed done: all canonical discovery/base rows and explicit high-precision, below-48-bit, galaxy-nine, and signed-edge controls are exercised through the public functions with complete expected components and portal values. | Pending independent review |
| AD-02 — Strict packed syntax and typed failures | Proposed done: both packed APIs cover the strict type/string/width/error-priority matrix and diagnostic paths. | Pending independent review |
| AD-03 — Expanded current presence and ranges | Proposed done: complete required-member, JSON-kind, range-boundary and extra-key behavior is checked before core masking. | Pending independent review |
| AD-04 — Previous absence and sentinel | Proposed done: absent and all-zero sentinel behavior is separated from present-invalid values and partial nonzero addresses, preserving planet and reality. | Pending independent review |
| AD-05 — Usable API and source-witness fixture | Proposed done: public docs/examples, external consumer calls, typed error traits/path/display, input-purity checks, and all 45 source-projection rows are covered. | Pending independent review |
| AD-06 — Existing interfaces remain unchanged | Proposed done for this bounded library increment: only the five authorized paths changed; legacy parsing/conversion and application/cache/native work are outside this commit. | Pending independent review |
| AD-07 — Attributable evidence and handback | Proposed done: raw-first capture inventory, retained failures/retries, exact-path commit, parsed trailers, and frozen return package are supplied. | Pending independent review |

These are the implementer's proposals, not acceptance. Slice20 does not prove whole-source application validation, caller migration, semantic cache transition, or real application/native behavior. Those remain required21; no closure of R06-R01 or other parent work is inferred.

## Return artifacts and evidence limits

- `execution.md` preserves intake, source/API/oracle readback, each iteration's actual gates and failures, source identities, final groups, and commit/handback details.
- `validation.json` lists all 29 attempt IDs and outcomes, the final commands and raw-channel hashes, source paths/hashes, and the seven proposed dispositions.
- `closing-report.md` is this concise proposed-done summary.
- The complete raw capture payload remains in `/private/tmp/nms-slice20-cc-attempts.json`; it is not copied or rewritten by the return package.

The separate helper `--help` invocation produced an observed wrapper `IndexError` because the helper requires positional arguments; it launched no child and is documented in `execution.md`, not represented as a product attempt. The actual gates used the inspected helper contract. CDC owns independent reproduction and final disposition; the source and three ignored return files are frozen for that review.

## Independent disposition, chronology, and bubble-up — 2026-10-21 UTC

This report belongs to the initial Slice20 assignment `cc-prompt.md`, source/test/documentation refinements `cc-prompt-iteration01.md`, `cc-prompt-iteration02.md`, and `cc-prompt-iteration03.md` (the last source assignment), plus record-only iteration04 `cc-prompt-iteration04.md`, all under `project01-mvp/arc06-fit-and-finish/slice20-save-address-decoder/`. Iteration04 changes only this report and `execution.md`; it does not change the validation JSON, captures, source, index, or gates.

CDC independently reproduced all 12 required groups on clean commit `0b37d8b41222b8e4a084c088246f777490575756` and accepted AD-01 through AD-06. AD-07 is the only remaining row in the original closing-report contract, and this section supplies that record. These dispositions are for Slice20 only; no foundation-only parent acceptance is claimed.

The earlier sentence “Captured final gates on the committed source” describes matching source file bytes, not gate execution at the later commit HEAD. CC's actual gate captures identify precommit HEAD `4cb745b7190e7463663a0ead29f169060e36dc05` and dirty working identity `454a5064bdd2b4ae926c91b14a1d8568be3ac4fdad1e4c99a06835c98e61a54f`; source commit `0b37d8b41222b8e4a084c088246f777490575756` followed those gates. CDC's independent fresh checks identify the clean committed source. CC's precommit diff checks covered tracked changes; CDC's postcommit base diff covered all five committed paths. This is an attribution clarification, not a missing-evidence claim or waiver.

### Bubble-up to the arc

The delivered usable increment is an opt-in public `nms_save::address` API with four documented field-specific decoders, typed path-aware errors, runnable public examples, and external consumer tests using independent packed and expanded expectations. The existing parser and legacy conversion/application behavior are preserved in this increment; API usability and legacy behavior preservation are accepted here, while caller migration remains assigned to required21.

Implementation revealed no new structural change requiring an arc-plan amendment. The preserved history records the actual arithmetic correction for a true below-2^48 positive, the consolidated completeness repairs to the positive/error/boundary/purity proof, and the generated documentation-link correction. These were bounded corrections to the approved field contract and evidence, not a new save policy. Required21 was already the owner of whole-source application validation and integration.

Delivery matches the specified foundation scope; it does not complete the parent Arc06 endpoint. Required21 still owns whole-source validation and caller migration; atomic live last-good publication; semantic cache transition3→4; and real CLI, normal-PTY, stdio, HTTP, watcher, and restart behavior. All broader parent obligations remain open. Slice20 makes no game-backed nonzero claim.

The mandatory CDC retrospective will live at [`cdc-retrospective.md`](cdc-retrospective.md) relative to the canonical Slice20 directory and is owned by CDC. Canonical supporting-artifact transfer paths are `artifacts/cc-execution.md`, `artifacts/cc-validation.json`, and `artifacts/cc-captures.json`. The complete actual attempt payload remains at `/private/tmp/nms-slice20-cc-attempts.json`; the inspected helper is `/private/tmp/nms-slice20-cc-capture.py`. Their original scope and availability limits are unchanged. No new artifact forest or schema is introduced.
