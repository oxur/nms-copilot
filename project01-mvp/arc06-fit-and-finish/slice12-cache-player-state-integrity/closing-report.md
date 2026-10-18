# Slice12 CC return — cached player-state integrity

## Disposition

**CC proposed-done for independent CDC review; not accepted.** All seven CI rows remain open. No Slice12, Arc06, Project01, parent-criterion, integration, install, or release acceptance is claimed.

## Assignment and source identity

- Assignment: `project01-mvp/arc06-fit-and-finish/slice12-cache-player-state-integrity/cc-prompt.md`, SHA-256 `1268796baf74dc66156b431b97314b34eafd9425150a88323d59a757e03ca17b`.
- Issuance: `30fdf09536089887395a88562ed748eb183c2e89`, parent `ad552d4abf25d183c96f9bfdec16c34acca1d0ff`. Planning intake was `72b23fb78848d9e231b423971cb11df9528e2404`. That intake adds only the actual issuance receipt to Slice12's plan; the issued plan hash, current plan hash, and metadata-only delta are retained in validation.
- Accepted source intake: `.worktrees/mcp-live-watch`, `codex/mcp-live-watch`, `239313cc109ca908f43f34ae5e04ba7e5487161f`, parent `46ca153c4418fcbf859f76ef59d600985a315b60`; clean.
- Candidate: `.worktrees/cache-player-state-integrity`, branch `codex/cache-player-state-integrity`.
- Source commit: `88c5980a0c0540c06790ca9d218c91c34dded606`, direct parent `239313cc109ca908f43f34ae5e04ba7e5487161f`. Its exact four paths are `crates/nms-cache/src/serialize.rs`, `crates/nms-cache/src/lib.rs`, `crates/nms-cache/src/freshness.rs`, and `crates/nms-cache/tests/load_integration.rs`. Both required co-author trailers occur once. Candidate is clean after commit and remains isolated.

## Delivered behavior

A private borrowed validator checks only `Some(player).current_reality == player.current_address.reality_index`; `None` passes. `serialize` checks before archive encoding and returns the existing `CacheError::Serialize(String)` on contradiction. `deserialize` retains length/magic/schema/semantic precedence, aligned checked rkyv decoding, then checks decoded logical state and returns the existing `CacheError::Deserialize(String)`. The message names both fields and values. There is no normalization, inferred repair, added public type/API/error, cache shape change, or change to schema2/semantic2. `write_cache`, loader fallback ordering, and direct `rebuild_model` are otherwise unchanged.

The loader docs now explain that a fresh contradictory cache follows the existing read-failure rebuild path, that source parse failure writes no replacement, and that selected galaxy, previous address, and freighter address are independent. Crate docs state the same boundary. `main::load_model` already calls `load_or_rebuild` before the headless/REPL split; this return claims only static caller composition, not native cache transport behavior.

Four integration tests were added. Both 0/9 and 9/0 contradictory values are built as genuine checked-rkyv-decodable payloads under a literal current 16-byte envelope, and direct checked payload decoding confirms the logical contradiction before `deserialize` rejects it. A matching 9/9 positive control succeeds. Actual byte and file readers reject both directions; serialize rejects them; write failure preserves an existing sentinel file and does not create a missing parent. Schema and semantic mismatches retain compatibility-error precedence.

For both mismatch directions, tests create a real fresh cache file and assert freshness before fallback. Direct file reading rejects it; `load_or_rebuild` returns the source model and source save version4720, rewrites a valid2/2 cache, and after deterministic `FileTimes` adjustment warm-loads with `was_cached=true`. The old sentinel cache has version9999 and player funds7/8/9; source funds are5,000,000/10,000/500. Malformed source input leaves the contradictory cache bytes untouched with cache enabled and with `no_cache`; `no_cache` with a valid source also bypasses and preserves those bytes.

The positive transport control uses same-packed systems in realities0/9, physical/current reality9, selected galaxy7, previous reality4, freighter reality6, and funds7/8/9. It passes serialize/deserialize/rebuild and write/read with all lookup/spatial maps intact. A `None` player-state control also passes. The sealed schema2/semantic2 route fixture and existing compatibility, stale, no-cache, and cache-write-failure tests remain intact.

## Final validation

All14 required gates passed against the final source. The complete 40-command-attempt history (two ordered gate passes, sandbox retries, and final planning diff-check receipts), exact launch metadata, separate stdout/stderr channels as base64/byte count/SHA-256, and per-harness counts are in `artifacts/validation.json`.

| Gate | Final result |
|---:|---|
| 1 `make format` | pass |
| 2 `make lint` | pass |
| 3 `cargo test -p nms-cache` | 38 passed, 0 failed, 0 ignored |
| 4 route cache integration, all features | 4 passed, 0 failed, 0 ignored |
| 5 native MCP live integration, serial | 2 passed, 0 failed, 0 ignored on unchanged host retry |
| 6 focused six-package all-feature suite | 737 passed across21 active harnesses; 0 failed/ignored on unchanged host retry |
| 7 copilot no-default-features library suite | 219 passed, 0 failed/ignored on unchanged host retry |
| 8 `make test` | 933 passed across24 active harnesses; 0 failed/ignored on unchanged host retry |
| 9 workspace all-feature build | pass |
| 10 workspace all-feature doc tests | pass across9 harnesses |
| 11 no-dependency docs for six packages | pass; existing compiler/rustdoc warnings retained in stderr |
| 12 strict cache docs with `-D rustdoc::broken_intra_doc_links` | pass |
| 13 source `git diff --check` | pass |
| 14 planning `git diff --check` | pass |

Sandbox attempts for gates5–8 are retained separately: loopback port binding was denied for the live HTTP test, and a native watcher file-replacement observation timed out in the focused/no-default/full suites. The exact same commands passed with host permissions and the original `TMPDIR`; no test, command, assertion, or input was changed. An earlier complete ordered gate pass and the final complete replay are both preserved. All21 protected source digests and both sealed fixture byte lengths/hashes match `author-baseline.json`; the only source modifications are the four allowlisted paths.

## Proposed row walk

| Row | CC evidence | CC disposition |
|---|---|---|
| CI-01 | Both directions fail through actual `deserialize`/`read_cache` and `serialize`/`write_cache`; existing bytes and missing parent are preserved | open for CDC |
| CI-02 | Matching and `None` positive controls; reality9/current9 with selected7, previous4, freighter6, full funds and same-packed cross-reality systems; sealed2/2 route control | open for CDC |
| CI-03 | Both actual bad files are fresh before validation; full source4720 state replaces sentinel9999; deterministic warm load is cached | open for CDC |
| CI-04 | Malformed source and `no_cache` preserve contradictory bytes; legacy no-cache/stale/write-failure controls pass | open for CDC |
| CI-05 | Existing shared loader caller precedes both startup modes; assigned route/MCP/focused/no-default/workspace gates pass; no full native cache-transport claim | open for CDC |
| CI-06 | Error/docs and precedence covered; revision2/2, public shape, protected hashes, and sealed fixtures unchanged | open for CDC |
| CI-07 | Intake, all attempts/channels, allowlists, direct parent, commit trailers, fence checks, and seven-row return recorded | open for CDC |

CC attests the implementation and evidence above for CDC replay. The seven CI rows remain open pending that independent disposition.

## Remainder, artifacts, and held fences

Planned13/CDC retains full startup/live-cache C04 and old/current/warm/write-through composition in both modes, unknown-system placeholder/promotion, watcher enabled/debounce settings and duplicate-producer composition, and broader historical-state questions. These remain required before raw-field repair and original parent acceptance. No policy inquiry was opened; unresolved policy remains with CDC/Operator. Original export/import, format, docs, distribution, coverage, independent-review, and Operator gates remain.

Artifacts in this slice home: immutable `artifacts/author-baseline.json`; `artifacts/execution.md`; `artifacts/validation.json`; this `closing-report.md`; and `ledger.md` with Evidence cells updated only. No CDC record was created. Main `6d183bde79dd7328c32d4739dca249f5f1f9cd6c`, the accepted Slice11 worktree, all dependency worktrees, planning intake fence, and Project02 remain held. Candidate is not integrated, installed, pushed, or released. Later integration, final-baseline composition, and parent acceptance remain CDC/Operator-owned.

Arc06 remains30 rows/11 done/19 open; all25 Project01 rows remain open. Slice12's seven rows all remain open. This return records only CC's isolated candidate and proposed-done evidence.

## Planning artifact commit

The planning close-set commit is `cb57370e12d006e8af45859cfca2365e3b9822e4`, direct parent `72b23fb78848d9e231b423971cb11df9528e2404`. It changes exactly the four paths listed above; each required co-author trailer occurs once. The first `git commit --only` attempt preceded staging of untracked artifacts and failed with pathspec errors without changing files; staging only the allowlisted paths and retrying succeeded. Validation retains both outcomes. This report and its three companion planning files receive a metadata-only receipt follow-up within the same four-path allowlist.
