# Source findings and project01 carry-over

## Current inheritance — 2026-10-27

[Project01 independently closed](../../project01-mvp/cdc-verification.md) at accepted source `01b601f49253987b1b9c596ad0fb1e01cb757dda`, planning close `8a56a405af459d945b6de508086b6d4c1bd0c131`. This reconciliation verified the source worktree remained clean at that revision, workspace Rust2024/MSRV1.91, the existing `nms-mcp` member, exported SaveParser/NmsSaveParser and main's LiveSaveCoordinator consumers. It was a targeted baseline read, not a rerun of tests or broad source audit. Main integration/PR14 state must be revalidated before dependent delivery; no merge is implied.

The current [six-arc plan](../project-plan.md) consumes accepted checked parsing, field-specific address/galaxy/model/cache semantics, one application intake coordinator, the separate MCP crate, current-session `last`, repaired Tab/map and the accepted genuine Euclid/Hilbert/native interactions. Earlier prose below about missing last/MCP/decoder/coordinator repairs describes old pins, not the present accepted baseline. Project01's strict-positive2opt proof is delivered, not assigned again.

Exact final source-head CI passed its recorded Linux/macOS/Windows, feature and MSRV-check populations. The historical Windows timeout remains a prior failed attempt, not an unresolved current-head CI result. Notify backend reconnection is still separate future Project02 scope; the old failure was not backend-recovery evidence. Project01's accepted limited warm-debug headless timings do not establish new Project02 chooser/full-save/REPL performance.

Current destinations: B03 compressed-offset arithmetic → Arc03; B04 jump arithmetic → Arc02; B06 existing query/detail/statistics/info → Arc02 and rich composition Arc05; B07 settings/output → Arc02; B08 saves parity → Arc01 and remaining existing parity → Arc02; B12 backend reconnection → Arc03. The [repository backlog](../../backlog.md) retains notebook/culture/research/exploration transfers and all other conditional deferrals. The current ledger preserves the original24 criteria and maps their amendments; historical tables below are evidence, not current assignments.

The original source findings about save resolution, identity metadata and other paths must be checked on the actual implementation baseline during their owning slice; they are not automatically re-certified by this notice. No dependency version, engine feasibility, native clipboard support or geometric capability was verified here.

Recorded 2026-10-03. Source checkout: main `a800950ef3558fcf463517696c3c140061e7353d`; planning base `85bc9a6597c62ecbaf2c5d649b4a46eef74e3719`. Findings are scoped inspections, not a complete code audit. For current ownership of inherited repairs and this augmentation's scope, see the dated 2026-10-11 and 2026-10-26 clarifications below.

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

## 2026-10-11 superseding ownership/status clarification

The inspections, examples and self-check above describe the 2026-10-03 packet and its pinned source; they are historical, not a current audit. In particular, the old proposed transfer of existing address defects to Project02 is superseded by the Operator's subsequent Project01 scope approval.

Read-only review of the current [Project01 plan](../../project01-mvp/project-plan.md) and [ledger](../../project01-mvp/ledger.md), at planning `c0351040b8fd6fba06c5ead8964b092a63aa7398`, confirms existing field-specific address repair, galaxy-qualified identity/cache compatibility, contributor docs and coverage remain Project01 responsibilities with their original gates. This pass does not review or accept ongoing Slice08 work. Source main at intake is `6d183bde79dd7328c32d4739dca249f5f1f9cd6c`.

Project02 Arc02 consumes accepted fixes and extends observational provenance; Arc07 verifies the new project's documentation/coverage and reconciles inherited findings, without silently moving Project01 obligations. No source file, Project01 artifact, private save or external reference checkout was modified for this augmentation.

New requirements come from the Operator's 2026-10-11 gameplay discussion: individual save narratives, discovery as claim, bases as attachment evidence, single-traveller cultures with arbitrary nouns, mathematical degrees of cohesion/completeness, late adjacency between historically separate cultures, and query scope across saves. [Journals/media](journals-and-media.md) and [cultures/exploration](cultures-and-exploration.md) distinguish those requirements from proposals and unresolved research. There is no new save decoding, candidate-star enumeration, geometry benchmark or native clipboard test in this pass.

## 2026-10-11 augmentation self-check

The nine Markdown documents pass local-target checks (35 links), balanced code-fence checks, consistent table-column checks and `git diff --check -- project02-personal-atlas`. The first local-link pass failed on the historical `/Users/oubiwann/.codex/skills/rust-guidelines/` location; those links now target the current installation and the repeated check passes. External references are research citations, not a claim of a complete web-link audit.

Structural checks find nine roadmap arcs with stable original IDs, 14 decision-register rows, 14 research gates and 20 unique open ledger rows. All 12 original criterion IDs/statuses remain; the four expanded rows are P02-01/06/07/11, the other eight original rows remain text-identical, and eight new rows capture the added capabilities/composition. The roadmap's release dependency follows the two added arcs. Manual scope review retained original creature/history/import/export/FTS decisions while adding the journal, media, narrative, attribution, culture, geometry and region cases.

This is planning-author self-check evidence, not independent acceptance or successful implementation research. No Rust code changed; no build, lint, test suite, native clipboard trial or geometric benchmark was run. Project01 evidence files were being updated concurrently by other work; the commit allowlist is exactly these nine Project02 documents, with both repository-required trailers. Existing source and Project01 changes are outside this task's writes.

## 2026-10-26 two explicit Project01 transfers

Duncan adopts ONLY groups2 and3 from the [complete Project01 clause matrix](../../project01-mvp/arc07-closeout-reconciliation/slice08-composition-handoff/artifacts/requirements-reconciliation.md#consolidated-decisions-to-confer-on). The matrix’s source/proof findings are reused; no new source audit or tests were performed. Current source pin3e8e460 retains application/library behavior from accepted1b79525d; later changes are CI/development-probe/test-fixture only. Historical observations above are not silently refreshed.

| Inherited source finding / approved scope | Next-project delivery and ownership |
|---|---|
| Shared query name predicate exists but CLI/REPL/MCP set it to None; standalone planet detail/seed is absent; statistics omit distance distributions/full discovery-type/explicit unnamed output and meaningful flag selection; info lacks biome overview and uses BaseContext Game Mode rather than active context | [B06](../../project01-mvp/backlog.md#b06-search-detail-statistics-and-info-next-project) / P02-21: expose shared name search consistently, standalone seed-bearing planet detail, distance/type/named-unnamed statistics with flag semantics, and info biome/active-context Game Mode. Existing Arc04/05; existing-query work may follow Arc01 directly without a database gate. |
| Default galaxy, tsp_algorithm, find_limit, emoji_glyphs, table_style and save.format are parsed but inert; color is not consistently applied; black/white palettes still contain ANSI and general JSON is limited to export | [B07](../../project01-mvp/backlog.md#b07-working-configuration-and-output-next-project) / P02-22: operational settings, consistent color, actual ANSI-free/plain piping and structured JSON beyond export. Existing Arc01 foundations AFTER friendly save/account selection; no database prerequisite. |

**Owner/re-entry:** explicitly assigned Project02 contributors, CDC independent review and Duncan prioritization/acceptance, during actual NEXT-project delivery at those existing arc boundaries. No source assignment, new arc/slice/prompt, implementation or proof is claimed. B02 configurable themes remains separately deferred; B03/B04 retain their next-two-future-project deadlines and B05 its unchanged no-deadline deferral. Groups1 (legacy compatibility/integrity),4 (routing/watcher detail/proof) and5 (promised-versus-implemented scope) have no new disposition from this discussion.

## 2026-10-26 REPL parity, required closeout and onward scope

Duncan transfers existing CLI-command REPL parity to Project02 [P02-23](../ledger.md): current REPL lacks `saves`, `export`, `import` and shell-script `completions`. Add `saves` early in existing Arc01 with friendly selection, export/import in Arc06 interchange and shell-script completions in Arc07 polish. Existing TAB completion is distinct and is not described as absent. Owner/re-entry: explicitly assigned Project02 contributors, CDC review and Duncan acceptance during those actual deliveries; no source implementation or new arc/slice/prompt.

Current SessionState lacks the detailed D0026 introduction’s promised last-query results. Root’s read-only local registry inspection confirms reedline0.46 FileBackedHistory persists command text (`h.command_line`), does not support update, and reconstructs `more_info` as None; it is not application result storage. Duncan requires the actual most recent SUCCESSFUL query result within the CURRENT REPL session BEFORE Project01 closure; that meaning is now agreed, distinct from existing command history. Delivery/API remains open; cross-restart result history is future, with no last command, persistence or backend inferred. He also requires extraction to a separate `nms-mcp` crate BEFORE that closure, retaining current headless stdio/HTTP/shared coordinator/cache semantics and avoiding cycles; a separate binary was not required. These are Project01 responsibilities, not completed work or new Project02 prerequisites manufactured by this note.

[B09](../../project01-mvp/backlog.md#b09-named-client-and-gameplay-integrations-project03-reservation) reserves named-client/gameplay integrations for Project03 AFTER Project02; Claude Desktop/web/Anthropic apps are explicitly excluded. Current generic stdio/HTTP MCP protocol implementation/tests remain. Duncan chooses later targets/priority, then an explicitly assigned contributor owns bounded delivery. This reserves an ID and re-entry condition only; no Project03 directory/full plan is created. Existing legacy/routing-watch decisions and cache/representation/SaveParser information requests receive no new disposition here.

## 2026-10-26 notify backend transfer

Duncan transfers the [original D0034 backend-error recovery clause](../../project01-mvp/arc07-closeout-reconciliation/slice08-composition-handoff/artifacts/requirements-reconciliation.md#consolidated-decisions-to-confer-on) to NEXT Project02 [B12](../../project01-mvp/backlog.md#b12-notify-backend-reconnection-next-project)/P02-24, existing Arc02 observation intake after friendly Arc01 selection. Reuse the established source finding (watcher backend Err batches ignored, no reconnect branch); no new source inspection/test here. Delivery must bound re-registration/resumption, respect shutdown/cancellation and handle duplicate backend events without recovery-induced duplicate registrations/application ingestion. Ordinary replacement/deletion/corruption recovery and the Windows CI6h timeout are separate. Owner/re-entry: future explicitly assigned Project02 contributor/CDC independent review/Duncan acceptance during actual next-project Arc02 delivery, no database prerequisite. No implemented recovery, new arc or prompt is claimed. Reachability visualization is B11 without project/deadline; the strict-positive2opt regression remains required Project01 proof, not a proven runtime defect.
