# Slice06 CC candidate result

Status: implementation and evidence returned for CDC review; no independent acceptance is claimed. Source is committed at `6b47cd3d758e6b44e28041ae52eceba0439c35db` on `codex/workspace-coverage-contract`; source worktree is clean. Main remains at `b616ab2cfa9c67dfb6361bb1355ca9e6680d18cc`, clean and untouched.

## Reconciliation and implementation

| Original clause | Current disposition and evidence | Boundary |
|---|---|---|
| D0040 art banner before model summary; independent show/custom/color controls and `{version}` handling | Existing banner/config implementation and retained S44–48 tests remain unchanged. README now accurately says `[display].color` controls the art banner; it does not claim a CLI `--color` option. | Existing acceptance is retained; no banner redesign or fresh display suite. |
| D0041 save/config priority, interactive first-run setup, saved settings and noninteractive error | `main.rs` now invokes the existing wizard after automatic detection fails only for non-headless TTY stdin. Priority remains `--save` → explicit `--setup` → env/config → auto-detection → TTY wizard. One helper shares setup failure/manual guidance. `setup.rs` asks for a directory truthfully. Two Unix process controls cover non-TTY and headless explicit setup. `cargo test --offline -p nms-copilot --test startup_integration`: 4 passed. One authorized native PTY run selected a copied synthetic account/save, persisted config, loaded 6 systems/13 planets/2 bases and bound MCP; a second start bypassed the wizard and bound MCP again. | Reedline CPR query timed out in the bounded `expect` interaction (`Input error: The cursor position could not be read within a normal duration`), followed by `Goodbye!`; no command/history continuation is claimed. PTY transcript and scratch home are retained in `/tmp/nms-s06-native-home.vc21ia`. Initial restricted PTY bind denial is also retained as a limited attempt; the one authorized retry succeeded through bind. |
| D0041 history, context-aware completion and persistent session | Existing implementation and retained Arc04 interaction rows/tests remain unchanged; no new completion/history code was added. | The native PTY CPR limit prevents fresh command/history composition proof. |
| D0042 map behavior and snapshot lifetime | Existing S01 native map evidence and S44–48 component evidence are reused. `crates/nms-copilot/src/map/mod.rs` documentation now states that the map holds its owned snapshot while open. | No map runtime change or recapture. |
| D0043 table output and theme recommendation | Existing table implementation/tests and retained S44–48 evidence remain unchanged. README no longer advertises configurable themes: the current palette is fixed, and `[display].color` controls art-banner color. | M7.10 remains a pending human scope decision; D0043's table recommendation does not resolve it. No theme implementation or inferred deferral. |

The source change is restricted to root README, regular root/per-package license files, `crates/nms-copilot/src/main.rs`, `setup.rs`, map module documentation, and `tests/startup_integration.rs`. Ten package roots each contain identical regular license files. Root and package license SHA-256 values are Apache `ac7fb99bf28422b4a7301a4844b2b56d1c3f006d200dccb53629b7bb944dcba2` and MIT `05272a3305411dbf4f1902581f5466ce875d8c9d0dd4e28b34a08598f72fbf2c`. The README uses local license links and makes no configurable-palette promise. The map comment changes no runtime behavior.

## Commands and outcomes

Source baseline was clean `8d572529445d80e404c1d45813c8b4a5b4b732fc`.

1. `make format` — passed; rerun after the final refinement and passed.
2. `make lint` — passed after fixing the test input API; the final prescribed format-then-lint pair passed. Cargo emitted its existing `proc-macro-error2 v2.0.1` future-incompatibility warning.
3. `cargo test --offline -p nms-copilot --test startup_integration` — 4 passed, 0 failed.
4. `cargo test --offline --doc -p nms-copilot` — 33 passed, 0 failed.
5. `make test` — failed in the `nms-copilot` library suite: 247 passed and these seven failed:
   - `mcp::watcher_lifecycle_tests::test_retained_watcher_receives_later_write_and_owner_drop_disconnects`
   - `coordinator::observer_and_drop_tests::test_observers_receive_order_without_backpressure_or_history`
   - `coordinator::additional_tests::test_no_cache_path_never_invokes_writer`
   - `coordinator::tests::test_blocked_writer_releases_model_and_preserves_update_order`
   - `coordinator::observer_and_drop_tests::test_drop_waits_for_inflight_cache_writer_and_update_publication`
   - `coordinator::additional_tests::test_failed_cache_write_does_not_stop_later_update_or_observers`
   - `coordinator::tests::test_real_watcher_applies_complete_state_and_persists_matching_version`

   Failure output showed watcher/coordinator timeouts. This was the one required full test attempt, run through default restricted `exec_command` (not host/native execution), with the host's original `TMPDIR` and no temporary-directory override. Cargo reported 101; the command tool's returned shell exit code was not captured, so no `make` exit code is asserted. These failures are retained without diagnosis or retry. The separate existing startup graph-oracle failure is owned/carried to07 and was not changed here.

### Local package and archive evidence

`cargo package --list --workspace --locked --offline` passed. It showed `LICENSE-APACHE`, `LICENSE-MIT`, and README in every package. The copilot payload also includes its useful example, tests, fixtures, and support files; these are honestly retained as development material, not mislabeled runtime assets.

Plain `cargo package --workspace --locked --offline` generated `.crate` payloads under `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/workspace-coverage-contract/target/package/tmp-registry/`, verified `nms` and `nms-core`, then failed at `nms-save` with `no hash listed for nms-core v0.3.0` (Cargo error 101). The extracted archive tree is `/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/workspace-coverage-contract/target/package/archive-overlay/`; scratch overlay config is `/tmp/nms-s06-archive-patch.toml`. One archive-only check succeeded:

`cargo check --offline --manifest-path target/package/archive-overlay/nms-save-0.3.0/Cargo.toml --config /tmp/nms-s06-archive-patch.toml`

Its project dependencies came from extracted `.crate` payloads, not checkout paths. This establishes a labelled local composition check, not pristine registry verification. Independent CDC inspection confirmed current archives for all ten packages, the two license hashes above in every archive, and VCS source `6b47cd3d758e6b44e28041ae52eceba0439c35db` with the expected package path.

The first offline install attempt could not write Cargo's normal user registry source cache and did not install globally. One bounded correction used scratch Cargo home `/tmp/nms-s06-install-offline.G4wwNx/cargo-home`, scratch install root `/tmp/nms-s06-install-offline.G4wwNx/install-root`, existing read-only cached index/archive inputs and the archive overlay. It remained blocked because cached `clap_derive v4.6.7` is absent. No binaries were installed; no dependency download, user/global install, or further retry occurred. Therefore install smoke and clean install readiness remain unproved.

### Historical 0.2.0 provenance

The first normal-TLS official crates.io API metadata/download route returned HTTP 403 for all five packages (`/tmp/nms-s06-history.7ac8uk`); it is retained as an unsuccessful route. One bounded alternate official route using native curl with normal TLS succeeded for sparse-index metadata and static crate archives. Artifacts are under `/tmp/nms-s06-history-index.zoKUD6/`. Index checksums match archive SHA-256 values:

| Package 0.2.0 | Archive SHA-256 |
|---|---|
| `nms` | `d97c2f616724422dab19bd3d386e19b492dd6758c6f69fc7e651a21eda67e16f` |
| `nms-core` | `c323620f6d06396826d18a9d93a916b77b62a8a4a15e5c16a1de6aaa35422b2c` |
| `nms-save` | `8b9c6d99c6a00b88d1430cefd8367503ed25d9532a42f133605ecd51d24e19b6` |
| `nms-graph` | `8999e443b4b1d97ac83f310c38728e9353ab49e5e69b5afffca58bcb8ddc5fc3` |
| `nms-copilot` | `66653a43f410054334220d0c9f43fc074af69ac5a2c8e87de6cab5c4cac4f28d` |

All five archive `.cargo_vcs_info.json` values point to source commit `079f034e3a07707aa6a366c275ca6495aeba1740` (exactly 40 hexadecimal characters); the source tree identifies workspace version `0.2.0` and MSRV 1.91. Historical provenance is complete for these five named packages only. It is not current 0.3.0 install/publication readiness, and no release or publication action was taken.

## Proposed row status and remaining ownership

- **ID-01:** propose done after CDC review for the bounded original-clause reconciliation, retained prior behavior evidence, startup controls, and wizard/config native observations. The CPR limitation is explicit; it does not establish history-command continuation.
- **ID-02:** open pending the human M7.10 disposition. README is corrected to current behavior; this is not a theme deferral or implementation decision.
- **ID-03:** propose done after CDC's all-ten-archive inspection and identical regular license-file hashes.
- **ID-04:** open/mixed. Five historical 0.2.0 provenance checks passed; clean current workspace packaging failed at the unpublished sibling hash boundary, the one archive overlay check passed, and isolated install remains blocked by absent cached `clap_derive 4.6.7`.
- **ID-05:** open. The exact full-test failures and limits are recorded; project-level coverage, package install, platform/features and other carried Arc07/04/05 gates remain with their owners.

Genuine nonzero save input remains pending the root/human response; no private save files were searched. The startup graph oracle remains owned by07. No themes, dependency changes, publication, or other unissued scope were introduced. Ownership returns to CDC for independent disposition; source is pinned above and this result is the sole CC evidence report.
