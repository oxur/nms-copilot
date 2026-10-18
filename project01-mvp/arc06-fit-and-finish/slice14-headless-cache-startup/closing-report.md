# Slice14 CC closing report — proposed-done

Date: 2026-10-17. Assignment: `arc06-fit-and-finish/slice14-headless-cache-startup/cc-prompt.md`, issued in planning commit `d333d5518999dc242b4eac89e3902b6c4d84b5c7` from `320de77a5ff3fab85ca0649cf5f905090f4b45ba`. The immutable prompt SHA-256 is `1ab0abba4768262aecdb94dbaac1088baea48366244a4c7f53e755146bdbb61b`. Planning intake was `3c91b9bb47293f2a416311a4cc5ae7cd7285d158`.

## Delivered

Source commit `88b184f999d4fde13e59b736dbd3a32896afd425` directly parents accepted source `16c078725492744db22bae837d87c89489a6add5` (parent `a101bd44b277f06c75ed7260b470b31cd5510130`). It changes only `crates/nms-copilot/tests/mcp_live_integration.rs` and `README.md`; required co-author trailers appear once each. The candidate worktree `.worktrees/headless-cache-startup` on `codex/headless-cache-startup` is clean, isolated, unintegrated and uninstalled.

The native integration test now starts the real headless binary over stdio and HTTP with separate source/cache/home/child fixtures. It observes the actual `where_am_i` tool and `nms://player/location` resource, checks their full location fields and equal system labels, verifies stdio/HTTP stdout hygiene and checks the cache before and after child termination. The sentinel is independently constructed and checked before launch; source expectations are literal and parser-checked. The test confirms the CLI bypass flag only for bypass cases and uses exact `FileTimes` freshness preconditions immediately before launch.

Both transports passed all eight cases:

| Case | Source → served | Cache result |
|---|---|---|
| `cold_missing` | C/X1 → C/X1 | Current 2/2 cache written |
| `fresh_current` | A/X0 → B/X0, `Cached-9` | Exact valid sentinel bytes preserved |
| `stale_current` | A/X0 → A/X0 | Rebuilt from source |
| `fresh_schema1` | C/X1 → C/X1 | Typed schema mismatch then rebuild |
| `fresh_semantic1` | A/X0 → A/X0 | Typed synthetic semantic mismatch then rebuild |
| `fresh_corrupt` | C/X1 → C/X1 | Typed deserialize error then rebuild |
| `bypass_existing` | A/X0 → A/X0 | Actual `--no-cache`; exact bytes preserved |
| `bypass_missing` | C/X1 → C/X1 | Actual `--no-cache`; cache and missing parent absent |

The full 16 actual tool/resource observations and per-case setup/effects are stored in `artifacts/validation.json`. For each case the source bytes remained unchanged. Rebuilt caches were checked for the current header, source version 4720, selected source reality, complete literal player state and empty source discovery maps, then rebuilt and compared with the parser-derived source model whose fields were independently checked against literals. The unnamed physical player placeholder remains an in-memory post-cache-load behavior; this work makes no promotion claim.

README now explains headless stdio/HTTP startup cache use, default enabled per-save paths, explicit `cache.path`, `--no-cache` read/write bypass, mtime freshness, source rebuild/write behavior, selected versus physical galaxy, watcher startup settings, and the remaining composition limits.

## Validation and retained attempts

All 14 prescribed gates passed on the final replay, in order: format, lint, cache tests, route-cache integration, all-feature native integration, focused package set, no-default library, `make test`, workspace build, workspace doctests, selected package docs, strict cache docs, candidate diff check and planning diff check. Final gate-level harness counts and complete per-attempt argv/cwd/time/environment/effective flags/offline configuration/separate stdout and stderr base64, byte counts, SHA-256, exit and harness summaries are in `artifacts/validation.json`.

The initial lint failure (three test helper compile errors) and sandbox-denied loopback/filesystem attempts are retained. The focused native gate, broader package gate, no-default gate and `make test` were retried unchanged on the authorized host with original inherited `TMPDIR`; each retry passed. Existing dependency future-incompatibility and rustdoc warnings remain recorded. No test was weakened and no gate waived.

At exit, all 28 protected source hashes and both immutable fixture identities match the author baseline. All 10 held worktree pins remain at their expected clean revisions. Planning prompt/baseline/closure inputs remain unchanged. Only the two allowlisted source files and four allowed planning return paths are included in the Slice14 return.

## Ledger and independent review

HC-01 through HC-07 remain **open**. This is CC proposed-done evidence; it is not independent CDC acceptance. Sofie/CDC must review and independently replay the assigned contract before changing any row status. The planning close commit hash and exact path/trailer receipt are recorded in the follow-up metadata entry in `artifacts/validation.json`.

## Required remaining ownership

This slice proves native headless initial cache startup composition only. Planned15/CDC still owns normal REPL startup composition and complete live-cache write-through behavior in both modes, unknown-system resolution/promotion, duplicate normal-mode producer/settings composition, broader historical-state questions, and final-baseline semantic transitions when raw-field repair lands. The original export/import/format/docs/distribution/coverage and R06-R02/R03/M7.4/C04/parent independent/Operator/integration/release gates remain required. No full C04 or parent acceptance is claimed.
