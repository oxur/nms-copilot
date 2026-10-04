# CC iteration01 — correct selected-member provenance

Execute this evidence-only correction before returning to CDC. Project01 / arc06 / slice02, current assignment `arc06-fit-and-finish/slice02-packed-format-authority/cc-prompt-iteration01.md`; predecessor `cc-prompt.md` preserved. CC initial packet ends at `0d1a6b4fbbf86ee104a9e562cd862ae1bf0cc1bb`. Source main `/Users/oubiwann/lab/oxur/nms-copilot`, pinned `a800950ef3558fcf463517696c3c140061e7353d`; planning `.worktrees/planning`, branch planning. Record the actual current planning HEAD after reviewer commit and check both worktrees. Two-Contributor Workflow, Expedited Mode; Operator relays to fresh CC, CDC reviews independently.

**Outcome:** resolve R02 and PA-05/PA-07 by recomputing the same two failed encrypted selections and correcting save-B's fingerprint. PA-01/02/03/04/06 are independently accepted within recorded limits; assess applicability before retaining. No new format research, save selection, decrypted population, source fix, API/galaxy/invalid-value/cache policy or parent closure is authorized.

## Required reading and preflight

| Order | Exact input/state | Required scope and purpose |
|---|---|---|
| 1 | This prompt, sibling slice-plan.md/ledger.md/cdc-verification.md, current planning HEAD | Required-full: assignment, R02, two open rows, five accepted rows and retained evidence |
| 2 | Preserved cc-prompt.md | Required-section: Assignment, fences and permitted changes; investigation steps 4/5 (public population and private condition); Return, commit and author readiness. Carry forward sample/privacy/source boundaries; this iteration narrows authorized writes below |
| 3 | artifacts/witnesses.json, initial execution.md and closing-report.md at 0d1a6b4 (with reviewer material separate) | Required-data: entire witness JSON, all 11 witnesses and population/reference metadata; required-section: execution Archive inventory/selections/decode failures and commit/path failures; report Seven-row disposition, Artifacts/scope, Bubble-up/next gate, Final scope/commit checks. Correct provenance without rewriting the failed attempt |
| 4 | artifacts/cdc-review-receipts.json, current HEAD | Required-data: source/CC fences, reference pin reconciliation, all 11 computed checks, archive counts, both selected attempts, mismatch, limits and failures. Establish retained applicability and exact independent discrepancy |
| 5 | Parent arc-plan.md §§Workflow, Current corrective preparation, Current bounded closeout sequence; project01-mvp/closure-preparation.md §Gates and authority | Required-section: preserve held production/higher-scale gates |
| 6 | Pinned libNom Steam ZIP at source workbench/libNom.io/libNOM.test/Resources/TESTSUITE_ARCHIVE_PLATFORM_STEAM.zip, reference commit 652aabb0147888776a088909cdb13bccbd3d9588 | Required-data: all central-directory metadata for class/count/selection, stored bytes of exactly the same ordinals 9/10, and one zipfile.read failure attempt each. No bulk extraction or additional payload |
| 7 | rust-guidelines guides/02-api-design.md §API-37, guides/11-anti-patterns.md §AP-33; work-verification guides/05-independent-verification.md §Evidence During Correction Review, under /Users/oubiwann/.codex/skills | Required-section, complete examples/caveats: distinguish ciphertext identity from payload identity and preserve failures; retain prior independent semantic evidence with explicit applicability |

Applicable repository/framework instructions still govern. Load required text completely, recover truncation in bounded reads, and keep data-query coverage separate. Before dependent edits, append a source-cited iteration readback to artifacts/execution.md explaining the exact digest mismatch, preserved ciphertext-vs-payload distinction, permitted metadata replacements, retained evidence and unresolved policies. Existing receipts alone do not establish your own intake or comprehension. Missing/mismatched inputs block only dependent work; report them without enlarging scope.

## Binding correction and concrete recipe

1. Confirm source/reference fences and selection population. Use Python zipfile central-directory metadata over the pinned Steam ZIP. Retain member counts/classes and choose exactly the first two basename-matching save[digits].hg entries in original infolist order, excluding mf_ and accountdata. Expected baseline: 119 members, 110 files, 9 directories, 51 metadata, 46 eligible saves, 5 accountdata, 8 other files, zero JSON; ordinals 9/10. Derive actual counts and report drift rather than force those numbers.

2. Rehash stored encrypted bytes, not an attempted/decrypted save. For each selected ZipInfo, read its local header at header_offset, validate signature b'PK\x03\x04', obtain filename/extra lengths from header offsets 26/28, advance by those lengths, then read exactly compress_size bytes. The following author-exercised route illustrates the extraction; keep member names local and out of evidence:

```python
with archive_path.open('rb') as raw:
    raw.seek(info.header_offset)
    header = raw.read(30)
    assert len(header) == 30 and header[:4] == b'PK\x03\x04'
    name_len, extra_len = struct.unpack('<HH', header[26:30])
    raw.seek(name_len + extra_len, 1)
    stored = raw.read(info.compress_size)
    assert len(stored) == info.compress_size
stored_sha256 = hashlib.sha256(stored).hexdigest()
```

Preserve aliases, ordinals, advertised uncompressed sizes and ciphertext lengths. Expected save-A SHA256 `fce94ae978e0f698309ee3ed8e05811cbec6668244c0babb0c7f62df5cf1da9c` remains a same-route control. Fresh save-B result must equal `01fdfb956a2cde624ebbbab501e6df6ad73d73455a749bd786029b5348a3373f`, replacing current JSON's `01fdf6f956a2cde624ebbbab501e6df6ad73d73455a749bd786029b5348a3373f`. Compute first, compare second; copying the desired value is not a fresh receipt. Record commands/method, lengths, hashes and actual exits in execution.md. Reattempt zipfile.read once per selected member; preserve encryption failure with error kind/reason, sanitizing internal filenames. Do not request/search for passwords or substitute later selections.

3. Correct witnesses.json population.archive.selected_attempts[1].encrypted_stored_bytes_sha256 to the observed value. Resolve its currently abbreviated Companion pin everywhere it identifies a current reference: `729c319a15f3be66d01587984ef6fff96a50401` resolves to full `729c319a15f3be66d01587984ef6fff96a50401d`. This is the same source, verified in CDC receipts by independent full-commit retrieval and matching source SHA256; do not repin to a newer revision. All 11 witness inputs, values, outcomes and limitations, sample selections/sizes, null populations and empty retained-pair collection remain unchanged. Current digest/pin metadata may be replaced; historical report/execution text is preserved in the next step.

4. Append dated iteration01 corrections to execution.md and closing-report.md. Explicitly retain the original incorrect table as historical and identify the corrected current save-B fingerprint/full source pin. Link this prompt, describe R02, exact fresh checks and five accepted rows' retention basis. Do not relabel prior calculations/game/export/runtime observations as fresh. Leave base/generalized discovery/nonzero-galaxy/API/compatibility/cache policies held; unavailable populations are still unknown. Field-authority.md needs no semantic change and is outside this correction's write scope.

5. Update only PA-05/PA-07 evidence/notes in ledger.md, keeping them open pending CDC review. Preserve five independently accepted rows/criteria. Validate the complete current witness JSON: 11 unique unchanged witness IDs; same two attempted selections; unchanged values/distributions/pair states; only prescribed digest/reference-pin metadata changed. Compare computed ciphertext digest against both JSON and current correction sections. Check all named-path changes, no raw member/account identity disclosure, required trailers, whitespace and clean pinned source/reference states. Keep every failed/unrun command visible.

## Verification and retained evidence

| Row/claim | Fresh required proof | Exact rejection/limit |
|---|---|---|
| PA-05 / R02 | Recount same ZIP metadata, recompute both ciphertext hashes from local-header bytes, reattempt the two encrypted reads | Old save-B fingerprint fails exact equality; sizes/ordinals/save-A control reject wrong population or byte interval. No payload digest or JSON/pair evidence follows from ciphertext hashing |
| PA-07 | Current JSON/diff/scope/link/sanitation checks and all-seven-row walk; correction sections match computed data | Merely replacing a string without replay, hiding original failure, altering semantic witnesses or claiming source/runtime work fails |
| PA-01/02/03/04/06 retention | Check main/source/inputs/criteria and semantic JSON remain unchanged; cite CDC review plus receipts | New pin spelling is same commit; no general source re-search/calculation/runtime rerun required unless an affected claim or input is actually changed |

This unchanged-source correction has no broad build/test/lint/coverage gate and no disposable Rust probe. The JSON/metadata validator checks current correction properties, not general format truth. If the digest, archive identity or selections disagree, stop dependent correction and return the discrepancy; do not normalize to the requested output. API-37 (SHOULD) applies to evidence identity roles; AP-33 (AVOID) requires failure/unknown states remain explicit. No public type or behavioral decision is delegated here.

## Allowed writes, return and commit

Only these four existing files may change:

```text
project01-mvp/arc06-fit-and-finish/slice02-packed-format-authority/artifacts/witnesses.json
project01-mvp/arc06-fit-and-finish/slice02-packed-format-authority/artifacts/execution.md
project01-mvp/arc06-fit-and-finish/slice02-packed-format-authority/closing-report.md
project01-mvp/arc06-fit-and-finish/slice02-packed-format-authority/ledger.md
```

Do not edit issued prompts, plans, reviewer record/receipts, author lead, field-authority.md, source/tests/dependencies/fixtures/config/cache/game data or reference bytes. Retain earlier execution/report sections. Commit only these exact paths with git commit --only on planning and both trailers exactly once:

    Co-authored-by: Codex <noreply@openai.com>
    Co-authored-by: Billo AI <ai-engineering@billo.systems>

Return this assignment identity, commit, exact fresh hashing/read outcomes and failures, metadata diff, applicability of five retained rows and remaining semantic gates to the Operator for CDC review. The initial report's proposed-done does not bypass this assignment; slice/arc/project remain open. No external thread messaging is authorized. Consequential scope/acceptance/policy changes go to CDC before affected work; this prompt authorizes only the described correction.

Author readiness is recorded in slice-plan.md: exact source/ZIP interval inspected and independently exercised; discriminating wrong/correct hashes, two-selection control and metadata-only diff bound this pass. Existing evidence files hold all output, with one preserved current prompt and a bounded reading manifest. Author self-check is not acceptance of the correction.
