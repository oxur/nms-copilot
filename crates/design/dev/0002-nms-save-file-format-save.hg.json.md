# NMS Save File Format (save.hg.json)

## Source

Exported by goatfungus NMSSaveEditor (Java, .jar only, no source)

- Repo: <https://github.com/goatfungus/NMSSaveEditor>
- Produces JSON with non-standard escape sequences

## JSON Parsing Issues

The exported JSON contains:

1. `\xNN` hex escapes (not valid JSON — must convert to `\u00NN`)
2. `\v` and other non-standard single-char escapes (must double the backslash)
3. Embedded binary-like ID strings with these escapes

**Safe parsing approach**: Character-by-character state machine that tracks
whether we're inside a JSON string literal. Only fix escapes within strings.
Do NOT use naive global regex — risk of corrupting structure outside strings.

## Top-Level Structure

```
{
  "Version": 4720,
  "Platform": "Mac|Final",
  "ActiveContext": "Main",
  "CommonStateData": { ... },      // 14 keys: save name, playtime, settings
  "BaseContext": {                  // Main game state
    "GameMode": int,
    "PlayerStateData": { ... },    // 254 keys — the motherlode
    "SpawnStateData": { ... }      // 15 keys
  },
  "DiscoveryManagerData": { ... }, // All discoveries
  "ExpeditionContext": { ... }     // Season/expedition data
}
```

## DiscoveryManagerData Structure

```
DiscoveryManagerData.DiscoveryData-v1:
  ReserveStore: 3200
  ReserveManaged: 3250
  Store.Record: [3200 items]      // Cached discoveries (from server/other players)
  Available: [172 items]           // Player's own discoveries
  Enqueued: []                     // Pending uploads
```

### Discovery Record Format

```json
{
  "TSrec": 1741910964,            // Unix timestamp (Available only)
  "DD": {                          // Discovery Data
    "UA": "0x242700FE0A56A3",     // Universe Address (hex string or int)
    "DT": "Planet",                // Discovery Type: Planet|SolarSystem|Sector|Animal|Flora|Mineral
    "VP": [                        // Value Pairs
      "0x2E62C00346219B6",        // VP[0]: Procedural generation seed hash
      9                            // VP[1]: Type-specific data (biome for planets)
    ]
  },
  "DM": {                          // Discovery Metadata (Store only, optional)
    "CN": "Planet Name"            // Custom Name (only if player-renamed)
  },
  "OWS": {                         // Owner State (Store only)
    "LID": "76561198025707979",   // Local ID
    "UID": "76561198025707979",   // User ID
    "USN": "oubiwann",            // Username
    "PTK": "ST",                   // Platform Token (ST=Steam, PS=PlayStation)
    "TS": 1740345088               // Timestamp
  },
  "FL": { "C": 1, "U": 1 },      // Flags
  "RID": "base64string"           // Record ID
}
```

### VP[1] Biome Encoding for Planets

Lower 16 bits = biome index (GcBiomeType::BiomeEnum):

```
 0 = Lush          8 = Red (stellar)
 1 = Toxic         9 = Green (stellar)
 2 = Scorched     10 = Blue (stellar)
 3 = Radioactive  11 = Test
 4 = Frozen       12 = Swamp
 5 = Barren       13 = Lava
 6 = Dead         14 = Waterworld
 7 = Weird        15 = GasGiant
                   16 = All
```

Upper bits: `0x10000` flag = Infested variant

Source: <https://github.com/monkeyman192/MBINCompiler/blob/development/libMBIN/Source/NMS/GameComponents/GcBiomeType.cs>

### Biome SubTypes (GcBiomeSubType::BiomeSubTypeEnum)

```
 0 = None           16 = HugeRing
 1 = Standard       17 = HugeRock
 2 = HighQuality    18 = HugeScorch
 3 = Structure      19 = HugeToxic
 4 = Beam           20 = Variant_A
 5 = Hexagon        21 = Variant_B
 6 = FractCube      22 = Variant_C
 7 = Bubble         23 = Variant_D
 8 = Shards         24 = Infested
 9 = Contour        25 = Swamp
10 = Shell          26 = Lava
11 = BoneSpire      27 = Worlds
12 = WireCell       28 = Remix_A
13 = HydroGarden    29 = Remix_B
14 = HugePlant      30 = Remix_C
15 = HugeLush       31 = Remix_D
```

Source: <https://github.com/monkeyman192/MBINCompiler/blob/development/libMBIN/Source/NMS/GameComponents/GcBiomeSubType.cs>

## Universe Address (UA) Encoding

48-bit packed integer encoding galactic location:

```
Bits  0-3:  PlanetIndex       (4 bits, 0-15)
Bits  4-15: SolarSystemIndex  (12 bits, 0-4095)
Bits 16-23: VoxelY            (8 bits, unsigned; signed = raw - 0x7F)
Bits 24-35: VoxelZ            (12 bits, unsigned; signed = raw - 0x7FF)
Bits 36-47: VoxelX            (12 bits, unsigned; signed = raw - 0x7FF)
```

Coordinate ranges (signed):

- VoxelX: -2047 to 2048
- VoxelY: -127 to 128
- VoxelZ: -2047 to 2048

### Portal Glyph Conversion

12 hex digits: `P SSS YY ZZZ XXX`

- P = PlanetIndex (1 digit)
- SSS = SolarSystemIndex (3 digits)
- YY = VoxelY unsigned (2 digits)
- ZZZ = VoxelZ unsigned (3 digits)
- XXX = VoxelX unsigned (3 digits)

### Signal Booster Format

`XXXX:YYYY:ZZZZ:SSSS` (all unsigned hex)

### Distance Calculation

Each voxel ≈ 400 light-years.
Euclidean distance: `sqrt(dx² + dy² + dz²) × 400 ly`

Systems in the same voxel (same X/Y/Z, different SSI) are co-located (~0-400 ly).

### Portal Glyph Names & Icons

```
0=Sunset    4=Eclipse    8=Dragonfly  C=Rocket
1=Bird      5=Balloon    9=Galaxy     D=Tree
2=Face      6=Boat       A=Voxel      E=Atlas
3=Diplo     7=Bug        B=Canoe      F=BlackHole
```

## PlayerStateData (key planet-related fields)

```
UniverseAddress          — Current location (structured, with RealityIndex)
PreviousUniverseAddress  — Previous system
FreighterUniverseAddress — Freighter location
PlanetPositions[16]      — 3D positions of planets in current system
PlanetSeeds[16]          — Proc-gen seeds for current system planets
PrimaryPlanet            — Current planet index
VisitedSystems[512]      — Array of visited system GA integers
PersistentPlayerBases[N] — All player bases with:
  - Name, GalacticAddress, Position[3], BaseType
  - Objects[] (building parts)
  - Owner (LID, UID, USN, PTK)
WonderPlanetRecords[11]  — Notable planet records
GalaxyWaypoints[3]       — Active waypoints (Atlas, BlackHole, Mission)
TerrainEditData          — Terrain modifications
```

## Typical Ship Warp Ranges

```
Stock hyperdrive:   ~100 ly
B-class upgrade:    ~250 ly
A-class upgrade:    ~800 ly
S-class (max):    ~2,500 ly
Freighter (max):  ~6,000 ly
```
