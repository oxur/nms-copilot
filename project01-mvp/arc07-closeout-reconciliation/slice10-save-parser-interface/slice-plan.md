# Slice10 — Usable checked SaveParser interface

2026-10-26. Explicit Duncan request, not automatic arc growth. CDC GPT6.1 Sol High; sole existing CC GPT6 Luna High. Source baseline3e8e46066136978aad1d878ca1608515862ff8c4 clean, dedicated codex/workspace-coverage-contract. Root owns source index/commit/remote; CC returns WIP, CDC independently reviews. Planning index stays CDC-owned.

Outcome: an external Rust caller can parse either supported on-disk representation through one public common interface, and production checked file consumers use it. Four source paths only: new nms-save/src/parser.rs, lib.rs, validated.rs, new tests/parser_interface.rs (all under crates/). No dependencies, format/encoding changes, compatibility parser change, config, last-results or MCP extraction. Goatfungus/NomNom remain outside support.

Binding recipe and literal oracles are in cc-prompt.md. The object-safe borrowed-byte API returns existing owned ValidatedSave/SaveError; stateless NmsSaveParser implements the unchanged checked file-byte pipeline. parse_validated_save_file keeps its signature/I/O and delegates through the trait; parse_validated_save remains deobfuscated-JSON-only and legacy APIs retain semantics. No extra injection/framework/format adapters. Inherited32-bit offset/bundled-mapping panic limits remain documented, not repaired.

Author readiness: inspected original file pipeline validated.rs:192–207, canonical validation/row pairing, error enum, first-byte decompression, existing checked fixture/literal oracles and real CLI/watch callers. API-02 borrowed/owned and EH-04 typed errors support the signature; TR-17 keeps production concrete dispatch while external tests demonstrate dyn compatibility; DC-01/04 require public examples/errors/panics. Sans-I/O is byte-boundary here, with existing file wrapper owning I/O. No semantic choice remains for CC; private helper mechanics within recipe are local discretion.

Verification: format→lint→focused save/checked CLI tests→native all-feature workspace test, all-feature workspace build, strict nms-save rustdoc. CC retains exact commands/results and failure limits in closing-report.md; raw logs may remain temporary named paths, no new evidence framework. CDC independently inspects actual diff and executes focused external API/consumer/docs checks; reuse unchanged prior proof, do not duplicate whole native gate without new reason. Existing Windows CI6h cancellation remains OPEN; no CI rerun. Maximum5 actual refinements for this new explicitly requested task; no07 budget reset. Stop on consequential drift/scope/facility issue rather than expand.

Artifact home: artifacts/ only if needed for actual supporting evidence; none pre-created. cc-prompt.md is initial preserved assignment.

## Assignment history

- Initial cc-prompt.md —2026-10-26, active; no predecessor for this expressly requested increment. CC intake/readback in closing-report.md, not another record family.
