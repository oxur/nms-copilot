# Slice10 CC closing report — WIP

## Intake and contract readback

Loaded in full at planning revision `e62c6ba6faa0a2001ac9aa5b0bdc72a1444abf1c`: `cc-prompt.md` (22 lines), `slice-plan.md` (17), and `ledger.md` (10). The source worktree was clean at `3e8e46066136978aad1d878ca1608515862ff8c4`.

Required source sections were read: `nms-save/src/lib.rs` exports and byte sanitation/obfuscation helpers; all of `validated.rs` including `ValidatedSave` accessors, `parse_validated_save`, file parsing and canonical row pairing; all of `error.rs`; format detection/decompression and the existing LZ4 header construction in `decompress.rs`; the fixture's first literal positive in `tests/validated_save.rs`, its 57-bit legacy opposition, and the checked-file/mapping/LZ4 tests at lines 830–938; `nms-cli/tests/checked_save_integration.rs` and `nms-cli/src/info.rs`. The relevant Rust-guideline sections AP-02/03/06, API-02–04, TR-17, EH-01/04 and DC-01–04 were read in full.

Required data checks: `git status --short --branch` and `git rev-parse HEAD` confirmed the clean source baseline above. `rg -n 'parse_validated_save_file' crates/nms-cli/src crates/nms-watch/src` returned all matches: eight CLI source call sites (`info`, `show`, `stats`, `import`, `list`, `find`, `route`, `export`) and two watcher source call sites (`snapshot`, `watcher`), plus snapshot docs. Each complete match was inspected; existing production consumers enter through the checked file parser.

Contract readback: add an object-safe, borrowed-byte `SaveParser` returning the existing owned `ValidatedSave` and typed `SaveError`, with a stateless `NmsSaveParser` that accepts the supported raw plaintext or standard LZ4 on-disk bytes. The easy-to-miss compatibility constraint is that the new trait must use the exact existing checked-file pipeline order (decompress, sanitize, F2P heuristic and mapping, validate original JSON, deserialize/pair), while `parse_validated_save` remains plaintext deobfuscated-JSON-only and legacy parsing stays unchanged. The existing file API must read and delegate through the concrete parser without recursion or a permissive fallback. External `dyn` tests need literal output and error-path/source-chain expectations for both byte forms; file-wrapper checks and the existing obfuscated checked-file test cover the real consumer path. Only the four assigned Rust paths and this report are in scope. No conditional trigger or plan conflict was found.

## Implementation and validation

Added the documented object-safe `SaveParser` contract and stateless `NmsSaveParser`, then re-exported them from the crate root. Extracted the former checked-file byte pipeline unchanged into `parse_validated_save_bytes`; `parse_validated_save_file` still owns file I/O and now invokes the concrete parser through the trait. `parse_validated_save` and legacy APIs were left unchanged. The external integration tests assert literal player/discovery/base output for plaintext and constructed standard LZ4 inputs, exact invalid-address paths and error source, malformed JSON/LZ4 errors, plus checked file-wrapper acceptance/rejection. The initial no-run API example was corrected to read a supported on-disk file before parsing.

**Ledger attestation (CC proposed-done; CDC independent review pending):** SP-01, SP-02, SP-03, SP-04. No source scope drift or unresolved implementation deviation. In particular, `validated_from_value` and its source-row pairing remain unchanged. Cache code was not modified; literal output checks and unchanged checked extraction semantics support retaining the current cache schema 4 / semantic 6 compatibility revisions. This is a compatibility assessment, not a separate cache test campaign.

**Commands and outcomes:**

- `CARGO_NET_OFFLINE=true make format` — exit 0 (run after implementation and again after the doc-example correction).
- `CARGO_NET_OFFLINE=true make lint` — exit 0 (both runs); Clippy and format checks passed. Cargo reported the existing future-incompatibility warning for `proc-macro-error2 v2.0.1`.
- `cargo test --offline --locked -p nms-save` — exit 0 after the final doc correction: 105 unit, 8 address integration, 4 parser interface, 17 validated-save, and 38 doctests passed.
- `cargo test --offline --locked -p nms-cli --test checked_save_integration` — exit 0, 3 passed. This ran before the later documentation-only example correction; the nms-save suite was rerun afterward.
- `CARGO_NET_OFFLINE=true make test` — one native, escalated all-feature workspace gate, exit 0 (`✓ All tests passed`). It ran before the later documentation-only example correction. Output was streamed and partly truncated by the tool, so no aggregate test denominator is claimed. The corrected crate doctest was subsequently covered by the nms-save test run above.
- `cargo build --offline --locked --workspace --all-features` — exit 0; the same existing future-incompatibility warning was emitted.
- `RUSTDOCFLAGS="-D warnings -D missing_docs -D rustdoc::broken_intra_doc_links" CARGO_NET_OFFLINE=true cargo doc --offline --locked -p nms-save --no-deps` — exit 0.
- `git diff --check` — exit 0.

No command failed. The implementation remains WIP for CDC review; root owns source Git/index/commit/push. The current cache compatibility statement does not resolve the separate inherited Windows CI cancellation item.

## Bubble-up to the arc

Slice10 delivers the usable parser trait and production checked-file boundary at reviewed WIP based on source `3e8e46066136978aad1d878ca1608515862ff8c4`; root commit and integration remain pending. No unanticipated scope, dependency, or follow-on work was discovered. The silent-drop comparison found no omitted assigned behavior. External review/acceptance, Windows CI cancellation, and other inherited gates remain unchanged and open. No supporting artifacts were created: the canonical source changes and close-set record suffice, and the streamed native workspace output was partly truncated.
