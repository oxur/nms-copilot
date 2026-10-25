# Slice44 CC finite intake and source-cited readback

Intake only; no implementation, index operation, validator, compiler, test, documentation build, or gate has been launched. The dedicated source checkout is `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/documentation-reconciliation`, branch `codex/documentation-reconciliation`, HEAD `a3f74a86f0ceeb523d7cda62c8c2bc38215669de`; `git status --short --branch` was clean. The exact file was absent before this write and is ignored by `.gitignore:2` (`/workbench`). This file records the required readback only.

## Actual intake extents

**Full reads:** Slice44 CC prompt, sibling slice plan and ledger, dedicated checkout `AGENTS.md`; collaboration-framework and rust-guidelines skill entrypoints plus the complete Rust anti-patterns guide; agent-coordination, testing, work-verification, and methodology-stewardship skill entrypoints; project-management README and guides 02/03/04; engineering-methods guide 07; agent-coordination guide 01; testing guide 03; work-verification guides 03/05; both capture helpers and reconciliation helper; both canonical support snapshots and all 2 CC / 4 CDC support records including their raw channels/status/identities; all 21 contract regions, 83 source identities, 9 package entries, 70 landing-page entries, and 8 retained evidence-input entries; full CONTRIBUTING and README; Slice29 and Slice30 CDC verification records; the 0039 M7.6 source criterion; Arc06 Slice31–43 dispositions; current 20 Rustdoc paragraphs and the relevant following declarations/function behavior; all nine library `lib.rs` declarations, copilot map/MCP child declarations, CLI declaration/help-input boundary, ten Cargo workspace members/targets, core archive feature, and lockfile pin.

**Required-section reads:** Rust documentation guide 13, DC01–12; methodology guide 01 Notes for Codex/Roles and Shared Invariants, Two-Contributor, Autonomous Mode, and model selection; Arc06 plan entries relevant to Slice29–44 and higher open owners. The DC04 required-section order is not applicable to this summary-only change; the established documentation guide's example detail order remains governing. DC03 is one physical summary line, blank line, then details; the approximate 15-word target is not a hard cutoff.

**Required data and visual reads:** the exact 12-path/21-region contract and source manifest; all 20 current paragraphs and their needed source behavior; full core address constants 1–45; full README/CONTRIBUTING; all retained29 raw and render inputs including the 887-byte help capture and 2,029-byte stats capture; capture README and renderer; both complete PNGs viewed at original resolution. The 8 retained29 files are unchanged evidence inputs; raw ANSI SGR and two trailing LF bytes remain part of the capture evidence. The images are safe-fixture CLI stdout rendered monochrome, not desktop/game/theme screenshots. The prior Slice29 broad default-diff EOF reds remain historical reds and are not reclassified.

## Scope and worksheet readback

Authority is one Arc06 slice. Exact baseline and branch are stated above. The contract is `/private/tmp/nms-slice44-contract.json` and the immutable source counterpart is the sibling `artifacts/documentation-contract.json`. It authorizes only 20 existing public Rustdoc first-paragraph regions and one CONTRIBUTING preview namespace block: exactly 12 paths and 21 literal regions. All other docs, examples, headings, links, executable examples, API/non-doc tokens, ordinary comments, tests, dependencies, fixtures, help text, layouts, and paths remain frozen. In particular the private `apply_and_snapshot` summary and CLI `Info.save` help text are excluded. No runtime repair, hidden API, suppression, dependency change, formatter/converter/cache/schema change, or extra exception is authorized. Main's pre-existing protected nine-line agent addition `2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e` remains untouched. Five actual refinements are allowed after the initial attempt; current actual refinement count is zero.

The 20 paragraph targets and source facts to preserve are:

1. `crates/nms-copilot/src/config.rs:171`: `None` selects the embedded default banner; an empty string disables it (`config.rs:171–174`, following `banner: Option<String>` declaration).
2–4. `crates/nms-core/src/address.rs:27,31,36`: same-voxel estimated distance is Robbins' constant times voxel size; same-voxel distance standard deviation uses the documented unit-cube expectation equation and approximately 99.8 ly; cross-voxel standard deviation uses its distinct documented formula/approximation. The complete numeric constants and formulas are in the unchanged source at `address.rs:1–45` and exact contract bytes; do not recompute or alter them.
5. `crates/nms-graph/src/import.rs:22`: `ImportStats.systems_added` counts catalog rows accepted as additions, including promotion of a marked placeholder; a promotion need not increase the final system count (`ImportStats` declaration and import behavior at `import.rs:22–35`).
6. `crates/nms-graph/src/model.rs:696`: base lookup keys are lowercased without trimming; insertion ignores an empty normalized key and a later same-key base replaces the prior record (`insert_base`, `model.rs:696+`).
7. `crates/nms-graph/src/query.rs:220`: radius search uses active-galaxy scope, while the explicit-address variant searches that address's galaxy without changing active selection (`query.rs:220+`).
8. `crates/nms-graph/src/spatial.rs:14`: coordinates are X/Y/Z voxel coordinates and the R-tree query uses squared Euclidean voxel distance (`spatial.rs:14+`).
9. `crates/nms-query/src/find.rs:32`: query fields include biome/subtype/infested, radius, nearest limit, name and discoverer patterns, named-only, and reference; filter combination precedes nearest limiting. Current-position requires player galaxy to equal active galaxy; base/address choose their own galaxy; negative radius yields no rows; NaN behavior depends on whether nearest is set; deterministic sort and no active-galaxy mutation are documented (`find.rs:11–169`).
10–13. `crates/nms-query/src/route.rs:13,34,53,78`: targets are biome, named, or explicit IDs; named lookup tries bases then system names; starts are current/base/address; route options include warp range, radius, target limit, algorithm, and return leg; result includes ordered hops, total distance, warp/jump metadata and visited target count. IDs are validated before radius/limit, biome targets rank from origin after deduplication, other target modes preserve caller order, and warp constraints are best effort (`route.rs:13–255`).
14–16. `crates/nms-query/src/show.rs:12,43,57`: system resolution tries active-galaxy name before packed hexadecimal address; base lookup is global/case-insensitive without trimming. Results expose cloned records and display metadata; player distance exists only within the same galaxy (`show.rs:12–122`). Packed hex is not a save-format decoder.
17–19. `crates/nms-query/src/stats.rs:8,18,41`: the two exposed flags are ignored; totals count all model records/galaxies; biome counts are held in a HashMap and row ordering is not guaranteed (`stats.rs:8–105`).
20. `crates/nms-query/src/theme.rs:72`: plain style returns input unchanged; styled output wraps it in ANSI SGR/reset sequences and does not escape embedded controls (`Style::paint`, `theme.rs:72–112`).

The 21st region is `CONTRIBUTING.md:26`, a preview namespace correction only. Its ordinary recipe separates library docs (`cargo doc --workspace --lib --all-features --no-deps`; nine libraries including umbrella `nms`) from the private CLI docs (`nms-cli --bin nms --document-private-items` in `target/cli-docs`). The distinct destinations are library `target/doc/nms_copilot/index.html`, umbrella `target/doc/nms/index.html`, and CLI `target/cli-docs/doc/nms/index.html`. Preserve the ordinary build command and all other CONTRIBUTING wording. Strict flags, JSON, offline, and dedicated targets are proof settings, not ordinary recipe claims.

## Surface, retained proof, and exception

The complete Rust inventory has 83 source identities. Nine library crates expose 70 root/public-module landing pages (9 roots + 61 public modules); all-features includes HTTP/archive. The CLI is a binary with 11 private-module landing pages in its separate namespace. The copilot executable root/types/functions are private and unchanged; public application APIs live in its library. External dependency reexports/trait docs are not new authored APIs. Actual built output and diagnostics determine acceptance; counts alone do not.

Slice29's independent full README/CONTRIBUTING, source-backed command/route controls, complete image/pixel evidence, and narrow raw byte/style evidence are retained only through exact unchanged relevant bytes and the eight exact evidence inputs. No new screenshot or rerender claim is made. The current CONTRIBUTING recipe still requires its corresponding current outputs. M7.6 also requires the fresh original full-workspace all-feature test configuration and the current composed docs surfaces; a child-only count cannot substitute.

The sole prior Operator-approved exception is the generated `DiscoverySourceResolver::Community.source` missing-doc warning emitted by `rkyv_derive 0.8.15`, at `crates/nms-core/src/system.rs:38:12–25` under `#[derive(rkyv::Archive)]`. The field has source docs but the resolver generator drops them. Original Slice30 strict reds remain visible. For new all-library JSON docs, exact flags are `-D warnings --force-warn missing_docs`; the full diagnostic population must contain exactly one warning from this same origin, with a successful build, all 70 public pages, both archive pages and resolver anchors. The predicate must reject an extra warning and a wrong-origin warning; all other warnings remain fatal. No blanket waiver, suppression, hidden API or dependency repair.

The full original0039 M7.6 criterion remains the acceptance target: README features/install/quickstart/glyphs/MCP/config/completions/examples and attributable usage images; contributor setup/build/test/fixture privacy/style/architecture; public APIs/key examples across core/save/graph/query and other crates; actual compiled navigation and isolated CLI. README command sketches do not override actual interfaces. M7.7 coverage/CI, M7.8 distribution/missing LICENSE assets, M7.10 themes, D0040–43 UI, R01 nonzero-game/raw/history/final semantics, R02/R03/cache freshness, authentic NomNom input, and parent/Operator/integration/release owners remain open. No publish-ready or coverage claim is implied. If full M7.6 is not demonstrated, preserve its exact open finding and owner.

## Eleven gates and mandatory hold

No gate has run. Every future first compiler/gate/validator attempt must use `/private/tmp/nms-slice44-cc-capture.py`, preserving UUID, argv/cwd/environment/overrides/original TMPDIR, timestamps, pre/post HEAD and status/tracked/untracked identity, and raw stdout/stderr bytes/base64/hash/exit. Raw channels must be fsynced before post metadata or parsing. Keep all failures, retries, unknowns and prefixes; do not infer quiet success or duplicate diagnostics. Source/inventory/edit guards and interface lookup are not SUT validators. Make/Cargo work remains offline and gates sequential with no writes during a gate.

1. `make format`, `CARGO_NET_OFFLINE=true`.
2. `make lint`, same override. Then immediately hold source/index/gates and send actual attempt IDs/identities. CDC must complete its full 21-region/source/namespace/helper/criterion walk before releasing runtime work; no source edit after pairing.
3. First released runtime: `cargo test --workspace --all-features --offline` on the authorized native watcher/loopback substrate, preserving inherited TMPDIR and required host execution. Preserve a restricted failure and only retry identically through the authorized host path if needed. Report real per-harness/feature denominators; do not infer a combined count or native matrix.
4. `cargo test --workspace --doc --no-default-features --offline` as the separate overlapping configuration.
5. All-library strict JSON rustdoc: `cargo doc --workspace --lib --all-features --no-deps --offline --message-format=json`, exact `RUSTDOCFLAGS=-D warnings --force-warn missing_docs`, `CARGO_TARGET_DIR=SOURCE/target/m76-cc-library-docs`. Verify and record target absent before first build; existing target is a blocker, not permission to delete.
6. Archive helper against actual group 5 population; verify exact one warning and both negative controls, 70 pages, two archive pages and anchors.
7. Isolated CLI docs: `cargo doc -p nms-cli --bin nms --document-private-items --all-features --no-deps --offline --target-dir SOURCE/target/cli-docs`, exact `RUSTDOCFLAGS=-D warnings -D missing_docs`; exception A does not transfer. Verify 11 internal pages/root links.
8. Reconcile `scope`: exact 12 paths/21 regions, all other docs/examples/non-doc frozen, all 83 inventory members, eight retained29 inputs, no untracked files, real insert_base casing mutant rejected.
9. Reconcile `composition` using actual groups 3/4/7 populations: CLI root links, retained29 exact Git bytes, current preview and full M7.6 mapping. This supports but does not replace full criterion review. Groups 6 and 9 must precede commit.
10. `git diff --check`.
11. `git diff a3f74a86f0ceeb523d7cda62c8c2bc38215669de --check`.

After gates 1–2, wait for CDC ACK. Do not inspect/prepare targets, launch runtime, run validators, or make source/index changes until ACK. A red or unexpected diagnostic leaves affected acceptance open; do not weaken a gate or substitute a scenario. The source/index/gate pair remains held at this point.


## Additive readback correction (2026-10-25)

The Slice44 import statistic is the public field `ImportStats::added`, not `systems_added`. I inspected the current declaration at `crates/nms-graph/src/import.rs:19–29`: `pub added: usize` is documented as accepted catalog contributions, including marked-placeholder promotions; such a promotion does not increase `GalaxyModel::system_count`. The complete authorized paragraph region at `import.rs:22–23` remains exactly the contract's original old bytes and proposed new bytes below; no source or contract change was made.

Old (unchanged):
```rust
    /// Number of accepted catalog contributions, including marked-placeholder
    /// promotions. Promotion does not increase [`GalaxyModel::system_count`].
```

New (unchanged):
```rust
    /// Number of accepted catalog contributions, including marked-placeholder promotions.
    ///
    /// Promotion does not increase [`GalaxyModel::system_count`].
```


## Gate 5 pre-build target check

At 2026-10-25T14:36:48.236754+00:00, before the first library JSON documentation build, `target/m76-cc-library-docs` was verified absent (`exists=false`, `is_symlink=false`) under the dedicated source root. No target content was created or removed.


## Initial implementation, captured gates, and commit handback

The exact 21-region/12-path contract batch was applied from an in-memory candidate after resolving the dedicated root, verifying accepted43 HEAD/branch/clean status, exact allowlist, and one current-byte occurrence of every old region. No other authored path changed. All attempts below are raw-first in `/private/tmp/nms-slice44-cc-attempts.json`; each stdout/stderr raw channel, hash, exit, timing, and source identity remains there. Gate sequence and outcomes:

- 1 `make format`: pass, `006bca999c094bf1829e956e9601b79b`.
- 2 `make lint`: pass, `2e2cc630e8c440279df3d64f8d5a9899` (Clippy and formatting).
- 3 full workspace/all features: pass, `7c27b441856840c9abc3fc6505b84bb0`; 1,097 runtime tests and 118 all-feature doctests (91 executing, 27 compile-only).
- 4 no-default-feature workspace doctests: pass, `c63bb86aed8c4ff0a12e9df82dec591d`; 114 doctests (89 executing, 25 compile-only), a separate overlapping population.
- Before the first library JSON docs build, `target/m76-cc-library-docs` was absent and recorded above. The first group 5 build, `9b46c66557be4c77bc89579505c74094`, exited 0 and generated the first output, but included an extra `CARGO_NET_OFFLINE` override. Group 6 verifier `1178e5d24be644259456a1d2bf06e674` rejected that proof metadata because its exact accepted override set is only `RUSTDOCFLAGS` and `CARGO_TARGET_DIR`. Both records remain preserved.
- Corrected group 5, `00d33077da9c41509c05cdd443c7caf3`, exited 0 with exact overrides and the same source/strict flags; the existing target was reused with cached artifacts, so it is not claimed as a second fresh rendering. Group 6 verifier `442f39115a7d4074a16f2ea693bcbd03` passed: exactly one same-origin exception warning, extra-warning and wrong-origin controls rejected, 70 public landing pages plus archive pages/resolver anchors verified.
- 7 isolated CLI docs: pass, `50156095bdda40bea2a0eee364fe928c`, with 11 private pages in the separate namespace and no transferred exception.
- 8 exact scope: pass, `42605abba14647ff9d9919635a994d77`; exact 12 paths/20 paragraphs, other docs/examples/non-doc frozen, retained29 inputs exact, casing mutant rejected.
- 9 composition: pass, `1960337cb45c4cef990d5b335fa0c43a`; retained29 bytes exact, current preview and original M7.6 mapping verified, CLI module set confirmed.
- 10 working diff check: pass, `2996dfdcd3f046e98e94a0ffc17f3006`. 11 accepted43 baseline diff check: pass, `248e968e4afd4b42a398eacbcc36ed3f`.

The local commit is `9b632b665885d1a8b596006dd96ebcaaad41a71a`, parent accepted43 `a3f74a86f0ceeb523d7cda62c8c2bc38215669de`, tree `974f51254e2304df17a5f806669a22b5c8b07299`. It contains exactly the 12 authorized files. Raw commit body equals `/private/tmp/nms-slice44-cc-source-message.txt` byte-for-byte; both required co-author trailers are exact and consecutive. Source status is clean. No owned gate/compiler/test/capture process remains running; source/index/gate ownership is handed back to CDC for independent review. No actual refinement was consumed; count remains initial 0.
