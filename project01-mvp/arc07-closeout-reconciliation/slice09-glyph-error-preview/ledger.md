# Slice09 acceptance ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| GP-01 | Rejected glyph diagnostic is UTF-8 safe and bounded to20 bytes with existing ASCII/error behavior | Four baseline-failing regressions: 3/4-byte preview splits, Kelvin abbreviation panic and name wrong-suffix; then exact corrected boundaries and literals | serious | Explicit Duncan correction gate | done | [CDC review](cdc-verification.md), source1b79525d | Four baseline-red oracles retained; scoped UTF-8 correction accepted |
| GP-02 | Successful parse/order/aliases and source-boundary suffix consumption plus direct CLI error propagation remain usable; obsolete docs corrected only for repaired branches | Literal emoji/name/abbrev/hex suffix tests; real convert rejection diagnostic, ordinary error exit/no panic; exact diff | correctness-grade | Existing public contract | done | [CDC review](cdc-verification.md), native1256 | Predicate/order/alias/API preserved; exact suffix and CLI checks pass |
| GP-03 | Applicable format/lint/focused and serialized native workspace gates independently reviewed | CC format→lint→focused checks; CDC actual review/affected checks and ONE native make test | correctness-grade | Repository gates and explicit follow-up | done | [CDC review](cdc-verification.md); two retained logs | Host1256 pass, restricted247/7 retained; no coverage/other acceptance work |

All3 rows independently accepted at1b79525d; one records-only refinement, zero source corrective iterations after initial delivery. Glyph closure gate MET; pause, no next assignment.
