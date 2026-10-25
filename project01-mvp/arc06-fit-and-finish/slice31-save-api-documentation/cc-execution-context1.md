# Slice31 execution record

## Intake and assignment

Assignment: initial Slice31 complete save API documentation, issued at planning commit 1362d32f230dc470cfd96baa00444f348e664df7. The governing prompt, sibling slice plan, ledger, and source allowlist were read in full. Dedicated source is /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-api-documentation, branch codex/save-api-documentation, clean at 40e6bc8f2b26606393931a232e604ce0e6f38e86. The manifest names exactly 11 allowed files: crates/nms-save/src/{address,convert,decompress,error,lib,locate,mapping,metadata,model,validated,xxtea}.rs. Its only non-prose allowance is 17 named existing payload declaration whitespace/trailing-comma layouts. No source edit, compiler, test, or validator has run.

## Required reading coverage

- The preserved original Arc06 M7.6 source 0039 lines 1–112 and current arc-plan Slice30 closure/Slice31 issue were loaded. Arc history keeps M7.6 open, assigns this save-crate documentation increment, and preserves later graph/query and remaining API owners.
- Dedicated AGENTS.md, crates/nms-save/Cargo.toml, and all 11 current allowed Rust source files were read completely. This included existing in-file tests/doctests in address.rs, locate.rs, metadata.rs, model.rs, validated.rs, and xxtea.rs. The two integration test files, tests/address_decode.rs (914 lines) and tests/validated_save.rs (938 lines), were each read in full.
- Framework sources were read at assigned extents: collaboration-framework/SKILL.md; methodology shared invariants and Autonomous Mode; project-management guides README, 02-canonical-planning-worktree and 03-planning-top-down planning/intake/iteration sections; implementation-prompt-authoring Required reading and CC intake; work-verification evidence-strength and silent-drop guides; testing validation-gates; methodology-stewardship ENTRYPOINT. The full Rust skill plus full 11-anti-patterns.md (2188 lines) and 13-documentation.md (1242 lines) were loaded in contiguous chunks through each file’s end.
- Current contract readback follows. Prompt, plan, ledger, and manifest remain governing; prior Slice30 records are applicability/background only and do not constitute Slice31 acceptance.

## Support evidence inspected

Planning artifact slice31-save-api-documentation/artifacts/capture-preflight.json was read as complete JSON. It contains two support-only kernel controls: 53e7f5195b1845ec912342db51e17bf6 (expected exit 7) and 370f4b18a7144616b99a724e9490850f (expected exit 0), both complete with explicit status and before/after metadata at source HEAD 40e6bc8f2b26606393931a232e604ce0e6f38e86, clean status, and matching working identity. Each captured stdout is 10 bytes (out, NUL, bytes, LF; SHA-256 96a4f40c904d0541a3302777be960d06bfe169e72d6eef4665ff472d3a092980); stderr is 5 bytes (err, NUL, LF; SHA-256 c4846b09bf3f403d7239b90bfa61a92221d5af3d2cebf3075fa6a21d17b21383). These are author support controls, not Slice31 product gates.

The complete raw-first kernel /private/tmp/nms-slice31-cc-capture.py and its planning artifact copy are byte-identical, SHA-256 9ebce04427bd85df4403c2a78784f207020b76c980cff445f99326242f509f47. It records command identity, inherited/overridden environment, before-state, persists exact child exit/stdout/stderr bytes and end time before post-state, and prints a compact receipt. The full supplied scope-audit template was read; SHA-256 1cf81f01b96920857dd68270f4b7ce69775153b226d4bf862c798121d52ddf87. It compares ordered source content after removing doc lines/blank lines and normalizes only its enumerated payload layouts; its required in-memory opposing mutation must be rejected. No support diagnostic was run.

Main-repository fences at intake: HEAD=b616ab2cfa9c67dfb6361bb1355ca9e6680d18cc; only the pre-existing crates/nms-copilot/tests/mcp_live_integration.rs working-tree modification is visible. Its Git blob is 2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e; crates/nms-core/src/lib.rs is affe7713497d7ead37551ebb5942df9f78c0a4a2. Main index is empty. These paths and all held pins are outside this assignment and remain protected. For later fence checks, compare Git blob OIDs from git hash-object, not SHA-256 file hashes.

## Contract readback

The deliverable is complete, truthful public Rust documentation for the existing save API, not a code/API repair. The only tracked scope is the 11 manifest paths. Add prose and blank lines, plus only the exact 17 existing payload-layout normalizations. Keep every executable statement, signature, type, field meaning, derive/attribute, body, test, fixture, dependency, and out-of-scope file unchanged. Use canonical Examples, Errors, Panics order where applicable; explain current errors and actual panic conditions rather than promising stronger behavior.

A key distinction is the checked parser’s validation of original JSON before serde defaults versus the legacy typed parse path. The checked path validates active context and discovery rows and returns paired addresses from the original source; it does not claim the typed model preserves every raw JSON field. Examples must preserve the contract’s literal checked-address oracle 0x20010900000003 (GG9/P2/X3/SSI1 and core address 0x200100000003) and distinguish legacy conversion’s intentionally different address/name semantics. The filesystem and format sections must reflect actual byte-level behavior: first-byte JSON detection, no implied UTF-8 validation for passthrough, strict mapping precedence/detection, actual metadata fallback, encrypted metadata constraints, and locator ordering/error-skip behavior. Examples use only isolated temporary fixtures, never host saves or game-backed claims.

Validation next is one captured make format then one captured make lint through the inspected kernel, followed immediately by a full source/index/gate hold for CDC review. No tests or docs gates will run before that review release.

## Intake coverage addendum

Before any source edit, completed the remaining exact required-section reads: project-management 02-canonical-planning-worktree section “Slice iteration filenames and preservation” (lines 302–334) and 03-planning-top-down section “Issuing and executing an iteration” (lines 202–256); work-verification 05-independent-verification in full, including “Evidence During Correction Review”; methodology-stewardship 01-retrospectives-and-follow-through in full and project-management 04-closing-slices in full. These reinforce preserved assignment/history, reviewer independence, evidence applicability, and CDC's later close/retrospective duties; none changes today's initial implementation scope or grants self-acceptance. All assigned reading is now complete; no compiler, validator, test, or source mutation has occurred.

## Initial documentation candidate and captured pair

Completed public docs for the 11 allowed source files after the initial readback. The changes are limited to rustdoc text/blank lines plus the manifest's enumerated payload declaration layouts. The manual API/example pass covered checked versus legacy parsing, address/metadata/mapping behavior, decompression boundaries, and locator selection/error semantics. No source/test/API behavior was intentionally changed.

### slice31-format-initial

Captured by `/private/tmp/nms-slice31-cc-capture.py`; attempt ID `dc457f123d6e4bf1986dbfee9ddaa17e`, argv `["make", "format"]`, cwd `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-api-documentation`, override `CARGO_NET_OFFLINE=true`, status `complete`, exit `0`, ended `2026-10-25T02:49:41.923573+00:00`. Before and after metadata both report HEAD `40e6bc8f2b26606393931a232e604ce0e6f38e86`, working identity `e86a3f3e99811bc08e43f33998430014c87efe34472f5852b15cefe8e054e1fd`, tracked-diff SHA-256 `47b8f523ad7bfda6f020e7e49b42e498e8f76375acf291ab64589add25473881`, and no untracked files.

Raw stdout: 107 bytes, SHA-256 `1205c3992cdb019f09e137aec75930a5e4c45d5d1f8993574c2ace0b8d86ce57`. Raw stderr: 0 bytes, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

### slice31-lint-initial

Captured by `/private/tmp/nms-slice31-cc-capture.py`; attempt ID `1c6954e80d1642ecadefff9e9a2ede60`, argv `["make", "lint"]`, cwd `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-api-documentation`, override `CARGO_NET_OFFLINE=true`, status `complete`, exit `0`, ended `2026-10-25T02:50:18.249234+00:00`. Before and after metadata both report HEAD `40e6bc8f2b26606393931a232e604ce0e6f38e86`, working identity `e86a3f3e99811bc08e43f33998430014c87efe34472f5852b15cefe8e054e1fd`, tracked-diff SHA-256 `47b8f523ad7bfda6f020e7e49b42e498e8f76375acf291ab64589add25473881`, and no untracked files.

Raw stdout: 176 bytes, SHA-256 `f810636e1c8999d41a9367929b15ad516bf33fcefcc8a7c5a1582c0859800ad7`. Raw stderr: 12040 bytes, SHA-256 `45e8307ac97ec24cf9f85a8b4203e0991bf434efa1ed58c01bcdf878c8cde9fc`.

`make format` reported rustfmt completed; its captured output was 107 stdout bytes and empty stderr. `make lint` exited 0, and captured stdout reports Clippy passed and format check passed; stderr contains Cargo build output and a future-incompatibility warning for `proc-macro-error2 v2.0.1` (not a gate failure). The final identity remains `e86a3f3e99811bc08e43f33998430014c87efe34472f5852b15cefe8e054e1fd`; the current diff SHA-256 is `47b8f523ad7bfda6f020e7e49b42e498e8f76375acf291ab64589add25473881`. Source/index/gates are now held for CDC review. Both nested gate calls returned completed results without a session identifier; no owned child process remains. No runtime/test/docs gate has been run.


## Iteration01 readback and start

Read the complete immutable iteration01 prompt, current slice plan/ledger, and source-allowlist.json. The iteration is the first actual refinement (1/5; four remain). Current dedicated source HEAD remains 40e6bc8f2b26606393931a232e604ce0e6f38e86 with the held initial 11-file candidate; index is expected empty. I will change only rustdoc prose/blank lines in convert.rs, decompress.rs, lib.rs, locate.rs, mapping.rs, metadata.rs, model.rs, validated.rs, and xxtea.rs. address.rs/error.rs remain byte-exact, and all executable/API/test/data/Cargo/other-path content stays frozen.

The correction covers canonical Examples/Errors/Panics order, actually executed owned-tempdir locator examples, the checked-versus-legacy address distinction and complete assigned literals, nested mapping and metadata observations, and the precise validation traversal/projection, decompression panic, XXTEA bounds, mapping collision, and timestamp contracts. The 32-bit decompression overflow is source-derived conditional behavior only; no runtime fix or 32-bit execution is included. After the full affected-document walk, I will run the raw-first captured offline format then lint pair and immediately hold the candidate for CDC review. No tests or strict-doc gate are released yet.


## Iteration01 bounded edit completion before renewed gates

Completed the closed F1–F3 documentation changes in the nine authorized prose files. The work preserves the initial candidate in `address.rs` and `error.rs`; no executable statement, public signature/type, test body, fixture, manifest, or dependency was changed. The checked parser docs now state the actual traversal/projection and mixed `Address`/JSON error boundary; legacy and checked address examples share the assigned literal; locator examples execute their owned temporary-directory operations; metadata/mapping observations use the assigned nested and synthetic values; and decompression, XXTEA, collision, and timestamp prose reflects the source-derived limits. The conditional 32-bit malformed-size panic remains documentation of existing source behavior, not runtime evidence or a code repair.

The edit orchestration included several guarded Python replacement scripts that stopped on mismatched source-string assertions. The first append preflight exited 1 before writing due to an empty `Path.glob` pattern. Later scripts wrote only earlier authorized targets before an expected-text assertion stopped the remaining targets; I inspected the actual file text and completed the remaining authorized documentation edits. These were editing-script incidents, not compiler, validator, test, or product attempts. No out-of-scope path was targeted.

Next are the renewed raw-first `make format` then `make lint` attempts with `CARGO_NET_OFFLINE=true`; after those complete, source/index/gates return to HOLD for the full CDC review.


## Iteration01 captured format→lint pair and immediate hold

The raw-first kernel `/private/tmp/nms-slice31-cc-capture.py` captured both commands sequentially with `CARGO_NET_OFFLINE=true`; both returned completed results without a nested session identifier. The exact full records, including inherited environment, child raw bytes/base64, timestamps, and before/after metadata, are retained in `/private/tmp/nms-slice31-cc-attempts.json`.

- `slice31-iteration01-format`, ID `5af8ff999a0b4030b81ccc5ef86ed5c6`: argv `make format`; started `2026-10-25T03:03:31.042204+00:00`, child exit 0 at `2026-10-25T03:03:32.334220+00:00`. Stdout 107 bytes, SHA-256 `1205c3992cdb019f09e137aec75930a5e4c45d5d1f8993574c2ace0b8d86ce57`; stderr empty (0 bytes, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`). Output reports rustfmt completed.
- `slice31-iteration01-lint`, ID `f754553dae354e479d536737b1011d76`: argv `make lint`; started `2026-10-25T03:03:35.725030+00:00`, child exit 0 at `2026-10-25T03:03:40.023712+00:00`. Stdout 176 bytes, SHA-256 `f810636e1c8999d41a9367929b15ad516bf33fcefcc8a7c5a1582c0859800ad7`; stderr 1287 bytes, SHA-256 `0b798ed2450bd13a97fc58dc16729ffc21ebd5c8d0d1e506ee35c9dc2b52cf6d`. Output reports Clippy and format checks passed; stderr also retains the existing future-incompatibility warning for `proc-macro-error2 v2.0.1`.

Both records have status `complete`, exit 0, HEAD `40e6bc8f2b26606393931a232e604ce0e6f38e86`, no untracked files, and identical before/after working identity `55d4edf98619d7663582ca2ee282e9d5e226cabac68a78bedf3e1163d041a28a` and tracked-diff SHA-256 `8b75c62ff29d6fdc63c756dfa6fd8e507fa315b776d4a92cbca5aadd6337d723`. The tracked status lists exactly the original 11 manifest files as modified; the index remains empty. Current source/index/gates are now held for CDC's full F1–F3 review. No runtime tests or strict-doc validation ran.

## Iteration02 readback and start — 2026-10-25 UTC

Read the complete immutable `cc-prompt-iteration02.md`, current slice plan and ledger, and the current `source-allowlist.json`. The prompt is the second actual refinement (2/5; three remain). The exact change subset is `crates/nms-save/src/{mapping.rs,lib.rs,model.rs,decompress.rs}`. The other seven already-modified candidate files remain frozen; examples, executable statements, public API/type/attribute declarations, tests, fixtures, and data remain unchanged. The first-pair records and every earlier editing incident remain part of the history. No runtime or strict-doc gate has been released.

Before editing, the dedicated source is still at HEAD `40e6bc8f2b26606393931a232e604ce0e6f38e86`, with empty index and tracked-diff SHA-256 `8b75c62ff29d6fdc63c756dfa6fd8e507fa315b776d4a92cbca5aadd6337d723`, matching the held iteration01 candidate. The manifest lists the original eleven-file envelope; this iteration further restricts writes to the four paths above. Main remains at `b616ab2cfa9c67dfb6361bb1355ca9e6680d18cc`, its only tracked modification is the protected `crates/nms-copilot/tests/mcp_live_integration.rs` blob `2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e`, `crates/nms-core/src/lib.rs` is blob `affe7713497d7ead37551ebb5942df9f78c0a4a2`, and the main index is empty. These are Git blob OIDs, not file SHA-256 values.

The bounded prose work is: (1) correct the actual indented `KeyMapping::bundled` and `KeyMapping::from_json` doc section order while keeping their examples verbatim; clarify recursive traversal of unknown-key descendants, scalar preservation, collision replacement, and the separate later-applied Save Wizard fixups; (2) describe trailing-NUL trimming at the crate intro, file parser, and sanitizer step accurately, including retaining all-NUL input; (3) correct the two specified one-line summaries and state that core address construction masks encoded widths without validating source presence/ranges; and (4) document the 16-byte little-endian `0xFEEDA1E5` header and ignored fourth reserved word. All surrounding behavior and examples remain frozen.

Next I will make only those four absolute-path, manifest-checked prose edits, inspect the complete affected docblocks and unchanged-file/example boundaries, then run captured offline `make format` followed by `make lint`. Immediately after that pair I will hold all source/index/gates for CDC review; no tests or runtime validation are included in this pass.

### Readback fence attribution correction (append-only)

The preceding iteration02 readback incorrectly labeled two Git blob OIDs from the dedicated-worktree command as main-repository values. The command was run with cwd `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-api-documentation`; its outputs `6f77e00a0f298b4fc567bbbb32c748b9e933f2cf` and `fa382ca481ceef133d7a41df5b46adaa537b9e67` are dedicated-worktree blob observations, not main fence values. The command made no mutation. CDC revalidated the actual main repository separately: HEAD `b616ab2cfa9c67dfb6361bb1355ca9e6680d18cc`, only the protected native test is modified, its Git blob is `2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e`, `crates/nms-core/src/lib.rs` is `affe7713497d7ead37551ebb5942df9f78c0a4a2`, and main index is empty. No Slice31 iteration02 Rust edit or gate had occurred at the time of correction. The held source/index/gate ownership is now active for the exact four-file prose pass.


## Iteration02 failed patch submissions (editing incidents, no source mutation)

For exact attribution, the following are the two `tools.apply_patch` payloads and their returned errors. Both calls were made through `functions.exec`; neither ran a compiler, validator, test, or product process. The second payload mistakenly named one main-root model.rs target. The apply operation returned a context error, and the subsequent read-only hashes establish that the dedicated candidate remained byte-identical to the held iteration01 pin; the main protected blobs also remained unchanged.

### Failed submission 1 — exact patch payload

```text
*** Begin Patch
*** Update File: /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-api-documentation/crates/nms-save/src/mapping.rs
@@
-//! Recursive
-//! deobfuscation preserves unknown keys and values unless mapped keys collide
-//! on one output name. This module handles JSON keys only; it does not
+//! Deobfuscation traverses objects and arrays recursively, so nested keys may
+//! be renamed even when their parent key is unknown. Unknown key spellings and
+//! scalar values are otherwise preserved. If keys map to the same output name,
+//! the later processed entry replaces the earlier value, so the result is not
+//! lossless. This module handles JSON keys only; it does not
@@
-/// The primary direction maps obfuscated keys such as `"F2P"` to readable
-/// names such as `"Version"`. [`Self::bundled`] also merges older keys and
-/// spelling fixups; [`Self::from_json`] loads one mapping file without those
-/// additional sources.
+/// The primary direction maps obfuscated keys such as `"F2P"` to readable
+/// names such as `"Version"`. [`Self::bundled`] adds older keys only when the
+/// primary mapping lacks them and loads separate Save Wizard spelling fixups
+/// for later deobfuscation. Those fixups are not mapping entries returned by
+/// [`Self::get`] or counted by [`Self::len`]. [`Self::from_json`] loads one
+/// mapping file without those additional sources.
@@
-    /// Loads `mapping_mbincompiler.json` first, adds keys from
-    /// `mapping_legacy.json` only when the primary mapping lacks that key, then
-    /// applies spelling fixups from `mapping_savewizard.json`.
-    ///
-    /// # Panics
-    ///
-    /// Panics if any of the mapping files compiled into this crate is invalid.
-    /// Those files are project data and are expected to be valid at build time.
+    /// Loads `mapping_mbincompiler.json` first, then adds keys from
+    /// `mapping_legacy.json` only when the primary mapping lacks that key. It
+    /// also loads separate spelling fixups from `mapping_savewizard.json` for
+    /// use during later deobfuscation; they do not alter a JSON value here or
+    /// count as entries returned by [`Self::get`] or [`Self::len`].
@@
     /// assert_eq!(mapping.get("F2P"), Some("Version"));
     /// ```
+    ///
+    /// # Panics
+    ///
+    /// Panics if any of the mapping files compiled into this crate is invalid.
+    /// Those files are project data and are expected to be valid at build time.
@@
-    /// # Errors
-    ///
-    /// Returns [`SaveError::MappingParseError`] when the JSON is malformed or
-    /// does not have the expected mapping-file shape.
-    ///
     /// # Examples
@@
     /// # Ok::<(), nms_save::SaveError>(())
     /// ```
+    ///
+    /// # Errors
+    ///
+    /// Returns [`SaveError::MappingParseError`] when the JSON is malformed or
+    /// does not have the expected mapping-file shape.
@@
-    /// obfuscated-key mapping and then any spelling fixup. Unknown keys and
-    /// their values remain unchanged unless a mapped key collides with the same
-    /// output name. Collisions do not return an error: the later processed
-    /// entry replaces the earlier value in the output map.
+    /// obfuscated-key mapping and then any spelling fixup. Unknown key spellings
+    /// are retained, but their object or array values are still traversed and
+    /// may contain renamed keys; scalar values remain unchanged. If two entries
+    /// produce the same output name, the later processed entry replaces the
+    /// earlier value, so the result is not lossless.
*** Update File: /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-api-documentation/crates/nms-save/src/lib.rs
@@
-//! LZ4 blocks when selected, strips trailing NUL bytes, replaces invalid UTF-8,
+//! LZ4 blocks when selected, removes trailing NUL bytes only when a non-NUL
+//! byte exists (an all-NUL input is retained), replaces invalid UTF-8,
@@
-/// decompression, strips trailing NUL bytes, replaces invalid UTF-8, checks
+/// decompression, removes trailing NUL bytes only when a non-NUL byte exists
+/// (an all-NUL input is retained), replaces invalid UTF-8, checks
@@
-/// 1. Strips trailing null bytes (NMS LZ4 blocks are padded with nulls)
+/// 1. Removes trailing null bytes after the last non-null byte, when one
+///    exists (NMS LZ4 blocks are padded with nulls); all-null input is retained.
*** Update File: /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-api-documentation/crates/nms-core/src/model.rs
@@
-    /// This typed-model conversion does not validate original JSON member
-    /// presence or narrower packed-save coordinate ranges.
+    /// Core construction masks components to their encoded widths. This
+    /// conversion does not validate original JSON member presence or narrower
+    /// packed-save coordinate ranges.
@@
-    /// Convert by masking into the legacy core 48-bit portal layout and using
-    /// the supplied galaxy index.
+    /// Convert the legacy packed value using the supplied galaxy index.
     ///
-    /// This does not recover the source 56-bit galaxy byte or validate the
-    /// original packed value. Use [`crate::address::decode_discovery_address`]
-    /// or the corresponding checked decoder when the source JSON is available.
+    /// This masks into the core 48-bit portal layout; it does not recover the
+    /// source 56-bit galaxy byte or validate the original packed value. Use
+    /// [`crate::address::decode_discovery_address`] or the corresponding
+    /// checked decoder when the source JSON is available.
@@
-    /// Optional numeric uploaded flag; the legacy conversion treats values
-    /// greater than zero as uploaded.
+    /// Optional numeric uploaded flag.
+    ///
+    /// The legacy conversion treats values greater than zero as uploaded.
*** Update File: /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-api-documentation/crates/nms-save/src/decompress.rs
@@
-/// headers, validates each block, and concatenates decompressed payloads. An
-/// empty slice is classified as compressed input and produces an empty output.
+/// headers with little-endian magic `0xFEEDA1E5`, validates each block, and
+/// concatenates decompressed payloads; the fourth reserved header word is
+/// ignored. An empty slice is classified as compressed input and produces an
+/// empty output.
*** End Patch
```

Returned error: `Script error: apply_patch verification failed: Failed to find expected lines in /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-api-documentation/crates/nms-save/src/mapping.rs:\n//! Recursive\n//! deobfuscation preserves unknown keys and values unless mapped keys collide\n//! on one output name. This module handles JSON keys only; it does not`.

### Failed submission 2 — exact patch payload

```text
*** Begin Patch
*** Update File: /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-api-documentation/crates/nms-save/src/mapping.rs
@@
-//! compatibility entries, and Save Wizard spelling fixups. Recursive
-//! deobfuscation preserves unknown keys and values unless mapped keys collide
-//! on one output name. This module handles JSON keys only; it does not
+//! compatibility entries, and Save Wizard spelling fixups. Deobfuscation
+//! traverses objects and arrays recursively, so nested keys may be renamed even
+//! when their parent key is unknown. Unknown key spellings and scalar values
+//! are otherwise preserved. If keys map to the same output name, the later
+//! processed entry replaces the earlier value, so the result is not lossless.
+//! This module handles JSON keys only; it does not
@@
-/// The primary direction maps obfuscated keys such as `"F2P"` to readable
-/// names such as `"Version"`. [`Self::bundled`] also merges older keys and
-/// spelling fixups; [`Self::from_json`] loads one mapping file without those
-/// additional sources.
+/// The primary direction maps obfuscated keys such as `"F2P"` to readable
+/// names such as `"Version"`. [`Self::bundled`] adds older keys only when the
+/// primary mapping lacks them and loads separate Save Wizard spelling fixups
+/// for later deobfuscation. Those fixups are not mapping entries returned by
+/// [`Self::get`] or counted by [`Self::len`]. [`Self::from_json`] loads one
+/// mapping file without those additional sources.
@@
-    /// Loads `mapping_mbincompiler.json` first, adds keys from
-    /// `mapping_legacy.json` only when the primary mapping lacks that key, then
-    /// applies spelling fixups from `mapping_savewizard.json`.
-    ///
-    /// # Panics
-    ///
-    /// Panics if any of the mapping files compiled into this crate is invalid.
-    /// Those files are project data and are expected to be valid at build time.
+    /// Loads `mapping_mbincompiler.json` first, then adds keys from
+    /// `mapping_legacy.json` only when the primary mapping lacks that key. It
+    /// also loads separate spelling fixups from `mapping_savewizard.json` for
+    /// use during later deobfuscation; they do not alter a JSON value here or
+    /// count as entries returned by [`Self::get`] or [`Self::len`].
@@
     /// assert_eq!(mapping.get("F2P"), Some("Version"));
     /// ```
+    ///
+    /// # Panics
+    ///
+    /// Panics if any of the mapping files compiled into this crate is invalid.
+    /// Those files are project data and are expected to be valid at build time.
@@
-    /// # Errors
-    ///
-    /// Returns [`SaveError::MappingParseError`] when the JSON is malformed or
-    /// does not have the expected mapping-file shape.
-    ///
     /// # Examples
@@
     /// # Ok::<(), nms_save::SaveError>(())
     /// ```
+    ///
+    /// # Errors
+    ///
+    /// Returns [`SaveError::MappingParseError`] when the JSON is malformed or
+    /// does not have the expected mapping-file shape.
@@
-    /// obfuscated-key mapping and then any spelling fixup. Unknown keys and
-    /// their values remain unchanged unless a mapped key collides with the same
-    /// output name. Collisions do not return an error: the later processed
-    /// entry replaces the earlier value in the output map.
+    /// obfuscated-key mapping and then any spelling fixup. Unknown key spellings
+    /// are retained, but their object or array values are still traversed and
+    /// may contain renamed keys; scalar values remain unchanged. If two entries
+    /// produce the same output name, the later processed entry replaces the
+    /// earlier value, so the result is not lossless.
*** Update File: /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-api-documentation/crates/nms-save/src/lib.rs
@@
-//! LZ4 blocks when selected, strips trailing NUL bytes, replaces invalid UTF-8,
+//! LZ4 blocks when selected, removes trailing NUL bytes only when a non-NUL
+//! byte exists (an all-NUL input is retained), replaces invalid UTF-8,
@@
-/// decompression, strips trailing NUL bytes, replaces invalid UTF-8, checks
+/// decompression, removes trailing NUL bytes only when a non-NUL byte exists
+/// (an all-NUL input is retained), replaces invalid UTF-8, checks
@@
-/// 1. Strips trailing null bytes (NMS LZ4 blocks are padded with nulls)
+/// 1. Removes trailing null bytes after the last non-null byte, when one
+///    exists (NMS LZ4 blocks are padded with nulls); all-null input is retained.
*** Update File: /Users/oubiwann/lab/oxur/nms-copilot/crates/nms-save/src/model.rs
@@
-    /// This typed-model conversion does not validate original JSON member
-    /// presence or narrower packed-save coordinate ranges.
+    /// Core construction masks components to their encoded widths. This
+    /// conversion does not validate original JSON member presence or narrower
+    /// packed-save coordinate ranges.
@@
-    /// Convert by masking into the legacy core 48-bit portal layout and using
-    /// the supplied galaxy index.
+    /// Convert the legacy packed value using the supplied galaxy index.
     ///
-    /// This does not recover the source 56-bit galaxy byte or validate the
-    /// original packed value. Use [`crate::address::decode_discovery_address`]
-    /// or the corresponding checked decoder when the source JSON is available.
+    /// This masks into the core 48-bit portal layout; it does not recover the
+    /// source 56-bit galaxy byte or validate the original packed value. Use
+    /// [`crate::address::decode_discovery_address`] or the corresponding
+    /// checked decoder when the source JSON is available.
@@
-    /// Optional numeric uploaded flag; the legacy conversion treats values
-    /// greater than zero as uploaded.
+    /// Optional numeric uploaded flag.
+    ///
+    /// The legacy conversion treats values greater than zero as uploaded.
*** Update File: /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-api-documentation/crates/nms-save/src/decompress.rs
@@
-/// headers, validates each block, and concatenates decompressed payloads. An
-/// empty slice is classified as compressed input and produces an empty output.
+/// headers with little-endian magic `0xFEEDA1E5`, validates each block, and
+/// concatenates decompressed payloads; the fourth reserved header word is
+/// ignored. An empty slice is classified as compressed input and produces an
+/// empty output.
*** End Patch
```

Returned error: `Script error: apply_patch verification failed: Failed to find expected lines in /Users/oubiwann/lab/oxur/nms-copilot/.worktrees/save-api-documentation/crates/nms-save/src/model.rs:\n    /// This typed-model conversion does not validate original JSON member\n    /// presence or narrower packed-save coordinate ranges.`. The erroneous main-root path was part of this submission; no hunk was applied, as confirmed by the unchanged full dedicated diff SHA and unchanged protected main Git blobs above.


## Iteration02 frozen handoff and execution accounting — 2026-10-25 UTC

The iteration02 prose batch is frozen for continuation by a fresh CC context with the same authorized gpt-6-luna High selection. Current candidate HEAD is `40e6bc8f2b26606393931a232e604ce0e6f38e86`, index empty, and full tracked `git diff --binary` SHA-256 is `a904df30281af6372b112e7557dce9ad9e6e28156c10c0b7e497ab0c21ca56a5`. The four iteration02 working-file SHA-256 values are: `decompress.rs` `74dbbbae5624744cf77077c0d006a6cf9ccc084c89c6a2ad2bf704104017bb03`; `lib.rs` `022be70f4319f799251e2d702a7e22e4a26fcd6937fc234499f8ad22cdac57e6`; `mapping.rs` `a94ab8d6ab201bbefcea44c070838bfdafbbe59368f157b26b7f09c9b227fb79`; `model.rs` `93b68004976c68e8264a93a71e0233a8b2807c4f5cdef64b49601f3506afcb00`. No iteration02 compiler, test, runtime, or documentation gate has started; there is no owned process. The actual six capture-log entries at this point are the two author support controls plus initial format/lint and iteration01 format/lint: `53e7f5195b1845ec912342db51e17bf6`, `370f4b18a7144616b99a724e9490850f`, `dc457f123d6e4bf1986dbfee9ddaa17e`, `1c6954e80d1642ecadefff9e9a2ede60`, `5af8ff999a0b4030b81ccc5ef86ed5c6`, and `f754553dae354e479d536737b1011d76`. These earlier records do not cover the current four-file edit.

### Additive construction and transcription qualifications

The initial readback's main-fence hashes were obtained in the dedicated source worktree and were mislabeled as main values. The correction above preserves the original statement and records CDC's verified main values. No main file was written; main remained at HEAD `b616ab2cfa9c67dfb6361bb1355ca9e6680d18cc`, with only protected native-test blob `2cd450ce5e0a74f5017b6e4cb3a2ea2719e4d77e`, core-library blob `affe7713497d7ead37551ebb5942df9f78c0a4a2`, and empty index.

Two `apply_patch` submissions failed before applying, followed by two pre-write Python assertions. The first failed patch call included a model.rs target outside the exact four-file iteration02 subset (`crates/nms-core/src/model.rs`); the second included a main-root `crates/nms-save/src/model.rs` target, while the tool's context error named the dedicated worktree model.rs path. The earlier incident subsection preserves the submitted text as a contemporaneous transcription, not as independently recovered byte-exact tool-call serialization; retain this qualification rather than treating that subsection as an exact transport payload. The returned errors and following read-only diff/hash checks showed no applied hunk. The first Python preflight stopped because `.strip()` removed the leading porcelain status space before comparison. The next pre-write transformation stopped because a lib.rs expected string used an extra closing brace in the `F2P` code span. Both stopped before any file write. After those failures, one guarded Python batch validated every generated target under the resolved dedicated root and inside the exact four-file subset, validated all replacement counts in memory, checked dedicated HEAD/index and main HEAD/status/protected blobs, and then wrote the four assigned files. No other path was targeted by that successful write.

Source/index/gates are now handed to the same-model successor context. This note is append-only; prior prompts, records, failures, and return bytes remain preserved. CDC owns planning and independent review.
