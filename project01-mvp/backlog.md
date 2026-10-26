# Project01 backlog

Operator-adopted deferrals,2026-10-25. These entries authorize no new implementation or project/arc. Original evidence and obligations remain linked from the [current handoff](arc07-closeout-reconciliation/slice08-composition-handoff/artifacts/handoff.md#operator-decisions-and-residual-obligations--2026-10-25).

## B01 Clean installation

**Deferred; current package/install evidence accepted for now.** Remaining work: demonstrate clean dependency-resolved package installation and binary smoke in an isolated scratch root, preserving exact pristine sibling-hash and uncached `clap_derive4.6.7` failures and partial archive-overlay limits. Licenses and historical five0.2 provenance are complete; no need to repeat them automatically. Origin: P01-G05/G06, M7.8, ID-04. **Owner:** Duncan for priority/external dependency authority, future explicitly assigned contributor for proof. **Re-entry:** publication is blocked by installation, or a future project needs clean installation. No download/install/publication authority is implied.

## B02 Configurable themes and biome wiring

**Explicitly deferred.** Remaining work: configurable terminal color schemes and semantic biome-color wiring against original M7.10; preserve existing fixed palettes/API and corrected README claims until then. **Owner:** Duncan for priority/scope, future explicitly assigned UI/config contributor for implementation and meaningful behavior proof. **Re-entry:** Duncan elects future UI/configuration work and approves its concrete scope. Original D0043/table and higher composition duties are not blanket deferred by this theme decision.

## B03 Conditional 32-bit LZ4 compressed-size offset overflow

**Explicitly deferred by Duncan,2026-10-25; not a current closeout blocker.** Malformed compressed-length input can overflow the `usize` payload offset (`16 + u32::MAX`) on 32-bit. This is source-derived conditional evidence, **not a directly encountered or 32-bit-reproduced failure**; no fix or successful platform proof is claimed. Retain [Slice31 plan:53](arc06-fit-and-finish/slice31-save-api-documentation/slice-plan.md#L53) and [CDC:43](arc06-fit-and-finish/slice31-save-api-documentation/cdc-verification.md#L43). Remaining work: address the unchecked offset arithmetic with meaningful malformed-length boundary regression and appropriate platform evidence. **Owner:** Duncan for prioritization, a future explicitly assigned contributor for correction/proof. **Re-entry deadline:** one of the next **two future projects**; earlier if encountered directly. No particular project/arc is automatically assigned, and no plan, source work or test is opened here.

## B04 warp_jump_count unchecked range and cast-sum overflow

**Explicitly deferred by Duncan,2026-10-25; not a current closeout blocker.** Range is unchecked; `ceil(leg_distance/range)` casts to `usize`, and summing leg counts can overflow/profile-dependently panic. Invalid floats use Rust cast semantics and do not uniformly panic; this documented limit has **not been directly encountered**. No fix/reproduction is claimed. Retain [Slice32 plan:27](arc06-fit-and-finish/slice32-graph-api-documentation/slice-plan.md#L27) and [CDC:45](arc06-fit-and-finish/slice32-graph-api-documentation/cdc-verification.md#L45). Remaining work: explicitly resolve invalid-range/overflow behavior and verify it with meaningful public-helper/consumer boundary tests; any consequential policy choice returns to Duncan. **Owner:** Duncan for prioritization, a future explicitly assigned contributor for correction/proof. **Re-entry deadline:** one of the next **two future projects**; earlier if encountered directly. No particular project/arc is automatically assigned, and no routing redesign, source work or test is opened here.
