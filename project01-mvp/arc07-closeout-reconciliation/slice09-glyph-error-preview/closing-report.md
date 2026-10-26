# Slice09 CC closing report

## Intake and contract readback

At initial intake, read the issued `cc-prompt.md`, `slice-plan.md`, and Arc07 `ledger.md`. During this completion-record correction, also read the Slice09 acceptance `ledger.md`; the GP row mapping below reflects it. Confirmed source baseline `ae0ef360389f555e5ec0d39930005cbfc374faad` on `codex/workspace-coverage-contract`, clean before edits. Read `parse_next_glyph` and its complete existing tests; `PortalAddress::parse_mixed`, `FromStr`, `GalacticAddress::from_portal_string` and their immediate docs; CLI convert implementation/docs, Convert argument/dispatch; Make format/lint/test targets; and existing CLI assertion style. Read Rust AP-09/AP-25, EH-01/02, testing discipline, and validation gates including Proportionate Proof and execution-population guidance. Contract readback: only the glyph diagnostic preview and successful name/abbreviation original-boundary mapping change at runtime; preserve typed error/API, prefix, table order, lowercase predicates, aliases, and output. Four literal baseline regressions precede repair. CDC owns the single final workspace test and planning disposition.

## Implementation

`crates/nms-core/src/glyph.rs` now backs the 20-byte preview index down to a UTF-8 boundary. Successful lowercase name and abbreviation matches share a private original-input boundary mapper, so Kelvin-sign aliases consume their original bytes and preserve the right suffix. Added four required baseline-failing regressions, exact preview boundary/unchanged error reason checks, and positive emoji/name/abbreviation/colon/hex suffix assertions.

`crates/nms-core/src/address.rs` removes the three propagated obsolete diagnostic-panic claims. `crates/nms-cli/src/convert.rs` removes only the inherited panic claim and retains the stdout-failure claim. Added `crates/nms-cli/tests/glyph_error_integration.rs` with direct 3-byte and 4-byte rejection cases asserting exit 1, empty stdout, bounded diagnostics, and no panic text. No executable change was made to Convert dispatch or address wrappers.

## Baseline regressions

Before the runtime correction, `CARGO_NET_OFFLINE=true cargo test --offline --locked -p nms-core test_parse_next_glyph_` exited 101. It selected 7 tests: 2 passed, 5 failed. Both required three/four-byte preview cases panicked at byte index 20; the Kelvin abbreviation case panicked at byte index 4; the Kelvin name case returned `(13, "ettail")` instead of `(13, "tail")`; the preview-boundary group also reproduced the split-character panic. These failures are retained as the actual baseline observation.

## Validation and disposition

Before the baseline test, `CARGO_NET_OFFLINE=true make format` and `CARGO_NET_OFFLINE=true make lint` passed (Clippy and format check; the existing `proc-macro-error2 v2.0.1` future-incompatibility warning was emitted). After repair, both targets passed again in the same order. `CARGO_NET_OFFLINE=true cargo test --offline --locked -p nms-core` exited 0: 95 unit tests and 14 doctests passed, none failed or ignored. `CARGO_NET_OFFLINE=true cargo test --offline --locked -p nms-cli --test glyph_error_integration` exited 0: 2 direct CLI tests passed, none failed or ignored. `git diff --check` passed. The focused selector used for the pre-fix witness selected 7 tests, of which 5 failed as described above; its other 88 unit tests and doctests were filtered out. No global workspace `make test` was run; that single gate belongs to CDC after this CC stops. No coverage measurement or unrelated LZ4/warp disposition was attempted.

Source commit: `1b79525dcaabd3cd06388a787734c8377b34c6e9`. No durable command logs or other slice artifacts were produced.

Proposed GP rows for CDC: GP-01 preview UTF-8 validity/20-byte bound and existing empty/separator behavior; GP-02 lowercase name/abbreviation alias and original suffix preservation including Kelvin, direct CLI invalid-input exit/diagnostic behavior, and the three doc corrections; GP-03 CC format/lint/focused gates plus CDC review and the serialized native workspace gate. CC's format, lint, and focused checks passed as recorded above. CDC's native workspace gate is pending. This implementation has not been independently accepted.

Bubble-up/silent-drop: This is the one expressly authorized Slice09 correction, delivered as a candidate for independent disposition; no next slice is issued. The LZ4 and warp risks and all other unrelated gates remain unchanged and undispositioned. No further Arc07 or Project01 closure is claimed.

CDC closure addendum: independent [verification](cdc-verification.md) accepts all3 rows at1b79525d; actual host1256pass, restricted attempt retained. [CDC retrospective](cdc-retrospective.md) completed; two CDC logs in artifacts/ added after CC return. This addendum is CDC-authored; preceding CC observations remain attributed to CC.
