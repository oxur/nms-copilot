---
number: 4
title: "libNOM Reference Guide for Rust Reimplementation"
author: "meta length"
component: All
tags: [change-me]
created: 2026-03-06
updated: 2026-03-06
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# libNOM Reference Guide for Rust Reimplementation

**Purpose:** Technical reference for building `nms-save` and `nms-core` crates in Rust, derived from analysis of the C# libNOM.io (v0.14.2) and libNOM.map (v0.13.8) libraries by zencq.

**Primary consumer:** AI assistants (Claude Code / Claude Desktop) working on the NMS Copilot project.

---

## 1. Architecture Overview

libNOM.io is a .NET library organized as partial classes across directories:

```
libNOM.io/
├── Container/          Container.cs — save slot abstraction (metadata, JSON, file refs)
├── Enums/              GameVersionEnum, StoragePersistentSlotEnum, SaveTypeEnum, etc.
├── Extensions/         Newtonsoft.cs (key mapping integration), Span.cs
├── Global/             Analyze.cs (entry point), Constants.cs, LZ4.cs, Json.cs, Convert.cs
├── Meta/               GameVersion.cs, SaveSummary.cs, SaveName.cs — metadata extraction
├── Models/             ContainerExtra, Difficulty, TransferData
├── Platform/           Platform_Read.cs, Platform_Write.cs — base read/write pipeline
├── PlatformSteam/      PlatformSteam.cs, _Read.cs, _Write.cs — XXTEA, key derivation
├── PlatformGog/        Inherits from PlatformSteam (identical format)
├── PlatformMicrosoft/  Xbox Game Pass WGS container handling
├── PlatformPlaystation/ PS4/PS5 via SaveWizard
├── PlatformSwitch/     Nintendo Switch saves
├── Services/           Steam service integration
└── Settings/           PlatformSettings, PlatformCollectionSettings
```

**Read pipeline:** `AnalyzeFile() → ExtractHeader → DetectPlatform → LoadMeta() → DecryptMeta() → LoadData() → DecompressData() → DeobfuscateKeys()`

---

## 2. Complete Constants Catalog

### LZ4 Block Format

| Constant | Value | Notes |
|----------|-------|-------|
| Block magic | `0xFEEDA1E5` | Little-endian: bytes `[0xE5, 0xA1, 0xED, 0xFE]` |
| Block header size | `0x10` (16 bytes) | Magic(4) + CompressedSize(4) + DecompressedSize(4) + Padding(4) |
| Max chunk size | `0x80000` (524,288 bytes) | Maximum decompressed size per block |

### Metadata (mf_save.hg)

| Constant | Value | Notes |
|----------|-------|-------|
| Meta magic (decrypted) | `0xEEEEEEBE` | First uint32 after XXTEA decryption — verification sentinel |
| Meta format 0 | `0x7D0` (2000) | Vanilla 1.0 — **not supported** |
| Meta format 1 | `0x7D1` (2001) | Foundation 1.10 through Prisms 3.53 |
| Meta format 2 | `0x7D2` (2002) | Frontiers 3.60 through Adrift |
| Meta format 3 | `0x7D3` (2003) | Worlds Part I (5.00) |
| Meta format 4 | `0x7D4` (2004) | Worlds Part II+ (5.50) |

### Meta File Lengths (Steam/GOG, bytes)

| Format | Total | Notes |
|--------|-------|-------|
| Vanilla (2001) | `0x68` (104) | 26 × uint32 |
| Waypoint (2002) | `0x168` (360) | Extended with save name |
| Worlds Part I (2003) | `0x180` (384) | |
| Worlds Part II (2004) | `0x1B0` (432) | |

### XXTEA Encryption

| Constant | Value | Notes |
|----------|-------|-------|
| Base key | `"NAESEVADNAYRTNRG"` | 16 ASCII bytes → 4 × uint32 little-endian |
| Key as uint32s | `[0x5345414E, 0x44415645, 0x5259414E, 0x47524E54]` | `"NAES"`, `"EVAD"`, `"NAYR"`, `"TNRG"` |
| Key derivation XOR | `0x1422CB8C` | XOR'd with storage slot enum value |
| Key derivation rotate | 13 bits left | Applied after XOR |
| Key derivation multiply | 5 | Applied after rotate |
| Key derivation add | `0xE6546B64` | Added after multiply → becomes key[0] |
| TEA delta | `0x9E3779B9` | Standard golden ratio constant |
| TEA reverse delta | `0x61C88647` | Added to hash during reverse iteration |
| Rounds (format 2001) | 8 | Vanilla meta length `0x68` |
| Rounds (format 2002+) | 6 | All newer meta lengths |

### SpookyHash V2

| Constant | Value | Notes |
|----------|-------|-------|
| Seed 1 | `0x155AF93AC304200` | uint64 |
| Seed 2 | `0x8AC7230489E7FFFF` | uint64 |
| Input | SHA-256(decompressed) ++ decompressed | Concatenation |
| Output | 128-bit hash → two uint64 | Stored at meta offset 0x08–0x17 |

**Important:** SpookyHash and SHA-256 verification are only used for META_FORMAT_1 (2001). Format 2002+ does NOT use them for verification (see `PlatformSteam_Write.cs` line 41).

### Save Slot Management

| Constant | Value | Notes |
|----------|-------|-------|
| Max save slots | 15 | Per account |
| Saves per slot | 2 | Auto (0) + Manual (1) |
| Max total saves | 30 | |
| Index offset | 2 | MetaIndex 0 = AccountData, 1 = unused, 2+ = saves |
| Gamemode offset | 512 | Multiplied by PresetGameModeEnum |
| Season offset | 128 | |
| Version threshold | 4098 | THRESHOLD_VANILLA — boundary for account detection |

### File Naming (Steam/GOG)

| MetaIndex | File | Meta File | Identity |
|-----------|------|-----------|----------|
| 0 | `accountdata.hg` | `mf_accountdata.hg` | Account data |
| 2 | `save.hg` | `mf_save.hg` | Slot 1 Auto |
| 3 | `save2.hg` | `mf_save2.hg` | Slot 1 Manual |
| 4 | `save3.hg` | `mf_save3.hg` | Slot 2 Auto |
| 5 | `save4.hg` | `mf_save4.hg` | Slot 2 Manual |
| ... | ... | ... | ... |
| 31 | `save30.hg` | `mf_save30.hg` | Slot 15 Manual |

### Platform Save Paths

| Platform | Account Pattern | Default Path |
|----------|-----------------|--------------|
| Steam (Win) | `st_76561198*` | `%AppData%/HelloGames/NMS/<account>/` |
| Steam (Linux) | `st_76561198*` | `~/.local/share/Steam/steamapps/compatdata/275850/pfx/drive_c/users/steamuser/Application Data/HelloGames/NMS/<account>/` |
| Steam (Mac) | `st_76561198*` | `~/Library/Application Support/HelloGames/NMS/<account>/` |
| GOG | `DefaultUser` | Same as Steam |
| Xbox Game Pass | `*_29070100B936489ABCE8B9AF3980429C` | `%LocalAppData%/Packages/HelloGames.NoMansSky_bs190hzg1sesy/SystemAppData/wgs/` |

---

## 3. Save File Reading Pipeline (Format 2002+)

### Step 1: Read Storage File (save.hg)

The file is a sequence of LZ4 blocks. Parse until EOF:

```
loop {
    read 16-byte header:
        magic:             u32 (must be 0xFEEDA1E5)
        compressed_size:   u32
        decompressed_size: u32
        padding:           u32 (always 0)

    read compressed_size bytes of LZ4 payload
    decompress using LZ4 block decompression → decompressed_size bytes
    append to output buffer
}
concatenate all decompressed chunks → complete JSON string (UTF-8)
```

**Edge case:** If the first bytes are `0x7B 0x22` (ASCII `{"`), the file is uncompressed plaintext JSON — skip decompression entirely.

**Reference:** `Platform_Read.cs:228-240` (block parsing), `LZ4.cs:32-45` (decompression wrapper)

### Step 2: Read and Decrypt Metadata (mf_save.hg)

1. Read entire meta file (104–432 bytes depending on format version)
2. Cast bytes to `uint32[]` (little-endian)
3. Determine iteration count: 8 for 104-byte files, 6 for all others
4. Derive key[0] from storage slot:

   ```
   key[0] = rotate_left((slot_enum_value ^ 0x1422CB8C), 13) * 5 + 0xE6546B64
   key[1..3] = META_ENCRYPTION_KEY[1..3]  // from "NAESEVADNAYRTNRG"
   ```

5. Run XXTEA decryption (see algorithm below)
6. Verify: `decrypted[0] == 0xEEEEEEBE`
7. If verification fails, try all other StoragePersistentSlot values (file may have been moved)

**Reference:** `PlatformSteam_Read.cs:12-79`

### Step 3: XXTEA Decryption Algorithm

```rust
fn xxtea_decrypt(data: &mut [u32], key: &[u32; 4], iterations: usize) {
    let last = data.len() - 1;
    let mut hash: u32 = 0;

    // Pre-compute hash (sum of deltas)
    for _ in 0..iterations {
        hash = hash.wrapping_add(0x9E3779B9);
    }

    // Reverse iteration
    for _ in 0..iterations {
        let key_index = (hash >> 2 & 3) as usize;
        let mut current = data[0];

        // Process elements last..1 (backwards)
        for j in (1..=last).rev() {
            let prev = data[j - 1];
            let t1 = (current >> 3) ^ (prev << 4);
            let t2 = (current.wrapping_mul(4)) ^ (prev >> 5);
            let t3 = prev ^ key[(j & 3) ^ key_index];
            let t4 = current ^ hash;
            data[j] = data[j].wrapping_sub(
                (t1.wrapping_add(t2)) ^ (t3.wrapping_add(t4))
            );
            current = data[j];
        }

        // Process element 0 (wraps around to last)
        let prev = data[last];
        let t1 = (current >> 3) ^ (prev << 4);
        let t2 = (current.wrapping_mul(4)) ^ (prev >> 5);
        let t3 = prev ^ key[key_index];
        let t4 = current ^ hash;
        data[0] = data[0].wrapping_sub(
            (t1.wrapping_add(t2)) ^ (t3.wrapping_add(t4))
        );

        hash = hash.wrapping_add(0x61C88647);
    }
}
```

**Reference:** `PlatformSteam_Read.cs:35-79`

### Step 4: Metadata Structure (After Decryption)

For META_FORMAT_1 (104 bytes = 26 × uint32):

| Offset | Size | Field |
|--------|------|-------|
| 0x00 | 4 | Magic (`0xEEEEEEBE`) |
| 0x04 | 4 | Format version (`0x7D1`, `0x7D2`, etc.) |
| 0x08 | 8 | SpookyHash key[0] (uint64) |
| 0x10 | 8 | SpookyHash key[1] (uint64) |
| 0x18 | 32 | SHA-256 of raw storage file |
| 0x38 | 4 | Decompressed JSON size |
| 0x3C | 4 | Compressed data size |
| 0x40 | 4 | Profile hash (0 if none) |
| 0x44 | 36 | Padding (zeros) |

For Waypoint+ formats, additional fields follow (save name, summary, difficulty, playtime, season, gamemode) — see `Meta/` directory for extraction logic.

### Step 5: Key Deobfuscation

After decompression, JSON keys are obfuscated (e.g., `"F2P"` instead of `"Version"`, `"6f="` instead of `"PlayerStateData"`).

**Mapping format** (`mapping.json` from MBINCompiler releases):

```json
{
  "libMBIN_version": "6.11.0.1",
  "Mapping": [
    { "Key": "F2P", "Value": "Version" },
    { "Key": "6f=", "Value": "PlayerStateData" },
    ...
  ]
}
```

**Algorithm:**

1. Load mapping as `HashMap<String, String>` (obfuscated → readable)
2. Walk JSON tree recursively
3. For each object key, look up in map — if found, replace key
4. Handle collisions: one obfuscated key maps to different readable keys depending on JSON path context (only one known collision: `"NE3"` → context-dependent)
5. Account data uses a subset of the mapping (split at `"UserSettingsData"` entry)

**Three mapping sources** (all merged):

- `mapping_mbincompiler.json` (54KB) — primary, latest game version
- `mapping_legacy.json` (1.9KB) — older keys from pre-Beyond versions
- `mapping_savewizard.json` (81 bytes) — single correction: `"MultiTools"` → `"Multitools"`

**Reference:** `libNOM.map/Mapping_Deobfuscation.cs:56-140`, `libNOM.map/Mapping.cs:79-98`

### Step 6: Platform Detection

Detection logic from file header bytes:

1. Read first bytes of storage file
2. If starts with `[0xE5, 0xA1, 0xED, 0xFE]` (LZ4 magic) or `{"F2P":` or `{"Version":`:
   - Check for `"NX1|Final"` in first ~160 bytes → **Switch**
   - Check for `"PS4|Final"` → **PlayStation**
   - Otherwise → **Steam** (or GOG — identical format)
3. If SaveWizard header detected → **PlayStation**
4. Default fallback → **Microsoft** (Xbox Game Pass)

**Reference:** `Global/Analyze.cs:163-192`

---

## 4. JSON Path System

libNOM.io uses a path dictionary with four variants per identifier:

```
JSONPATH["KEY"] = [
    "vanilla-obfuscated-path",
    "vanilla-plaintext-path",
    "omega-obfuscated-path",     // post-Worlds Part II structure
    "omega-plaintext-path"
]
```

Key examples:

| Identifier | Obfuscated | Plaintext |
|------------|------------|-----------|
| VERSION | `F2P` | `Version` |
| PLATFORM | `8>q` | `Platform` |
| SAVE_NAME | `6f=.Pk4` | `PlayerStateData.SaveName` |
| TOTAL_PLAY_TIME | `6f=.Lg8` | `PlayerStateData.TotalPlayTime` |
| BASE_GALACTIC_ADDRESS | `oZw` | `GalacticAddress` |
| BASE_NAME | `NKm` | `Name` |
| BASE_TYPE | `peI.DPp` | `BaseType.PersistentBaseTypes` |
| OWNER_UID | `K7E` | `UID` |
| OWNER_LID | `f5Q` | `LID` |

"Omega" refers to the restructured save format (post-Worlds Part II) which wraps PlayerStateData inside `BaseContext`/`ExpeditionContext` containers, with shared data in `CommonStateData`.

---

## 5. Key Differences: What NMS Copilot Needs vs What libNOM.io Does

**libNOM.io is a full save editor** — it supports read, modify, and write across all platforms. NMS Copilot only needs:

| Feature | libNOM.io | NMS Copilot (Phase 1) |
|---------|-----------|----------------------|
| Read save.hg | Yes | Yes |
| Write save.hg | Yes | **No** |
| XXTEA (meta) | Decrypt + encrypt | Decrypt only |
| SpookyHash/SHA-256 | Verify + generate | Verify only (optional) |
| Platform support | Steam, GOG, Xbox, PS, Switch | Steam/GOG only |
| Key deobfuscation | Full bidirectional | Deobfuscate only |
| Full JSON model | Everything | DiscoveryManagerData, PlayerStateData (position, bases) |
| Format 2001 | Yes | No (stretch goal) |
| Format 2002+ | Yes | Yes |

**Simplification opportunities:**

- Skip all write-path code (no encryption, no hash generation)
- Skip Xbox/PS/Switch platform detection
- Skip format 2001 (XXTEA-encrypted storage files)
- Only deserialize the JSON fields we need (discoveries, bases, player position)
- Use `serde_json` with `#[serde(rename)]` instead of runtime key replacement — apply mapping.json as a pre-processing step on raw JSON bytes

---

## 6. Recommended Rust Implementation Order

1. **LZ4 block decompression** — trivial with `lz4_flex::block::decompress`
2. **Format detection** — check first 4 bytes for magic vs `{"`
3. **Block reader** — iterate headers, decompress chunks, concatenate
4. **Key deobfuscation** — load mapping.json, string-replace on raw JSON
5. **Serde deserialization** — typed structs for top-level + DiscoveryManagerData + PlayerStateData subset
6. **XXTEA decryption** — ~50 lines, only for mf_save.hg verification
7. **Metadata parsing** — extract format version, sizes, optional hashes
8. **Save slot enumeration** — scan directory for save*.hg files

---

## 7. Source File Quick Reference

### libNOM.io Key Files

| File | What It Contains |
|------|-----------------|
| `Global/Constants.cs` | ALL magic numbers, format versions, JSON paths, meta lengths |
| `Global/Analyze.cs` | Entry point, platform detection, file header extraction |
| `Global/LZ4.cs` | LZ4 block decompression wrapper |
| `Platform/Platform_Read.cs` | Base read pipeline, LZ4 block parsing (lines 228-240) |
| `Platform/Platform_Write.cs` | Write pipeline (reference only for hash generation) |
| `PlatformSteam/PlatformSteam.cs` | Steam constants, XXTEA key, meta lengths, save paths |
| `PlatformSteam/PlatformSteam_Read.cs` | XXTEA decryption algorithm (lines 35-79) |
| `PlatformSteam/PlatformSteam_Write.cs` | SpookyHash/SHA-256 generation (lines 87-103) |
| `Container/Container.cs` | Save slot abstraction (slot index, save type, identifier) |
| `Extensions/Newtonsoft.cs` | libNOM.map integration, obfuscation state detection |
| `Enums/StoragePersistentSlotEnum.cs` | Slot enumeration (AccountData=1, PlayerState1-30=2-31) |
| `Enums/GameVersionEnum.cs` | Game version detection thresholds |
| `Meta/GameVersion.cs` | Version detection by meta length and base version |

### libNOM.map Key Files

| File | What It Contains |
|------|-----------------|
| `Mapping.cs` | Public API, version management, map creation (271 lines total) |
| `Mapping_Deobfuscation.cs` | Recursive tree walker, collision handling, key replacement |
| `Mapping_Obfuscation.cs` | Reverse mapping (not needed for NMS Copilot) |
| `Data/MappingJson.cs` | JSON deserialization model for mapping files |
| `Extensions/Newtonsoft.cs` | JProperty.Rename() implementation |
| `Resources/mapping_mbincompiler.json` | Primary mapping data (54KB, ~2000+ entries) |
| `Resources/mapping_legacy.json` | Legacy key compatibility (1.9KB) |
| `Services/GithubService.cs` | Downloads latest mapping from MBINCompiler releases |

---

## 8. goatfungus NMSSaveEditor (Java)

The goatfungus editor at `workbench/NMSSaveEditor/` is a compiled JAR with obfuscated bytecode — **no readable source code**. It remains useful as:

- A reference for **which fields the community considers important** to edit (currencies, ships, inventories, bases, settlements, companions)
- **Validation tool** — can be used to verify our parser's output matches what a known-good editor shows
- The README documents the full feature matrix for NMS save editing

It is **not useful** as a code reference for Rust reimplementation. Use libNOM.io instead.

---

## 9. Open Questions for Discussion

From the detailed project plan, with research findings:

### Q1: Spatial index — rstar vs kiddo

**Recommendation: `rstar`**

- `rstar` supports dynamic insert/remove — required for live updates from file watcher
- `kiddo` requires full rebuild on insert (designed for static datasets)
- For ~300 systems, performance difference is negligible
- `rstar` is well-maintained and widely used in the Rust ecosystem

### Q2: Key mapping updates

**Recommendation: All three strategies**

- **(a)** Bundle `mapping_mbincompiler.json` as a compiled-in resource (like libNOM.map does)
- **(b)** Support `--mapping` CLI flag / config file override for user-supplied mapping.json
- **(c)** Auto-detect obfuscation state: check for `"Version"` key (plaintext) vs `"F2P"` (obfuscated) — if already plaintext, skip mapping entirely

### Q3: Multi-galaxy support

**Finding:** RealityIndex (0–255) is part of GalacticAddress. Discoveries from different galaxies coexist in the same save file's DiscoveryManagerData. The graph model should use `HashMap<u8, GalaxyModel>` keyed by RealityIndex, with most users having data primarily in galaxy 0 (Euclid).

### Q4: Community data integration

**Finding:** NMSCE uses Firebase with portal glyph-based lookups. The NMSCD Coordinate-Converter TypeScript library documents the exact conversion math. Community data could be imported as CSV/JSON with portal addresses, tagged with source. Design the `Discovery` type with an `origin: DiscoveryOrigin` enum (SaveFile, Community, Manual).
