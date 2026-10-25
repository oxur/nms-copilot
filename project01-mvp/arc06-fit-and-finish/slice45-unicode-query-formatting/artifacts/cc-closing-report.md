# Slice45 CC closing report — proposed-done

Assignment: Project01 Arc06 Slice45, initial `cc-prompt.md` plus the one executed corrective pass `cc-prompt-iteration01.md`. This is a CC implementation return, not CDC/CRC acceptance or Arc06 closure.

The committed candidate is `fe5e9d1287a353bcfbf67fc8db6c5445102c03f5` on `codex/unicode-query-formatting`, parent `9b632b665885d1a8b596006dd96ebcaaad41a71a`. It changes exactly `crates/nms-query/src/display.rs`, `crates/nms-copilot/src/dispatch.rs`, and adds `crates/nms-cli/tests/unicode_display_integration.rs`. The working tree and index are clean.

## Ledger row walk

| Row | CC disposition | Evidence and limits |
|---|---|---|
| UF-01 | Proposed-done | Private truncation counts Unicode scalar values, preserves UTF-8 boundaries, respects the zero-to-three-period budgets, and preserves in-bound inputs. The 17-case literal matrix and focused Unicode query run `f958e14cafae42de88e37b95f05d0a0c` passed (3 tests). No grapheme or terminal-width guarantee is made. |
| UF-02 | Proposed-done | Plain and dark formatter witnesses cover find plus normal-route and waypoint branches, literal labels, values, addresses, summaries, and unchanged source names. Included in the 3 passing focused query tests. |
| UF-03 | Proposed-done | Two compiled `nms` child tests used isolated temporary checked-save copies/config homes and verified useful find/route output and unchanged fixture bytes. `76d4dda8d1e24dcabe5fc3fef867ecf3` passed both tests. Earlier real readiness children remain preserved as the original 101/inside-`é` negative witnesses in the issued readiness receipts. |
| UF-04 | Proposed-done | Full CDC source/U1–U4 review and targeted F1/F2 whole-file review closed without another finding. Strict all-features documentation gate `9fc64ed85418436ea91cc277d57cf5d5` passed. I inspected the generated find, route and dispatch pages: scalar limits/qualifications appear; only the name-truncation panic cause disappeared; the parser, jump-count and theme caveats remain. |
| UF-05 | Proposed-done for CC evidence | All nine original gate groups completed with exit 0 on their final authorized attempts; raw IDs, channels, environment, times and identities are retained in the capture file referenced by `validation.json`. Initial lint failure and the mistaken sandbox `make test` failure are preserved. Identical authorized host `make test` passed. Commit has the exact three paths, required consecutive trailers, clean status, and raw commit body equal to the prepared message. CDC's fresh reproduction on this commit is pending. |
| UF-06 | Open — CDC/Operator-owned | Fresh independent reproduction/review of the final commit, acceptance, retrospective, bottom-up verification and parent/Operator gates are not completed by this CC return. |

## Bottom-up to Arc06

1. **Capability and delivery state.** The candidate implements Unicode-scalar-safe private query-name formatting while preserving existing field limits and ASCII output. Query formatter witnesses and two real CLI child tests passed. The capability is committed on the dedicated feature branch; integration to the project’s accepted baseline remains with CDC/Arc06 owners.
2. **Unexpected findings and owners.** No new source scope or dependency emerged. The initial lint found two helper-signature/type-shape issues; the authorized first refinement corrected only those two shapes. One `make test` invocation was mistakenly run with default sandbox settings despite the prompt requiring the first attempt on the host. That exit-2 attempt (228 passed, seven watcher/coordinator timeouts) is preserved; the identical `require_escalated` retry with inherited `TMPDIR` passed. The execution-setting miss is recorded for CDC’s verification/retrospective; no causal claim about sandbox behavior is made. No additional slice or arc-plan change is proposed.
3. **Scope-as-specified versus delivered.** All three authorized source paths were delivered, with no dependency, fixture, parser, map, CLI grammar or public API changes. Existing tests and fences were reviewed as preserved by CDC. No scoped item was silently dropped; unrelated M7.7 coverage/CI/platform/full-command work, D0043/theme reconciliation, parser/map Unicode debt, authentic NomNom input, and parent/Operator/release gates remain open under their existing owners.

## Artifacts, handback and status

Supporting return artifacts in the authorized ignored override are `execution.md` (readbacks and intake correction), `validation.json` (gate/commit identities and receipts), and this report. The complete raw attempt channels remain at `/private/tmp/nms-slice45-cc-attempts.json` and are referenced, not copied into this prose. The initial and iteration prompts, slice plan and ledger are first-class planning records; the three committed source/test files are the deliverable, not supporting artifacts. Generated `target/doc` output is local build output.

All captured command sessions have completed; no CC-owned process remains. There are no staged or unstaged source changes, no source/index/gate action is pending, and the exact-three-path commit is clean. CC status is proposed-done only. CDC still owns fresh final-candidate verification, independent acceptance, retrospective and remaining Arc06/Project01 gates.
