# Adventure journals and managed media

**Current destination — 2026-10-27:** Duncan moved this capability to the unnumbered **Adventure notebook** project in the [repository roadmap](../../backlog.md). The original requirements/proposals below remain preserved; references to Project02 Arc06/G08/G09 describe the pre-split plan. Project02 provides identity and durable atlas inputs; it does not implement notes, clipboard capture, image storage or their restore contract. Research/interaction/native trials require their own agreed scope/budget before execution. No later project directory or assignment is opened by this notice.

Version 1.0 — 2026-10-11. Design for expanded Arc06; no implementation or native capture test. Confirmed needs come from the Operator's gameplay discussion. Command grammar, keybindings and library adoption below are proposals to validate in G08/G09.

## Experience and scope

Switch from the game to the shell, record an experience with little friction, and return to play. Notes can concern systems, planets, animals/companions, plants, minerals, bases or cultures as supported by the atlas. Unsupported subject identity must be reported; preserve text/drafts without inventing an entity association. A journal entry may link several subjects, with one body and one author, rather than duplicating content for each subject.

Each logical save retains its distinct narrative, labelled by traveller and main/expedition context. Sharing a world, account, image bytes or prose never merges narratives. A synced replica is a source of the same save, not another author. World identity is shared for comparison; visits, observations, bases and entries remain attributed. Intentional links between narratives are permitted and do not silently copy or reassign authorship.

Use the active save's most recent accepted snapshot for implicit location. “Current” means last observed in that save, not guaranteed live game position. Display save name, target, galaxy and snapshot freshness. An unavailable or ambiguous location requires explicit selection or an unresolved draft. Never use a more recent file from another save to fill that gap.

## Proposed shell interactions

Examples express intended workflows; they are not implemented commands or a final clap grammar:

```text
note add planet "Chili's World" "Tiny insects beside the landing site."
note add system "Gohairt Platform" "A place worth returning to."
note add planet --comment "A quiet evening beneath the rings."
note add planet
notes list
notes list planet
notes list planet "Chili's World"
notes list --scope all
notes journal --profile "main - Xbox"
```

The explicit `--comment` form avoids confusing an omitted planet name with the comment. With no comment, offer multiline composition. With no name, resolve the current subject in the selected save. Duplicate names produce a chooser or structured ambiguity, never a fuzzy automatic selection. `notes list` defaults to entries concerning the current system and its supported contained subjects, grouped by system-level, planet, animal, mineral, etc. A narrower subject filter and a chronological view use the same underlying entries. All-save results keep author labels; explicit global listing need not require a current system. Final grammar must distinguish “all authors here” from “all entries everywhere.”

On implicit targeting, show a message such as:

```text
Adding note for planet Chili's World
main - Xbox · Euclid · snapshot captured 2 minutes ago
[compose; Escape to cancel]
```

Freeze author, context, resolved subjects and source snapshot when composition begins. If autosave advances or the user switches saves, the note still belongs to that frozen context. Show the difference visibly. Submit/cancel behavior must be tested with multiline paste: a newline must not unexpectedly save half a note. Escape cancels the pending operation without committing an entry; do not use a timed auto-submit. Failed writes retain a recoverable draft and an actionable error. Define draft recovery/cleanup and explicit discard before implementation. One-shot CLI/MCP operations never rely on a keyboard cancellation window and never prompt without a terminal.

Entries carry created/edited times and optional user-reported event time with an explicit basis. They do not turn save observation timestamps into visit times. Editing/retargeting an existing entry preserves attribution and revision evidence. A named culture can have narrative entries before its complete spatial extent is known.

## Images: input and display are separate capabilities

Ordinary bracketed terminal paste is text: crossterm exposes `Event::Paste(String)`. Do not assume Reedline will receive binary screenshot bytes. An explicit application action can ask the operating-system clipboard for an image; `arboard::Clipboard::get_image` is a candidate API. File import is the portable baseline, including console screenshots transferred locally. [crossterm API](https://docs.rs/crossterm/latest/crossterm/event/enum.Event.html), [arboard API](https://docs.rs/arboard/latest/arboard/struct.Clipboard.html#method.get_image).

```text
image add "/path/to/screenshot.png" planet "Chili's World"
image paste planet
images list
images list planet "Chili's World"
image show <image-id>
images show planet "Chili's World"
images size
images du
```

Validate singular/plural aliases, selector ambiguity and output behavior before freezing commands. A deliberate capture keybinding can be explored after the command works. Do not read the clipboard continuously. Clipboard capture refers to the host running the process; SSH/headless sessions may not expose the user's desktop clipboard. Report unavailable/unsupported clipboard formats clearly and offer file import.

Displaying an image is a separate terminal-capability decision. Evaluate supported terminal graphics and an external-viewer fallback; opening a viewer happens on explicit `show`, not ingestion. A headless/noninteractive result can return managed attachment metadata/path without emitting graphics. The [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) is a reference, not a promise of universal terminal support. Validate the actual target terminal/platform matrix in G09.

## Storage and lifecycle

- Managed content lives in an application data directory, outside the database; catalog records hold stable attachment IDs, content digests, size, detected media type/dimensions, import/capture time and source, caption, author and typed links. Choose the concrete directory with Arc03's storage layout, preserving existing configuration conventions.
- Copy imported files into managed storage by default so moving the original does not break the atlas. Preserve original file bytes; if clipboard capture supplies decoded pixels, encode a documented format and record that conversion. No promise of recovering the clipboard producer's original encoded file.
- Deduplicate exact stored bytes. Two attachment records may reference one blob and retain different captions/authors. Perceptual image deduplication is outside this scope. Bound bytes, decoded dimensions and temporary work; reject unsupported/corrupt input without publishing a false success.
- Stage and publish content before committing attachment/link records. Interrupted transactions can leave unreferenced blobs; reconcile them explicitly. A missing or altered referenced blob is an integrity failure with its affected records identified.
- `images size`/`du` distinguish unique physical content bytes, attributed logical bytes, and derived thumbnails. Collection totals can overlap; never sum them as physical usage. Count all retained originals/captures and report missing files or incomplete scans.
- Unlinking an attachment from one subject does not delete bytes needed by other links. Define explicit deletion and reference-aware garbage collection; cancel/failure paths cannot leave a committed dangling link. Save-snapshot retention never prunes authored journal/media content.
- Backup/portable export includes media and a manifest with identity, links, digests and attribution. Restore into a new data directory without dependencies on original absolute paths. Existing community imports, tags, generic collections and portable exports remain Arc06 scope; keep their source/galaxy evidence distinct from native save claims.

## Acceptance scenarios and owners

| Case | Required evidence | Owner / gate |
|---|---|---|
| Capture during movement/switch | Start on planet A in save A; advance to B and switch saves before submit; stored entry still names A/its original author; Escape commits nothing | Arc06, G08 |
| Stale/unknown target | Visible source age; no invented current planet; duplicate-name and unavailable-save handling preserve draft | Arc01/06, G08 |
| Two narratives, one world | Two saves under one account make distinct entries/bases; cross-save view keeps both; synced replica adds no duplicate author | Arc01/03/06, G01/G10 |
| Browsing | Grouped system/planet/fauna/mineral views and chronological view return exact expected entries; multi-subject entry has stable identity and declared grouping behavior | Arc05/06, G08 |
| Native clipboard and file input | Supported desktop capture; empty/nonimage clipboard; SSH/headless fallback; quoted/non-ASCII file path; real terminal display/fallback | Arc06, G09 |
| Attachment integrity | Duplicate bytes/different authors, caption edits, broken file, partial write, disk exhaustion and reference-aware unlink leave explainable state | Arc03/06, G06/G09 |
| Durability | Rename/reimport/restart, then backup/restore elsewhere; authored text, original media digests and all associations survive | Arc03/06/07, G07–G09 |

These are future proof obligations. Basic system/planet note capture should be an early Arc06 slice after durable storage; it need not wait for creature, mineral or culture extraction to be complete. Later subjects compose through the same typed linkage model.
