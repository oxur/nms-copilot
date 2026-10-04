# CC assignment — Slice03 decoded local location witnesses

Issued 2026-10-03, initial pass. Current assignment `arc06-fit-and-finish/slice03-decoded-location-witnesses/cc-prompt.md`, relative to project01-mvp. Operator relays to fresh CC; CDC independently reviews. Two-Contributor Workflow, Expedited Mode. Slice02 is independently closed after its provenance correction; generalized format and production policies remain unresolved.

## Outcome, fences and allowed writes

Evidence-only investigation of **one known local save**. Produce complete scoped raw-field populations, independently expanded location oracles and carefully classified relationships/candidate comparisons. This continues evidence-before-repair, selecting no public API/compatibility/galaxy/cache or invalid-value policy. No production implementation, private identity/account assignment or game-file mutation.

Source root `/Users/oubiwann/lab/oxur/nms-copilot`, main `a800950ef3558fcf463517696c3c140061e7353d`, must remain clean. Planning root `.worktrees/planning`, branch planning; record actual HEAD after author packet commit. Input is only source-root `workbench/save-games/save.hg`, alias local-save-A, expected raw bytes 2918235 and SHA256 `ee786384b19ae3a7974cb3130cf16b8823f3dc2a930bd55cfa092015a244c89e`. Before/after file metadata and read digest must agree. If source/input drifts or decoding fails, preserve failure and return that dependent blocker; never silently use save2.hg, another save or a new source revision. The author's temporary decoded JSON is not your input or fresh receipt.

Only these six committed CC paths are authorized beneath this slice:

- artifacts/population.json
- artifacts/location-witnesses.json
- artifacts/field-assessment.md
- artifacts/execution.md
- closing-report.md (after actual work)
- ledger.md (evidence/disposition/notes only, criteria unchanged)

Preserve author-input.md, author-probe-summary.json, issued prompt, plans, parent ledgers and prior close sets. Raw input/decoded JSON/disposable probes may remain in a private temporary directory outside Git, then be removed after retaining sufficient sanitized replay evidence. Never commit raw dumps, owner/account/native IDs, names, seeds or full record bodies. Allowed: byte/mapping hashes, anonymous aliases, record ordinals/JSON pointers, location scalars/coordinates/galaxy, counts/types and relation booleans. Do not install/download/run a save-editor app. No external thread messages authorized.

## Required reading and preflight

Read this prompt, slice-plan.md, ledger.md, artifacts/author-input.md and artifacts/author-probe-summary.json fully. Applicable collaboration-framework/project-management/work-verification/engineering-methods obligations remain. Rust-guidelines applies to any optional disposable Rust probe; no new production Rust is requested. Author applied API-37 (SHOULD, representation distinctions) and AP-33 (AVOID, explicit error/unknown states); load those complete sections at `/Users/oubiwann/.codex/skills/rust-guidelines/guides/02-api-design.md` and `guides/11-anti-patterns.md`.

| Input/state | Required extent and purpose, before dependent work |
|---|---|
| Parent arc-plan.md | Capability, Preserved authority/review units, Closure gate, Workflow, Current corrective preparation/Current bounded closeout sequence: no scope/acceptance or production policy change |
| project01-mvp/closure-preparation.md | Preserved scope/acceptance population and Gates/authority: original capabilities stay pending |
| Sibling Slice02 cdc-verification.md | Entire final Iteration01 review/close and initial Independently checked evidence: accepted source applicability, historical dates, absent pair limits |
| Slice02 artifacts/field-authority.md and witnesses.json | Required-full field packet; all 11 witnesses/reference pins and population limits. These are retained predecessor evidence, not local-save results |
| Slice01 artifacts/caller-map.csv | Required-data: all 20 rows/every column; associate any prospective change with existing consumers without proposing a patch |
| Fenced main crates/nms-save/src/lib.rs/decompress.rs/mapping.rs | Full parse_save_file/sanitize_for_json; detect_format/decompress_save/decompress_blocks/parse_block_header and constants; KeyMapping::bundled/deobfuscate/walk, mapping definitions, is_obfuscated/deobfuscate_json: raw JSON route and caveats |
| Fenced model.rs / convert.rs | Full non-test SaveRoot/GameContext/PlayerStateData/UniverseAddress/GalacticAddressObject/PackedGalacticAddress/DiscoveryManagerData/DiscoveryDataV1/DiscoveryStore/RawDiscoveryRecord/DiscoveryData/PersistentPlayerBase definitions and conversion impls: field spelling/defaults and forced galaxy zero |
| Fenced nms-core address.rs / system.rs | Address bit layout/constants and full new/from_packed/accessors/portal conversion; SystemId definition/from_address: exact existing core contract, not the save oracle |
| Pinned primary reference bodies | Slice02 recorded NMSCoordinates PersistentBases/HexToAll/GetGalacticCoordHex/GalacticToPortal and Companion decodeHexAddress/signed plus discovery/base callers, full bodies/constants. Use full pins/hashes from accepted receipts; no repin/new repository hunt |
| Installed native LZ4 route, only if used | `/opt/homebrew/opt/lz4/include/lz4.h` complete LZ4_decompress_safe comment/declaration and version constants; author library/digest from summary. Explain any changed native environment before adopting it |
| local-save-A raw JSON | Required-data population/projection below over the whole pinned input, plus complete relevant address/relationship wrappers. Unknown location/identity fields encountered are inspected locally and counted, without publishing identity values |

Load complete required sections, recover truncation, record source/revision and actual read extents separately from structured queries/denominators/exclusions/failures in execution.md. Hashes/grep hits/author summaries are not your full-body reading. Before dependent artifact edits, append a source-cited readback: input fence, JSON route, spelling variants, raw association versus candidate decoding, independent expanded oracle, sanitation and unresolved policies. Missing/conflicting inputs block affected work; do not broaden the sample.

## Source-grounded execution recipe

1. Fence and privately copy/read exactly local-save-A. Record before/after metadata, bytes and digest, excluding identifying paths in public summaries beyond the generic workbench path. Derive the observed record count from decoded bytes, not the author's expected count. Failures keep the last independent evidence unchanged; no private home crawl or account/root guess.

2. Decode complete raw JSON, preserving fields omitted by typed SaveRoot. Recommended primary route: a disposable temporary probe calls existing nms_save::decompress_save, applies the inspected trailing-NUL/lossy UTF-8 handling, then nms_save::deobfuscate_json with KeyMapping::bundled into serde_json::Value. Existing Rust deps/public APIs are verified; source/dependency/workspace manifests stay untouched. No typed deserialization/serialization round trip may remove unknown endpoint fields.

   A concrete exercised alternative is the existing native Python route described in author-input: Python ctypes loads already installed liblz4, declares LZ4_decompress_safe pointer/pointer/int/int -> int, reads each NMS little-endian 16-byte header, requires magic 0xFEEDA1E5, complete compressed interval and declared output <=0x80000; allocates output capacity, rejects negative result, and concatenates exactly returned bytes. Trim trailing NUL, replace invalid UTF-8 as the fenced parser does, json.loads with integer precision preserved, then apply the pinned primary/legacy/fixup mappings using first-primary-key precedence and unknown-key preservation. Do not substitute LZ4 frame decoding for raw blocks. Explain which route ran; preserve commands and sufficient sanitized probe source/algorithm for independent replay, not a new tooling product. Capture input/decompressed lengths, blocks, trim count, UTF-8 invalid/replacement caveat, mapping hashes/versions, duplicate JSON-key/map collisions and numeric representability caveats. A duplicate/collision must not silently create a trustworthy identity pair. Unknown field/value kinds remain observable. Author observation (20 blocks/10278541 bytes/one NUL/invalid UTF-8) is a baseline check, not proof or a forced outcome.

3. Project the entire scoped population into population.json. Discovery root paths: DiscoveryManagerData['DiscoveryData-v1'].Store.Record[*] and any legacy PlayerStateData discovery wrapper actually present. Record each discovery type and DD.UA JSON kind/value-state/width, absent DD/DT/UA or malformed wrappers, duplicate records and exclusions. Preserve native integer representation precision; do not coerce through JS Number or classify format solely by width.

   For BaseContext and ExpeditionContext and a legacy top-level PlayerStateData if present, count every player wrapper and all current/previous **UniverseAddress and UniversalAddress spellings separately** (including PreviousUniverseAddress/PreviousUniversalAddress). Inspect complete expanded GalacticAddress/RealityIndex members with kinds/ranges/state, without filling missing fields from source defaults. Count all PersistentPlayerBases[*].GalacticAddress and base kinds (including freighter/player-ship/other entries); all TeleportEndpoints[*], teleporter types and both address spellings. Count absent/null/empty/malformed/populated per collection/field and all explicit reality values separately from candidate-decoded reality distributions. Include denominator for every statistic and excluded record reason. Author shape baseline: 3243 discoveries; BaseContext 54 bases/207 endpoints and ExpeditionContext 0/17; endpoint spellings are UniverseAddress in this sample. Recompute, report discrepancies.

4. Classify relationships **before** running any packed candidate comparison. First inspect native explicit cross-record identifiers/links locally. If none supports a shared identity, record absence and keep native identities out of artifacts. For bounded raw corroboration, consider only nonempty unaltered Name and finite three-element Position fields: HomePlanetBase records and Base endpoints in the same context with exact equality of both fields and bidirectional unique matches. Missing/duplicate/ambiguous/type-incompatible/lossy-key cases are excluded with complete counts. Conservatively exclude a Name containing U+FFFD from this association rule unless its unaltered original bytes are independently demonstrated; report that exclusion separately from confirmed replacement. This is raw-corroborated association, not a shared ID or game-confirmed pair. Neither name-only, scalar equality nor candidate-decoded coordinate equality provides independent linkage. Do not let the candidate select a convenient endpoint or break ties.

   Author example to check: BaseContext base ordinal0 / endpoint39, scalar 0x40050003AB8C07; expanded endpoint RealityIndex0/X=-1017/Y=3/Z=-1352/SSI5/P4. Validate full raw eligibility, equality and reverse uniqueness afresh. Preliminary all-base matching differed from this stricter rule; its 44 matches/one ambiguity are not acceptance counts. Retain all unmatched/ambiguous ordinals and reason counts. If no strong or corroborated link survives, explicit unavailable outcomes are allowed; do not invent pairs.

   Exercise the association rule with one accepted raw-corroborated case if available, one actual ambiguous/nonmatching case if available, and labelled synthetic controls made only in temporary copies: duplicate an endpoint (must become ambiguous), change Position while holding Name equal (must reject that edge), and change Name while holding Position equal (must reject that edge). Candidate decoding is excluded from this predicate in every case. Record controls as synthetic transformations of a pinned witness, never new game observations. These checks reject name-only matching and silent tie selection.

5. Build independent expanded oracles and compare source-applicable candidates. From explicit P/SSI/Y/Z/X fields derive portal string `P(1 hex) + SSI(3 hex) + (Y&255)(2 hex) + (Z&4095)(3 hex) + (X&4095)(3 hex)`, keeping RealityIndex separate. Validate required member types/ranges before formatting; outside-range/missing values remain diagnostic/unresolved rather than silently masked into a valid oracle. For the example above, independently expanded portal is `400503AB8C07`. Raw ordinal174 expanded endpoint has RealityIndex18, X=-75/Y=-60/Z=-145/SSI324/P3; it is a nonzero expanded observation only until an independently meaningful scalar relationship is shown. Do not call this a nonzero packed/game witness by itself.

   For every eligible raw-corroborated base association compare the pinned base candidate to expanded components and portal, preserving all agreements/contradictions and complete comparison counts. Original NMSCoordinates quirks and field applicability remain; separately state literal source versus candidate math. Seek discovery-specific direct raw relationships independently; exact DD.UA/base scalar equality is at most an address-equality correlation, never proof of shared family layout. If no independent discovery relation exists, record it explicitly and keep Chili as the one retained game witness. Do not derive expected values from nms-core or the same candidate decoder. Neither runtime original reference execution nor generalized save meaning is established by a transcription.

6. Produce a field assessment mapping each claim to raw JSON pointer/ordinal + applicable source pin + independently expanded expected values + actual candidate observation + relationship class + limits/remaining gate. Keep independent game-confirmed, explicit native-ID linkage, raw-corroborated association, address-equality correlation, source-derived synthetic and unavailable classes distinct. No silent widening from a one-save result to a platform/version/general decoder. Explain which bounded repair design now has enough evidence for CDC consideration and which decisions need more input/producer authority. CDC owns discovery/base adapters, invalid/zero/missing/over-width behavior, galaxy attribution, public identity/API/cache transitions; do not implement or choose them.

## Evidence schema and validation

population.json contains pinned input/mapping/decoder provenance, every scoped collection/context denominator, field-state/type/range/width distributions, explicit-versus-candidate galaxy counts, all exclusions/failures, and relationship/comparison counts. Record unknown coverage as null with reason, never zero.

location-witnesses.json retains all relevant association outcomes as anonymous ordinals/locators and flags; minimal atomic locations/scalars and expected/observed/error, confidence basis, actual source pin, derivation method and caveats. No raw Name/Position/identity values: equality/type/finite/lossy/uniqueness flags plus record pointers permit CDC to replay privately. Include ambiguous/excluded relation locators and complete counts rather than storing only successful witnesses. Include the example/nonzero expanded case if present and synthetic controls with transformation provenance. Every assessment claim links stable witness IDs. Do not retain full save/record bodies.

| Row | Meaningful gate / incorrect result rejected |
|---|---|
| DW-01 | Fresh fenced bytes/metadata and actual intake/readback; source or input drift cannot be hidden |
| DW-02 | Complete raw decode and wrappers/mapping provenance; typed dropping of endpoints, silent collisions/replacement and integer narrowing rejected |
| DW-03 | Entire one-input recount with contexts/spellings/states/type denominators; wrong spelling or default zero cannot masquerade as absent/populated data |
| DW-04 | Full bidirectional association accounting and duplicate/changed-name/changed-position controls; candidate-only matches and tie selection rejected |
| DW-05 | All eligible comparisons against explicit expanded oracles plus nonzero-case limit; candidate self-oracle and correlation-as-proof rejected |
| DW-06 | Source/witness/relationship applicability and consequential-owner gates checked against accepted caller inventory; no patch or universal claim from one save |
| DW-07 | JSON syntax/ID/link/count consistency, exact allowed diff, privacy scan, raw/source/reference hashes/status, artifact inventory, failures and seven-row walk |

No broad workspace test/build/coverage/lint suite is a gate for this evidence-only slice. Temporary probe compile/check is only route validation. Report all actual failures/unrun operations and probe platform/dependency limits; no described step becomes an executed receipt. Reading and structural checks do not independently accept semantic claims.

## Return and commit

Write closing-report only after actual execution, with this assignment/source/input state, intake/readback, every row's attested evidence/disposition, commands/results/failures, full artifact inventory, scope/sanitation checks, deviations, remaining gates, bubble-up and readiness assessment. Include two already-authored supporting artifacts separately from CC outputs. CC proposed-done is pending CDC acceptance; higher-scale/project gates remain open.

Commit the six explicitly allowed paths only on planning with git commit --only; stage new files by exact path, preserving unrelated edits. Both trailers exactly once:

    Co-authored-by: Codex <noreply@openai.com>
    Co-authored-by: Billo AI <ai-engineering@billo.systems>

Return commit, fences, per-row evidence, relationship limits and unresolved policies to the Operator. Expansion to another input/repository, source/fixture/API/policy/criterion changes or source/input drift returns to CDC before affected work; continue independent in-scope analysis where possible. Current bounded author readiness is in slice-plan.md and author-input; it is not acceptance of your future results.
