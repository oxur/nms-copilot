# NMS Save File Analysis — Session Findings (2026-03-05)

## Save File Overview

- File: `save.hg.json` (22MB, 1.3M lines)
- Exported by: goatfungus NMSSaveEditor (Java .jar, no source)
- Game version: 4720, Platform: Mac|Final
- Player: oubiwann (Steam), ~685 hours playtime

## What's In the Save

### Discovery Data

| Type | Available (yours) | Store (cached) | Total |
|------|:-:|:-:|:-:|
| **Planets** | 44 | 600 | 644 |
| Solar Systems | 30 | 300 | 330 |
| Sectors | 35 | ~270 | ~305 |
| Animals | 24 | ~180 | ~204 |
| Flora | 18 | ~130 | ~148 |
| Minerals | 21 | ~150 | ~171 |

### Unique Systems with Planet Data: 148

### Total Systems (incl. no-planet): 293

### Biome Distribution (644 planets)

```
Lush:         116    Red:           11
Frozen:        70    Swamp:         12
Scorched:      63    Lava:            7
Barren:        57    Waterworld:      5
Dead:          54    GasGiant:        6
Radioactive:   54    Green:          16
Toxic:         47    Blue:           15
Weird:         40
                     + 71 Infested variants (flag 0x10000)
```

### Named Discoveries

- 85 planets with custom names (DM.CN field)
- 39 solar systems with custom names
- 52 animals, 19 flora, 14 minerals named
- Your named planets: "Metok-Kalpa" (Lush), "Sushimi" (Lush), "Dipadri Grosso" (GasGiant)

### Player Bases: 53

- 49 HomePlanetBase, 1 FreighterBase, 3 PlayerShipBase
- Notable: "Sealab 2038", "Acadia National Park", "Void Egg Basejump Spire",
  "In-N-Outpost", "Dyphoti-benthic Research Base (sea bed)"

### Distance Metrics (from Gugestor Colony base)

- Nearest discovered system: ~18K ly
- Farthest: ~1.7M ly
- Inter-system range: 0 ly (co-located) to ~1.6M ly
- 675 system pairs within 100K ly
- Current player position is ~783K ly from nearest discoveries

## Key Technical Discoveries

### 1. Biome is encoded in VP[1]

The second value pair in planet discovery records directly maps to GcBiomeType enum.
Lower 16 bits = biome index, bit 16 = Infested flag.

### 2. Custom names in DM.CN

Only ~13% of planets have names. Unnamed planets have proc-gen names
computed at runtime from the seed — algorithm is in game binary.

### 3. Portal glyphs from Universe Address

UA packs location into 48 bits: `[VoxelX:12][VoxelZ:12][VoxelY:8][SSI:12][PI:4]`
Directly converts to 12 portal glyph hex digits: `P SSS YY ZZZ XXX`

### 4. Distance ≈ Euclidean voxel distance × 400 ly

Simple, effective approximation. Good enough for routing.

### 5. JSON needs careful parsing

The goatfungus export has `\xNN` and `\v` escapes that break standard JSON parsers.
Must use string-aware state machine — NOT global regex.
