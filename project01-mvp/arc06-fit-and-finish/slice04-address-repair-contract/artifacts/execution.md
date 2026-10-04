# Slice04 execution record

## CC intake and readback

This is a bounded evidence and design-preparation assignment for the address-repair contract. It does not authorize production changes, adoption of a public API, save-format policy, or closure of this slice. The assigned output is a decision packet grounded in the pinned source, accepted predecessor evidence, the immutable author baseline, and a fresh synthetic CLI characterization. All ledger criteria remain open for CDC disposition.

### Worktree and immutable inputs

- Planning worktree: `.worktrees/planning`, branch `planning`, starting HEAD `6b553d6711c677655f3b49f37a1595e0ab55fb17`, initially clean; `main` source worktree was clean at `a800950ef3558fcf463517696c3c140061e7353d`.
- Source inspected is the pinned `main` checkout at `a800950ef3558fcf463517696c3c140061e7353d`; no source files were changed or save files opened for this assignment.
- Immutable author baseline: `artifacts/author-baseline.json`, SHA-256 `0c3d0208a79f23cd8ff44ea101ad5d4909b2c0921aef4435d8291b3d11e30bdf`. Its synthetic templates and declared packed/expanded cases are the only source for fixture values. Author-observation notes are not independent evidence and are not copied as runtime outcomes.
- Slice03 Iteration02 CDC receipt was read in full. It accepted all seven predecessor rows and closed Slice03. Its retained decoded-location evidence may be reused with its original scope: one pinned game save; 44 unique raw Name+Position base/endpoint corroborations; not proof of shared native IDs, all-save behavior, or a general address policy.
- Slice01's complete 20-row caller-map and Slice03's full field assessment were read. The parent Arc06 preparation and sequence were read; they authorize contract preparation before any separate repair implementation.

### Source read set and limits

All reads below were from source commit `a800950ef3558fcf463517696c3c140061e7353d`.

- `crates/nms-core/src/address.rs:1-544`; `system.rs:1-58`; `spatial.rs:1-40`; `delta.rs:1-110`.
- `crates/nms-save/src/lib.rs:1-65`; `model.rs:1-165,197-239,279-340`; `convert.rs:1-208,292-end`.
- `crates/nms-graph/src/extract.rs:104-180`; `model.rs:20-167,165-256,256-357,706-734`; `import.rs:36-83,92-181`; `edges.rs:40-240`; graph query and route implementations; `crates/nms-query/src/find.rs:32-end` and `show.rs:1-104`.
- `crates/nms-watch/src/snapshot.rs:1-61` and `delta.rs:1-69`.
- `crates/nms-cache/src/data.rs` (complete), `freshness.rs:1-99`, and `serialize.rs:1-114`.
- `crates/nms-cli/src/convert.rs:1-128,278-292,425-435`; `info.rs:1-121`; `import.rs` (complete); `export.rs:1-106`; `main.rs` relevant command declarations and dispatch arms.
- `crates/nms-copilot/src/dispatch.rs:454-533`; MCP conversion tools `:646-696`; copilot model loading `:353-360`.
- A recursive source search for `from_packed`, `to_galactic_address`, `galactic_address.0`, `SystemId`, `address_to_id`, `node_map`, and `PlanetKey` covered `nms-core`, `nms-save`, `nms-graph`, `nms-query`, `nms-watch`, `nms-cache`, `nms-cli`, and `nms-copilot`. Test matches were distinguished from runtime call sites. Relevant consumers are enumerated in `impact-map.csv`.
- Build metadata read: root `Cargo.toml`, `crates/nms-cli/Cargo.toml`, and `Cargo.lock`. No dependency or lockfile change is authorized.

### Readback: behavior and evidence classes

The public core `GalacticAddress` is a 48-bit portal address: planet, system, and voxel fields are packed into the low 48 bits; reality/galaxy is a separate `u8`. `from_packed` and field construction mask their inputs. Portal text is twelve glyph/hex digits; the signal-booster transform has its own coordinate offsets. Those are current source facts, not evidence that save `DD.UA` uses the same representation.

The save model has a distinct packed wrapper whose deserializer accepts integer and string forms and whose conversion calls the 48-bit core constructor. Expanded save address objects carry voxel/system/planet/reality fields separately and also route through the core constructor. Discovery conversion currently supplies galaxy zero; base conversion currently supplies galaxy zero; player state derives reality from its expanded wrapper. Current implementation therefore loses or collapses some galaxy identity on save-derived paths. `SystemId` clears the planet nibble from packed core address and does not include reality. Graph maps key systems by that ID while spatial indexes are partitioned by galaxy. These facts create a concrete same-address-across-galaxies identity question; they do not establish the correct product policy.

Three evidence classes stay distinct:

1. Accepted game-save evidence from Slice03 is bounded to its pinned input and its specific decoded-location corroborations.
2. The Slice04 parser probes use only synthetic baseline fixtures and characterize present CLI behavior. They cannot establish how a live game save should be interpreted or what production repair to adopt.
3. Repository source establishes current code paths and likely effects, but a source-flow inference is not a runtime observation. In particular, import statistics do not expose which colliding row survives.

A successful core `from_packed` call, portal roundtrip, or core unit test establishes only the core representation's behavior. It does not prove save decoder field width, accepted save encodings, discovery/base semantic equivalence, galaxy ownership, or correctness against an NMS save oracle. Conversely, an `nms info` parser failure identifies current typed parsing behavior for the synthetic fixture; it does not determine whether the fixture value is valid in a real save.

### Required fresh work and output boundary

The planned fresh characterization is 45 explicit `nms info --save <synthetic fixture>` invocations (16 packed-address injections independently into discovery and base templates, plus 13 expanded-wrapper cases) and six prompt-declared controls. Each invocation must retain argv, exit status, errors, relevant output projection and a digest of complete stdout where the projection is used. Failures and non-observable outcomes remain recorded. Fixtures, runner, and temporary raw captures belong under `/private/tmp`; the six authorized planning paths are the only Git changes.

The decision packet is eight recommendations, each with evidence, tradeoffs, caller and compatibility impacts, a discriminating acceptance oracle, owner, unresolved state, and re-entry condition. The impact map accounts for every one of the 20 Slice01 seed rows and the direct consumers found in the source scan. Recommendations remain proposals; unresolved product and format policy remains with its named authority. The closing report can attest only CC proposed-done. No ledger status advances pending CDC.

### Read/tool failures retained

- An initial source-read command was run from the planning worktree, which has no `crates/` tree; the requested source paths were unavailable there. All required reads were repeated from the pinned source root and completed.
- An initial project-management guide lookup used the wrong path (`/Users/oubiwann/.codex/skills/project-management/README.md`). The correct collaboration-framework guide index was located and read. This did not change repository state.

## Fresh execution and results

### Build and fixture method

At the pinned source root ran exactly:

- argv: cargo build --offline --locked -p nms-cli
- exit: 0
- stdout: Finished dev profile [unoptimized + debuginfo] target(s) in 0.51s
- stderr warning: proc-macro-error2 v2.0.1 contains code rejected by a future Rust version; Cargo offered future-incompatibilities report id 1.
- binary: /Users/oubiwann/lab/oxur/nms-copilot/target/debug/nms

All fixtures were under /private/tmp and deep-copied independently from the baseline synthetic_save. Packed JSON values retained their JSON types; each of 16 baseline cases was injected separately into discovery DD.UA and base GalacticAddress. Each of 13 expanded cases started with expanded_valid_wrapper at BaseContext.PlayerStateData.UniverseAddress and applied the exact declared operation. Every parser command used the explicit binary and explicit --save fixture path; no auto-detection occurred. Complete argv, injected value/kind or remove operation, exit, stderr, ANSI-free relevant output projection, original stdout SHA-256 and evidence limitation are in behavior-probes.json. The projections are not a save dump and contain only synthetic labels/values.

Fresh counts: 45 parser invocations, 27 accepted and 18 rejected (discovery 16, base 16, expanded 13); all exits were 0 or 1. Both packed families accepted the same nine cases: int-zero, hex-zero, hex14, uppercase-prefix, bare-short-hex, numeric-string, signed-negative, over56 and max56. Both rejected fraction, boolean, null, missing, empty, bad-hex and over-u64. Expanded accepted valid, missing-wrapper, empty-wrapper, zero-object, partial-object, x2048, ssi4095, ssi4096 and planet16. Expanded rejected null-wrapper, y128, planet256 and reality256.

Observed consequences are deliberately scoped to info: base output masks to low48, including negative/max56 to FFFFFFFFFFFF and the 56-bit witness to 2B0004B2446A. Discovery output reports record counts, not its injected address. Expanded info prints raw fields; e.g. X=2048 and SSI=4096 are accepted and shown raw. Core constructor masks inferred from source are not measured by these projections. The accepted SSI4095 and planet16 cases likewise show that parser acceptance is not game-validity evidence.

Six additional CLI executions, all exit 0:

1. convert --ga 0x212B0004B2446A -> Portal Glyphs 2B0004B2446A; SSI 2816 (0xB00), planet 2, Euclid (0).
2. convert --glyphs 212B04B2446A -> portal output; SSI 299 (0x12B), planet 2, Euclid (0).
3. convert --ga 212B04B2446A -> same portal output and SSI 299.
4. import first-order collision rows in galaxies 0 and 9 -> Imported 1 systems (1 duplicates, 0 skipped).
5. import reversed collision rows -> same stats. Neither command exposes retained row. Import source flow suggests the first inserted row survives, but that is not a runtime observation.
6. import distinct 212B/212C system nibbles, both galaxy 0 -> Imported 2 systems (0 duplicates, 0 skipped). This fresh same-galaxy SSI control is deliberately distinct from the baseline CSV that uses different galaxy values.

Import handlers create only an invocation-local model; no persistence is claimed. These 51 final executions are the declared evidence set. No save file was read.

### Attempt log and failures

- The required source extent command was first run from the planning worktree, which lacks crates/. Those paths were unavailable; all extents were re-read from the pinned source root before edits.
- A project-management README was first sought at a nonexistent skill-root path; the correct collaboration-framework project-management guide README was located and read before edits.
- An exploratory attempt opened the baseline by a planning-relative path while cwd was the source root; it failed with FileNotFoundError. The same command then passed /dev/null to info, which correctly failed JSON parsing at EOF. It was a harness-location check, not a declared probe.
- A prompt/source paired sed command requested prompt lines beyond its 110-line file and also tried to read info.rs from the planning worktree. That read failed; the full prompt and info.rs required extent had already been, and were again, read from their correct roots.
- First runner invocation used bare nms rather than the required root target/debug/nms, so subprocess creation failed before a CLI process started. Replaced it with the explicit binary path.
- Next runner pass completed the 32 packed invocations but aborted constructing missing-wrapper because the synthetic root had no UniverseAddress member to delete. Those duplicate outcomes were not retained by the aborted script. The required 32 packed IDs were freshly rerun in the final 51-run pass and retained there; no failed parser case is omitted from the declared results.
- The next harness correction also made absent-wrapper represent absence without deleting a nonexistent key. A full rerun then completed.
- Initial projection included terminal color escape sequences. Updated the projection to strip ANSI styling only; kept the SHA-256 of original stdout. Repeated all 51 final invocations and retained that clean pass.
- No dependency/build blocker, source drift, or user-cache access occurred. No broad tests, format, lint, full save parsing, REPL/MCP live protocol, watcher, cache, or raw game-save reads were run. Those are outside this packet's authorization/proof boundary; future composition tests are specified in the decision packet.

## Analysis applications and preservation checks

Applied the required Rust-guidelines recommendations only to decision framing: API-37 and TD-03 SHOULD support distinct save-packed, portal and identity roles; AP-33 AVOID supports explicit invalid/unknown/default states; EH-04 MUST applies to a proposed future public library error type, not opportunistic alteration of current Box<dyn Error> functions. Read the required engineering-methods investigation/semantic-evidence and implementation-prompt intake sections, and work-verification independent-review/evidence-retention sections. No source changes, Rust code, traits, dependencies, edition change, or reusable test tool were introduced.

The impact map retains all 20 caller-map seed rows and adds eight source-backed consumer rows (graph identity/mutation, import, query association, glyph/base display, watcher, cache, and startup). Total impact-map rows: 28. Search names and source sections are identified in the intake; tests are labeled as tests, not runtime contracts.

Source and planning inputs remained pinned at final check: source main a800950ef3558fcf463517696c3c140061e7353d, clean; planning started at 6b553d6711c677655f3b49f37a1595e0ab55fb17. Immutable author baseline SHA-256 remains 0c3d0208a79f23cd8ff44ea101ad5d4909b2c0921aef4435d8291b3d11e30bdf. Planning criterion inputs as read before ledger dispositions: slice-plan SHA-256 26ac147687fe2590737fc31ca9c713bf147cb8c02aebe05024423fa805b0f204; open ledger SHA-256 57a7d4d1c472004a01781576da3854708c6686eb2ab858b55e0520fba2edc507. Preserved reference hashes: arc-plan 2db0b5d5ff5cfacb929f8b29e15054a7d2d06e013ddf244779355a5a2d293e3f; closure-preparation 739d0503c3f970895c2a128074a9191e0a2524e51b8e501f418ef243e8e7bdab; Slice01 caller-map 73429ceff562f1eaffb3078b5b692d561a428d74bfeb0acd1dd888291865fb86; Slice03 receipt 6bd1e8cc3db0292108fe60b7f9be39dd8e56cfcb9bb481cf8710495b2dda5575; Slice03 field assessment 4cbc91ac914a851eecaf0cfb942b372eac844675218e1f0d65b17e269505888d.

Pre-commit inventory is confined to the six authorized paths: artifacts/decision-packet.md, artifacts/impact-map.csv, artifacts/behavior-probes.json, artifacts/execution.md, closing-report.md and ledger.md. No author baseline, prompt, plan, parent record, predecessor evidence or source file changed. Structural checks and commit outcome are appended after they are performed.

The final output-projection completeness pass added the Hex Glyphs row to the convert-control projections and repeated the complete 51-command set; behavior-probes.json contains that final pass. Counts and exit distribution were unchanged.

The required named-pattern search was run against source directories nms-core/src, nms-save/src, nms-graph/src, nms-query/src, nms-watch/src, nms-cache/src, nms-cli/src, and nms-copilot/src for from_packed, to_galactic_address, galactic_address.0, SystemId, address_to_id, node_map and PlanetKey. Runtime to_galactic_address definitions/callers are in core portal conversion, save conversion, CLI and REPL; there are no such matches in graph/query/watch/cache. The literal galactic_address.0 search finds the base-display mask in CLI info and a save-model test assertion; the cache stores typed addresses and cached identity fields through its own serialization path rather than those named operations. All SystemId and PlanetKey matches in graph/query/watch/delta/route/edges were classified as runtime or tests during inspection. No further direct packed-address decoder or unlisted galaxy source surfaced in the scanned crate set. Nonmatches are search evidence only, not proof of no indirect behavior.

Structural validator attempts: the first local-link check script incorrectly unpacked a one-capture regex and aborted before a result; corrected the validator. The next check exposed that the ledger rewrite script had emitted literal backslash-n separators; ledger was restored to normal Markdown newlines. Rechecked all seven rows against the original criterion, verify, significance, origin and open status; only Evidence/Notes differ. Final structural results follow.

Final pre-commit structural check passed: behavior-probes.json parses and contains 51 unique declared IDs matching the full 45+6 manifest, 27 accepted/18 rejected parser outcomes and recorded argv/status/stderr/projection/digest for every result; impact-map.csv parses and preserves all seven original seed columns for all 20 rows plus S01–S08; decision packet has all D01–D08; all seven local Markdown links resolve; ledger has seven rows whose criterion/verify/significance/origin/status cells match the original and remain open. Source is clean and pinned; planning HEAD is still the recorded starting commit; status contains exactly the six authorized paths. Markdown and CSV/JSON line endings and trailing whitespace are clean.

## Commit receipt

The first six-path commit was eee6af5cfba55c97b2cd3ad38c75b4355512baa1. Its committed path list was exactly the six authorized paths; the commit message contained each required co-author trailer once. Post-commit at that point: planning branch was ahead 19 with no worktree changes; source main remained clean at a800950ef3558fcf463517696c3c140061e7353d; author-baseline and Slice01 caller-map hashes still matched their recorded values. Git reported the working impact-map.csv as CRLF against repository LF policy, so the authorized CSV was normalized to LF and this receipt was added. The commit is amended only for those two authorized artifact updates; the final commit path/trailer/pin/worktree check follows.

A trailer-parser check on the amended commit found that, although both required lines appeared once in the message, separate message paragraphs meant git interpret-trailers --parse recognized only the final line. The message is being normalized to one contiguous trailer block. After that correction the final receipt verifies both parsed trailers exactly once, the six-path commit, clean planning/source worktrees, unchanged source pin, and unchanged baseline/seed hashes.

Final amended-commit audit passed: committed path list is exactly artifacts/decision-packet.md, artifacts/impact-map.csv, artifacts/behavior-probes.json, artifacts/execution.md, closing-report.md and ledger.md. git interpret-trailers --parse returns each required trailer exactly once in a contiguous block. Planning is clean on branch planning, ahead 19; source main is clean at a800950ef3558fcf463517696c3c140061e7353d. Author baseline remains SHA-256 0c3d0208a79f23cd8ff44ea101ad5d4909b2c0921aef4435d8291b3d11e30bdf; Slice01 caller-map remains 73429ceff562f1eaffb3078b5b692d561a428d74bfeb0acd1dd888291865fb86. impact-map.csv is LF in both index and worktree. Ledger RP-01–RP-07 remain open for CDC.
