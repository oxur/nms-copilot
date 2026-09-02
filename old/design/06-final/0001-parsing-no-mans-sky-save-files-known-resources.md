---
number: 1
title: "Parsing No Man's Sky save files - Known Resources"
author: "model path"
component: All
tags: [change-me]
created: 2026-03-05
updated: 2026-03-05
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# Parsing No Man's Sky save files - Known Resources

**No Man's Sky saves are LZ4 block-compressed JSON with obfuscated keys, wrapped in XXTEA-encrypted metadata — not a proprietary binary format.** This means a Rust parser needs five core components: XXTEA decryption (metadata only), LZ4 block decompression, SHA-256 and SpookyHash V2 verification, JSON parsing, and a key-mapping table. No existing Rust implementation exists anywhere. The ecosystem is dominated by C#/.NET (libNOM), Java (goatfungus), and Python (Chase-san gists, NMSCD decoder), with ~30+ community projects spanning every aspect of save manipulation. Below is the complete technical catalog.

---

## The raw binary format: two files per save slot

Each save slot produces a **paired file system**: a storage file (`save.hg`) containing compressed game data and a manifest file (`mf_save.hg`) containing integrity hashes and metadata. Steam saves live at `%AppData%\HelloGames\NMS\st_<SteamID>\`, with slots numbered `save.hg`/`save2.hg` through `save29.hg`/`save30.hg` (15 slots × 2 saves each: auto and manual). Account-wide data lives in `accountdata.hg`/`mf_accountdata.hg`.

**Three format generations** have existed. Format **2000** (vanilla 1.0, August 2016) used an encryption scheme now handled only by MetaIdea's abandoned nms-savetool. Format **2001** (Foundation 1.10 through Prisms 3.53) introduced XXTEA encryption of both the storage file and metadata, with LZ4 compression of the JSON payload. Format **2002** (Frontiers 3.60 onward) dropped XXTEA encryption of the storage file entirely, retaining it only for the metadata — the storage file is now purely LZ4 block-compressed. All modern tools target format 2002. The game also writes uncompressed JSON in certain edge cases; detection is simple: if the first two bytes are `0x7B 0x22` (ASCII `{"`), the file is plaintext JSON.

### Metadata file structure (mf_save*.hg)

The manifest file is exactly **0x68 bytes (104 bytes)**, structured as 26 little-endian uint32 values, XXTEA-encrypted. After decryption:

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0x00 | 4 | Magic | Always `0xEEEEEEBE` |
| 0x04 | 4 | Format Version | `0x7D0` (2000) for legacy; varies for newer |
| 0x08 | 8 | SpookyHash Key[0] | First half of 128-bit integrity hash |
| 0x10 | 8 | SpookyHash Key[1] | Second half of 128-bit integrity hash |
| 0x18 | 32 | SHA-256 Hash | SHA-256 of the raw storage file bytes |
| 0x38 | 4 | DecompressedSize | Size of decompressed JSON in bytes |
| 0x3C | 4 | CompressedSize | Size of compressed data |
| 0x40 | 4 | ProfileHash | Hash of Steam/GOG profile key (0 if none) |
| 0x44 | 36 | Padding | Zero-filled |

Newer game versions (post-Waypoint 4.0) may extend this with an additional **~280 bytes** of metadata, as observed in goatfungus debug logs. The Xbox/Game Pass format uses a completely different WGS container system with its own `containers.index` file structure, documented extensively in goatfungus issue #306.

The XXTEA key for metadata decryption derives from the archive number (save index + 2) mixed with constant `0x1422CB8C` via a MurmurHash3-like operation, then overlaid onto the base key string **`NAESEVADNAYRTNRG`**. XXTEA uses **8 rounds** (not the standard `6 + 52/n`) with delta `0x9E3779B9`.

### Storage file structure (save.hg) — format 2002+

The storage file is a concatenation of LZ4 blocks, each preceded by a **16-byte header**:

```
struct NmsSaveBlock {
    magic: u32,              // Always 0xFEEDA1E5 (little-endian)
    compressed_size: u32,    // Bytes of LZ4-compressed payload following
    uncompressed_size: u32,  // Bytes after LZ4 block decompression
    padding: u32,            // Always 0x00000000
}
// Followed by `compressed_size` bytes of LZ4 block data
```

Each uncompressed chunk maxes at **0x80000 bytes (512 KB)**. This is standard LZ4 **block** compression — not LZ4 frame format. The `lz4_flex` crate in Rust provides the correct block-level API. After decompressing and concatenating all blocks, the result is a UTF-8 JSON string with obfuscated keys.

### Integrity verification

**SpookyHash V2** (Bob Jenkins) with seeds `0x155AF93AC304200` and `0x8AC7230489E7FFFF` is computed over: the SHA-256 of the decompressed JSON concatenated with the decompressed JSON itself. The resulting 128-bit hash must match Key[0]/Key[1] in the metadata. Editing a save without regenerating both hashes causes the game to reject the file.

---

## Complete catalog of parsers and libraries

### Tier 1: Primary libraries (actively maintained, production-quality)

**libNOM.io** — The authoritative cross-platform .NET library. Handles all save formats (2001, 2002, Waypoint extensions) across Steam, GOG, Xbox Game Pass, PlayStation, Switch, and Mac. Uses K4os.Compression.LZ4, SpookilySharp, and Newtonsoft.Json. NuGet package v0.14.2. This is the single most complete reference implementation for building a Rust parser — its source code at `github.com/zencq/libNOM.io` documents every platform's binary quirks.

**libNOM.map** — Companion .NET library for JSON key obfuscation/deobfuscation using MBINCompiler's `mapping.json`. NuGet v0.13.8. Supports legacy keys back to Beyond 2.11. The mapping file is a flat `{"obfuscated": "readable"}` JSON object downloadable from MBINCompiler releases.

**libNOM.collect** — Backup/restore library for save collections (ships, companions, multitools). NuGet package.

**MBINCompiler / libMBIN** — Game data decompiler by monkeyman192. Generates the critical `mapping.json` for save key deobfuscation. Also contains all C# struct and enum definitions (GcPlayerStateData, GcBiomeType, etc.) that define the save file's data model. Version v6.24.0-pre2 (Feb 2025). Documented at `monkeyman192.github.io/MBINCompiler/`.

### Tier 2: Save editors and tools (useful for format understanding)

| Project | Language | URL | Notes |
|---------|----------|-----|-------|
| NomNom | C#/.NET | github.com/zencq/NomNom | Most complete GUI editor, 708 stars, GPL-3.0 |
| goatfungus NMSSaveEditor | Java | github.com/goatfungus/NMSSaveEditor | Original editor, contains items.xml/words.xml data |
| vectorcmdr fork | Java/C# | github.com/vectorcmdr/NMSSaveEditor | Actively maintained fork, C# port underway |
| nmssavetool | C# | github.com/matthew-humphrey/nmssavetool | CLI tool, documents format 2001 encryption |
| MetaIdea/nms-savetool | C# | github.com/MetaIdea/nms-savetool | **Critical**: Storage.cs contains definitive format 2001 byte-level spec |

### Tier 3: Community tools (Python, JavaScript, Kotlin, others)

| Project | Language | URL | What it does |
|---------|----------|-----|-------------|
| NMSCD/NMS-Save-Decoder | Python+TS | github.com/NMSCD/NMS-Save-Decoder | LZ4 decode/encode pipeline with key mapping |
| NMSCD/nms-save-web-editor | Vue | github.com/NMSCD/nms-save-web-editor | Online save editor (active, March 2026) |
| UNOWEN-OwO/NMS-Save-Parser | Python | github.com/UNOWEN-OwO/NMS-Save-Parser | GUI JSON parser with compress/decompress/mapping |
| Chase-san decode.py | Python | gist.github.com/Chase-san/704284e4acd841471d9836e6bc296f2f | Minimal, clean decoder — best reference for format 2002 |
| Chase-san nmssavetool.py | Python | gist.github.com/Chase-san/16076aaa90429ea6170550926b70f48b | De/compress CLI using lz4 package |
| NightCodeOfficial/NMS-Base-File-Editor | Python | github.com/NightCodeOfficial/NMS-Base-File-Editor | PySide6 GUI, auto-downloads mappings, handles LZ4+deobfuscation |
| waryder/BBB-NMS-Save-File-Manipulator | Python | github.com/waryder/BBB-NMS-Save-File-Manipulator | Base sorting, inventory management, active 2024-2025 |
| BrayanIribe/NMS-SaveEditor | JavaScript/Vue | github.com/BrayanIribe/NMS-SaveEditor | Web-based difficulty editor |
| joshbarrass/NMS-Save-Editor | Python | github.com/joshbarrass/NMS-Save-Editor | Simple save modification utilities |
| Kevin0M16/NMSCoordinates | C# | github.com/Kevin0M16/NMSCoordinates | Coordinate calculator + fast travel, uses libNOM.map |
| IzzyTheDreamingFox/FoxTech-DNA | Kotlin | github.com/IzzyTheDreamingFox/FoxTech-DNA | Android app, ship/multitool/freighter catalog, 53 stars |
| okranger1777/nms-mission-progress | Python | github.com/okranger1777/nms-mission-progress | Mission progress extraction/injection |
| cwmonkey/nms-expeditions | JavaScript | github.com/cwmonkey/nms-expeditions | Expedition JSON generator, 50 stars |
| jaszhix/NoMansConnect | JavaScript/Electron | github.com/jaszhix/NoMansConnect | Location sync service (archived) |
| BradfordBach/NMSLocator | Python | github.com/BradfordBach/NMSLocator | Coordinate extraction from saves |
| djmonkeyuk/nms-base-builder | Python/Blender | github.com/djmonkeyuk/nms-base-builder | Blender base building integration |
| zencq/Pi | Python | github.com/zencq/Pi | Procedural item seed collection, 126 stars |

### NMSCD web tools (operate on exported JSON)

The No Man's Sky Community Developers organization maintains several TypeScript/JavaScript tools: **NMSCD/Coordinate-Converter** and **NMSCD/Coordinate-Conversion** (TypeScript library for glyph/galactic/voxel/save-format conversion), **NMSCD/NMS-Teleport-Editor**, **NMSCD/Base-Relocator**, and **NMSCD/Base-Reorder**. All operate on decompressed JSON and are useful reference implementations for coordinate math.

### Nexus Mods-only tools (no public repos)

**NMSBaseJsonEditor** (nexusmods.com/nomanssky/mods/3849) — lightweight C#/.NET 4.8 base editor with LZ4 decompression and deobfuscation. **NMS Coordinate Tool** (nexusmods.com/nomanssky/mods/3857) — coordinate reading and teleportation. **NMS Companion** (nexusmods.com/nomanssky/mods/1879) — collection sharing tool built on libNOM.io with 20,000+ member Discord.

### What doesn't exist

**No Rust crates, Go libraries, npm packages, or PyPI packages** exist for NMS save parsing. No 010 Editor templates or ImHex patterns exist either — the format is compressed JSON, not a fixed-layout binary structure amenable to hex templates.

---

## The JSON data model: PlayerStateData and beyond

After decompression and key deobfuscation, the save JSON has this top-level structure:

```json
{
  "Version": 6726,
  "Platform": "Win|Final",
  "ActiveContext": "BaseContext",
  "CommonStateData": { /* account-wide: seasons, rewards */ },
  "BaseContext": {
    "PlayerStateData": { /* 200+ fields */ },
    "SpawnStateData": { /* position/spawn state */ },
    "GameMode": 0,
    "DifficultyState": { /* difficulty config (4.0+) */ },
    "GameKnowledgeData": { /* words, lore */ }
  },
  "ExpeditionContext": { /* same structure as BaseContext */ },
  "DiscoveryManagerData": { /* discoveries and uploads */ }
}
```

The **Version** field double-encodes format version and game mode: 4616 = Normal, 5128 = Creative, 5640 = Survival, with Permadeath and Custom having their own values.

### PlayerStateData: the 200+ field behemoth

The canonical field reference is MBINCompiler's GcPlayerStateData struct definition at `monkeyman192.github.io/MBINCompiler/classes/gc/GcPlayerStateData/` (GUID 0xA18051E8320F1145, size 0x2A030 bytes). Key field groups include:

**Universe position** uses `GcUniverseAddressData` containing `RealityIndex` (galaxy number, 0-indexed), `GalacticAddress` (VoxelX/Y/Z signed integers, SolarSystemIndex, PlanetIndex), with separate fields for current, previous, home, and multiplayer locations.

**Inventories** follow a uniform `GcInventoryContainer` structure with `Slots` (array of items with Type, Id string, Amount, MaxAmount, DamageFactor, and X/Y Index), `ValidSlotIndices`, Width, Height, and IsCool flag. The save contains **20+ separate inventory containers**: Exosuit (main, tech, cargo), current ship, current multitool, grave, freighter (main + tech), 10 base storage chests, magic chests, cooking ingredients, and corvette storage. Each also has a companion `GcInventoryLayout` with seed data.

**Ship ownership** is an array of `GcPlayerOwnershipData` (expanded to 9+ slots in recent updates), each containing Name, Resource (model path + procedural seed), Inventory, InventoryLayout, and position data. Ship types are identified by model path: `FIGHTERS/FIGHTER_PROC.SCENE.MBIN` for fighters, `SCIENTIFIC/SCIENTIFIC_PROC.SCENE.MBIN` for explorers, `SAILSHIP/SAILSHIP_PROC.SCENE.MBIN` for solar ships, and so on.

**Base building** uses `PersistentPlayerBases`, an array of `GcPersistentBase` objects each containing Name, BaseType, GalacticAddress (uint64 encoded), Position/Forward vectors, Owner (LID/UID/USN/PTK), and an **Objects array** of base parts with ObjectID (e.g., `^BASE_FLAG`, `^WALL_L`), Position/Up/At vectors, Timestamp, UserData, and Message. This is the structure that base editors like NMS-Base-File-Editor and Base-Relocator manipulate.

**Currencies** are simple Int32 fields: Units, Nanites, and Specials (Quicksilver). Knowledge tracking includes KnownTech, KnownProducts, KnownSpecials (all string lists), KnownWords (GcWordKnowledge objects), and KnownPortalRunes (int, 0–16). Mission progress is tracked in MissionProgress as `{Name, Progress, Data}` tuples with IDs like `ACT1_STEP1` through `ACT3_STEP3` (Artemis), `ATLAS1` through `ATLAS11`, and expansion-specific IDs.

**Fleet data** includes FleetFrigates (all owned frigates), FleetExpeditions (active missions), and freighter data with its own universe address, orientation matrices, and NPC reference. Vehicles use the same GcPlayerOwnershipData structure indexed 0–6 (Roamer, Nomad, Colossus, Pilgrim, Dragonfly, Nautilon, Minotaur).

Newer updates added Pets/Companions (3.2), SettlementStates (Frontiers 3.6), DifficultyState (Waypoint 4.0), SquadronPilots, CorvetteStorageInventory (Breach), and purple system flags (Worlds Part I). The schema is additive — fields are added but rarely removed.

---

## Galactic coordinates: 48-bit addressing across 256 galaxies

The galactic coordinate system encodes locations as a 48-bit value decomposed into five fields:

| Component | Bits | Range | Description |
|-----------|------|-------|-------------|
| PlanetIndex | 4 | 0–F | Planet/moon within system |
| SolarSystemIndex | 12 | 0x000–0xFFE | Star system within region |
| VoxelY | 8 | 0x00–0xFF | Height (vertical position) |
| VoxelZ | 12 | 0x000–0xFFF | North-south position |
| VoxelX | 12 | 0x000–0xFFF | East-west position |

Each VoxelX/Y/Z coordinate identifies a **region** containing 200–600+ accessible star systems. The galaxy is a **4096 × 256 × 4096 voxel grid** — a flattened disc with four non-rotating spirals. The center is void for ~3,000 light years. Special system indices are hardcoded: **0x079 always contains a black hole**, **0x07A always contains an Atlas Interface**, and **0x3E8–0x429 are purple star systems** (Worlds Part II, requires Atlantid Drive).

**Portal glyph encoding** uses 12 hexadecimal glyphs (16 symbols, 0–F) arranged as `P-SSS-YY-ZZZ-XXX` where P=PlanetIndex, SSS=SolarSystemIndex, YY=VoxelY, ZZZ=VoxelZ, XXX=VoxelX. Portal coordinates use **galaxy center as origin**, while signal booster coordinates (`ALPHA:XXXX:YYYY:ZZZZ:SSSS`) use a **corner origin**. Conversion: `Portal_X = (SignalBooster_X + 0x801) mod 0x1000`, similar for Y (mod 0x100 with offset 0x81) and Z. Save files store VoxelX/Y/Z as **signed decimal integers** (can be negative), requiring conversion to/from the unsigned hex representation used by portals and signal boosters.

The game has **256 unique galaxies** numbered 0–255 (RealityIndex in save data), plus galaxy 256 (Odyalutai) as an overflow. Four types exist: **Norm** (178 galaxies, cyan hologram), **Lush** (25, green), **Harsh** (26, red), and **Empty** (26, blue). Galaxy type affects biome generation probabilities. The type follows a fixed pattern: Euclid (0)=Norm, Hilbert (1)=Norm, Calypso (2)=Harsh, with Lush galaxies at indices 10, 19, 30, 39, 50, 59, 70... Region names within galaxies are procedurally generated from coordinate-derived syllable combinations plus one of ~20 suffixes (Adjunct, Void, Expanse, Terminus, Boundary, Fringe, Cluster, Mass, Band, Cloud, Nebula, Quadrant, Sector, Anomaly, Conflux, Instability, Spur, Shallows) or prefixes ("Sea of", "The Arm of").

---

## MBIN game data and how it connects to saves

Save files are JSON; game data files are MBIN (a binary serialization format). **MBIN files contain only data values with no structural metadata** — property names, types, and nesting are defined in the NMS executable. MBINCompiler reverse-engineers these structures into C# classes under libMBIN, enabling MBIN↔EXML (XML) conversion.

The MBIN header (format v2) contains a 4-byte MagicID, FormatID (low short = FormatNMS 2500, high short = FormatAPI), VersionID, 8-byte TemplateGUID, 64-byte TemplateName (the C# class name), and MetaOffset. Game data lives in `GAMEDATA/PCBANKS/*.pak` (PSARC archives extractable with PSArcTool). Key files include `METADATA/REALITY/TABLES/BASEBUILDINGTABLE.MBIN` (base parts), `REWARDTABLE.MBIN` (rewards), and various tables defining items, technologies, and recipes.

Save files reference game data through **string IDs**: substances like `FUEL1` (Carbon), `OXYGEN`, `LAND1` (Ferrite Dust); technologies like `HYPERDRIVE`, `SHIELD`; products like `ANTIMATTER`, `ATLAS_SEED_1`. These IDs are defined in MBIN game tables and appear directly in save JSON inventory slots. The `mapping.json` file shipped with MBINCompiler releases is the bridge — it maps obfuscated save file keys to human-readable names derived from these same game data structures.

Since NMS v5.50 (Worlds Part II), the modding system changed significantly: PAK files are no longer used for mods. EXML/MBIN files go directly into `GAMEDATA\MODS\<MOD_NAME>\`. The game now natively reads MXML format (matching Hello Games' internal format), making MBINCompiler output MXML instead of the older EXML.

---

## Community infrastructure and discovery services

The **NMS Modding Discord** (~7,800+ members, discord.gg/no-man-s-sky-modding-215514623384748034) is the primary knowledge hub where format details circulate informally. Key figures include monkeyman192 (MBINCompiler), zencq (NomNom/libNOM), Wbertro (AMUMSS), and gregkwaste (model viewer/RE). The **NMSCD GitHub organization** (github.com/NMSCD) maintains community developer tools including the Save-Decoder, web editor, coordinate tools, and wiki page creators.

Hello Games operates a **closed discovery service** at `nms.hellogames.co.uk` using AWS infrastructure and PlayFab backend. It stores universal addresses, discovery types, names, and upload timestamps — but **no planet/biome data** (all regenerated procedurally). **No public API exists.** Network analysis reveals HTTPS/TLS endpoints but no documented schema. The **AssistantNMS API** (api.nmsassistant.com) is the closest public alternative, serving game data (recipes, items, costs) extracted from MBIN files.

The **NMS Coordinate Exchange** (nmsce.com, r/NMSCoordinateExchange with ~200K members) uses a Firebase backend for storing portal addresses of ships, multitools, and planets. A seed engineering spreadsheet exists at `docs.google.com/spreadsheets/d/1kcEtrAdkSd-eeOp4rxOX8XUCHUSBPt5T_FVy2AbI8tY` documenting upgrade module seeds. Key wiki resources include the **NMS Modding Wiki** (nmsmodding.fandom.com), **Step Modifications Wiki** (stepmodifications.org/wiki/NoMansSky:Game_Structure), and **NMS Retro Wiki** (nomansskyretro.com/wiki) for technical format documentation.

The only academic-adjacent resource is gregkwaste's reverse engineering writeup on procedural generation (sudonull.com/post/77048) and Sean Murray's **GDC 2017 talk** "Building Worlds Using Math(s)" covering terrain noise functions and voxel generation.

---

## Building the Rust parser: a dependency map

A complete Rust implementation needs these components, mapped to recommended crates:

| Component | Purpose | Crate | Complexity |
|-----------|---------|-------|------------|
| LZ4 block decompress/compress | Storage file I/O | `lz4_flex` (pure Rust) | Low |
| XXTEA | Metadata decrypt/encrypt | Hand-implement (~50 lines) | Low |
| SHA-256 | Storage file integrity | `sha2` | Trivial |
| SpookyHash V2 | Data integrity verification | `spooky` or hand-implement | Medium |
| JSON parse/serialize | Save data manipulation | `serde_json` + `serde` | Low |
| Key mapping | Obfuscation/deobfuscation | Custom `HashMap<String,String>` | Low |
| WGS container parsing | Xbox Game Pass support | Custom (per issue #306 spec) | Medium |

The critical constants are: metadata magic **`0xEEEEEEBE`**, LZ4 block magic **`0xFEEDA1E5`**, SpookyHash seeds **`0x155AF93AC304200`** and **`0x8AC7230489E7FFFF`**, XXTEA base key **`NAESEVADNAYRTNRG`**, XXTEA rounds **8**, max LZ4 block size **0x80000**, and metadata size **0x68 bytes**. The definitive byte-level reference for format 2001 encryption is MetaIdea/nms-savetool's `Storage.cs`. For format 2002 (current), Chase-san's 30-line Python gist is the cleanest reference — the storage file is just sequential LZ4 blocks with no encryption.

## Conclusion: what this catalog reveals

The NMS save format is **well-understood but poorly documented** — knowledge lives in source code rather than specifications. The format is simpler than it appears: format 2002 storage files are trivially decompressible (no encryption, just LZ4 blocks), with the only cryptographic complexity in the 104-byte metadata file (XXTEA + SpookyHash). The JSON payload after decompression is large (several MB) but structurally straightforward, with MBINCompiler's `mapping.json` providing the Rosetta Stone for obfuscated keys. The absence of any Rust implementation represents a genuine gap in the ecosystem. The three essential reference implementations are: **Chase-san's decode.py** (simplest correct decompressor), **MetaIdea/nms-savetool Storage.cs** (definitive format 2001 encryption spec), and **libNOM.io** (most complete cross-platform implementation). A Rust parser targeting only format 2002+ on Steam could be functional in under 500 lines of code; full cross-platform support with format 2001 backward compatibility would require perhaps 2,000–3,000 lines.
