# Save and account experience — Arc01 design

**Current ownership — 2026-10-27:** Friendly inventory/selection/switching remains FIRST in current [Project02 Arc01](../project-plan.md). General settings/output and remaining existing-command REPL parity are current Arc02. This design's concrete selector grammar, replica-link mechanisms and startup preference remain proposals until their stated decisions are settled. Reserve separate play/read/future-authorship identities, while the notebook/culture mutation behaviors described in the dated augmentation belong to their later projects in the [repository backlog](../../backlog.md). No database or notebook implementation is a prerequisite for friendly saves.

Status: proposed behavior, 2026-10-03. This document is a design artifact, not an arc implementation plan.

## What the user sees

Lead with the in-game save name or the user's local alias. For example:

```text
Save                 Player       Last saved on   Context   Status
main - Xbox          oubiwann*    Xbox            Main      Available
main - Steam         oubiwann*    Mac             Main      Available

* Player name comes from matching save records.
```

These names illustrate observed data; the layout is a proposal. The normal view should not show numeric account identifiers. Details can reveal the source folder, slot, physical file versions, exact timestamps, identity evidence, and stable machine ID. A platform token identifies the last recorded platform, not necessarily the platform of origin or the account folder's service.

User-assigned aliases take display precedence but do not rename the game save. Keep the in-game name visible in details and searchable in the chooser. Empty/missing names get a readable fallback such as “Unnamed save · slot 3”; duplicates gain an account alias, mode, or slot suffix. Internal IDs remain stable when labels change.

One flat searchable save list is the default when several local account folders exist. Grouping/filtering by a friendly account alias remains available. Avoid a forced numeric-account screen before the user can see recognizable save names. If a local account has no reliable label, assign a stable readable fallback such as “Local account A” and allow an alias.

## Identity rules

| Concept | Identity and evidence | Must not imply |
|---|---|---|
| Source root | Registered filesystem location with a local ID | A person or platform identity |
| Local account | Container/account directory within a root; optional user alias | Every save belongs to the directory's platform account |
| Recorded player identity | Namespaced platform token plus UID/LID evidence and observed names | A verified real-world person or current online display name |
| Logical save | App-assigned stable ID; native `SaveUniversalId` retained as correlation evidence | Filename, slot number, or display name is globally unique |
| Replica/slot | A save's concrete location and paired physical files | Two copies are always interchangeable or equally fresh |
| Snapshot | Exact captured bytes plus source observation metadata | File modification time is the player's last visit time |
| Context | Main, expedition, or preserved unknown identifier | Context changes necessarily create a new logical save |

`SaveUniversalId` is not a UUID: sampled JSON contains a string while the reference game schema declares an unsigned 64-bit field. Preserve its representation and validate its encoding. In the four examined files, paired files agreed and the two logical saves differed. Copy/rename/cross-save/slot-reuse stability still needs tests; do not silently merge separate roots on this field alone.

Within a registered slot, matching native IDs support pairing. Different IDs produce a conflict, not an arbitrary winner. If the native ID is absent, retain a provisional local association and expose that limitation. Replaced slots with a new native ID create a new save identity. Moves/copies across roots become candidate links requiring an explicit user choice; path aliases must be reconciled so one file reached through a symlink is not ingested twice. Preserve unavailable registrations rather than reassigning their IDs.

Player-name enrichment first uses explicit local aliases, then matched identity evidence in owned records. `UsedDiscoveryOwnersV2` supplies candidate identities; in the sampled saves its `USN` names were empty. Owned base records supplied `oubiwann`. Match identity fields and record type; do not label a save using the first or most common arbitrary discoverer. If multiple identities remain plausible, show “Multiple recorded identities” with details. No online account lookup is required.

## Selection contract

Proposed command spellings, to validate against clap before implementation:

```text
nms saves
nms saves --details
nms saves --json
nms saves use "main - Xbox"
nms --profile "main - Xbox" info
nms --account "My Xbox saves" --slot 3 info
nms --save /path/to/save.hg info

REPL: saves
REPL: use "main - Xbox"
```

Preserve `--save PATH` for exact physical-file selection. Add a distinct friendly selector rather than changing the meaning of that flag. Selectors resolve exact aliases/names or stable IDs; fuzzy matching helps the interactive chooser but must not silently choose a close match in scripts. Duplicate names return candidates with readable disambiguators.

Resolution order:

1. Explicit `--save` path. Incompatible profile/account/slot options are a clear usage error, not silently ignored flags.
2. Explicit profile selector, optionally constrained by account; or explicit account plus slot. A bare slot may use the remembered account only if that account is unambiguous and shown in status; otherwise request selection interactively or report ambiguity in noninteractive mode.
3. Remembered explicit selection, if still available and consistent. A missing selection produces an actionable unavailable state; do not fall through to someone else's newest file.
4. A sole available candidate can be selected and named. Otherwise show the interactive chooser, or return a structured selection-required error when noninteractive.

“Latest file anywhere” becomes an explicit opt-in policy. Within a logical save, file mtime may rank candidate physical versions, but the result says “file updated,” not “last played.” Retain both candidates and expose equal-time/metadata disagreements. A malformed newest file must not silently turn a read into an older successful save; show the failure and offer the last readable version explicitly.

The startup default above is provisional pending the user preference. Explicit `use` persists the preference; one-shot `--save`/`--profile` overrides do not. Listing saves does not alter the preference. Cancellation is a normal outcome and leaves the current selection intact.

## Scanning, configuration, and switching

Metadata inventory reads existing parser output without constructing the whole galaxy graph. Cache metadata by registered source and file fingerprint; mtime/size are cheap invalidation hints, not proof of equal content. Inspect valid save candidates, avoid treating unrelated subdirectories as accounts, and report unreadable files separately from empty slots. Limit decompression/input sizes and bound concurrent work. Refresh progressively if cold scanning is slow.

Keep `~/.nms-copilot/config.toml` for user-editable policy and existing settings. Use a versioned app-owned `profiles.json` for registration IDs, aliases, links, and remembered selection. A separate stable lock file protects registry read/modify/write; do not lock a data inode that atomic replacement will swap away. Write a temporary file in the same directory, sync as supported, then replace. A parse error preserves the original and reports repair instructions. Back up migration inputs and preserve unknown configuration keys; avoid rewriting unrelated formatting where practical.

Import existing `save.dir`/`save.file` into the registry once, recording their values and a migration marker in the registry. Unchanged migrated values no longer override a subsequent explicit `use` selection. If the user later changes those legacy settings, treat that changed path as a new explicit configuration selection and record its resolution. CLI overrides still win. Keep the old configuration readable for compatibility; no transaction spanning the two files is needed. A failed migration leaves the original path usable. Profile changes never modify game saves.

Switching builds a complete new session before publication. The active selection, graph, completions, filters/reference location, cache key, and watcher generation move together. A failed switch leaves the old session usable. Discard late watcher events from the previous generation. Reset save-dependent filters and explain that reset. The REPL prompt/status always identifies the active save and context.

MCP gets the same inventory and resolution service. Prefer explicit per-request save/context scope; an assistant query must not silently change the human's REPL selection. Any future switch tool must be explicit and session-scoped. JSON results include opaque IDs, display labels, provenance/status, and a schema version; diagnostics/prompts stay off stdout.

## Arc01 acceptance scenarios to carry into its plan

- Recognizable save names on the real synced Xbox/Mac fixture; unknown account names remain useful.
- Duplicate names, multiple account roots, non-ASCII names and paths, missing names, unavailable files, and conflicting IDs resolve predictably.
- `--slot` no longer binds to the first directory; all save-consuming commands, including import, share resolution.
- A corrupt file or invalid configuration remains visible as an error; another readable save can still be listed.
- Startup/chooser cancellation, missing remembered save, one-shot overrides, and noninteractive use follow the selection contract.
- Alias changes survive restart and an in-game rename without altering game files; same basenames under different roots do not collide in cache keys.
- Switching during a watcher update preserves one coherent session generation; MCP queries remain scoped.
- Registry lock contention, failed replacement, and legacy migration retain recoverable prior state.

These are proposed observable requirements, not passing tests. Detailed commands and test fixtures belong in the Arc01 plan after design review.

## 2026-10-11 augmentation — traveller and scope foundations

Arc01 must reserve stable, separate identities for logical save, replica, recorded account/player and narrative traveller. Four main saves on two accounts are an acceptance case: sharing an account name/UID must not merge their narratives. An evidenced synced replica is one logical save/traveller; a divergent copy has a visible unresolved relationship until explicitly linked or separated. Main/expedition context stays attached to its save narrative unless an explicit supported lifecycle says otherwise.

The player's current save/location, a query's read scope, and journal authorship are separate. Ordinary reads default to the selected save; explicit multi-save/account/all-save queries preserve labels and never change selection. Culture-filtered reads do not switch the traveller. Every applicable query family must define its multi-save behavior; operations requiring a single location (for example routing) retain an explicit origin and galaxy rather than selecting one arbitrarily. Cross-galaxy summaries are allowed; spatial routes remain galaxy-scoped.

A note/image composition captures author, resolved subjects and source snapshot at entry. Switching saves or accepting a newer autosave must not move it. Prompt/status shows both the active play save and any differing frozen author/target. A name is a selector, not an identity; same-name planets require disambiguation. Automated interfaces take explicit scope and return structured ambiguity/errors instead of interactive prompts.

G01/G10 must test whether discovery-owner fields identify an account or a specific traveller. Until supported, an owner-account match alone cannot assign a claim to one of that account's saves. This uncertainty does not block separate save journals.

These requirements expand the 2026-10-03 proposal; startup preference remains provisional. Command spellings and editor keybindings are still to be validated. See [journals/media](journals-and-media.md) and [cultures/exploration](cultures-and-exploration.md).
