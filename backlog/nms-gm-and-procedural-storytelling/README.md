# NMS GM, coplayer and procedural storytelling

Recorded: 2026-10-27. Status: retained discovery discussion for the accepted, unnumbered Procedural Narratives grouping; no implementation assignment or source-adoption decision. Owner: Duncan for priorities and scope, with Sofie maintaining these notes.

This note preserves ideas and reading leads from the earlier conversation in this chat after a file-coverage check found them only partly represented in the planning records. It is an edited synthesis, not a verbatim archive. The [repository backlog](../../backlog.md#accepted-project-grouping) indexes it under Procedural Narratives. Its authoritative home is `backlog/nms-gm-and-procedural-storytelling/README.md` on the `planning` branch; Duncan corrected the temporary placement on `main` in commit `a866d6d`. Existing project requirements remain in their own records; this note does not expand them.

## What Duncan wants

Make NMS comfortable as a player-facing console and expressive as an AI-facing situational model. Duncan plays while chatting with Sofie, who can query game context through MCP, help remember intent and discoveries, and eventually act as GM for private adventures.

NMS supplies the gameplay mechanics. The desired addition is story: purposeful quests, mysteries, discoveries, recurring threads and consequences woven around actual play. D&D combat, character statistics and replacement resolution rules are not the target. The linked DMG screenshots illustrate the useful ingredients: small roll tables, connected follow-up questions and procedures that turn a seed into a situation.

Grow the tools in the order play makes them necessary. Begin with a simple hunt, learn from playing it together, and build toward richer missions and layered mystery expeditions. This is a preference for discovery through use, not an approved implementation sequence.

## GM dispatch and the hunt loop

Duncan's preferred dispatch mechanism is a planet/portal address. Destinations need not be limited to worlds already visited. A candidate could be chosen randomly, but an address alone must not be presented as proof of a reachable world or a particular landmark.

Rough out the quest's premise and possibilities ahead of time, then develop local particulars from the player's findings after arrival. A GM cannot assume it can log in, construct bases, place labels or objects, spawn characters, or alter the native world. Missions should be playable without those capabilities.

The reusable hunt mechanism discussed was:

1. Establish a location anchor, with galaxy and address and any supported local coordinates.
2. Present a clue, using riddles, lore or observations to give the journey meaning.
3. Let the player travel, explore and choose how to investigate.
4. Verify the relevant finding using available save observations, a player report or optional screenshot evidence.
5. Record what changed in the quest and what the player has learned.
6. Offer the next lead, revelation or agreed reward.

Hard location anchors can support softer interpretation. A clue might direct attention to something the player can actually find rather than require an author-placed prop. Exact completion criteria, fallback clues and reward mechanisms remain design questions; narrative rewards must not imply Copilot can grant native inventory items.

Potential destination sources include known discoveries, cached teleport destinations, community/player bases and player-supplied coordinates. Other-player base availability, provenance and reachability need validation; cached entries are not a global base directory. Existing sites could supply scenery without requiring their authors to participate.

## From a hunt to a mystery

Retain these ambitions for later exploration:

- A small Easter egg hunt with a few meaningful stops and an observable conclusion.
- Multiple clue chains that can be followed in different orders and converge on a revelation.
- Persistent threads across sessions: people or fictional correspondents, factions, unresolved questions, recurring images and consequences.
- Local details discovered during play that develop the larger story without erasing established facts or invalidating a player's deduction.
- GM-private material separated from the player's journal/handbook so queries do not accidentally reveal solutions.

Keep native save facts, computed results, looked-up interpretations, player reports, visual observations and authored fiction distinguishable. Observations can inspire a fictional explanation without becoming evidence that the explanation is native NMS lore. Preserve committed campaign facts and allow choices to matter; do not continually move the answer to force a predetermined ending.

## Procedural story tools

Duncan particularly values die-roll lookup tables as starting points for invention. The supplied DMG pages combine event categories with questions about causes, motives, affected groups and consequences. The useful pattern is a small generative choice followed by prompts that make it causally specific.

Candidate tooling to investigate includes original NMS tables for hooks, discoveries, anomalies, motives, factions, complications, clues and consequences; linked or weighted tables; and ways to record a generated result as stable campaign state. These are proposed tool shapes, not selected data formats or commands.

Source selection should favor material useful for narrative preparation and play: adventure seeds, situations, mystery structure, clue delivery, improvisation, pacing, factions and campaign continuity. Publicly accessible or free-to-download material is not automatically an openly reusable source. Record the exact work/edition, author, license, attribution and content scope before incorporating its text or tables into a distributable guide or tool.

## Gameplay history and intent

Two concrete player needs from this conversation remain especially important:

- "What were the last N planets of biome type X, Y, or Z that I visited?" Return most recent first, with a clear date/evidence basis.
- Session logs in time order: planets visited, travel by warp/teleport where supported, and other useful events. Make them filterable and quiet by default, with richer detail available.

Discovery/scanning time, file modification/sync time, Copilot observation time and actual visit/revisit evidence must stay distinct. A cached teleport destination does not prove a particular journey. Travel mode should be marked unknown or inferred unless its source establishes it; observation gaps cannot be reconstructed as certain events.

Additional discussion leads were intent memory, route/session objectives, "what should I do next?" queries, live-event narration and MCP tools that expose gameplay context beyond atlas primitives. Their interfaces and persistence are not designed here.

## Screenshots as observations

Duncan wants taking a screenshot to become a natural way to show Sofie what he is seeing. The desired chain is capture, detection/ingestion, context association, notification, image analysis and useful annotations/classification.

Candidate behavior includes waiting for a file to finish writing, retaining the original image, linking it to the applicable traveller/save/session/world, and recording labels or observations with their origin and uncertainty. Reversible sidecar metadata was proposed as the default; embedding metadata in the image would be a separate opt-in write operation. No concrete schema is selected.

File watching, MCP notification and actually waking an assistant are different capabilities. The full path needs an integration that can receive the event and initiate analysis; a watcher alone does not establish it. Image interpretation is also distinct from a separately adopted OCR pipeline.

The notebook already retains explicit file/clipboard capture, captions and managed media. It does not yet specify this automatic sensing/analysis workflow. Console screenshots transferred to the laptop are a possible input; no platform acquisition mechanism has been selected.

## Companion presence and in-game chat

Duncan proposed playing his main character on Xbox and putting a second character, running NMS on his laptop, aboard the Xbox character's corvette. Sofie could potentially remain aboard and deliver GM text in native chat, while the full conversation continues in the desktop assistant connected to MCP.

This is an integration idea, not an established control channel. App access, text entry, sending, receiving/reading replies, game focus and multiplayer behavior need their own evidence. Observing the application does not prove reliable control. Autonomous movement is not needed for the proposed first experience, and no game-control implementation is commissioned by this note.

## Reading and source-material register

The following were supplied or discussed as leads. This capture did not fetch them, read every work, verify current availability or complete a license audit. Prior attempted searches did not produce a usable retained research report; do not treat a candidate list as completed comparative research.

| Resource | Why retain it | Reading/reuse status |
|---|---|---|
| [Our Narrative Handbook: Cognitive Dispatch Across Scales](</Users/oubiwann/Dropbox/cowboys.n.beans/Creative Writing/Guides/Narrative Handbook/Cognitive Dispatch Across Scales/src>) | Duncan explicitly identified it as a primary creative resource: narrative across scales, reader inference, gaps/rests, recurrence, polyphony and interlace. | Local source directory was inventoried, not fully read for this task. Its bibliography/source ledger matters when adapting material drawn from other authors. |
| [Sly Flourish: The Lazy Dungeon Master Creative Commons](https://slyflourish.com/the_lazy_dungeon_master_cc.html) | Candidate preparation procedures for an attributed LLM GM guide. | Verify the exact covered work, license and attribution; do not extend the permission to all Sly Flourish books or articles. |
| [Sly Flourish: Top Advice](https://slyflourish.com/top_advice.html) | Candidate GM practices and improvisation guidance. | Separate source whose reuse terms and useful sections need review. |
| [Galactic Hub NMS-RPG System](https://nmsgalactichub.miraheze.org/wiki/Guide:NMS-RPG_System) | Existing NMS roleplaying example and comparison for native-world storytelling. | License and practical assumptions need review. Our proposed hunts should not require authors who can build or label sites. |
| [D&D SRD 5.2.1 PDF](https://media.dndbeyond.com/compendium-images/srd/5.2/SRD_CC_v5.2.1.pdf) and [SRD Markdown repository](https://github.com/downfallx/dnd-5e-srd-markdown) | Source leads already located by Duncan; Markdown could ease later source processing. | Primarily rules material, so lower priority for this narrative-focused work. Check edition, provenance and applicable notices before reuse. |
| [Open5e API documentation](https://open5e.com/api-docs) | Possible structured source/content discovery route. | Determine whether it offers useful narrative material and inspect source-specific licenses; do not assume one blanket content license. |
| [Dungeon Master's Guide reading copy](https://archive.org/details/dungeon-masters-guide/Dungeon%20Master%27s%20Guide/) and Duncan's seven pasted screenshots | Concrete examples of the desired small-table-plus-follow-up-question technique, campaign growth and event consequences. Duncan also owns the physical book. | Supplied excerpts can ground discussion. The conversation did not establish an open license for the full DMG; do not treat it as an approved redistributable corpus. |

Handbook entry points to read later include `00-index.md`, `01-core-model.md`, `02-scales-of-story.md`, `04-gaps-rests-reader-room.md`, `05-recurrence.md`, `06-polyphony-counterpoint.md`, `07-interlace.md`, `09-practice-tools.md` and `10-bibliography-and-source-ledger.md`. This is a reading pointer, not a claim of completed extraction.

Duncan's proposed material-development path is to gather sources in Markdown, read them for useful narrative procedures, build an attributed LLM GM storytelling guide, then adapt that guide fully to NMS. He subsequently requested broader OSS/CC resource reconnaissance before planning. That reconnaissance remains outstanding; this preservation pass does not fulfill it or adopt any source.

## Existing records and remaining gaps

| Existing record | What it retains | What this note preserves beyond it |
|---|---|---|
| [Project02 plan](../../project02-personal-atlas/project-plan.md) and [ledger](../../project02-personal-atlas/ledger.md) | Durable observations/history, recent-world queries, explicit evidence/date basis and cross-save scope. | Rich session timeline, travel-mode evidence, intent/objectives and GM consumers. No new Project02 acceptance criterion is added here. |
| [Journals and media](../../project02-personal-atlas/artifacts/journals-and-media.md) | Frozen author/target, composition, grouped/chronological entries, captions, managed images and restoration; now allocated to the Adventure notebook. | Automatic screenshot sensing, assistant notification/analysis and classification metadata. |
| [Cultures and exploration](../../project02-personal-atlas/artifacts/cultures-and-exploration.md) | Traveller histories, authored culture identities, evidence-qualified geometry and exploration; now allocated to later work. | Authored quests, clue graphs, campaign truth and fictional factions/consequences. Culture analysis is not a prerequisite for a small hunt. |
| [Research and library decisions](../../project02-personal-atlas/artifacts/research-and-library-decisions.md) | Engineering storage, interface and spatial-library leads. | Narrative source selection, exact reuse terms, GM guide creation and procedural story material. |
| [Repository roadmap](../../backlog.md) | Project03's reserved client/gameplay integration and the accepted atlas/notebook/culture grouping. | Detailed GM/coplayer ambitions and the boundaries between observation, chat, notification and control. |

## Questions to resume with

- Which open narrative sources provide the most useful tables and procedures, under terms suitable for our intended guide and software distribution?
- Which observations can support a first enjoyable hunt without promising landmarks or data we cannot obtain?
- What should remain private GM state, and what should be visible in the player's handbook and session journal?
- Which visit/session facts can Project02 actually retain, and what additional instrumentation would later gameplay integration need?
- What concrete screenshot-to-assistant path is available, and which parts require an active assistant session?

These questions are retained for future scope decisions. No dependencies, commands, schemas, source licenses, native control abilities or full campaign architecture are accepted by recording them.
