# CC assignment — Slice02 packed-format authority and witnesses

Issued 2026-10-03, initial pass. Current assignment: `arc06-fit-and-finish/slice02-packed-format-authority/cc-prompt.md`, relative to project01-mvp. Operator relays to fresh CC; CDC independently reviews the returned evidence. Two-Contributor Workflow, Expedited Mode. Predecessor slice01 is independently accepted, including iteration01; its unresolved format questions remain unresolved.

## Assignment, fences and permitted changes

Evidence-only investigation: produce field-applicable primary-source claims and independent witnesses for discovery DD.UA, persistent-base scalar addresses and galaxy interpretation. A supported conclusion or a precisely bounded unresolved result is acceptable; neither is permission to repair production code or select public policy.

Source checkout `/Users/oubiwann/lab/oxur/nms-copilot`, branch main, exact revision `a800950ef3558fcf463517696c3c140061e7353d`, must remain clean. Work only in the planning worktree `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/planning`, branch planning. Record its HEAD after this assignment has been committed; do not invent that future commit. Check both worktrees at intake and return. If main drifts, stop dependent comparisons and return the mismatch to CDC; unrelated planning dirt is not permission to include it.

Permitted committed changes are only these five paths beneath this slice:

- `artifacts/field-authority.md`
- `artifacts/witnesses.json`
- `artifacts/execution.md`
- `closing-report.md`
- `ledger.md` (evidence, disposition and notes only; criteria unchanged)

Preserve this issued prompt, slice plan, author-source-leads, all predecessor records, parent plans and original project acceptance population. No production/dependency/config/fixture/cache/game-file changes. Temporary read-only probes and extracted public payloads may live under a disposable directory in /private/tmp; no new installed application, repository tooling product or committed probe harness is authorized. Do not run binaries from downloaded save editors.

## Required reading and intake

Read the active prompt, slice-plan.md, ledger.md and artifacts/author-source-leads.md completely. Load applicable collaboration-framework/project-management/work-verification/engineering-methods skills by name; obey their intake, evidence and return guidance. Rust-guidelines governs any disposable Rust probe; the author applied API-37 (SHOULD, separate field representations) and AP-33 (AVOID silent error loss). Their exact sections are in `/Users/oubiwann/.codex/skills/rust-guidelines/guides/02-api-design.md` and `guides/11-anti-patterns.md`. Read those sections including examples and caveats; no public newtype design is prescribed by this investigation.

The following bounded readings supply the contract before dependent work:

| Input, revision | Required scope | Purpose |
|---|---|---|
| Parent arc-plan.md, current planning HEAD | Capability; Preserved authority and review units; Closure gate; Workflow; Current bounded closeout sequence | Evidence-before-repair sequence and authority |
| project01-mvp/closure-preparation.md, current HEAD | Ordered route to closure; Active assignment; Gates and authority | Parent closure gates remain pending |
| Sibling slice01 cdc-verification.md, current HEAD | Initial review finding R01; entire Iteration01 independent review and final close | Accepted population and correction limits |
| Sibling slice01 artifacts/format-contract.md, CC c94a208 plus unchanged current bytes | Entire Field-family findings, Downstream identity and cache/query effects, Bounded next repair recommendation, Iteration01 correction | Format ambiguity, caller effects and validation distinctions |
| Sibling slice01 artifacts/caller-map.csv, c94a208 | All 20 data rows, every column; check totals and field-family classifications | Complete accepted caller inventory; final CDC note corrects the raw definition count |
| Source crates/nms-save/src/model.rs at main fence | Complete non-test definitions SaveRoot through PackedGalacticAddress and its impl; DiscoveryManagerData through DiscoveryData; PersistentPlayerBase | Context paths, raw representation, serde behavior |
| Source crates/nms-save/src/convert.rs at fence | Complete non-test conversion implementations | Current DD.UA/base Euclid substitution, active context and expanded player handling |
| Source crates/nms-core/src/address.rs and system.rs at fence | Address module representation/range documentation; full GalacticAddress new/from_packed/accessor/portal/glyph/signal methods; full SystemId definition/identity methods | Existing 48-bit contract and downstream identity implications |
| Source crates/nms-save/src/lib.rs, decompress.rs, mapping.rs at fence | Full parse_save_file/sanitize_for_json helpers, detect_format/decompress_save; KeyMapping bundled/deobfuscate/walk, is_obfuscated/deobfuscate_json and required mapping-file definitions | Concrete lossless-field JSON probe route |
| Pinned NMSCoordinates files in author-source-leads | Full PersistentBases, HexToAll, GetGalacticCoordHex, GalacticToPortal, PortalToHex, VoxelToHex and every directly called coordinate/sign helper or constant | Trace actual field caller; establish reference applicability before comparing outputs |
| Existing libNom.io at 652aabb0147888776a088909cdb13bccbd3d9588 | Archive inventory query specified below; relevant full address schema/decoder bodies discovered by bounded field search | Public population and independent field structure |

For each text read, record actual loaded bounds/revision and recover truncations. For structured queries record commands, population/projection, complete bounded outputs, excluded records and failures separately. Hashes, successful commands and grep hits are not full-body reading evidence. Follow newly found semantic calls to complete bodies before claiming their meaning; avoid unrelated whole-repository reads. At context recovery revalidate fences and reload contracts no longer available.

Before writing dependent artifacts, record a short source-cited readback in execution.md: field families, what the base reference does and does not establish, independent oracle provenance, sampling limits, preserved ambiguity, sanitation, and source immutability. The easily missed constraint is that a base decoder does not establish DD.UA semantics, and numeric width does not identify format. Public fixtures are observations, not independent expected outputs by themselves.

## Concrete investigation method

1. Fence source, planning and each reference. Pin NMSCoordinates to `b8f765b03082cf5e130a538896db6d67481f9bae`; verify file digests against author-source-leads. GitHub raw/API acquisition or existing local bytes are permissible. Trace the actual PersistentBases caller to HexToAll and its helpers, distinguishing comments, executable behavior and game/save applicability. Record field paths, source ranges, commit, digest and language behavior. Check C# string slicing, prefix handling, short input/leading zeros and planet formatting above nine before treating any output as an oracle. A transcription into another language must be labelled and checked against complete reference logic; it is not execution of the original app.

2. Seek discovery-specific primary authority independently. Search the pinned reference and existing libNom.io/MBINCompiler reference trees for DD.UA, DiscoveryData, DiscoveryManagerData, PersistentPlayerBases, GalacticAddress and RealityIndex; record all relevant hits and full-body follow-ups, including schema-only results. Bound additional acquisition to at most two primary source repositories selected for an explicit discovery/save-field decoder lead. Pin revisions, record negative searches and unavailable bodies. Exhausting this bounded search leaves discovery authority unresolved; it does not establish absence of authority everywhere. Do not use forum claims or portal-only conversions as save-format proof.

3. Compare exact values with provenance. Chili's independent user-confirmed game-visit witness is `0x212B0004B2446A` => portal `212B04B2446A`, Euclid, planet 2, SSI 299, voxel X=1130/Y=4/Z=-1244. Accepted slice01 records show current discovery export `2B0004B2446A` and the portal-shaped control `212B04B2446A`. Reference computations must keep candidate format applicability explicit; agreement with Chili alone supports one witness, not all discovery/base/galaxy cases.

   Also evaluate reference-derived synthetic diagnostics: `0x212B0904B2446A` (candidate galaxy 9), `0x012B0004B2446A` (planet zero; width may fit 48 bits), zero, leading-zero strings, malformed strings, negative integers and u64 maximum. Add a planet A/F case when checking decimal planet formatting. State which cases are representational probes versus valid game inputs. Derive expected components/portal strings independently of nms-core using the traced field-applicable reference or independently expanded observation. If reference code fails, disagreeing comments or string quirks are findings; do not silently normalize them into a supposedly authoritative oracle. Keep literal reference behavior, candidate mathematical interpretation and prospective product policy separate. The reference-derived galaxy-nine expectation is conditional on the candidate split being applicable, not a discovered nonzero-galaxy fact.

4. Use a reproducible bounded public-save population. The existing archive is `workbench/libNom.io/libNOM.test/Resources/TESTSUITE_ARCHIVE_PLATFORM_STEAM.zip`, relative to main. Inventory all ZIP members without extracting all payloads; author inventory was 119 members, 51 .hg save-family members excluding mf_ metadata, zero JSON. Verify counts, actual naming and payload formats rather than assuming modern compatibility. Select at most two eligible non-metadata save payloads in archive member order, record zero-based ZIP member ordinal, byte digest, byte length and selection rule, and use anonymous save-A/save-B aliases. Do not commit internal member names, account identifiers, raw save bodies or broad logs. Preserve every selected decode failure; do not replace failed selections or expand the sample silently. Other platform archives are outside this population.

   A concrete probe route preserving fields absent from typed models is: read selected bytes; call existing `nms_save::decompress_save`; trim trailing NUL bytes and use the same UTF-8 handling as inspected sanitize_for_json; call `nms_save::deobfuscate_json(bytes, &nms_save::KeyMapping::bundled())` to retain a `serde_json::Value`. Do not serialize a typed SaveRoot back to JSON: unknown fields such as teleporter data would disappear. A disposable Rust probe in /private/tmp may use an isolated Cargo manifest referencing the fenced nms-save path and serde_json = "1" (the inspected existing requirement); no workspace manifest/lockfile mutation. Use a temporary target directory or existing compatible build dependencies. Record its exact temporary source/commands/digest in sanitized execution evidence sufficient for CDC replay; build/parse failures remain explicit. This route has inspected public APIs, not a claim of an already executed fixture decode.

   For every selected successfully decoded save, query all discovery records at DiscoveryManagerData.DiscoveryData-v1.Store.Record[*].DD.UA, all available BaseContext and ExpeditionContext PlayerStateData UniverseAddress/PreviousUniverseAddress (with RealityIndex and all GalacticAddress fields), all PersistentPlayerBases[*].GalacticAddress in both contexts, and all TeleportEndpoints found under those player contexts (full location subobjects only). Verify paths on actual JSON, include alternate legacy PlayerStateData paths if present, and report absent/null/empty/malformed separately. Inspect any field-bearing wrapper completely. Record denominators, JSON kinds, bit widths, counts by context/discovery type/galaxy when present, excluded unrelated payloads and decode failures. Store aggregate distributions and minimal atomic location witnesses, not complete owner/name/seed/discovery payloads.

   Pair scalar and expanded location only through an explicit independently meaningful identity/location relationship in the raw save, or a separate game observation. Shared names, coincident values and equality under the candidate decoder alone are insufficient. Describe pairing rule, matched/unmatched counts and every retained atomic pair; if no independent pair exists, say so and leave that gate open. Do not infer shared DD.UA/base layout from nearby co-occurrence. Absent independent nonzero-galaxy pairs must remain a limitation.

5. Optional private evidence is conditional. No decoded candidate was located in the author's bounded /private/tmp/*nms* search, and old project02 snapshots are not supplied. If a known prior snapshot path is already available in session evidence, inspect that named path; otherwise mark private population unavailable without a broad home-directory crawl or stalling public work. Apply the same population/identity/pairing discipline and stricter sanitation to any supplied snapshot. Unavailable means unknown coverage, not a zero-record finding.

6. Assemble a decision packet. Each claim in field-authority.md must link actual save-field caller -> complete primary function/schema -> independently expected or observed witness -> applicability/confidence -> remaining gate. Keep documented, observed, inferred and unresolved labels. Cover discovery, persistent base, expanded player/teleporter, portal/core 48-bit, packed galaxy and SystemId/query/cache implications individually. State which bounded repair can now be designed and which consequential choices remain CDC-owned: format adapters and width compatibility; negative/zero/missing/error policy; galaxy attribution; any API/identity/cache transition. Give a concrete re-entry condition/evidence owner for unresolved choices; no patch or invented target policy.

## Evidence representation and checks

Use witnesses.json as a compact array or keyed object of atomic witnesses, with stable IDs, kind (game-confirmed, save-paired, reference-derived-synthetic or diagnostic), field family, anonymous input provenance, raw scalar kind/value if relevant, independently expected components and portal (or null with reason), observed output/error, decoder/source pin, derivation method, applicability and limitations. Keep unavailable expected values null rather than filling them with the candidate's output. Include separate population summaries/failed selections; every claim links its witness ID.

Example W-chili is a game-confirmed DD.UA witness with the scalar and portal above, expected galaxy 0 and X/Y/Z/SSI/P components, observed current export copied from accepted prior execution and clearly marked retained (not fresh), plus any newly executed reference result in a separate observation. A synthetic W-g9 has no live-game provenance and cannot close a general nonzero-galaxy gate merely by splitting bits. This schema supplies replayable claim-level evidence without forcing a successful decoder conclusion.

| Rows | Required check and expected observation | Rejects / limits |
|---|---|---|
| PA-01 | Exact fences and complete bounded intake; pre-edit readback cites actual constraints | Silent drift/truncated reading/using base authority for discovery |
| PA-02/03 | Read full reference callers and bodies, search bounded discovery sources; each family supported or precisely unresolved | Comments/schema-only/portal-only treated as semantic decoder proof |
| PA-04 | Replay Chili and all synthetic diagnostics with independently derived expected values or explicit unavailable oracle | Current 48-bit decoder treated as its own oracle; width heuristic; failed reference silently corrected |
| PA-05 | Independently recount archive/selected saves and JSON populations; validate atomic pairs and sanitation | Hidden exclusions, unavailable = zero, matching via candidate algorithm alone |
| PA-06 | Walk accepted caller inventory against conclusions and future gates | Starting repair or changing public policy on a partial witness |
| PA-07 | Parse witnesses.json, verify every evidence link/ID, inspect entire named-path diff, scan identity leakage, diff whitespace and both worktree states | Structural pass mistaken for semantic acceptance; omitted failure/unrun attempts |

No production test/build/coverage suite is a gate for this evidence-only slice; previous source validation remains dated evidence. Disposable probe compilation validates only its executable route. If any accidental production edit occurs, restore only your own edit and report it; no opportunistic repair. Preserve all attempted command outcomes and separate unavailable, failed, unrun and observed results. Do not claim original C# execution when only reading/transcribing it.

## Return, commit and author readiness

Walk all seven unchanged ledger criteria with attested evidence or explicit unresolved outcomes permitted by each criterion. Write closing-report.md only after actual execution: intake/readback extent, changed paths, commands/outcomes, complete artifact inventory, per-row rationale, self-review, deviations, remaining gates and next repair readiness. CC proposed-done is not CDC acceptance; leave arc/project closure pending and identify independent review as the gate. Do not pre-create CDC decisions or success receipts.

Commit only the five allowed paths on planning using explicit named-path discipline (git commit --only); verify both required trailers exactly once:

    Co-authored-by: Codex <noreply@openai.com>
    Co-authored-by: Billo AI <ai-engineering@billo.systems>

Return commit ID, fences, row disposition and unresolved semantic gates to the Operator for CDC review. No external thread messaging is authorized by this prompt. Source mismatch, need for more than two primary repositories/two save payloads, production/API/policy changes or criterion changes go to CDC before affected expansion; proceed with independent in-scope work where possible.

Author readiness: inspected primary base caller and complete decoder/helper chain, current field families/caller inventory and the concrete public JSON decoding APIs. The archive is inventoried, not decoded; private data availability is explicit. This packet supplies bounded source/data queries, discriminating Chili/width/galaxy witnesses, a viable temporary probe route and precise unresolved-result rules. Field distinctions (API-37) and preserved errors (AP-33) interact with the oracle and population rules without prescribing public policy. No new product API is necessary. Full required reading is confined to active contracts and semantic bodies; queries are capped at two public payloads and two additional source repositories. Author self-check establishes handoff readiness, not acceptance of future evidence.
