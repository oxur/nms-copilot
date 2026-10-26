# Slice09 acceptance ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|---|---|---|---|---|---|---|---|
| GP-01 | Rejected glyph diagnostic is UTF-8 safe and bounded to20 bytes with existing ASCII/error behavior | Four baseline-failing regressions: 3/4-byte preview splits, Kelvin abbreviation panic and name wrong-suffix; then exact corrected boundaries and literals | serious | Explicit Duncan correction gate | open | — | No claim all parser panic paths repaired |
| GP-02 | Successful parse/order/aliases and source-boundary suffix consumption plus direct CLI error propagation remain usable; obsolete docs corrected only for repaired branches | Literal emoji/name/abbrev/hex suffix tests; real convert rejection diagnostic, ordinary error exit/no panic; exact diff | correctness-grade | Existing public contract | open | — | API/dependencies frozen |
| GP-03 | Applicable format/lint/focused and serialized native workspace gates independently reviewed | CC format→lint→focused checks; CDC actual review/affected checks and ONE native make test | correctness-grade | Repository gates and explicit follow-up | open | — | No coverage or other acceptance work |
