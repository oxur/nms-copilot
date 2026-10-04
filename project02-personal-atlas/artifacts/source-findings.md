# Source findings and project01 carry-over

Recorded 2026-10-03. Source checkout: main `a800950ef3558fcf463517696c3c140061e7353d`; planning base `85bc9a6597c62ecbaf2c5d649b4a46eef74e3719`. Findings are scoped inspections, not a complete code audit.

## Current implementation

| Finding | Evidence at inspected main revision | Design consequence |
|---|---|---|
| `CommonStateData` already parses `SaveName` and total play time | [model.rs](https://github.com/oxur/nms-copilot/blob/a800950ef3558fcf463517696c3c140061e7353d/crates/nms-save/src/model.rs#L27) | Friendly names need not wait for a richer atlas parser. |
| Save listing shows raw account names/IDs and physical-file presence, not in-game names; scan errors become “No save files found” | [saves.rs](https://github.com/oxur/nms-copilot/blob/a800950ef3558fcf463517696c3c140061e7353d/crates/nms-cli/src/saves.rs) | Add metadata inventory and preserve partial errors. |
| Slot selection searches `accounts[0]`; default resolver selects newest file globally | [main.rs](https://github.com/oxur/nms-copilot/blob/a800950ef3558fcf463517696c3c140061e7353d/crates/nms-cli/src/main.rs#L295) | Resolve account and logical save together; do not conflate latest sync with intent. |
| Import has a separate save resolution path | [import.rs](https://github.com/oxur/nms-copilot/blob/a800950ef3558fcf463517696c3c140061e7353d/crates/nms-cli/src/import.rs) | All save-consuming commands must use one policy. |
| Setup uses directory/slot labels and treats invalid TOML as an empty table while writing config | [setup.rs](https://github.com/oxur/nms-copilot/blob/a800950ef3558fcf463517696c3c140061e7353d/crates/nms-copilot/src/setup.rs#L269) | Metadata chooser; validation-preserving, recoverable configuration writes. |
| Cache namespace uses only parent basename + save stem | [paths.rs](https://github.com/oxur/nms-copilot/blob/a800950ef3558fcf463517696c3c140061e7353d/crates/nms-copilot/src/paths.rs#L33) | Distinguish different roots/replicas and extraction generations. |
| Discovery extraction passes packed UA to portal-address constructor and forces galaxy zero | [extract.rs](https://github.com/oxur/nms-copilot/blob/a800950ef3558fcf463517696c3c140061e7353d/crates/nms-graph/src/extract.rs#L113) and second call at line 142 | Fix the encoding boundary before relying on richer origins/relationships. |
| REPL and MCP setup each contain watcher startup | [REPL main](https://github.com/oxur/nms-copilot/blob/a800950ef3558fcf463517696c3c140061e7353d/crates/nms-copilot/src/main.rs#L171), [MCP setup](https://github.com/oxur/nms-copilot/blob/a800950ef3558fcf463517696c3c140061e7353d/crates/nms-copilot/src/mcp/mod.rs#L40) | Centralize ingestion ownership and verify simultaneous-interface composition. |

The workspace currently has no persistent SQL catalog. Its graph and cache cover a narrower projection than the full JSON. Discovery names/owners/times and nonsystem/nonplanet records need explicit extraction work; the graph's existence does not mean all raw entities are currently searchable. Existing rkyv loading reconstructs the in-memory model, so do not base performance promises on an assumed fully zero-copy query path.

## Local save evidence

Read-only examination used four decoded files from two logical saves, each with two physical versions. Private raw snapshots and account/player IDs are not included in this planning packet. These observations establish test cases, not universal format guarantees:

- The in-game names were `main - Steam` and `main - Xbox`.
- The Xbox-labelled save resided in a Steam-style local account directory; its root platform value was `XBX|Final`, while the Steam-labelled save's was `Mac|Final`.
- `CommonStateData.SaveUniversalId` was nonempty, identical within each sampled pair, and different between the two saves. Its JSON representation was a string.
- `UsedPlatforms` included Mac, Windows, and Xbox tokens. Ordering has not been established as chronological.
- `UsedDiscoveryOwnersV2` entries carried `LID`, `UID`, `USN`, `PTK`, and `TS`; sampled `USN` values were empty. Owned base records contained the name `oubiwann`.
- Main and retained expedition contexts coexist in a save; the active context does not describe all retained data.

The checked-out [MBINCompiler common-state schema](https://github.com/monkeyman192/MBINCompiler/blob/04a76be59439911fd9db1ea588e03ac604112e71/libMBIN/Source/NMS/GameComponents/GcPlayerCommonStateData.cs) declares `SaveUniversalId` as `ulong` and lists the platform/owner arrays. That schema supports field interpretation but does not prove cross-save ID lifecycle semantics.

The checked-out [libNOM.io identity implementation](https://github.com/zencq/libNOM.io/blob/652aabb0147888776a088909cdb13bccbd3d9588/libNOM.io/Platform/Platform_UserIdentification.cs) prioritizes filtered owned bases, then settlements, then discoveries, with identity/platform filtering and frequency selection. This is a useful reference, not a guarantee for our cross-platform inputs. Its Steam adapter can use an online lookup; project02's initial UX deliberately relies on local evidence and aliases. No reference implementation code was copied.

## Address regression to preserve

The user's independently successful in-game visit established Chili-Chilista's origin in **Gohairt Platform**, Euclid, portal **212B04B2446A**. The pet species was **U. Xenohidnia**. The user reported a **Worm-ridden** planet with **Star Bulb**, consistent with the creature's **Verdant/Lush** native biome. Keep the user's observation distinct from decoded fields when building fixtures.

The observed packed discovery/companion address was `0x212B0004B2446A`, laid out as `P SSS GG YY ZZZ XXX`. The portal omits the `GG` byte: `P SSS YY ZZZ XXX`. For this field family, a candidate conversion is `((v >> 40) << 32) | (v & 0xffffffff)`, retaining galaxy separately as `(v >> 32) & 0xff`. It yields planet 2, system index 299, X 1130, Y 4, Z -1244, galaxy 0. Current truncation yields the incorrect `2B0004B2446A`.

This is an independent golden case for Arc02. Validate bit widths and malformed inputs before conversion. Audit bases, teleport endpoints, discovery records, and companions individually: not every field named “address” uses the same encoding, and expanded coordinates already take a different path.

## Project01 disposition

The previous status pass found the seven broad MVP phases delivered and reproduced **807 passing tests, zero failures and zero ignored tests**, plus lint success. It did not run coverage; the coverage target only exercises the REPL library without default features, so the project's broad 95% target remains unproven. The test result is prior-turn evidence, not a rerun for this planning packet. Temporary logs were `/private/tmp/nms-project01-status-test.log` and `/private/tmp/nms-project01-status-lint.log`; do not treat temporary files as durable closure evidence.

| Item | Treatment |
|---|---|
| Discovery-address decoding defect | Arc02 prerequisite; eligible for an earlier narrow corrective release. |
| Friendly save/account selection shortcomings | Arc01, newly prioritized by the user. |
| Community import persistence | New durable-catalog scope; original implementation explicitly deferred persistence. |
| Missing CONTRIBUTING guide and incomplete coverage contract | Carry into Arc07 documentation/validation planning; preserve as unresolved project01 findings. |
| Formal project01 closeout | Not performed by this design pass; retain historical status and distinguish implementation evidence from independent closure. |

## Evidence limits

No new Rust tests, database trial, latency benchmark, cross-platform build, or migration test was performed for this design. Before implementing identity heuristics, turn the observations into sanitized fixtures and expand the sample to missing/changed IDs and other household saves. Research URLs were consulted as primary documentation; local reference clones were inspected at the pinned commits above, not assumed to be the latest upstream code.

## Design-packet self-check

The seven Markdown documents were checked for existing local-link targets, balanced code fences, accidental numeric account identifiers, and whitespace errors. All 12 project ledger IDs are unique and remain open. The seven roadmap arcs map to P02-01 through P02-07, with whole-project composition and inherited-findings rows P02-08 through P02-12. The source checkout remains unchanged. These are structural self-checks, not independent acceptance of the architecture or implementation.
